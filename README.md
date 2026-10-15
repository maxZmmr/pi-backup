# pi-backup 🛡️

[![CI](https://github.com/mxjhf/pi-backup/actions/workflows/build.yml/badge.svg)](https://github.com/mxjhf/pi-backup/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

**A lightweight, single-binary homelab backup tool & watchdog designed specifically for Raspberry Pis and Linux micro-servers.**

Traditional backup tools like BorgBackup or Restic are great for multi-gigabyte media archives, but for services like **Vaultwarden**, databases, and server configurations, they introduce heavy deduplication caches that thrash SD cards and can leave your backup history vulnerable if the central repository corrupts.

`pi-backup` takes a different approach: **Zero unencrypted intermediate files on disk**, **in-memory streaming**, **isolated independent snapshots**, and **asymmetric public-key encryption**.

---

## 🚀 Key Features

- **SD-Card Friendly In-Memory Pipeline:**  
  Files and databases are streamed directly through a nested `std::io::Write` pipeline:  
  `Files -> tar -> zstd (compression) -> age (encryption) -> cloud destination`.  
  No unencrypted or uncompressed intermediate files ever touch the storage card.
- **Consistent SQLite Online Dumps:**  
  Safe live backups of running SQLite databases (e.g. Vaultwarden) using the SQLite Online Backup API dumped to Linux RAM-disk (`/dev/shm`).
- **Asymmetric Encryption with `age`:**  
  Encrypted using modern X25519 elliptic-curve cryptography. The Raspberry Pi only needs the **Public Key** to create backups. Even if your server is compromised, attackers **cannot decrypt your backups**!
- **Automated Cloud Sync & Rotation via `rclone`:**  
  Uploads directly to Google Drive, S3, Nextcloud, OneDrive, or SFTP and automatically purges historical archives older than `keep_last` days.
- **Healthchecks.io & Dead-Man's-Snitch Monitoring:**  
  Sends start and finish signals (with detailed error traces) to [Healthchecks.io](https://healthchecks.io) to ensure you are instantly alerted if a cron job fails.
- **Interactive Cloud Restore:**  
  Interactive CLI restore with numbered backup versions fetched directly from cloud storage.
- **Zero Runtime Dependencies:**  
  SQLite is bundled directly into the statically linked Rust binary (`aarch64-unknown-linux-musl`).

---

## 📦 Architecture Pipeline

```text
               ┌────────────────────────────────────────────────────────┐
               │                     Raspberry Pi                       │
               │                                                        │
[Local Files] ─┼────────┐                                               │
               │        ▼                                               │
[SQLite DB] ───┼─▶ [/dev/shm Snapshot]                                  │
               │        │                                               │
               │        ▼                                               │
               │   ┌─────────┐   ┌─────────┐   ┌─────────┐              │
               │   │   tar   │──▶│  zstd   │──▶│   age   │ (In-RAM)     │
               │   └─────────┘   └─────────┘   └─────────┘              │
               │                                    │                   │
               │                                    ▼                   │
               │                         [Encrypted Archive]            │
               │                                    │                   │
               │                                    ▼                   │
               │                                [ rclone ]              │
               └────────────────────────────────────┬───────────────────┘
                                                    │
                                                    ▼
                                          [ Google Drive / S3 ]
```

---

## 📥 Installation

### Option 1: Download Prebuilt ARM64 Binary (Recommended)
Download the latest statically compiled `pi-backup-arm64` binary from your GitHub Actions artifacts or releases:

```bash
sudo mv pi-backup-arm64 /usr/local/bin/pi-backup
sudo chmod +x /usr/local/bin/pi-backup
```

### Option 2: Build From Source
```bash
git clone https://github.com/mxjhf/pi-backup.git
cd pi-backup
cargo build --release
sudo cp target/release/pi-backup /usr/local/bin/
```

---

## ⚙️ Configuration (`config.toml`)

Create a configuration file (e.g. `/etc/pi-backup/config.toml`):

```toml
# Public key for encryption (generate one with: pi-backup keygen)
age_public_key = "age1t7m8curm2pzdmlye5hkka5u66xpd942ye7amyy0l27vv065twqjsqkwzws"

# Rclone remote destination
rclone_remote = "gdrive:pi-backups"

[jobs.vaultwarden]
sqlite_db = "/var/lib/vaultwarden/db.sqlite3"
include_paths = [
    "/var/lib/vaultwarden/attachments",
    "/etc/vaultwarden.env"
]
keep_last = 14
healthcheck_url = "https://hc-ping.com/your-healthchecks-uuid-here"

[jobs.system_config]
include_paths = [
    "/etc/fstab",
    "/etc/network/interfaces",
    "/home/pi/docker-compose.yml"
]
keep_last = 7
healthcheck_url = "https://hc-ping.com/another-uuid-here"
```

---

## 🛠️ Usage

### 1. Generate an Encryption Keypair
```bash
pi-backup keygen
```
> ⚠️ **Important:** Add the printed **Public Key** to `config.toml`. Store the **Private Key** in your password manager. Never leave the private key on the server!

### 2. Run a Backup Job
```bash
pi-backup --config /etc/pi-backup/config.toml backup -j vaultwarden
```

### 3. List Available Backups from Cloud
```bash
pi-backup --config /etc/pi-backup/config.toml list -j vaultwarden
```

### 4. Restore an Archive
Restore interactively from cloud:
```bash
pi-backup --config /etc/pi-backup/config.toml restore -j vaultwarden -t /restore/path -k "AGE-SECRET-KEY-1..."
```
Or restore from a local `.tar.zst.age` file:
```bash
pi-backup restore -a ./backup.tar.zst.age -t /restore/path -k "AGE-SECRET-KEY-1..."
```

---

## ⏰ Automated Cron / Systemd Setup

Set up a nightly cronjob with `crontab -e`:

```bash
# Run Vaultwarden backup every night at 03:00 AM
0 3 * * * /usr/local/bin/pi-backup --config /etc/pi-backup/config.toml backup -j vaultwarden > /var/log/pi-backup.log 2>&1
```

---

## 🔒 Security Model

- **Zero-Knowledge Server:** The server only possesses the public key. Compromising the Raspberry Pi does not yield access to historical backups.
- **Pure Authenticated Encryption:** Every archive uses `age` (ChaCha20-Poly1305 / HMAC) ensuring cryptographic integrity.
- **Fail-Safe Watchdog:** Files smaller than 1 byte or corrupted dumps are caught before upload, and immediate `/fail` alerts are dispatched to Healthchecks.io.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
