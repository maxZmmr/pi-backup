use std::{collections::HashMap, fs, path::PathBuf};
use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub age_public_key: String,
    pub rclone_remote: String,
    pub jobs: HashMap<String, JobConfig>,
}

#[derive(Deserialize, Debug)]
pub struct JobConfig {
    pub sqlite_db: Option<PathBuf>,
    pub include_paths: Vec<PathBuf>,
    #[serde(default = "default_keep_last")]
    pub keep_last: u32,
    pub healthcheck_url: Option<String>,
}

fn default_keep_last() -> u32 {
    7
}

pub fn load_config(path: &PathBuf) -> anyhow::Result<Config> {
    let content = fs::read_to_string(path).with_context(|| format!("Konnte Konfigurationsdatei '{}' nicht lesen", path.display()))?;
    let config: Config = toml::from_str(&content).context("Fehler beim parsen der TOML-Konfiguration")?;

    Ok(config)
}