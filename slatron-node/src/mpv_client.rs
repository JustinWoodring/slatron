use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

pub fn spawn_mpv(socket_path: &str) -> Result<Child> {
    if Path::new(socket_path).exists() {
        // Try to remove existing socket file
        let _ = std::fs::remove_file(socket_path);
    }

    let child = Command::new("mpv")
        .arg("--idle")
        .arg(format!("--input-ipc-server={}", socket_path))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Wait for socket to be created
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if Path::new(socket_path).exists() {
            return Ok(child);
        }
        thread::sleep(Duration::from_millis(100));
    }

    // If we timed out, check if process is still alive
    // If process died, return error.
    // If process is alive but no socket... maybe slow start?

    Ok(child)
}

pub struct MpvClient {
    socket_path: String,
}

impl MpvClient {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub fn play(
        &self,
        path: &str,
        start_time: Option<f64>,
        loop_enabled: Option<bool>,
    ) -> Result<()> {
        let mut args = vec![
            "loadfile".to_string(),
            path.to_string(),
            "replace".to_string(),
        ];

        let mut options = Vec::new();
        if let Some(start) = start_time {
            options.push(format!("start={}", start));
        }
        if let Some(true) = loop_enabled {
            options.push("loop-file=inf".to_string());
        }

        // loadfile path replace options...
        // The JSON IPC format for loadfile with options is tricky.
        // It's strictly: ["loadfile", "URL", "flags", "k=v,k=v..."]?
        // No, MPV IPC is command args.
        // loadfile <file> <flags> <options>
        // options is key=value,key=value

        // MPV 0.38+ requires an insertion index argument between mode and options.
        // Signature: loadfile url [mode [index [options]]]
        // We use "0" as the index (placeholder for 'replace' mode).

        if !options.is_empty() {
            args.push("0".to_string());
            args.push(options.join(","));
        }

        self.send_command(json!({
            "command": args
        }))?;
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        self.send_command(json!({
            "command": ["set_property", "volume", volume]
        }))?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.send_command(json!({
            "command": ["set_property", "pause", true]
        }))?;
        Ok(())
    }

    pub fn resume(&self) -> Result<()> {
        self.send_command(json!({
            "command": ["set_property", "pause", false]
        }))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.send_command(json!({
            "command": ["stop"]
        }))?;
        Ok(())
    }

    pub fn seek(&self, position_secs: f64) -> Result<()> {
        self.send_command(json!({
            "command": ["seek", position_secs, "absolute"]
        }))?;
        Ok(())
    }

    pub fn add_overlay(&self, path: &str, x: i32, y: i32, opacity: f64) -> Result<()> {
        self.send_command(json!({
            "command": ["overlay-add", 0, x, y, path, opacity]
        }))?;
        Ok(())
    }

    pub fn get_position(&self) -> Result<f64> {
        let response = self.send_command(json!({
            "command": ["get_property", "time-pos"]
        }))?;

        response["data"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
    }

    pub fn get_duration(&self) -> Result<f64> {
        let response = self.send_command(json!({
            "command": ["get_property", "duration"]
        }))?;

        response["data"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
    }

    pub fn send_command(&self, cmd: Value) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;

        let cmd_str = format!("{}\n", serde_json::to_string(&cmd)?);

        // Log the command we are sending
        tracing::info!(target: "slatron_node::mpv_client", "Sending command: {}", cmd_str.trim());

        stream.write_all(cmd_str.as_bytes())?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        // Log the response
        // tracing::debug!(target: "mpv_client", "Response: {}", response.trim());

        Ok(serde_json::from_str(&response)?)
    }
}
