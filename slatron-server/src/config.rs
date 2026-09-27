use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Public URL nodes use to reach this server (e.g. "http://192.168.1.50:8080").
    /// Falls back to http://127.0.0.1:{port} when unset, which only works when
    /// nodes run on the same machine as the server.
    #[serde(default)]
    pub public_url: Option<String>,
    pub https: Option<HttpsConfig>,
    pub ui_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_template() -> &'static str {
        r#"[server]
host = "0.0.0.0"
port = 8080

# Optional: Public URL that nodes use to reach this server
# (e.g. "http://192.168.1.50:8080" or "https://tv.example.com").
# Required for DJ/TTS audio playback on nodes when binding to 0.0.0.0 or
# running behind a reverse proxy. If unset, audio URLs fall back to
# http://127.0.0.1:<port>, which only works when nodes run on this machine.
# public_url = "http://192.168.1.50:8080"

[server.https]
enabled = false
cert_path = "certs/cert.pem"
key_path = "certs/key.pem"

# Optional: Path to custom UI directory.
# If invalid or empty, server defaults to "./static" (or embedded UI if compiled with feature).
# ui_path = "./static"

[database]
# URL for the SQLite database. Ensure the directory exists.
url = "sqlite://slatron.db"

[jwt]
secret = "change-me-in-production"
expiration_hours = 24

[logging]
level = "info"
"#
    }
}
