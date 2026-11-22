# Changelog

Semua perubahan penting pada CATerm didokumentasikan dalam berkas ini.
Format mengacu pada [Keep a Changelog](https://keepachangelog.com/id/), dan proyek mematuhi [Semantic Versioning](https://semver.org/).

---

## [v2.1.44] - 2026-11-22

### Added
- **GCC License v2 (Ed25519 Signature)**: Transisi penuh dari HMAC legacy ke tanda tangan kriptografis asimetris Ed25519 (`key_version: 1`). Public key produksi `ac5b9954c8dc7648131e9682a90999de4d38a15deb00095eee4f6be9521e1dcf` ditanam langsung di biner aplikasi.
- **Device Deactivation Flow**: Menambahkan dukungan endpoint `POST /licenses/deactivate` untuk melepaskan ikatan perangkat Pro secara mandiri langsung dari aplikasi (tombol "Lepas perangkat ini" pada tab Settings).
- **Grace Period Offline 14 Hari**: Dukungan operasional penuh offline hingga 14 hari sebelum status otomatis turun ke Free tanpa mengunci data lokal pengguna.
- **Suite Regresi Lisensi Offline**: Menambahkan 6 unit test regresi komprehensif di `caterm-core/src/pro.rs` (boundary token expired, HWID mismatch, SKU mismatch, verifikasi signature rusak, clock rollback 300s, dan lifecycle offline grace).
- **Infrastruktur Local CI/CD Pipeline**:
  - `scripts/check-version-sync.sh`: Memvalidasi sinkronisasi 4 konfigurasi versi (`Cargo.toml` == `tauri.conf.json` == `package.json` == `package-lock.json`).
  - `scripts/ci.sh`: Eksekusi 5 tahap otomatis (`check-version-sync`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `svelte-check`).
  - `scripts/release.sh`: Skrip packaging rilis aman dengan proteksi fail-closed jika `TAURI_SIGNING_PRIVATE_KEY` belum diset.
  - `scripts/hooks/pre-push` & `scripts/install-hooks.sh`: Git hook opt-in pre-push.

### Changed
- **Device Limit Error Mapping**: Respons server `maximum device limit reached` kini dipetakan secara akurat ke kode translatable `DEVICE_LIMIT_EXCEEDED`.
- **Gating Fitur Cloud Sync & Team Space**: Diselaraskan dengan ketetapan D-3; Cloud Sync dan Team Space diberi label "Coming Soon (Q1 2027)" pada landing page dan dinonaktifkan secara aman saat aktivasi berbasis license key saja.
- **Pembersihan UI Lisensi**: Menghapus tombol "Owner Quick Unlock" yang menanam key di frontend.

### Fixed
- **Clippy Warnings Cleanup**: Menyelesaikan 57 peringatan clippy di seluruh workspace (`caterm-core`, `caterm-pro`, `caterm-cli`, `caterm-app`) tanpa suppress `#[allow]`.
- **Frontend Svelte Check**: Menyelesaikan diskrepansi tipe unik `Snippet` pada Svelte 5 / `@cecepazhar/caui` (0 errors pada `svelte-check`).
- **Audit String & Secret 100% Bersih**: Menghilangkan seluruh jejak rahasia, IP pribadi, dan secret lama dari biner rilis Linux (`.deb`, `.rpm`, `.AppImage`). Checksum `dist/SHA256SUMS.txt` tervalidasi.
