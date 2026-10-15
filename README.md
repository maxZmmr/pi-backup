# pi-backup

A simple, lightweight backup tool for Raspberry Pi and homelab servers.

It takes targeted backups of files, directories, and live SQLite databases (e.g. Vaultwarden), streams them through in-memory compression and encryption, uploads them to cloud storage via `rclone`, and pings Healthchecks.io.

## Features

- **SD-card friendly:** Streams archives directly in memory (`tar -> zstd -> age`). No unencrypted temporary files on disk.
- **Consistent SQLite dumps:** Uses the SQLite Online Backup API to dump databases into `/dev/shm` before archiving.
- **Asymmetric encryption:** Encrypted with [age](https://github.com/FiloSottile/age) (X25519). The Pi only stores the public key and cannot decrypt historical backups.
- **Rclone integration:** Uploads archives and automatically rotates backups older than `keep_last` days.
- **Healthchecks.io support:** Pings your healthcheck endpoint on start, success, and sends error traces on failure.
- **Restore CLI:** Select and restore backups directly from cloud storage or local files.

## Installation

### Prebuilt binary
Download the `pi-backup-arm64` binary from the GitHub Actions tab of this repository, then make it executable:

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

Create a `config.toml` (e.g. at `/etc/pi-backup/config.toml`):

```toml
# Public key to encrypt archives (generate with: pi-backup keygen)
age_public_key = "age1..."

# rclone remote path
rclone_remote = "gdrive:pi-backups"

[jobs.vaultwarden]
sqlite_db = "/var/lib/vaultwarden/db.sqlite3"
include_paths = [
    "/var/lib/vaultwarden/attachments",
    "/etc/vaultwarden.env"
]
keep_last = 14
healthcheck_url = "https://hc-ping.com/<uuid>"

[jobs.configs]
include_paths = [
    "/etc/fstab",
    "/home/pi/docker-compose.yml"
]
keep_last = 7
```

## Commands

```bash
# Generate a new age keypair
pi-backup keygen

# Run a backup job
pi-backup --config config.toml backup -j vaultwarden

# List cloud backups for a job
pi-backup --config config.toml list -j vaultwarden

# Restore interactively from cloud
pi-backup --config config.toml restore -j vaultwarden -t /target/dir -k "<AGE-SECRET-KEY>"

# Restore from a local file
pi-backup restore -a backup.tar.zst.age -t /target/dir -k "<AGE-SECRET-KEY>"
```

## Cron example

```cron
# Run daily at 03:00 AM
0 3 * * * /usr/local/bin/pi-backup --config /etc/pi-backup/config.toml backup -j vaultwarden
```

## License

MIT
