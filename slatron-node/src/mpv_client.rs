use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub struct MpvClient {
    socket_path: String,
}

impl MpvClient {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub async fn play(&self, path: &str) -> Result<()> {
        self.send_command(json!({
            "command": ["loadfile", path, "replace"]
        }))
        .await
    }

    pub async fn pause(&self) -> Result<()> {
        self.send_command(json!({
            "command": ["set_property", "pause", true]
        }))
        .await
    }

    pub async fn resume(&self) -> Result<()> {
        self.send_command(json!({
            "command": ["set_property", "pause", false]
        }))
        .await
    }

    pub async fn stop(&self) -> Result<()> {
        self.send_command(json!({
            "command": ["stop"]
        }))
        .await
    }

    pub async fn seek(&self, position_secs: f64) -> Result<()> {
        self.send_command(json!({
            "command": ["seek", position_secs, "absolute"]
        }))
        .await
    }

    pub async fn add_overlay(&self, path: &str, x: i32, y: i32, opacity: f64) -> Result<()> {
        self.send_command(json!({
            "command": ["overlay-add", 0, x, y, path, opacity]
        }))
        .await
    }

    pub async fn get_position(&self) -> Result<f64> {
        let response = self
            .send_command(json!({
                "command": ["get_property", "time-pos"]
            }))
            .await?;

        response["data"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
    }

    pub async fn get_duration(&self) -> Result<f64> {
        let response = self
            .send_command(json!({
                "command": ["get_property", "duration"]
            }))
            .await?;

        response["data"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
    }

    async fn send_command(&self, cmd: Value) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        let cmd_str = format!("{}\n", serde_json::to_string(&cmd)?);

        stream.write_all(cmd_str.as_bytes())?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        Ok(serde_json::from_str(&response)?)
    }
}
