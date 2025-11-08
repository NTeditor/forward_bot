use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub chat_id: i64,
    pub thread_id: Option<i32>,
    pub allowed_users: Option<Vec<u64>>,
}

impl Config {
    pub fn read_from_file(file: &Path) -> Result<Self> {
        if !file.exists() {
            bail!("Config file is not exists: '{}'", file.to_string_lossy());
        }
        let content = fs::read_to_string(file).context("Failed read config file")?;
        let config = toml::from_str(&content).context("Failed parse config file")?;
        Ok(config)
    }
}
