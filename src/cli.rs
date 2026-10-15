use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pi-backup")]
#[command(about = "Lightweight homelab backup tool & watchdog for Raspberry Pis", long_about = None)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a backup for the specified job
    Backup {
        /// Name of the job as defined in config.toml
        #[arg(short, long)]
        job: String,
    },
    /// Restore data from an encrypted .tar.zst.age backup
    Restore {
        /// Optional: Local backup file path. If omitted, backups are listed from cloud.
        #[arg(short, long)]
        archive: Option<PathBuf>,

        /// Optional: Job name to search for in remote cloud storage
        #[arg(short, long)]
        job: Option<String>,

        /// Destination directory for restored files
        #[arg(short, long)]
        target: PathBuf,

        /// Path to age secret key file or the raw key string itself
        #[arg(short, long)]
        key: String,
    },
    /// List all available backups on remote cloud storage
    List {
        /// Name of the job
        #[arg(short, long)]
        job: String,
    },
    /// Generate a fresh age X25519 keypair for encryption and decryption
    Keygen,
}
