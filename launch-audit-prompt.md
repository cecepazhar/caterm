# PROMPT AUDIT KESIAPAN LAUNCH — CATerm v2 (Cyber Monday + Product Hunt)

> Tempel ke Claude Code / coding agent sebagai prompt awal. Spesifik repo `~/Product/caterm`, bukan generik.
> Setiap temuan WAJIB disertai bukti (file:baris + log output), bukan opini.

## KONTEKS PRODUK (fakta, jangan diubah)
- CATerm v2: SSH/terminal client desktop. Rust workspace (`crates/caterm-core`, `caterm-app` = Tauri v2, `caterm-cli`) + SvelteKit 5 (`frontend/`).
- Vault zero-knowledge (SQLite terenkripsi). TOFU host-key verification wajib utuh.
- Monetisasi: Free vs Pro $10/bulan. Brand: Rose `#ef4444`, dual-wing SVG.
- i18n ID/EN wajib parity. Hana AI trigger kanan-bawah. Feedback modal maksimal 1x.
- Dokumen hidup: `ledger-v2.md` + `task-v2.md` (update format yang ada, jangan buat doc baru).
- Aturan arsitektur: badan `#[tauri::command]` maks 15 baris, tanpa `InsecureIgnoreHostKey`, tanpa `.unwrap()` baru dari input user.

## BUKTI QA WAJIB (Whitebox + Blackbox, standar owner)
1. Whitebox Crypto/Vault: buktikan kredensial tidak pernah plaintext di disk, log, atau crash dump (tunjuk file:baris).
2. Whitebox Arsitektur: `cargo test -p caterm-core --test arch` = 0 violation.
3. Whitebox Linter: `cargo clippy --workspace` 0 warning 0 error; `cargo fmt --check` bersih.
4. Blackbox: `cargo test --workspace` semua hijau; `npm run check` di `frontend/` bersih; uji build nyata pada jalur kritis.

## AREA AUDIT (status: PASS / WARNING / BLOCKER)
1. Core SSH: TOFU, reset host key, auto-reconnect, keep-alive, timeout handling.
2. Terminal: xterm render, resize window, copy-paste, ANSI colors, unicode.
3. SFTP: upload/download/drag-drop, cancel transfer, error permission.
4. Tunnels & Keys: local/remote port forwarding, import/export ed25519 & RSA.
5. Security: vault encryption, sanitasi input, klaim zero-telemetry terbukti di kode.
6. Packaging: `cargo tauri build` menghasilkan .exe/.msi/.dmg/.AppImage, uninstall bersih, versi bump, first-run wizard.
7. Monetisasi: gate Free vs Pro $10/bulan, validasi lisensi offline, kupon Cyber Monday tidak memblokir fitur dasar.
8. Launch assets: ikon, screenshot 1270x760 (6-8 gambar), GIF demo 15-30 detik, tagline maks 60 karakter, landing page (ikon GitHub + tombol Docs & Sponsor), docs `?lang=id`, link Privacy/Terms/GitHub tanpa 404.
9. i18n ID/EN parity, Hana AI trigger, feedback modal maks 1x.

## EXIT CRITERIA (100% SIAP)
- 0 BLOCKER, 0 test merah, 0 warning clippy/svelte-check.
- Installer teruji install + run + uninstall di Windows (dan OS target lain).
- Lisensi dan kupon Cyber Monday teruji end-to-end.

## FORMAT OUTPUT
1. Ringkasan eksekutif: SIAP LAUNCH / BELUM SIAP.
2. Tabel temuan per area + bukti (file:baris, log) + status.
3. Daftar fix blocker berurut prioritas.
4. Quick-win UI/UX dalam 48 jam.
5. Catat keputusan desain baru di `ledger-v2.md`.
