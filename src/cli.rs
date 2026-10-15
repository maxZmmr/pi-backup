use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pi-backup")]
#[command(about = "small backup tool for raspi", long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    pub config: PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Führt ein Backup für einen definierten Job aus
    Backup {
        /// Name des Jobs
        #[arg(short, long)]
        job: String,
    },
    /// Stellt ein Backup aus einer .tar.zst.age Datei wieder her
    Restore {
        /// Optional: Lokale Backup-Datei (.tar.zst.age)
        #[arg(short, long)]
        archive: Option<PathBuf>,

        /// Optional: Name des Jobs in der Cloud (falls aus der Cloud geladen wird)
        #[arg(short, long)]
        job: Option<String>,

        /// Zielordner für die wiederhergestellten Daten
        #[arg(short, long)]
        target: PathBuf,

        /// Pfad zur age Secret-Key Datei oder der Schlüssel selbst
        #[arg(short, long)]
        key: String,
    },
    /// Zeigt alle backups, die auf der rclone drive gespeichert sind
    List {
        /// Name des Jobs
        #[arg(short, long)]
        job: String,
    },
    /// generiert neue Schlüsselpaare
    Keygen,
}

