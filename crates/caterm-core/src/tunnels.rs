//! Port Forwarding / Tunneling Manager (`T2-TOOL-02`).
//! Supports Local, Remote, and Dynamic (SOCKS5 proxy) port forwarding over SSH.
//! Safe lifecycle management: guarantees clean closing upon exit or vault lock.

use crate::error::{CatermError, ValidationError};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ForwardType {
    Local,
    Remote,
    Dynamic, // SOCKS5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelRecord {
    pub id: String,
    pub host_id: String,
    pub forward_type: ForwardType,
    pub bind_addr: String,
    pub bind_port: u16,
    pub target_addr: String,
    pub target_port: u16,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInput {
    pub host_id: String,
    pub forward_type: ForwardType,
    pub bind_addr: Option<String>,
    pub bind_port: u16,
    pub target_addr: String,
    pub target_port: u16,
}

struct ActiveTunnel {
    shutdown_signal: Arc<AtomicBool>,
}

static ACTIVE_TUNNELS: Lazy<Arc<Mutex<HashMap<String, ActiveTunnel>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("tun-{nanos:x}")
}

/// Create the `tunnels` database table if it doesn't already exist.
pub fn init_table(conn: &rusqlite::Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tunnels (
            id TEXT PRIMARY KEY,
            host_id TEXT NOT NULL,
            forward_type TEXT NOT NULL,
            bind_addr TEXT NOT NULL,
            bind_port INTEGER NOT NULL,
            target_addr TEXT NOT NULL,
            target_port INTEGER NOT NULL
        );",
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to create tunnels table: {e}"
        )))
    })?;
    Ok(())
}

/// List all configured tunnels along with active live state.
pub fn list_tunnels() -> Result<Vec<TunnelRecord>, CatermError> {
    let conn = crate::db::open()?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port FROM tunnels")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let active_map = ACTIVE_TUNNELS.lock();

    let rows = stmt
        .query_map([], |row| {
            let ftype_str: String = row.get(2)?;
            let forward_type = match ftype_str.as_str() {
                "Remote" => ForwardType::Remote,
                "Dynamic" => ForwardType::Dynamic,
                _ => ForwardType::Local,
            };

            let id: String = row.get(0)?;
            let is_active = active_map.contains_key(&id);

            Ok(TunnelRecord {
                id,
                host_id: row.get(1)?,
                forward_type,
                bind_addr: row.get(3)?,
                bind_port: row.get(4)?,
                target_addr: row.get(5)?,
                target_port: row.get(6)?,
                is_active,
            })
        })
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result
            .push(r.map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?);
    }
    Ok(result)
}

/// Save a tunnel configuration in the vault.
pub fn save_tunnel(input: TunnelInput) -> Result<TunnelRecord, CatermError> {
    let bind_addr = input.bind_addr.unwrap_or_else(|| "127.0.0.1".to_string());
    let forward_type_str = match input.forward_type {
        ForwardType::Local => "Local",
        ForwardType::Remote => "Remote",
        ForwardType::Dynamic => "Dynamic",
    };

    let id = generate_id();

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO tunnels (id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            input.host_id,
            forward_type_str,
            bind_addr,
            input.bind_port,
            input.target_addr,
            input.target_port
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save tunnel: {e}"))))?;

    Ok(TunnelRecord {
        id,
        host_id: input.host_id,
        forward_type: input.forward_type,
        bind_addr,
        bind_port: input.bind_port,
        target_addr: input.target_addr,
        target_port: input.target_port,
        is_active: false,
    })
}

/// Save a tunnel record directly (used by cloud sync / import to preserve existing ID).
pub fn save_tunnel_record(record: &TunnelRecord) -> Result<(), CatermError> {
    let forward_type_str = match record.forward_type {
        ForwardType::Local => "Local",
        ForwardType::Remote => "Remote",
        ForwardType::Dynamic => "Dynamic",
    };

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO tunnels (id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            host_id = excluded.host_id,
            forward_type = excluded.forward_type,
            bind_addr = excluded.bind_addr,
            bind_port = excluded.bind_port,
            target_addr = excluded.target_addr,
            target_port = excluded.target_port",
        rusqlite::params![
            record.id,
            record.host_id,
            forward_type_str,
            record.bind_addr,
            record.bind_port,
            record.target_addr,
            record.target_port
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to upsert tunnel: {e}"))))?;

    Ok(())
}

/// Delete a tunnel configuration and close if currently active.
pub fn delete_tunnel(id: &str) -> Result<(), CatermError> {
    let _ = stop_tunnel(id);

    let conn = crate::db::open()?;
    init_table(&conn)?;

    let affected = conn
        .execute("DELETE FROM tunnels WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    if affected == 0 {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "Tunnel '{id}' not found"
        ))));
    }

    Ok(())
}

/// A dedicated blocking session for a tunnel listener. Delegates to
/// [`crate::ssh::open_authenticated_session`] so a tunnel goes through the same TOFU host key
/// verification as every other connection — this used to be a hand-rolled copy that skipped it.
fn connect_host_session(host_id: &str) -> Result<ssh2::Session, CatermError> {
    let (sess, _host) = crate::ssh::open_authenticated_session(host_id)?;
    Ok(sess)
}

/// libssh2's `LIBSSH2_ERROR_EAGAIN`: "would block", returned by every call on a non-blocking session.
const LIBSSH2_ERROR_EAGAIN: i32 = -37;

fn is_eagain(e: &ssh2::Error) -> bool {
    matches!(e.code(), ssh2::ErrorCode::Session(LIBSSH2_ERROR_EAGAIN))
}

/// Re-runs a libssh2 call that may report EAGAIN until it completes, the tunnel is shut down,
/// or 15 seconds pass.
fn retry_eagain<T>(
    shutdown: &AtomicBool,
    mut f: impl FnMut() -> Result<T, ssh2::Error>,
) -> Result<T, ssh2::Error> {
    let started = Instant::now();
    loop {
        match f() {
            Err(e) if is_eagain(&e) => {
                if shutdown.load(Ordering::Relaxed) || started.elapsed() > Duration::from_secs(15) {
                    return Err(e);
                }
                thread::sleep(Duration::from_millis(2));
            }
            other => return other,
        }
    }
}

/// `write_all` for a non-blocking writer: `WouldBlock` means "try again", not "failed".
fn write_all_retry<W: Write>(w: &mut W, mut buf: &[u8], shutdown: &AtomicBool) -> io::Result<()> {
    while !buf.is_empty() {
        match w.write(buf) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(n) => buf = buf.get(n..).unwrap_or(&[]),
            Err(e) if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted) => {
                if shutdown.load(Ordering::Relaxed) {
                    return Err(io::ErrorKind::Interrupted.into());
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Copies bytes both ways between a local socket and an SSH channel until either side closes.
///
/// The SSH session must be non-blocking. With a blocking session one `channel.read()` holds
/// libssh2's session lock until the remote answers, so a request larger than one buffer (the
/// client is still sending while the server waits for the rest of it) deadlocked, and every
/// other connection of the tunnel stalled behind that lock.
pub(crate) fn relay(mut local: TcpStream, mut channel: ssh2::Channel, shutdown: &AtomicBool) {
    if local.set_nonblocking(true).is_err() {
        return;
    }
    let mut buf = [0u8; 16 * 1024];
    let mut idle_rounds: u32 = 0;
    let mut local_done = false;
    while !shutdown.load(Ordering::Relaxed) {
        let mut active = false;

        if !local_done {
            match local.read(&mut buf) {
                Ok(0) => {
                    local_done = true;
                    let _ = retry_eagain(shutdown, || channel.send_eof());
                }
                Ok(n) => {
                    active = true;
                    if write_all_retry(&mut channel, buf.get(..n).unwrap_or(&[]), shutdown).is_err() {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }
        }

        match channel.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                active = true;
                if write_all_retry(&mut local, buf.get(..n).unwrap_or(&[]), shutdown).is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
        if channel.eof() {
            break;
        }

        if active {
            idle_rounds = 0;
        } else {
            idle_rounds = idle_rounds.saturating_add(1);
            thread::sleep(Duration::from_millis(u64::from(idle_rounds.min(10))));
        }
    }
    let _ = channel.close();
}

/// Sends SSH keepalives for as long as the tunnel runs. A failed one means the link is dead:
/// the tunnel is shut down and dropped from the active list so the UI stops showing it as up.
fn spawn_keepalive(sess: ssh2::Session, shutdown: Arc<AtomicBool>, id: String) {
    thread::spawn(move || {
        let mut since_last = Duration::ZERO;
        let tick = Duration::from_millis(500);
        while !shutdown.load(Ordering::Relaxed) {
            thread::sleep(tick);
            since_last += tick;
            if since_last < Duration::from_secs(15) {
                continue;
            }
            since_last = Duration::ZERO;
            match sess.keepalive_send() {
                Ok(_) => {}
                Err(e) if is_eagain(&e) => {}
                Err(_) => {
                    shutdown.store(true, Ordering::Relaxed);
                    ACTIVE_TUNNELS.lock().remove(&id);
                    return;
                }
            }
        }
    });
}

/// Minimal SOCKS5 (RFC 1928) server side: no-auth + CONNECT only. Answers the client with the
/// proper failure code when it asks for something unsupported, then returns an error.
fn socks5_handshake(stream: &mut TcpStream) -> io::Result<(String, u16)> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let bad = || io::Error::from(io::ErrorKind::InvalidData);

    let mut head = [0u8; 2];
    stream.read_exact(&mut head)?;
    if head[0] != 5 {
        return Err(bad());
    }
    let mut methods = vec![0u8; head[1] as usize];
    stream.read_exact(&mut methods)?;
    if !methods.contains(&0) {
        stream.write_all(&[5, 0xFF])?;
        return Err(bad());
    }
    stream.write_all(&[5, 0])?;

    let mut req = [0u8; 4];
    stream.read_exact(&mut req)?;
    if req[0] != 5 {
        return Err(bad());
    }
    if req[1] != 1 {
        // Only CONNECT: "command not supported".
        stream.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0])?;
        return Err(bad());
    }
    let host = match req[3] {
        1 => {
            let mut ip = [0u8; 4];
            stream.read_exact(&mut ip)?;
            std::net::Ipv4Addr::from(ip).to_string()
        }
        3 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len)?;
            let mut name = vec![0u8; len[0] as usize];
            stream.read_exact(&mut name)?;
            String::from_utf8_lossy(&name).into_owned()
        }
        4 => {
            let mut ip = [0u8; 16];
            stream.read_exact(&mut ip)?;
            std::net::Ipv6Addr::from(ip).to_string()
        }
        _ => {
            stream.write_all(&[5, 8, 0, 1, 0, 0, 0, 0, 0, 0])?;
            return Err(bad());
        }
    };
    let mut port = [0u8; 2];
    stream.read_exact(&mut port)?;
    stream.set_read_timeout(None)?;
    Ok((host, u16::from_be_bytes(port)))
}

fn bind_listener(addr: &str, port: u16) -> Result<TcpListener, CatermError> {
    let bind_str = format!("{addr}:{port}");
    let listener = TcpListener::bind(&bind_str).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to bind local port {bind_str}: {e}"
        )))
    })?;
    listener.set_nonblocking(true).ok();
    Ok(listener)
}

/// Start an active port forward tunnel listener.
pub fn start_tunnel(id: &str) -> Result<(), CatermError> {
    if ACTIVE_TUNNELS.lock().contains_key(id) {
        return Ok(()); // Already running
    }

    let tunnels = list_tunnels()?;
    let tun = tunnels
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| {
            CatermError::Validation(ValidationError::Generic(format!("Tunnel '{id}' not found")))
        })?
        .clone();

    let _ = crate::audit::log_event(
        "TUNNEL_START",
        Some(&tun.host_id),
        &format!(
            "Started tunnel ({:?}) on port {}",
            tun.forward_type, tun.bind_port
        ),
    );

    let shutdown_signal = Arc::new(AtomicBool::new(false));

    match tun.forward_type {
        ForwardType::Local => {
            let listener = bind_listener(&tun.bind_addr, tun.bind_port)?;
            let sess = connect_host_session(&tun.host_id)?;
            sess.set_keepalive(true, 15);
            sess.set_blocking(false);
            spawn_keepalive(sess.clone(), Arc::clone(&shutdown_signal), id.to_string());

            let target_addr = tun.target_addr.clone();
            let target_port = tun.target_port;
            let shutdown = Arc::clone(&shutdown_signal);
            thread::spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((local, _)) => {
                            let sess = sess.clone();
                            let shutdown = Arc::clone(&shutdown);
                            let target_addr = target_addr.clone();
                            thread::spawn(move || {
                                let Ok(channel) = retry_eagain(&shutdown, || {
                                    sess.channel_direct_tcpip(&target_addr, target_port, None)
                                }) else {
                                    return;
                                };
                                relay(local, channel, &shutdown);
                            });
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        ForwardType::Remote => {
            let sess = connect_host_session(&tun.host_id)?;
            // Listening is a blocking request; only afterwards does the session go non-blocking.
            let (mut listener, _bound_port) = sess
                .channel_forward_listen(tun.bind_port, Some(&tun.bind_addr), None)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!(
                        "Failed to remote bind: {e}"
                    )))
                })?;
            sess.set_keepalive(true, 15);
            sess.set_blocking(false);
            spawn_keepalive(sess.clone(), Arc::clone(&shutdown_signal), id.to_string());

            let target_addr = tun.target_addr.clone();
            let target_port = tun.target_port;
            let shutdown = Arc::clone(&shutdown_signal);
            thread::spawn(move || {
                // `sess` must outlive the listener; moving it in keeps the transport open.
                let _sess = sess;
                while !shutdown.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok(channel) => {
                            let shutdown = Arc::clone(&shutdown);
                            let target_addr = target_addr.clone();
                            thread::spawn(move || {
                                let Ok(local) = TcpStream::connect((target_addr.as_str(), target_port))
                                else {
                                    let mut channel = channel;
                                    let _ = channel.close();
                                    return;
                                };
                                relay(local, channel, &shutdown);
                            });
                        }
                        Err(ref e) if is_eagain(e) => {
                            thread::sleep(Duration::from_millis(20));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        ForwardType::Dynamic => {
            let listener = bind_listener(&tun.bind_addr, tun.bind_port)?;
            let sess = connect_host_session(&tun.host_id)?;
            sess.set_keepalive(true, 15);
            sess.set_blocking(false);
            spawn_keepalive(sess.clone(), Arc::clone(&shutdown_signal), id.to_string());

            let shutdown = Arc::clone(&shutdown_signal);
            thread::spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut local, _)) => {
                            let sess = sess.clone();
                            let shutdown = Arc::clone(&shutdown);
                            thread::spawn(move || {
                                let Ok((host, port)) = socks5_handshake(&mut local) else {
                                    return;
                                };
                                let channel = match retry_eagain(&shutdown, || {
                                    sess.channel_direct_tcpip(&host, port, None)
                                }) {
                                    Ok(c) => c,
                                    Err(_) => {
                                        // 0x05: connection refused.
                                        let _ = local.write_all(&[5, 5, 0, 1, 0, 0, 0, 0, 0, 0]);
                                        return;
                                    }
                                };
                                if local.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]).is_err() {
                                    return;
                                }
                                relay(local, channel, &shutdown);
                            });
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }

    ACTIVE_TUNNELS
        .lock()
        .insert(id.to_string(), ActiveTunnel { shutdown_signal });
    Ok(())
}

/// Stop an active port forward tunnel listener.
pub fn stop_tunnel(id: &str) -> Result<(), CatermError> {
    if let Some(active) = ACTIVE_TUNNELS.lock().remove(id) {
        active.shutdown_signal.store(true, Ordering::Relaxed);

        if let Some(tun) = list_tunnels()
            .ok()
            .as_ref()
            .and_then(|tunnels| tunnels.iter().find(|t| t.id == id))
        {
            let _ = crate::audit::log_event(
                "TUNNEL_STOP",
                Some(&tun.host_id),
                &format!(
                    "Stopped tunnel ({:?}) on port {}",
                    tun.forward_type, tun.bind_port
                ),
            );
        }
    }
    Ok(())
}

/// Stop all active tunnels cleanly (e.g. called on vault lock or app exit).
pub fn stop_all_tunnels() -> Result<(), CatermError> {
    let mut tunnels = ACTIVE_TUNNELS.lock();
    for (_, tunnel) in tunnels.drain() {
        tunnel.shutdown_signal.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression for C-03: save_tunnel/list_tunnels/delete_tunnel used to open the database
    // with `vault::load_or_create_local_key` (a plaintext key file used nowhere else) instead
    // of the vault DEK every other module uses via `db::open()`. Against a database created by
    // any other module the mismatch failed with "file is not a database". Unlocking the vault
    // first, as the real app does before any of these calls, reproduces that setup.
    #[test]
    fn save_list_and_delete_tunnel_use_the_same_key_as_the_rest_of_the_app() {
        let _data = crate::test_support::isolated_data_dir("tunnels_roundtrip");
        crate::vault::unlock_vault("12345678").expect("unlock vault");

        let saved = save_tunnel(TunnelInput {
            host_id: "host-1".into(),
            forward_type: ForwardType::Local,
            bind_addr: None,
            bind_port: 2222,
            target_addr: "127.0.0.1".into(),
            target_port: 22,
        })
        .expect("save_tunnel should use the vault DEK, not a separate local key");

        let all = list_tunnels().expect("list_tunnels should open the same database");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);
        assert_eq!(all[0].bind_port, 2222);

        delete_tunnel(&saved.id).expect("delete_tunnel should open the same database");
        assert!(
            list_tunnels()
                .expect("list_tunnels after delete")
                .is_empty()
        );
    }
}
