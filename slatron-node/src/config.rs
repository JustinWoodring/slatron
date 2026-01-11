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
    pub screenshot_tool_path: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_template() -> &'static str {
        r#"node_name = "Local"
server_url = "ws://127.0.0.1:8080/ws"
secret_key = "change-me"
heartbeat_interval_secs = 5
schedule_poll_interval_secs = 60
mpv_socket_path = "/tmp/mpv-socket"
offline_mode_warning_hours = 24
# screenshot_tool_path = "/usr/local/bin/screenshot.js"
"#
    }
}
