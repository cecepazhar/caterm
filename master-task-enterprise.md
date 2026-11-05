# CATerm — Master Task: Enterprise Readiness

> **Tujuan:** membawa CATerm (v2.1.41) dari "aplikasi produktivitas yang kuat" menjadi produk yang **lolos review IT, security, dan procurement perusahaan**: bisa di-deploy massal, dikelola terpusat, diaudit, dan didukung secara profesional.
>
> **Baseline:** branch `main` @ `5843207` (v2.1.41, November 2026).
> **Pemilik dokumen:** Cecep Azhar. Dokumen ini hidup: perbarui status setiap kali task selesai.

---

## Daftar Isi

0. [Cara Membaca Dokumen Ini](#0-cara-membaca-dokumen-ini)
1. [Definisi "Enterprise Ready"](#1-definisi-enterprise-ready)
2. [Baseline Audit: Kondisi Saat Ini](#2-baseline-audit-kondisi-saat-ini)
3. [Roadmap & Milestone](#3-roadmap--milestone)
4. [Workstream & Task Detail](#4-workstream--task-detail)
   - [WS-GOV: Repo Hygiene & Governance](#ws-gov--repo-hygiene--governance)
   - [WS-REL: Build, Release & Supply Chain](#ws-rel--build-release--supply-chain)
   - [WS-QA: Quality & Testing](#ws-qa--quality--testing)
   - [WS-OBS: Observability & Supportability](#ws-obs--observability--supportability)
   - [WS-SEC: Security Hardening](#ws-sec--security-hardening)
   - [WS-ADM: Enterprise Administration & Deployment](#ws-adm--enterprise-administration--deployment)
   - [WS-IAM: Identity, Teams & Licensing](#ws-iam--identity-teams--licensing)
   - [WS-CMP: Compliance, Audit & Privacy](#ws-cmp--compliance-audit--privacy)
   - [WS-UX: Accessibility, i18n & UX Consistency](#ws-ux--accessibility-i18n--ux-consistency)
   - [WS-DOC: Documentation & Support](#ws-doc--documentation--support)
5. [Urutan Eksekusi & Dependensi](#5-urutan-eksekusi--dependensi)
6. [Risiko & Mitigasi](#6-risiko--mitigasi)
7. [Lampiran](#7-lampiran)

---

## 0. Cara Membaca Dokumen Ini

### 0.1 Format ID task

`ENT-<WORKSTREAM>-<NN>`, contohnya `ENT-REL-03`. ID tidak boleh diubah setelah dibuat; task yang dibatalkan ditandai `❌ Dropped`, bukan dihapus.

### 0.2 Prioritas

| Kode | Arti | Aturan |
|---|---|---|
| **P0** | Blocker. Tanpa ini, perusahaan menolak atau CI rusak. | Dikerjakan sebelum task lain di milestone yang sama. |
| **P1** | Wajib untuk label "Enterprise Ready". | Harus selesai sebelum M3 ditutup. |
| **P2** | Diharapkan oleh pelanggan menengah–besar. | Boleh menyusul di M4. |
| **P3** | Nilai tambah / diferensiasi. | Backlog. |

### 0.3 Estimasi effort

| Kode | Durasi kerja efektif |
|---|---|
| **S** | ≤ ½ hari |
| **M** | 1–2 hari |
| **L** | 3–5 hari |
| **XL** | > 1 minggu (wajib dipecah sebelum dikerjakan) |

### 0.4 Pelaksana

| Kode | Arti |
|---|---|
| 🤖 **Claude** | Bisa dikerjakan penuh lewat kode/CI. |
| 👤 **Owner** | Butuh tindakan pemilik: pembelian, akun vendor, keputusan bisnis, tanda tangan legal. |
| 🤝 **Bersama** | Owner menyediakan input/kredensial, Claude yang mengimplementasikan. |

### 0.5 Status

`⬜ Todo` · `🟦 In Progress` · `🟨 Review` · `✅ Done` · `⛔ Blocked` · `❌ Dropped`

### 0.6 Definition of Done (berlaku untuk SEMUA task)

1. Kode lolos `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `npm run check`, `npm run build`.
2. Ada tes otomatis untuk perilaku baru, atau alasan tertulis kenapa tidak bisa dites.
3. Dokumentasi pengguna/admin diperbarui bila perilaku terlihat oleh pengguna.
4. Entri `CHANGELOG.md` ditambahkan (setelah ENT-GOV-04 selesai).
5. CI hijau di PR, dan PR di-review sebelum merge ke `main`.

---

## 1. Definisi "Enterprise Ready"

CATerm dianggap **Enterprise Ready** jika SEMUA kriteria berikut terpenuhi dan bisa dibuktikan:

| # | Kriteria | Bukti yang diminta pembeli | Task terkait |
|---|---|---|---|
| E1 | Binary resmi ditandatangani di semua OS desktop | Tidak ada peringatan SmartScreen / Gatekeeper; `signtool verify` & `spctl -a` lolos | ENT-REL-01..03 |
| E2 | Bisa di-deploy massal tanpa interaksi pengguna | Installer MSI + flag silent, terdokumentasi untuk Intune/SCCM/Jamf | ENT-ADM-01..04 |
| E3 | Admin bisa mengunci pengaturan secara terpusat | Managed policy (Registry/plist/JSON) dihormati & tidak bisa dioverride pengguna | ENT-ADM-05..09 |
| E4 | Aktivitas bisa diaudit dan dikirim ke SIEM | Audit log tamper-evident + export syslog/JSON | ENT-CMP-01..04 |
| E5 | Supply chain transparan | SBOM per rilis, provenance, checksum bertanda tangan | ENT-REL-05..08 |
| E6 | Proses keamanan formal | `SECURITY.md`, threat model, hasil pentest, SLA patch CVE | ENT-SEC-01..03 |
| E7 | Kualitas terukur | Coverage minimum, tes E2E di CI, zero warning a11y | ENT-QA-* |
| E8 | Bisa didukung (supportable) | Log terstruktur + bundle diagnostik satu-klik | ENT-OBS-* |
| E9 | Dokumentasi admin & keamanan lengkap | Admin Guide, Security Whitepaper, kuesioner vendor siap | ENT-DOC-* |
| E10 | Siklus rilis dapat diprediksi | Semantic versioning, kanal Stable/LTS, kebijakan EOL | ENT-REL-10, ENT-DOC-06 |

---

## 2. Baseline Audit: Kondisi Saat Ini

Semua temuan di bawah diverifikasi langsung dari repo pada commit `5843207`.

### 2.1 Kekuatan yang sudah ada (pertahankan)

| Area | Fakta | Lokasi |
|---|---|---|
| Enkripsi data lokal | SQLCipher + Argon2id | `crates/caterm-core/src/vault.rs`, `db.rs` |
| Verifikasi host SSH | `known_hosts` per data dir (TOFU) | `crates/caterm-core/src/ssh.rs:220`, `:402` |
| Masking secret di audit | Regex `password\|pass\|secret\|token\|key` | `crates/caterm-core/src/audit.rs:17` |
| Crash report ber-scrubber PII | `PiiScrubber`, opt-out `is_disabled()` | `crates/caterm-core/src/crash.rs` |
| CSP ketat | `script-src 'self'` | `crates/caterm-app/tauri.conf.json` |
| Tauri capabilities granular | Permission per command | `crates/caterm-app/capabilities/*.json` |
| CI cukup lengkap | fmt, clippy ×3 OS, test ×3 OS, arch-guard, secret-scan, cargo-audit, cargo-deny, frontend build, budget-check, clean-clone-build, repo-hygiene | `.github/workflows/ci.yml` |
| Tes Rust | ±144 tes unit + 4 file tes integrasi | `crates/caterm-core/tests/` |
| Updater bertanda tangan | Minisign pubkey di config | `tauri.conf.json → plugins.updater` |
| Lingkungan tes SSH | Docker `linuxserver/openssh-server` | `docker-compose.test.yml` |

### 2.2 Celah yang ditemukan

| ID Temuan | Severity | Temuan | Bukti | Ditangani oleh |
|---|---|---|---|---|
| F-01 | 🔴 P0 | **Binary ELF 15,8 MB ter-commit** di repo. Ini melanggar aturan CI sendiri (T-16 "no file over 1MB"), jadi job `repo-hygiene` seharusnya gagal di `main`. | `build/bin/caterm` | ENT-GOV-01 |
| F-02 | 🔴 P0 | **Tidak ada code signing OS.** Hanya ada tanda tangan updater (minisign); belum ada Authenticode (Windows) maupun Developer ID + notarization (macOS). | `.github/workflows/release.yml` hanya memakai `TAURI_SIGNING_*` | ENT-REL-01..03 |
| F-03 | 🟠 P1 | **Updater belum siap enterprise**: hanya satu kanal, tidak bisa dimatikan atau diarahkan ke mirror internal, dan belum ada prosedur rotasi kunci minisign. (Catatan: URL fallback `github.com/cecepazhar/caterm` sudah benar; `cecep-azhar` adalah nama lama yang di-redirect GitHub.) | `tauri.conf.json → plugins.updater` | ENT-REL-09 |
| F-04 | 🟠 P1 | **Dua lockfile frontend** (`package-lock.json` & `pnpm-lock.yaml`). CI memakai npm + Node 24, sedangkan release memakai pnpm + Node 20, dengan fallback `pnpm install --frozen-lockfile \|\| pnpm install`. Akibatnya build rilis tidak reproducible dan bisa berbeda dari yang dites CI. | `ci.yml` vs `release.yml` | ENT-REL-04 |
| F-05 | 🟠 P1 | **Release tidak digate oleh CI.** Workflow release tidak bergantung pada hasil test/clippy. | `release.yml` (tidak ada `needs:`) | ENT-REL-04 |
| F-06 | 🟠 P1 | **Frontend tanpa tes otomatis** (0 file `*.test.*` / `*.spec.*`). | `frontend/` | ENT-QA-01..04 |
| F-07 | 🟠 P1 | **Logging aplikasi praktis tidak ada** (±1 pemakaian `tracing`/`log` di seluruh crate). | `grep tracing:: crates/` | ENT-OBS-01..03 |
| F-08 | 🟠 P1 | **Audit log belum tamper-evident dan tanpa identitas aktor.** Struktur `CommandLog` hanya `id, event_type, timestamp, host_id, details`; tidak ada hash chain, user/device, atau export. | `crates/caterm-core/src/audit.rs:7` | ENT-CMP-01..04 |
| F-09 | 🟠 P1 | **Regex masking secret terlalu sempit.** Tidak menangkap pola seperti `Authorization: Bearer …`, `AKIA…` (AWS), private key PEM, `mysql -pPASSWORD`, atau URL `user:pass@host`. | `audit.rs:17` | ENT-SEC-06 |
| F-10 | 🟠 P1 | **Tidak ada managed policy / konfigurasi terpusat.** | Tidak ada modul policy di `crates/` | ENT-ADM-05..09 |
| F-11 | 🟠 P1 | **Tidak ada `SECURITY.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`.** | root repo | ENT-GOV-03..05 |
| F-12 | 🟡 P2 | **Root repo berantakan**: 18 file `.md` di root, sebagian besar prompt/plan internal (`prompt.md`, `agent-master-prompt.md`, `launch-audit-prompt.md`, `ledger-v2.md`, dst.). | `ls *.md` | ENT-GOV-02 |
| F-13 | 🟡 P2 | **File sisa ter-commit**: `crates/caterm-cli/src/main.rs_draft`, `crates/caterm-core/src/pro.rs.new`, dan `dist-release/`. | path terkait | ENT-GOV-01 |
| F-14 | 🟡 P2 | **±203 pemakaian `.unwrap()`/`.expect()`** di kode produksi, sehingga berpotensi panic dan menutup aplikasi. | `crates/*/src` | ENT-QA-07 |
| F-15 | 🟡 P2 | **19 warning aksesibilitas** dari `svelte-check` (label tanpa control, `div` dengan click tanpa keyboard handler). | `src/routes/+page.svelte`, `src/routes/sftp/+page.svelte`, `+layout.svelte` | ENT-UX-01 |
| F-16 | 🟡 P2 | **Klaim "Production Ready" di `STATUS.md` tanpa bukti tes** untuk banyak fitur (VFS S3/WebDAV/FTP, tunnels, sync). | `STATUS.md` | ENT-QA-05, ENT-DOC-01 |
| F-17 | 🟡 P2 | **Tidak ada dukungan proxy korporat / CA custom** yang terdokumentasi untuk koneksi keluar (updater, AI, sync, license). | `crates/caterm-core/src/ai`, `sync.rs`, `pro.rs` | ENT-ADM-10 |
| F-18 | 🟡 P2 | **WebView2 diunduh saat instal** (`downloadBootstrapper`), sehingga gagal di jaringan air-gapped. | `tauri.conf.json → bundle.windows` | ENT-ADM-02 |
| F-19 | 🟡 P2 | **SSO belum ada**; fitur tim (`teams.rs`, 239 baris) masih lokal, tanpa RBAC. | `crates/caterm-core/src/teams.rs` | ENT-IAM-* |
| F-20 | 🔵 P3 | **`ssh2` (libssh2) dipakai sebagai engine SSH.** Dukungan algoritma modern (mis. `sk-ssh-ed25519`, certificate auth) dan kecepatan patch upstream terbatas. | `Cargo.lock: ssh2, libssh2-sys` | ENT-SEC-10 |

---

## 3. Roadmap & Milestone

| Milestone | Tema | Target durasi | Exit criteria |
|---|---|---|---|
| **M0: Stabilize** | CI hijau, repo bersih, build rilis reproducible | 2–3 hari | F-01, F-04, F-05, F-13 selesai; semua job `ci.yml` hijau di `main` |
| **M1: Foundation** | Governance, tes, logging | 2 minggu | ENT-GOV-*, ENT-QA-01..04, ENT-OBS-01..03, ENT-REL-04..08 selesai |
| **M2: Trusted Distribution** | Signing, MSI, SBOM | 2 minggu (tergantung sertifikat) | E1, E2, E5 terpenuhi |
| **M3: Enterprise Control** | Policy, audit, SIEM, security | 3–4 minggu | E3, E4, E6, E8 terpenuhi |
| **M4: Enterprise Scale** | SSO, RBAC, compliance docs | 4–6 minggu | E9, E10 + ENT-IAM P1 selesai |
| **M5: Certification-ready** | Pentest, SOC 2 / ISO 27001 readiness | berkelanjutan | Laporan pentest bersih, control mapping lengkap |

```
M0 ──► M1 ──► M2 ──► M3 ──► M4 ──► M5
        │      ▲
        └──────┘  (ENT-REL-04 wajib selesai sebelum signing dipasang)
```

---

## 4. Workstream & Task Detail

Format setiap task:

> **ENT-XXX-NN · Judul** · Prioritas · Effort · Pelaksana · Status
> - **Masalah:** kenapa ini perlu
> - **Lingkup:** apa yang dikerjakan
> - **File:** file yang disentuh
> - **Acceptance criteria:** syarat dianggap selesai (harus bisa diverifikasi)
> - **Dependensi:** task yang harus selesai lebih dulu

---

### WS-GOV · Repo Hygiene & Governance

#### ENT-GOV-01 · Hapus binary & file sisa dari repo · P0 · S · 🤖 · ⬜
- **Masalah:** F-01, F-13. Binary di git membuat clone berat, melanggar T-15/T-16, dan menimbulkan pertanyaan "binary ini dibuild dari mana?" saat audit supply chain.
- **Lingkup:**
  1. `git rm --cached build/bin/caterm`, hapus `crates/caterm-cli/src/main.rs_draft`, `crates/caterm-core/src/pro.rs.new`, dan `dist-release/`.
  2. Tambahkan `build/`, `dist-release/`, `*.rs_draft`, `*.new` ke `.gitignore`.
  3. Jalankan step `repo-hygiene` secara lokal dan pastikan T-15/T-16/T-17 OK.
  4. (Opsional, keputusan Owner) bersihkan history dengan `git filter-repo`. Ini merusak clone lama, jadi lakukan hanya jika ukuran repo jadi masalah.
- **File:** `.gitignore`, path di atas.
- **Acceptance criteria:** `git ls-files | xargs stat -c%s | sort -n | tail -1` < 1 MB; job `repo-hygiene` hijau.
- **Dependensi:** –

#### ENT-GOV-02 · Rapikan root repo · P2 · S · 🤖 · ⬜
- **Masalah:** F-12. Dokumen prompt internal di root terlihat tidak profesional dan bisa membocorkan rencana bisnis.
- **Lingkup:**
  1. Root hanya berisi: `README.md`, `LICENSE`, `SECURITY.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `ARCHITECTURE.md`, file konfigurasi build.
  2. Pindahkan `DESIGN.md`, `STATUS.md` ke `docs/`.
  3. Pindahkan dokumen perencanaan (`prd-v2.md`, `task-v2.md`, `ledger-v2.md`, `optimization.md`, `plan-finishing-pro.md`, `CATERM_COMPONENT_GAP.md`, `PRD_AUDIT.md`, `cron.md`, dokumen ini) ke `docs/internal/`.
  4. **Keputusan Owner:** prompt AI (`prompt*.md`, `agent-master-prompt.md`, `launch-audit-prompt.md`, `caterm-execution-prompts-v2-addendum.md`) dipindah ke `docs/internal/prompts/` atau dikeluarkan dari repo publik.
  5. Perbaiki semua link relatif yang rusak akibat pemindahan.
- **Acceptance criteria:** `ls *.md` di root ≤ 7 file; tidak ada link 404 di README (`lychee` atau `markdown-link-check` di CI).
- **Dependensi:** –

#### ENT-GOV-03 · `SECURITY.md` + kebijakan disclosure · P1 · S · 🤝 · ⬜
- **Lingkup:** cara melapor kerentanan (email khusus, mis. `security@fathforce.com`, atau GitHub Private Vulnerability Reporting), target waktu respons (ack ≤ 72 jam, fix critical ≤ 14 hari), versi yang didukung, dan safe-harbor.
- **Owner:** menyediakan alamat email keamanan dan menyetujui SLA.
- **Acceptance criteria:** tab "Security" di GitHub menampilkan policy; Private Vulnerability Reporting aktif.

#### ENT-GOV-04 · `CHANGELOG.md` (Keep a Changelog) · P1 · S · 🤖 · ⬜
- **Lingkup:** buat `CHANGELOG.md` dengan format Keep a Changelog + SemVer; isi mundur minimal sampai v2.1.0 dari `git log`. Tambah job CI yang gagal bila PR mengubah `crates/` atau `frontend/src` tanpa menyentuh `CHANGELOG.md` (bisa di-skip dengan label `skip-changelog`).
- **Acceptance criteria:** release notes otomatis diambil dari section versi terkait.

#### ENT-GOV-05 · `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, template issue/PR · P2 · S · 🤖 · ⬜
- **Lingkup:** panduan setup dev, konvensi commit (Conventional Commits), cara menjalankan tes, `.github/ISSUE_TEMPLATE/{bug,feature}.yml`, `.github/pull_request_template.md`.

#### ENT-GOV-06 · Branch protection & CODEOWNERS · P1 · S · 👤 · ⬜
- **Lingkup:** proteksi `main`: wajib PR, wajib status check (`fmt`, `clippy`, `test`, `frontend`, `repo-hygiene`), larang force-push, aktifkan signed commits (opsional). Tambah `.github/CODEOWNERS`.
- **Owner:** setting di GitHub → Settings → Branches.
- **Acceptance criteria:** push langsung ke `main` ditolak.

#### ENT-GOV-07 · Seragamkan identitas & URL publik · P2 · S · 🤖 · ⬜
- **Masalah:** pemilik repo yang benar adalah `cecepazhar` (dipakai di `Cargo.toml`, README, FUNDING, updater). Nama `cecep-azhar` hanya redirect lama dari GitHub, tapi masih bisa muncul di clone URL, dokumen, atau skrip.
- **Lingkup:** cari `cecep-azhar` di repo, CI, dan dokumentasi, lalu ganti ke `cecepazhar`. Pastikan `repository`/`homepage` di semua `Cargo.toml` dan `frontend/package.json` konsisten.
- **Acceptance criteria:** `git grep cecep-azhar` kosong.

#### ENT-GOV-08 · Selaraskan `STATUS.md` dengan kenyataan · P2 · S · 🤖 · ⬜
- **Masalah:** F-16. Versi di `STATUS.md` masih v2.1.19, dan label "Production Ready" diberikan tanpa bukti.
- **Lingkup:** ganti kolom status menjadi `Stable` / `Beta` / `Experimental`, dengan syarat tegas: **Stable = ada tes otomatis + dipakai ≥ 1 rilis tanpa bug P0**. Tautkan setiap fitur Stable ke file tesnya.

---

### WS-REL · Build, Release & Supply Chain

#### ENT-REL-01 · Code signing Windows (Authenticode) · P0 · M · 🤝 · ⬜
- **Masalah:** F-02. `.exe` tanpa tanda tangan memicu SmartScreen dan diblokir AppLocker/WDAC di banyak perusahaan.
- **Opsi (keputusan Owner):**

  | Opsi | Biaya kira-kira/tahun | Kelebihan | Kekurangan |
  |---|---|---|---|
  | **Azure Trusted Signing** (direkomendasikan) | ± USD 120 | Murah, reputasi SmartScreen cepat, tanpa token fisik, cocok untuk CI | Syarat verifikasi identitas (badan usaha / individu sesuai wilayah yang didukung) |
  | Sertifikat OV (Sectigo/DigiCert) + HSM cloud | ± USD 300–600 | Umum diterima | Reputasi SmartScreen perlu waktu untuk terbangun |
  | Sertifikat EV | ± USD 400–900 | Reputasi SmartScreen langsung | Token hardware, sehingga sulit dipakai di CI |

- **Lingkup:**
  1. Owner membeli/menyiapkan akun signing.
  2. Konfigurasi `bundle.windows.signCommand` di `tauri.conf.json` (atau `certificateThumbprint` + `timestampUrl`).
  3. Simpan kredensial di GitHub Secrets / OIDC federated credential (tanpa file `.pfx` di repo).
  4. Tandatangani **semua** binary: `caterm-app.exe`, `uninstall.exe`, installer NSIS/MSI, dan `caterm-cli.exe`.
  5. Pakai timestamp server (RFC 3161) supaya tanda tangan tetap valid setelah sertifikat kedaluwarsa.
- **Acceptance criteria:** `signtool verify /pa /v` lolos untuk semua file; instal di Windows 11 bersih tidak memunculkan SmartScreen "Unknown publisher".
- **Dependensi:** ENT-REL-04.

#### ENT-REL-02 · Signing + notarization macOS · P0 · M · 🤝 · ⬜
- **Lingkup:** Apple Developer Program (USD 99/tahun, oleh Owner), sertifikat "Developer ID Application", secrets `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_KEY`/`APPLE_API_ISSUER` untuk `notarytool`. Aktifkan hardened runtime dan entitlements minimal (network client, tanpa JIT kecuali perlu). Lakukan stapling tiket ke `.dmg`.
- **Acceptance criteria:** `spctl -a -vvv CATerm.app` menampilkan `accepted, source=Notarized Developer ID`; `xcrun stapler validate` lolos.
- **Dependensi:** ENT-REL-04.

#### ENT-REL-03 · Signing paket Linux & Android · P1 · M · 🤝 · ⬜
- **Lingkup:** tanda tangan GPG untuk `.deb`/`.rpm` (`dpkg-sig` / `rpm --addsign`), repositori APT/YUM bertanda tangan (opsional P2), dan AppImage dengan `--sign`. Android: keystore rilis sesuai `docs/android-signing.md`, disimpan di secrets.
- **Acceptance criteria:** `rpm -K` dan `dpkg-sig --verify` lolos; kunci publik GPG dipublikasikan di website & README.

#### ENT-REL-04 · Build reproducible & release digate CI · P0 · M · 🤖 · ⬜
- **Masalah:** F-04, F-05.
- **Lingkup:**
  1. Pilih **satu** package manager. Rekomendasi: **npm**, karena sudah dipakai CI. Hapus `pnpm-lock.yaml`.
  2. Samakan versi Node di CI dan release (pin `node-version-file: frontend/.nvmrc`).
  3. Hapus fallback `|| pnpm install`; release **wajib** `npm ci` (frozen).
  4. Release dipicu oleh tag `v*`, lalu menjalankan reusable workflow `ci.yml` (`workflow_call`) dan `needs: [ci]` sebelum build/sign.
  5. Pin action ke commit SHA (bukan `@v4`), dan aktifkan Dependabot untuk `github-actions`.
  6. Set `permissions:` minimum per job (`contents: read`, dan `contents: write` hanya untuk upload release).
- **File:** `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `frontend/.nvmrc`, `.github/dependabot.yml`.
- **Acceptance criteria:** tag dengan tes gagal tidak menghasilkan rilis; dua build dari tag yang sama menghasilkan hash frontend bundle identik.

#### ENT-REL-05 · SBOM per rilis (CycloneDX) · P1 · S · 🤖 · ⬜
- **Lingkup:** `cargo cyclonedx` untuk Rust dan `@cyclonedx/cyclonedx-npm` untuk frontend. Gabungkan, lalu lampirkan `caterm-<ver>.cdx.json` ke GitHub Release.
- **Acceptance criteria:** setiap rilis punya SBOM yang lolos validasi `cyclonedx validate`.

#### ENT-REL-06 · Build provenance (SLSA level 2+) · P1 · S · 🤖 · ⬜
- **Lingkup:** `actions/attest-build-provenance` untuk semua artefak; dokumentasikan verifikasi dengan `gh attestation verify`.
- **Acceptance criteria:** setiap artefak rilis punya attestation yang bisa diverifikasi publik.

#### ENT-REL-07 · Checksum bertanda tangan · P1 · S · 🤖 · ⬜
- **Lingkup:** buat `SHA256SUMS` otomatis di CI (bukan file manual di repo seperti `SHA256SUMS.txt` sekarang), tandatangani dengan minisign/GPG, lalu lampirkan ke rilis. Hapus `SHA256SUMS.txt` dari root.
- **Acceptance criteria:** README menjelaskan cara verifikasi; tidak ada checksum statis di repo.

#### ENT-REL-08 · Scan kerentanan artefak · P2 · S · 🤖 · ⬜
- **Lingkup:** job `osv-scanner` / `trivy fs` terhadap lockfile; gagal untuk severity High/Critical tanpa pengecualian tertulis. Pastikan `npm audit --omit=dev` juga ikut dijalankan.
- **Dependensi:** ENT-REL-04.

#### ENT-REL-09 · Perkuat updater untuk enterprise · P1 · M · 🤝 · ⬜
- **Masalah:** F-03.
- **Lingkup:**
  1. Tambahkan kanal update (`stable`, `beta`, `lts`) via `latest-<channel>.json`.
  2. Updater harus **bisa dimatikan oleh policy** (`updates.enabled`, ENT-ADM-06) dan bisa diarahkan ke **server internal perusahaan** (`updates.endpoint`).
  3. Rotasi kunci minisign: dokumentasikan prosedur rotasi dan siapkan kunci cadangan offline.
  4. Tes otomatis bahwa kedua endpoint (`caterm.fathforce.com` dan GitHub Releases) menyajikan `latest.json` yang valid dan tanda tangannya cocok.
- **Acceptance criteria:** tes manual update v(n-1) → v(n) berhasil dari kedua endpoint; dengan endpoint utama dimatikan, fallback tetap berjalan.
- **Dependensi:** ENT-ADM-05 (untuk butir 2).

#### ENT-REL-10 · Kebijakan versi & kanal LTS · P2 · S · 👤 · ⬜
- **Lingkup:** SemVer ketat (breaking change pada data/policy = major), kanal **LTS** dengan dukungan 12 bulan dan hanya menerima security fix, serta jadwal EOL yang dipublikasikan.

---

### WS-QA · Quality & Testing

#### ENT-QA-01 · Setup Vitest + Testing Library untuk frontend · P1 · M · 🤖 · ⬜
- **Lingkup:** `vitest`, `@testing-library/svelte`, `jsdom`; script `npm test`; tambahkan ke job `frontend` di CI. Mock `@tauri-apps/api` (`invoke`, `listen`) di `frontend/src/test/setup.ts`.
- **Acceptance criteria:** `npm test` berjalan di CI; minimal 1 tes contoh lolos.

#### ENT-QA-02 · Tes unit logika frontend kritis · P1 · L · 🤖 · ⬜
- **Target minimal (urut prioritas):**
  1. `TerminalAutocomplete.svelte`: posisi/clamp/flip popup (regresi bug November 2026).
  2. `TerminalPane.svelte`: `updateSuggestions()` (history → snippet → command, dedup, limit 8).
  3. `lib/stores/terminalPrefs.svelte.ts`: batas font size.
  4. `lib/stores/sessionTabs.svelte.ts`: label tab dinamis (Local N / Host N).
  5. `lib/i18n`: semua key di `en.ts` ada di `id.ts` (dan sebaliknya).
- **Acceptance criteria:** coverage `frontend/src/lib` ≥ 60% baris; gagal CI jika turun lebih dari 2 poin.

#### ENT-QA-03 · Tes E2E aplikasi (WebdriverIO + tauri-driver) · P1 · XL → pecah · 🤖 · ⬜
- **Sub-task:**
  - ENT-QA-03a: setup `tauri-driver` di Linux CI (xvfb). Target: membuka app dan melihat layar unlock.
  - ENT-QA-03b: skenario vault, yaitu buat vault → lock → unlock → password salah ditolak.
  - ENT-QA-03c: skenario SSH ke container `docker-compose.test.yml` → ketik `ls` → output muncul.
  - ENT-QA-03d: skenario SFTP, yaitu upload & download file 10 MB dengan checksum sama.
- **Acceptance criteria:** suite E2E berjalan di setiap PR ke `main` dalam < 15 menit.

#### ENT-QA-04 · Coverage Rust + gate · P1 · M · 🤖 · ⬜
- **Lingkup:** `cargo llvm-cov` di CI, laporan diunggah sebagai artefak (opsional Codecov). Target awal: `caterm-core` ≥ 60%, dengan modul kripto (`vault.rs`, `keys.rs`, `secret.rs`, `totp.rs`) ≥ 85%.
- **Acceptance criteria:** CI gagal jika coverage modul kripto < 85%.

#### ENT-QA-05 · Tes integrasi per driver VFS & tunnel · P1 · L · 🤖 · ⬜
- **Masalah:** F-16. Driver FTP/WebDAV/S3 diklaim siap tanpa tes.
- **Lingkup:** tambahkan service ke `docker-compose.test.yml`: `delfer/alpine-ftp-server` (FTP/FTPS), `bytemark/webdav`, `minio/minio` (S3). Tes list/upload/download/rename/delete/permission error untuk setiap driver. Tunnel: tes local forward ke service HTTP di container.
- **File:** `crates/caterm-core/tests/vfs_*.rs`, `docker-compose.test.yml`, job CI baru `integration`.
- **Acceptance criteria:** setiap driver yang berlabel "Stable" punya tes integrasi hijau.

#### ENT-QA-06 · Fuzzing parser & input tak tepercaya · P2 · M · 🤖 · ⬜
- **Lingkup:** `cargo fuzz` untuk parser backup `.catb`, import `known_hosts`, parsing URL host, `mask_secrets`, dan import key OpenSSH/PEM. Jalankan 10 menit per target secara nightly.
- **Acceptance criteria:** tidak ada crash/panic baru selama 7 hari nightly.

#### ENT-QA-07 · Kurangi `unwrap()`/`expect()` di jalur produksi · P2 · L · 🤖 · ⬜
- **Masalah:** F-14 (±203 kemunculan).
- **Lingkup:** aktifkan `clippy::unwrap_used` dan `clippy::expect_used` sebagai `warn` di crate produksi (bukan di tes), lalu ganti dengan `?` + `CatermError`. Pengecualian harus diberi `#[allow]` + komentar alasan.
- **Acceptance criteria:** 0 `unwrap()` di `crates/*/src` tanpa `#[allow]` beralasan; `clippy -D warnings` hijau.

#### ENT-QA-08 · Matriks tes platform manual (release checklist) · P1 · S · 🤖 · ⬜
- **Lingkup:** dokumen `docs/release-checklist.md` berisi skenario smoke test per OS (Windows 10 22H2, Windows 11, macOS 14/15 Intel & ARM, Ubuntu 22.04/24.04, Fedora 41, Android 12+), termasuk skala layar 100/125/150%.
- **Acceptance criteria:** setiap rilis menyertakan checklist yang terisi.

#### ENT-QA-09 · Uji performa & regresi memori · P2 · M · 🤖 · ⬜
- **Lingkup:** perluas `scripts/bench.sh` + `budget-check.sh` dengan metrik terukur: RSS idle < 180 MB (sesuai klaim README), throughput `cat` 100 MB via SSH, dan waktu startup cold < 2 detik. Simpan hasilnya sebagai artefak dan bandingkan dengan `main`.

---

### WS-OBS · Observability & Supportability

#### ENT-OBS-01 · Structured logging dengan `tracing` · P1 · M · 🤖 · ⬜
- **Masalah:** F-07.
- **Lingkup:**
  1. `tracing` + `tracing-subscriber` + `tracing-appender` (rolling harian, simpan 7 file, maks 10 MB per file).
  2. Lokasi log: `<data_dir>/logs/caterm.YYYY-MM-DD.log` (lihat `crates/caterm-core/src/paths.rs`).
  3. Format JSON di file; format pretty di stderr saat dev.
  4. Level default `info`, bisa diubah lewat Settings, env `CATERM_LOG`, atau policy.
  5. Instrumentasi minimal: lifecycle app, unlock vault (tanpa data sensitif), connect/disconnect SSH (host di-hash), error di semua command Tauri, dan transfer SFTP (ukuran, durasi, hasil).
  6. **Larangan:** jangan pernah me-log password, private key, isi terminal, atau token. Tambahkan tes yang memastikan `PiiScrubber` diterapkan sebagai `Layer` tracing.
- **Acceptance criteria:** setiap command Tauri yang gagal menghasilkan 1 baris log `error` dengan kode error dari `docs/error-codes.md`.

#### ENT-OBS-02 · Bundle diagnostik satu-klik · P1 · M · 🤖 · ⬜
- **Lingkup:** menu **Settings → Help → Export Diagnostics** menghasilkan `.zip` berisi log 7 hari (sudah di-scrub), versi app/OS/WebView, daftar fitur aktif, policy efektif (ENT-ADM-06), dan hasil self-check (DB terbuka, izin folder). **Tanpa** data vault.
- **Acceptance criteria:** pengguna bisa mengirim satu file ke support; isi zip lolos `PiiScrubber` (diverifikasi dengan tes).

#### ENT-OBS-03 · Konsistensi kode error · P2 · S · 🤖 · ⬜
- **Lingkup:** pastikan setiap `CatermError` punya kode stabil (sudah ada tes `error_codes_unique.rs`) dan pesan UI menampilkan kode tersebut ("Error CT-SSH-012: …") dengan tautan ke dokumentasi.

#### ENT-OBS-04 · Kebijakan crash report untuk enterprise · P1 · S · 🤖 · ⬜
- **Lingkup:** crash report (`crash.rs`) default **opt-in**; bisa dipaksa off atau diarahkan ke endpoint internal via policy. Dokumentasikan isi laporan dan lokasi penyimpanannya.
- **Dependensi:** ENT-ADM-05.

#### ENT-OBS-05 · Health/telemetry opsional ke server internal · P3 · L · 🤖 · ⬜
- **Lingkup:** untuk pelanggan besar: endpoint OpenTelemetry (OTLP) opsional yang hanya aktif via policy, berisi metrik anonim (versi, crash rate, durasi koneksi). Default mati.

---

### WS-SEC · Security Hardening

#### ENT-SEC-01 · Threat model formal · P1 · M · 🤝 · ⬜
- **Lingkup:** `docs/security/threat-model.md` dengan metode STRIDE, untuk aset (vault, private key, TOTP seed, API key AI, sesi SSH), trust boundary (WebView ↔ Rust core, app ↔ server lisensi/sync/AI, app ↔ host SSH), dan daftar ancaman + mitigasi + task terkait.
- **Acceptance criteria:** setiap ancaman High punya mitigasi yang sudah diimplementasikan atau task terbuka.

#### ENT-SEC-02 · Pentest eksternal · P1 · L · 👤 · ⬜
- **Lingkup:** Owner mengontrak pihak ketiga untuk pentest aplikasi desktop + API backend (`gcc.fathforce.com`, `caterm.fathforce.com`, `api.caterm.com`). Temuan dimasukkan sebagai task `ENT-SEC-PT-*`.
- **Acceptance criteria:** laporan final tanpa temuan High/Critical terbuka; ringkasan eksekutif bisa dibagikan ke calon pelanggan di bawah NDA.
- **Dependensi:** M3 selesai.

#### ENT-SEC-03 · Proses respons CVE dependency · P1 · S · 🤖 · ⬜
- **Lingkup:** Dependabot untuk `cargo`, `npm`, dan `github-actions`; `cargo-audit` terjadwal harian (bukan hanya saat push); SLA patch: Critical ≤ 7 hari, High ≤ 30 hari. Dokumentasikan di `SECURITY.md`.

#### ENT-SEC-04 · Audit Tauri capabilities (least privilege) · P1 · M · 🤖 · ⬜
- **Lingkup:** review `capabilities/default.json` dan `desktop.json`: hapus permission yang tidak dipakai frontend, pisahkan capability per window, pastikan command berbahaya (`delete_*`, `trigger_task_run_now`, akses filesystem lokal) memvalidasi input di sisi Rust (path traversal, ukuran, tipe).
- **Acceptance criteria:** tabel di `docs/security/ipc-surface.md` mencantumkan setiap command beserta validasinya; tes Rust untuk path traversal (`../`, symlink) di `local_fs.rs` dan `sftp.rs`.

#### ENT-SEC-05 · Review CSP & WebView hardening · P2 · S · 🤖 · ⬜
- **Lingkup:** evaluasi penghapusan `style-src 'unsafe-inline'` (pakai hash/nonce bila memungkinkan), `freezePrototype: true`, nonaktifkan devtools di build rilis, dan pastikan `dangerousDisableAssetCspModification` tidak dipakai.

#### ENT-SEC-06 · Perluas masking secret · P1 · M · 🤖 · ⬜
- **Masalah:** F-09.
- **Lingkup:** satukan `audit::mask_secrets` dan `crash::PiiScrubber` menjadi satu modul `redact.rs`. Pola minimal:
  - `Authorization: Bearer <token>`, `Basic <b64>`
  - AWS access key `AKIA[0-9A-Z]{16}` dan secret 40 karakter setelah `aws_secret_access_key`
  - GitHub token `gh[pousr]_[A-Za-z0-9]{36,}`, Slack `xox[baprs]-…`
  - Blok `-----BEGIN … PRIVATE KEY-----` … `-----END … -----`
  - URL berkredensial `scheme://user:pass@host`
  - Argumen CLI `-p<pass>` (mysql), `--password=…`, `PGPASSWORD=…`, `sshpass -p …`
  - JWT `eyJ…\.eyJ…\.…`
- **Acceptance criteria:** tabel tes ≥ 30 kasus positif dan ≥ 10 kasus negatif (tidak over-mask); fuzz target di ENT-QA-06.

#### ENT-SEC-07 · Kunci di OS keychain (opsional) & zeroize memori · P2 · L · 🤖 · ⬜
- **Lingkup:** opsi menyimpan kunci pembuka vault di Windows Credential Manager / macOS Keychain / Secret Service (crate `keyring`) untuk mode "unlock dengan login OS", **dimatikan by default** dan bisa dikunci oleh policy. Pakai `zeroize` / `secrecy` untuk password dan private key di memori Rust.
- **Acceptance criteria:** tes memastikan buffer password di-zeroize setelah dipakai (unit test pada tipe wrapper).

#### ENT-SEC-08 · Auto-lock & proteksi sesi · P1 · M · 🤖 · ⬜
- **Lingkup:** auto-lock vault setelah idle N menit (default 15, bisa dikunci policy), lock saat OS sleep/screen lock, clipboard auto-clear setelah 30 detik untuk password/TOTP yang disalin, dan konfirmasi sebelum paste multi-baris ke terminal (sudah ada `paste_sentinel.rs` di Pro, sehingga perlu diputuskan apakah dipindah ke core).
- **Acceptance criteria:** tes E2E idle-lock (ENT-QA-03).

#### ENT-SEC-09 · Mode strict host key · P1 · S · 🤖 · ⬜
- **Lingkup:** policy `ssh.hostKeyPolicy = strict | tofu | ask`. Dalam mode `strict`, koneksi ke host yang belum dikenal ditolak; admin bisa mendistribusikan `known_hosts` terpusat. Perubahan host key **selalu** diblokir dengan peringatan merah.
- **File:** `crates/caterm-core/src/ssh.rs` (sekitar `:402`).

#### ENT-SEC-10 · Evaluasi engine SSH (`ssh2` → `russh`) · P3 · XL · 🤖 · ⬜
- **Masalah:** F-20.
- **Lingkup:** spike 3 hari untuk membandingkan `russh` (pure Rust, async native Tokio) dengan `ssh2` dari sisi algoritma (ed25519-sk, certificate auth, `rsa-sha2-*`), performa, dan agent forwarding. Hasilnya berupa ADR (`docs/adr/0001-ssh-engine.md`). Migrasi hanya jika ADR menyetujui.

#### ENT-SEC-11 · Dukungan SSH certificate & hardware key · P2 · L · 🤖 · ⬜
- **Lingkup:** OpenSSH user certificate (`*-cert.pub`) untuk perusahaan yang memakai SSH CA (Vault, Teleport, Smallstep), FIDO2 `sk-ssh-ed25519` (YubiKey), dan agent forwarding ke `ssh-agent`/Pageant/1Password agent.
- **Dependensi:** ENT-SEC-10 (kemungkinan butuh `russh`).

---

### WS-ADM · Enterprise Administration & Deployment

#### ENT-ADM-01 · Installer MSI (WiX) · P1 · M · 🤖 · ⬜
- **Lingkup:** aktifkan target `msi` di `tauri.conf.json` di samping NSIS. MSI harus mendukung instal **per-machine** (`ALLUSERS=1`), upgrade code stabil, dan properti publik untuk policy awal (mis. `CATERM_UPDATE_DISABLED=1`).
- **Acceptance criteria:** `msiexec /i CATerm.msi /qn ALLUSERS=1` berhasil tanpa UI; uninstall bersih via `msiexec /x`.

#### ENT-ADM-02 · Instal offline (air-gapped) · P1 · S · 🤖 · ⬜
- **Masalah:** F-18.
- **Lingkup:** varian installer dengan `webviewInstallMode: offlineInstaller` atau `fixedRuntime` untuk jaringan tertutup; dokumentasikan perbedaan ukurannya.
- **Acceptance criteria:** instal sukses di VM Windows tanpa internet.

#### ENT-ADM-03 · Silent install/uninstall terdokumentasi · P1 · S · 🤖 · ⬜
- **Lingkup:** `docs/admin/deployment.md` berisi perintah untuk NSIS (`/S`, `/D=`), MSI (`/qn`), `.pkg` macOS (`installer -pkg … -target /`), serta `.deb`/`.rpm`, plus exit code dan lokasi log instalasi.

#### ENT-ADM-04 · Paket untuk tool deployment · P2 · M · 🤖 · ⬜
- **Lingkup:** manifest `winget` (`microsoft/winget-pkgs`), panduan Intune Win32 app (`.intunewin`), contoh Jamf/Kandji, Chocolatey (opsional), serta Flatpak/Snap (folder `flatpak/` & `snap/` sudah ada, sehingga perlu diverifikasi masih bisa dibuild).

#### ENT-ADM-05 · Engine managed policy · P1 · L · 🤖 · ⬜
- **Masalah:** F-10. Ini fondasi semua kontrol admin.
- **Lingkup:**
  1. Modul baru `crates/caterm-core/src/policy.rs` dengan struct `Policy` (serde, semua field `Option`).
  2. Sumber policy, dari prioritas tertinggi ke terendah:
     - Windows: `HKLM\SOFTWARE\Policies\FathForce\CATerm`, lalu `HKCU\SOFTWARE\Policies\FathForce\CATerm`
     - macOS: managed preferences `/Library/Managed Preferences/com.fathforce.caterm.plist` (via MDM configuration profile)
     - Linux: `/etc/caterm/policy.json`
     - Semua OS: env `CATERM_POLICY_FILE` (khusus tes/CI)
  3. Nilai policy **mengalahkan** `prefs.rs`; setting yang dikunci ditandai `locked: true` ke frontend.
  4. Policy dibaca saat startup dan di-reload saat berubah (watch file / registry notify).
  5. Policy yang tidak valid di-log sebagai `error`, lalu app memakai nilai paling aman (fail-closed untuk field keamanan).
- **Acceptance criteria:** tes unit untuk precedence & parsing; tes integrasi dengan `CATERM_POLICY_FILE`.

#### ENT-ADM-06 · Daftar policy minimal v1 · P1 · M · 🤖 · ⬜
- **Lingkup:** implementasikan key berikut (nama final dibakukan di `docs/admin/policy-reference.md`):

  | Key | Tipe | Default | Efek |
  |---|---|---|---|
  | `updates.enabled` | bool | `true` | Matikan auto-update |
  | `updates.endpoint` | string | – | Mirror update internal |
  | `updates.channel` | enum | `stable` | `stable`/`beta`/`lts` |
  | `ai.enabled` | bool | `true` | Matikan seluruh fitur AI |
  | `ai.allowedProviders` | string[] | semua | Whitelist provider (mis. hanya `ollama`) |
  | `ai.customEndpoint` | string | – | Gateway AI internal |
  | `sync.enabled` | bool | `true` | Matikan cloud sync |
  | `telemetry.crashReports` | enum | `ask` | `off`/`ask`/`on` |
  | `ssh.hostKeyPolicy` | enum | `tofu` | `strict`/`tofu`/`ask` (ENT-SEC-09) |
  | `ssh.knownHostsFile` | path | – | known_hosts terpusat |
  | `ssh.allowedHosts` | glob[] | semua | Batasi tujuan koneksi |
  | `ssh.allowPasswordAuth` | bool | `true` | Paksa key-based auth |
  | `vault.autoLockMinutes` | int | `15` | Auto-lock (ENT-SEC-08) |
  | `vault.minPasswordLength` | int | `12` | Kekuatan master password |
  | `vault.allowOsKeychain` | bool | `false` | ENT-SEC-07 |
  | `sftp.allowedProtocols` | enum[] | semua | Mis. larang FTP non-TLS |
  | `network.proxy` | string | sistem | ENT-ADM-10 |
  | `network.caBundle` | path | – | CA korporat |
  | `audit.siemTarget` | object | – | ENT-CMP-03 |
  | `audit.retentionDays` | int | `90` | ENT-CMP-02 |
  | `features.disabled` | string[] | `[]` | Matikan modul: `port-scanner`, `speed-test`, `tunnels`, `scheduler`, … |
  | `license.key` | string | – | Pre-provision lisensi (ENT-IAM-05) |

- **Acceptance criteria:** setiap key punya tes, dokumentasi, dan indikator 🔒 di UI Settings.
- **Dependensi:** ENT-ADM-05.

#### ENT-ADM-07 · UI untuk setting yang dikunci · P1 · M · 🤖 · ⬜
- **Lingkup:** komponen Settings menampilkan ikon gembok + tooltip "Dikelola oleh organisasi Anda", kontrol menjadi disabled, dan menu yang dimatikan oleh `features.disabled` disembunyikan dari sidebar. Tambah halaman **Settings → About → Managed Policy** (read-only) yang menampilkan policy efektif.
- **Dependensi:** ENT-ADM-05.

#### ENT-ADM-08 · Template ADMX/ADML (Windows GPO) · P2 · M · 🤖 · ⬜
- **Lingkup:** `deploy/windows/admx/CATerm.admx` + `en-US/CATerm.adml` + `id-ID/CATerm.adml` yang memetakan semua key ENT-ADM-06 ke registry.
- **Acceptance criteria:** template bisa diimpor ke Group Policy Management Editor tanpa error.

#### ENT-ADM-09 · Configuration profile macOS (.mobileconfig) · P2 · S · 🤖 · ⬜
- **Lingkup:** contoh `deploy/macos/com.fathforce.caterm.mobileconfig` + JSON schema untuk Jamf "Application & Custom Settings".

#### ENT-ADM-10 · Dukungan proxy korporat & CA custom · P1 · M · 🤖 · ⬜
- **Masalah:** F-17.
- **Lingkup:** semua HTTP client (AI, updater, sync, license, speed test) menghormati proxy sistem / `HTTPS_PROXY` / policy `network.proxy`, termasuk autentikasi proxy dasar. Trust store menggabungkan root CA OS (mis. `rustls-native-certs`) + `network.caBundle`, sehingga app berjalan di balik proxy TLS-inspecting. Untuk SSH: dukung `ProxyJump` (sudah ada `jump_proxy.rs` di Pro), SOCKS5, dan HTTP CONNECT.
- **Acceptance criteria:** tes integrasi dengan `mitmproxy` di docker memakai CA custom.

#### ENT-ADM-11 · Mode portable & lokasi data yang bisa diatur · P2 · S · 🤖 · ⬜
- **Lingkup:** policy/flag `--data-dir`, deteksi mode portable (file `portable.flag` di samping exe), dan dukungan roaming profile Windows. Dokumentasikan lokasi data per OS (`paths.rs`).

#### ENT-ADM-12 · Import/export konfigurasi massal · P2 · M · 🤖 · ⬜
- **Lingkup:** import host dari `~/.ssh/config`, CSV, PuTTY (registry), MobaXterm, dan Termius export. Export daftar host (tanpa secret) ke JSON untuk distribusi tim.

---

### WS-IAM · Identity, Teams & Licensing

#### ENT-IAM-01 · Keputusan arsitektur tim (ADR) · P1 · M · 🤝 · ⬜
- **Masalah:** `teams.rs` saat ini menyimpan data tim secara lokal. Fitur tim enterprise butuh backend.
- **Lingkup:** ADR `docs/adr/0002-team-backend.md` yang memutuskan model zero-knowledge untuk vault bersama (envelope encryption per anggota; sudah ada fondasi di `sync.rs`, `vault_sync.rs`), opsi self-hosted vs SaaS, dan batas data apa yang boleh dilihat server.
- **Owner:** memutuskan model bisnis (SaaS saja / self-hosted / keduanya).

#### ENT-IAM-02 · SSO via OIDC · P1 · XL → pecah · 🤖 · ⬜
- **Sub-task:**
  - ENT-IAM-02a: login OIDC dengan Authorization Code + PKCE lewat browser sistem (loopback redirect `http://127.0.0.1:<port>`), tanpa WebView embedded.
  - ENT-IAM-02b: simpan refresh token di OS keychain; access token hanya di memori.
  - ENT-IAM-02c: konfigurasi issuer/client ID via policy (`identity.oidc.issuer`, `identity.oidc.clientId`).
  - ENT-IAM-02d: uji dengan Microsoft Entra ID, Google Workspace, Okta, dan Keycloak (docker untuk CI).
- **Acceptance criteria:** login SSO berhasil di 4 IdP; logout mencabut token.
- **Dependensi:** ENT-IAM-01, ENT-ADM-05.

#### ENT-IAM-03 · SAML 2.0 (via backend) · P3 · L · 🤖 · ⬜
- **Lingkup:** SAML ditangani backend (bukan desktop) yang lalu menerbitkan token ke app. Hanya dikerjakan jika ada permintaan pelanggan.

#### ENT-IAM-04 · RBAC tim · P2 · L · 🤖 · ⬜
- **Lingkup:** peran `Owner`, `Admin`, `Member`, `Read-only`; izin per grup host (lihat, connect, edit, lihat secret); dan log setiap perubahan peran ke audit.
- **Dependensi:** ENT-IAM-01.

#### ENT-IAM-05 · Lisensi volume & provisioning · P1 · M · 🤝 · ⬜
- **Lingkup:** lisensi organisasi (N seat) yang bisa dipasang via policy `license.key` atau file, tanpa aktivasi per pengguna. Aktivasi offline sudah ada (`license_local.rs`), jadi tambahkan format kunci organisasi, manajemen seat, dan grace period 30 hari saat server lisensi tidak terjangkau.
- **Owner:** model harga & seat.

#### ENT-IAM-06 · SCIM provisioning · P3 · L · 🤖 · ⬜
- **Lingkup:** endpoint SCIM 2.0 di backend untuk sinkron pengguna/grup dari IdP. Hanya untuk tier Enterprise tertinggi.

---

### WS-CMP · Compliance, Audit & Privacy

#### ENT-CMP-01 · Audit log tamper-evident · P1 · M · 🤖 · ⬜
- **Masalah:** F-08.
- **Lingkup:**
  1. Perluas skema `command_logs`: `actor` (user lokal/SSO), `device_id`, `app_version`, `severity`, `outcome` (`success`/`failure`), `prev_hash`, `hash`.
  2. `hash = SHA-256(prev_hash ‖ canonical_json(entry))` untuk membentuk hash chain.
  3. Tambah fungsi `verify_chain()` dan tampilkan statusnya di halaman `command-logs` ("✅ Integritas log terverifikasi" / "⚠️ Log telah diubah").
  4. Migrasi DB untuk data lama: entri lama diberi `prev_hash = "legacy"`.
  5. Evaluasi ulang `restore_log()` (`audit.rs:103`): mengembalikan log yang dihapus harus tetap konsisten dengan chain.
- **Acceptance criteria:** tes yang mengubah satu baris lewat SQL langsung lalu memastikan `verify_chain()` gagal.

#### ENT-CMP-02 · Retensi & rotasi audit log · P1 · S · 🤖 · ⬜
- **Lingkup:** policy `audit.retentionDays`, purge harian, dan setiap purge dicatat sebagai event audit.

#### ENT-CMP-03 · Export ke SIEM · P1 · L · 🤖 · ⬜
- **Lingkup:**
  - Format: JSON Lines (default), CEF, dan syslog RFC 5424.
  - Transport: file lokal (untuk diambil agen Splunk/Elastic/Wazuh), syslog TCP+TLS, dan HTTPS webhook (Splunk HEC / Elastic `_bulk`).
  - Buffer lokal + retry eksponensial saat target tidak tersedia; event tidak boleh hilang (at-least-once).
  - Konfigurasi hanya via policy (`audit.siemTarget`).
- **Acceptance criteria:** tes integrasi dengan `rsyslog` di docker dan mock HEC.
- **Dependensi:** ENT-CMP-01, ENT-ADM-05.

#### ENT-CMP-04 · Katalog event audit · P1 · S · 🤖 · ⬜
- **Lingkup:** `docs/admin/audit-events.md` berisi daftar event dengan ID stabil: `vault.unlock`, `vault.unlock_failed`, `vault.export`, `host.create/update/delete`, `ssh.connect/disconnect/hostkey_mismatch`, `sftp.upload/download/delete`, `tunnel.start/stop`, `secret.reveal`, `secret.copy`, `policy.applied`, `ai.request` (metadata saja), `license.activate`.
- **Acceptance criteria:** tes memastikan setiap event di katalog benar-benar dipancarkan oleh kode.

#### ENT-CMP-05 · Session recording (opsional, Pro) · P3 · L · 🤖 · ⬜
- **Lingkup:** rekam sesi terminal dalam format asciicast v2, terenkripsi, hanya bila diaktifkan policy dan dengan banner pemberitahuan ke pengguna. Perlu ditinjau bersama `caterm-pro/src/session_audit.rs` yang sudah ada.

#### ENT-CMP-06 · Dokumen privasi & alur data · P1 · M · 👤 · ⬜
- **Lingkup:** Kebijakan Privasi, diagram alur data (data apa ke server mana: `gcc.fathforce.com`, `caterm.fathforce.com`, `api.caterm.com`, provider AI, `speed.cloudflare.com`), daftar sub-processor, serta kepatuhan **UU PDP (UU 27/2022)** dan GDPR. Sediakan template **DPA** untuk pelanggan.
- **Owner:** review legal.

#### ENT-CMP-07 · Mapping kontrol SOC 2 / ISO 27001 · P2 · M · 🤝 · ⬜
- **Lingkup:** `docs/compliance/control-mapping.md` yang memetakan kontrol (A.5–A.8 ISO 27001:2022, SOC 2 CC6/CC7/CC8) ke fitur/proses CATerm dan buktinya (link ke CI, dokumen, tes). Ini mempercepat pengisian kuesioner keamanan pelanggan.

#### ENT-CMP-08 · Kontrol data AI · P1 · M · 🤖 · ⬜
- **Lingkup:** indikator jelas di UI kapan data dikirim ke AI dan ke provider mana; preview payload setelah di-scrub sebelum dikirim (opsional, bisa dipaksa via policy); log metadata request AI ke audit (provider, ukuran, tanpa isi); serta dukungan gateway AI internal (`ai.customEndpoint`).
- **Dependensi:** ENT-SEC-06, ENT-ADM-06.

---

### WS-UX · Accessibility, i18n & UX Consistency

#### ENT-UX-01 · Nol warning aksesibilitas · P2 · M · 🤖 · ⬜
- **Masalah:** F-15 (19 warning).
- **Lingkup:** perbaiki semua warning `a11y_*` dari `svelte-check`, lalu jadikan warning a11y sebagai error di CI (`svelte-check --fail-on-warnings` atau filter khusus a11y).
- **Acceptance criteria:** `npm run check` tanpa warning a11y.

#### ENT-UX-02 · Navigasi keyboard penuh & screen reader · P2 · L · 🤖 · ⬜
- **Lingkup:** semua aksi bisa dijangkau dengan keyboard; fokus terlihat; ARIA role pada sidebar, tab, dan modal; dukungan screen reader untuk xterm (`screenReaderMode` opsi xterm.js). Audit dengan axe-core di tes E2E. Target: **WCAG 2.1 AA** (sering diminta di tender pemerintah/BUMN).

#### ENT-UX-03 · Kelengkapan i18n · P2 · S · 🤖 · ⬜
- **Lingkup:** tes otomatis yang memastikan key `en.ts` == key `id.ts` (bagian dari ENT-QA-02), tidak ada string hardcode di komponen (lint), serta format tanggal/angka mengikuti locale.

#### ENT-UX-04 · Dukungan high-contrast & skala DPI · P3 · S · 🤖 · ⬜
- **Lingkup:** tema high-contrast; uji di skala 100–200% (lihat ENT-QA-08).

---

### WS-DOC · Documentation & Support

#### ENT-DOC-01 · Admin Guide · P1 · L · 🤖 · ⬜
- **Lingkup:** `docs/admin/` berisi `deployment.md` (ENT-ADM-03), `policy-reference.md` (ENT-ADM-06), `audit-events.md` (ENT-CMP-04), `network-requirements.md` (domain & port yang harus di-whitelist firewall), `data-locations.md`, `backup-restore.md`, `troubleshooting.md`, dan `upgrade.md`.

#### ENT-DOC-02 · Security Whitepaper · P1 · M · 🤝 · ⬜
- **Lingkup:** ringkasan 6–10 halaman tentang arsitektur zero-knowledge, kriptografi (algoritma, parameter Argon2id, manajemen kunci), model ancaman, proses SDLC aman, signing & supply chain, respons insiden, dan hasil pentest. Dipublikasikan sebagai PDF di website.
- **Dependensi:** ENT-SEC-01, ENT-REL-01..07.

#### ENT-DOC-03 · Jawaban kuesioner keamanan standar · P2 · M · 🤝 · ⬜
- **Lingkup:** jawaban siap pakai untuk **CAIQ Lite** (CSA) dan **SIG Lite**. Ini menghemat waktu setiap kali ada calon pelanggan baru.

#### ENT-DOC-04 · Dokumentasi pengguna · P2 · L · 🤖 · ⬜
- **Lingkup:** situs docs (mis. VitePress / Starlight) dari folder `docs/`, dengan panduan per fitur, FAQ, dan shortcut keyboard, dalam dua bahasa (ID/EN).

#### ENT-DOC-05 · Model support & SLA · P1 · S · 👤 · ⬜
- **Lingkup:** tier support (Community / Business / Enterprise), kanal (email, portal tiket), jam layanan, dan target respons (mis. Enterprise P1 ≤ 4 jam kerja).

#### ENT-DOC-06 · Kebijakan siklus hidup produk · P2 · S · 👤 · ⬜
- **Lingkup:** halaman publik berisi versi yang didukung, tanggal EOL, kebijakan LTS (ENT-REL-10), dan dukungan OS minimum.

---

## 5. Urutan Eksekusi & Dependensi

### 5.1 Urutan yang direkomendasikan

| Urutan | Task | Alasan |
|---|---|---|
| 1 | ENT-GOV-01 | CI harus hijau dulu |
| 2 | ENT-REL-04 | Build rilis harus sama dengan yang dites CI; prasyarat signing & SBOM |
| 3 | ENT-GOV-07 | Seragamkan identitas sebelum dokumen publik dibuat |
| 4 | ENT-GOV-02..05, ENT-GOV-08 | Murah, langsung menaikkan kesan profesional |
| 5 | ENT-QA-01, ENT-QA-02 | Jaring pengaman sebelum refactor besar |
| 6 | ENT-OBS-01, ENT-OBS-02 | Dibutuhkan untuk debugging semua fitur berikutnya |
| 7 | ENT-REL-05..08 | Supply chain |
| 8 | ENT-REL-01..03 | Tergantung sertifikat dari Owner, jadi **mulai proses pembelian sejak hari pertama** |
| 9 | ENT-ADM-05 → 06 → 07 | Fondasi kontrol admin |
| 10 | ENT-SEC-06, ENT-SEC-08, ENT-SEC-09, ENT-ADM-10 | Hardening yang memakai policy |
| 11 | ENT-CMP-01 → 02 → 03 → 04 | Audit & SIEM |
| 12 | ENT-ADM-01..04, 08, 09 | Paket deployment |
| 13 | ENT-QA-03..05, ENT-QA-07 | Tes E2E & integrasi, kurangi unwrap |
| 14 | ENT-SEC-01, ENT-SEC-04, ENT-SEC-05 | Threat model & IPC review sebelum pentest |
| 15 | ENT-IAM-01 → 02 → 05 | SSO & lisensi organisasi |
| 16 | ENT-DOC-01..03 | Dokumen penjualan enterprise |
| 17 | ENT-SEC-02 | Pentest setelah fitur stabil |

### 5.2 Graf dependensi utama

```
ENT-GOV-01 ─► ENT-REL-04 ─┬─► ENT-REL-01/02/03 (signing)
                          ├─► ENT-REL-05/06/07 (SBOM, provenance, checksum)
                          └─► ENT-REL-08

ENT-ADM-05 (policy engine) ─┬─► ENT-ADM-06 ─► ENT-ADM-07 / 08 / 09
                            ├─► ENT-SEC-08, ENT-SEC-09, ENT-ADM-10
                            ├─► ENT-OBS-04, ENT-CMP-08
                            ├─► ENT-CMP-03 ◄── ENT-CMP-01
                            └─► ENT-IAM-02 ◄── ENT-IAM-01

ENT-QA-01 ─► ENT-QA-02 ─► ENT-UX-03
ENT-QA-03 ◄── ENT-SEC-08 (tes idle-lock)
ENT-SEC-06 ─► ENT-QA-06 (fuzz), ENT-CMP-08
ENT-SEC-10 ─► ENT-SEC-11
ENT-SEC-01 + M3 ─► ENT-SEC-02 (pentest) ─► ENT-DOC-02
```

### 5.3 Tindakan Owner yang harus dimulai segera (lead time panjang)

| # | Tindakan | Untuk task | Lead time |
|---|---|---|---|
| 1 | Daftar Azure Trusted Signing / beli sertifikat OV | ENT-REL-01 | 1–3 minggu (verifikasi identitas) |
| 2 | Daftar Apple Developer Program | ENT-REL-02 | 2–14 hari |
| 3 | Siapkan email `security@…` | ENT-GOV-03 | 1 hari |
| 4 | Putuskan nasib prompt AI di repo publik | ENT-GOV-02 | – |
| 5 | Putuskan model tim: SaaS / self-hosted | ENT-IAM-01 | – |
| 6 | Cari vendor pentest & anggarkan biayanya | ENT-SEC-02 | 4–8 minggu |
| 7 | Review legal Kebijakan Privasi & DPA | ENT-CMP-06 | 2–4 minggu |

---

## 6. Risiko & Mitigasi

| Risiko | Dampak | Kemungkinan | Mitigasi |
|---|---|---|---|
| Verifikasi identitas sertifikat signing tertunda | M2 mundur | Sedang | Mulai proses di minggu pertama; kerjakan M3 paralel |
| Migrasi skema audit log merusak data lama | Kehilangan log | Rendah | Migrasi idempoten + backup otomatis sebelum migrasi + tes migrasi dari DB v2.1.41 |
| Policy engine salah membaca nilai → fitur keamanan mati | Celah keamanan | Sedang | Fail-closed untuk field keamanan; tes precedence; tampilkan policy efektif di UI |
| Hapus `pnpm-lock.yaml` mengubah versi dependensi rilis | Bug regresi | Sedang | Bandingkan `npm ls` vs `pnpm ls` sebelum menghapus; rilis beta dulu |
| Migrasi `ssh2` → `russh` memicu regresi koneksi | Fitur inti rusak | Tinggi | Hanya lewat ADR + feature flag + tes E2E SSH (ENT-QA-03c) sebelum default diganti |
| Scope creep fitur IAM/SaaS | Roadmap molor | Tinggi | ENT-IAM P2/P3 hanya dikerjakan bila ada pelanggan yang meminta (berbasis permintaan nyata) |
| `git filter-repo` merusak clone/fork lama | Kontributor bingung | Sedang | Hanya dilakukan dengan persetujuan Owner + pengumuman |
| Pembersih secret over-mask sehingga log tak berguna | Support sulit | Sedang | Kasus negatif di tes (ENT-SEC-06) |

---

## 7. Lampiran

### 7.1 Checklist Rilis Enterprise (setiap versi)

- [ ] Semua job CI hijau pada commit tag
- [ ] `CHANGELOG.md` diperbarui; versi di `Cargo.toml`, `tauri.conf.json`, `frontend/package.json` sama
- [ ] Artefak Windows ditandatangani + timestamp (`signtool verify`)
- [ ] Artefak macOS di-notarize + stapled (`spctl -a`)
- [ ] Paket Linux bertanda tangan GPG
- [ ] SBOM CycloneDX terlampir
- [ ] Provenance attestation terlampir
- [ ] `SHA256SUMS` + signature terlampir
- [ ] `osv-scanner` tanpa High/Critical
- [ ] Smoke test matriks OS (ENT-QA-08) terisi
- [ ] Update dari versi sebelumnya teruji di kanal yang relevan
- [ ] Migrasi DB teruji dari 2 versi sebelumnya
- [ ] Release notes menyebut perubahan policy / breaking change
- [ ] Dokumentasi admin diperbarui bila ada key policy baru

### 7.2 Domain jaringan yang dipakai aplikasi (bahan `network-requirements.md`)

Diambil dari kode saat ini. Verifikasi ulang sebelum dipublikasikan.

| Domain | Fungsi | Bisa dimatikan via policy (target) |
|---|---|---|
| `caterm.fathforce.com` | Manifest update | `updates.enabled` |
| `github.com` (releases) | Fallback update | `updates.enabled` |
| `gcc.fathforce.com` | Lisensi Pro | `license.key` (offline) |
| `api.caterm.com` | CATerm Hosted AI | `ai.allowedProviders` |
| `api.anthropic.com`, `api.openai.com`, dll. | AI BYOK | `ai.allowedProviders` |
| `speed.cloudflare.com` | Speed test | `features.disabled: ["speed-test"]` |
| Host SSH/SFTP/FTP/WebDAV/S3 pengguna | Fungsi inti | `ssh.allowedHosts`, `sftp.allowedProtocols` |

### 7.3 Template task baru

```markdown
#### ENT-XXX-NN · Judul · P? · S/M/L/XL · 🤖/👤/🤝 · ⬜
- **Masalah:**
- **Lingkup:**
- **File:**
- **Acceptance criteria:**
- **Dependensi:**
```

### 7.4 Ringkasan jumlah task

| Workstream | P0 | P1 | P2 | P3 | Total |
|---|---|---|---|---|---|
| WS-GOV | 1 | 3 | 4 | 0 | 8 |
| WS-REL | 3 | 5 | 2 | 0 | 10 |
| WS-QA | 0 | 6 | 3 | 0 | 9 |
| WS-OBS | 0 | 3 | 1 | 1 | 5 |
| WS-SEC | 0 | 7 | 3 | 1 | 11 |
| WS-ADM | 0 | 7 | 5 | 0 | 12 |
| WS-IAM | 0 | 3 | 1 | 2 | 6 |
| WS-CMP | 0 | 6 | 1 | 1 | 8 |
| WS-UX | 0 | 0 | 3 | 1 | 4 |
| WS-DOC | 0 | 3 | 3 | 0 | 6 |
| **Total** | **4** | **43** | **26** | **6** | **79** |

---

*Dokumen ini dibuat dari audit langsung terhadap kode di commit `5843207`. Setiap temuan (F-xx) menyertakan lokasi file agar bisa diverifikasi ulang.*
