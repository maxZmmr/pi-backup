use anyhow::Context;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Age public key used for encrypting backup archives
    pub age_public_key: String,
    /// Rclone remote destination (e.g. "gdrive:pi-backups")
    pub rclone_remote: String,
    /// Configured backup jobs mapped by job name
    pub jobs: HashMap<String, JobConfig>,
}

#[derive(Deserialize, Debug)]
pub struct JobConfig {
    /// Optional: Path to SQLite database file for safe online dumps
    pub sqlite_db: Option<PathBuf>,
    /// Directories and files to include in the backup archive
    pub include_paths: Vec<PathBuf>,
    /// Retention period in days for cloud backup rotation
    #[serde(default = "default_keep_last")]
    pub keep_last: u32,
    /// Optional: Healthchecks.io ping URL for execution monitoring
    pub healthcheck_url: Option<String>,
}

fn default_keep_last() -> u32 {
    7
}

pub fn load_config(path: &PathBuf) -> anyhow::Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read configuration file at '{}'", path.display()))?;
    let config: Config = toml::from_str(&content).context("Failed to parse TOML configuration")?;

    Ok(config)
}
