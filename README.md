# pi-backup

A simple, lightweight backup tool & watchdog for Raspberry Pi and homelab servers.

It takes targeted backups of files, directories, and live SQLite databases (e.g. Vaultwarden), streams them through in-memory compression and encryption, uploads them to cloud storage via `rclone`, and reports to Healthchecks.io.

## Features

- **SD-card friendly:** Streams archives directly in memory (`tar -> zstd -> age`). No unencrypted temporary files on disk.
- **Consistent SQLite snapshots:** Uses the SQLite Online Backup API to dump databases into `/dev/shm` (RAM) before archiving, preserving original database filenames.
- **Asymmetric encryption:** Encrypted with [age](https://github.com/FiloSottile/age) (X25519). The Pi only stores the public key and cannot decrypt historical backups.
- **Preserve paths option:** Support for `preserve_paths = true` (essential for system configs like Samba or CUPS) and flat mode for Docker volumes.
- **Smart config discovery:** Automatically checks `./config.toml` and `/etc/pi-backup/config.toml`.
- **Rclone integration:** Uploads archives and automatically rotates backups older than `keep_last` days.
- **Healthchecks.io monitoring:** Dispatches start, success, and detailed error traces on failure.
- **Restore CLI:** Interactively select historical backups or restore the latest version automatically with `--latest`.

## Installation

### Prebuilt binary
Download the `pi-backup-arm64` binary from GitHub Actions, make it executable, and move it to your system path:

```bash
chmod +x pi-backup-arm64
sudo mv pi-backup-arm64 /usr/local/bin/pi-backup
```

### Build from source
```bash
cargo build --release
sudo cp target/release/pi-backup /usr/local/bin/
```

## Configuration

Place your `config.toml` at `/etc/pi-backup/config.toml` (or in the current directory):

```toml
# Public key to encrypt archives (generate with: pi-backup keygen)
age_public_key = "age1..."

# rclone remote path
rclone_remote = "gdrive:pi-backups"

# --- Example 1: Docker / Container with SQLite (Vaultwarden) ---
[jobs.vaultwarden]
sqlite_db = "/home/raspi/programmieren/vw-data/db.sqlite3"
include_paths = [
    "/home/raspi/programmieren/vw-data/rsa_key.pem"
]
keep_last = 14
healthcheck_url = "https://hc-ping.com/<uuid>"

# --- Example 2: System Service with directory hierarchy (CUPS / Samba) ---
[jobs.cups]
preserve_paths = true
include_paths = [
    "/etc/cups/printers.conf",
    "/etc/cups/ppd",
    "/etc/cups/cupsd.conf"
]
keep_last = 7
healthcheck_url = "https://hc-ping.com/<uuid>"
```

## Commands

```bash
# Generate a new age keypair
pi-backup keygen

# Run a backup job (auto-detects /etc/pi-backup/config.toml)
sudo pi-backup backup -j vaultwarden

# List remote cloud backups for a job
pi-backup list -j vaultwarden

# Restore latest cloud backup automatically (headless / scripted)
sudo pi-backup restore -j vaultwarden --latest -t /home/raspi/programmieren/vw-data -k "<AGE-SECRET-KEY>"

# Restore interactively (choose from a numbered list of versions)
sudo pi-backup restore -j vaultwarden -t /home/raspi/programmieren/vw-data -k "<AGE-SECRET-KEY>"

# Restore system files directly to root filesystem
sudo pi-backup restore -j cups --latest -t / -k "<AGE-SECRET-KEY>"
```

## Cron example

```cron
# Run daily at 03:00 AM
0 3 * * * /usr/local/bin/pi-backup backup -j vaultwarden
```

## License

MIT
