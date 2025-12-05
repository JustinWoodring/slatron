use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub node_name: String,
    pub server_url: String,
    pub secret_key: String,
    pub heartbeat_interval_secs: u64,
    pub schedule_poll_interval_secs: u64,
    pub mpv_socket_path: String,
    pub offline_mode_warning_hours: u64,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
