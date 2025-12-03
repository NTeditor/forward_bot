use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub token: String,
    pub allowed_users: Option<Vec<u64>>,
    pub target: Target,
    pub messages: Messages,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        tracing::info!(config_path = path, "Loading config");
        tracing::info!("Reading config file");
        let content = fs::read(path).context("Failed to read config file")?;
        tracing::info!("Parsing config");
        let config: Self = toml::from_slice(&content).context("Failed to parse config")?;
        tracing::info!("Config loaded");
        Ok(config)
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Target {
    pub chat_id: i64,
    pub thread_id: Option<i32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Messages {
    pub start_command: String,
    pub success_forward: String,
    pub access_denied_forward: String,
    pub unknown_sender: String,
    pub failed_to_forward: String,
}
