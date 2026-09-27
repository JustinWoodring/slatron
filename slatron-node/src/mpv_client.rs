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

    let mut child = Command::new("mpv")
        .arg("--idle")
        .arg("--player-operation-mode=pseudo-gui")
        .arg(format!("--input-ipc-server={}", socket_path))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Wait for socket to be created, checking on every poll whether the
    // process is still alive: an mpv that died during startup (bad flag,
    // missing audio device, etc.) must surface as an error, not a silent Ok.
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if Path::new(socket_path).exists() {
            return Ok(child);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(anyhow::anyhow!(
                    "mpv exited during startup before creating IPC socket '{}': {}",
                    socket_path,
                    status
                ));
            }
            Ok(None) => {}
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "failed to check mpv status while waiting for IPC socket '{}': {}",
                    socket_path,
                    e
                ));
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Timed out with the process still alive (no exit status reported) but no
    // socket... maybe slow start? Return it and let IPC commands fail loudly.
    tracing::warn!(
        "mpv (socket '{}') did not create its IPC socket within 10s; assuming slow start",
        socket_path
    );
    Ok(child)
}

/// Supervises both MPV processes: every 5 seconds, checks each child via
/// try_wait(); if it has exited, logs the exit status and respawns it with
/// spawn_mpv on the given socket path, replacing the child in state's
/// mpv_process (voice=true → mpv_process_voice) under its Mutex.
pub async fn supervise(
    state: crate::NodeState,
    main_socket_path: String,
    voice_socket_path: String,
) {
    tracing::info!("MPV supervisor started (checking every 5s)");
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        supervise_slot(&state, &main_socket_path, false);
        supervise_slot(&state, &voice_socket_path, true);
    }
}

/// Checks one MPV slot for an exited child and respawns it in place.
fn supervise_slot(state: &crate::NodeState, socket_path: &str, voice: bool) {
    let label = if voice { "MPV (Voice)" } else { "MPV (Main)" };
    let slot = if voice {
        &state.mpv_process_voice
    } else {
        &state.mpv_process
    };

    // Non-blocking reap check: try_wait() never blocks, so the std Mutex is
    // only held for the check itself, never across the blocking respawn below.
    let exit_status = {
        let mut guard = match slot.lock() {
            Ok(guard) => guard,
            Err(_) => {
                tracing::error!(
                    "{} process slot mutex poisoned; skipping supervision tick",
                    label
                );
                return;
            }
        };
        match guard.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(status)) => Some(status),
                Ok(None) => None,
                Err(e) => {
                    tracing::warn!("Failed to check {} process status: {}", label, e);
                    None
                }
            },
            // No managed child (initial spawn failed or mpv is started
            // manually): nothing to supervise and no exit status to retry.
            None => None,
        }
    };

    let Some(status) = exit_status else {
        return;
    };

    tracing::error!(
        "{} process exited unexpectedly ({}); respawning on socket {}",
        label,
        status,
        socket_path
    );

    match spawn_mpv(socket_path) {
        Ok(mut child) => {
            drain_output(&mut child, voice);
            match slot.lock() {
                Ok(mut guard) => {
                    *guard = Some(child);
                    tracing::info!("{} respawned successfully on socket {}", label, socket_path);
                }
                Err(_) => {
                    tracing::error!(
                        "{} process slot mutex poisoned; killing respawned child",
                        label
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
        Err(e) => {
            // The slot keeps the old dead child, whose cached exit status is
            // re-detected on every tick, so the next tick retries the respawn.
            tracing::error!(
                "Failed to respawn {} on socket '{}': {}; will retry next tick",
                label,
                socket_path,
                e
            );
        }
    }
}

/// Forwards a respawned child's stdout/stderr to the tracing log, mirroring
/// the capture threads main.rs sets up for the initially spawned processes.
/// Without this, the piped output buffer fills and the respawned mpv stalls.
fn drain_output(child: &mut Child, voice: bool) {
    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    if voice {
                        tracing::debug!(target: "slatron_node::mpv_voice", "{}", l);
                    } else {
                        tracing::debug!(target: "slatron_node::mpv_main", "{}", l);
                    }
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    if voice {
                        tracing::error!(target: "slatron_node::mpv_voice", "{}", l);
                    } else {
                        tracing::error!(target: "slatron_node::mpv_main", "{}", l);
                    }
                }
            }
        });
    }
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

    pub fn queue(&self, path: &str) -> Result<()> {
        // queue implies append to playlist.
        // If nothing playing, it plays.
        // "loadfile" "path" "append-play"
        let args = vec![
            "loadfile".to_string(),
            path.to_string(),
            "append-play".to_string(),
        ];

        self.send_command(json!({
            "command": args
        }))?;
        Ok(())
    }

    pub fn get_volume(&self) -> Result<f64> {
        let response = self.send_command(json!({
            "command": ["get_property", "volume"]
        }))?;

        response["data"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
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

    pub fn screenshot(&self, path: &str) -> Result<()> {
        // "screenshot-to-file" "<filename>" "<mode>"
        // mode: "video" (no subtitles/osd), "window" (with osd)
        // We probably want "video" or "subtitles" (video+subs)
        self.send_command(json!({
            "command": ["screenshot-to-file", path, "video"]
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

    pub fn get_path(&self) -> Result<String> {
        let response = self.send_command(json!({
            "command": ["get_property", "path"]
        }))?;

        response["data"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid response or no path"))
    }

    pub fn is_paused(&self) -> Result<bool> {
        let response = self.send_command(json!({
            "command": ["get_property", "pause"]
        }))?;

        response["data"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
    }

    pub fn is_idle(&self) -> Result<bool> {
        let response = self.send_command(json!({
            "command": ["get_property", "idle-active"]
        }))?;

        response["data"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))
    }

    pub fn send_command(&self, cmd: Value) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;

        let cmd_str = format!("{}\n", serde_json::to_string(&cmd)?);

        // Log the command we are sending
        tracing::debug!(target: "slatron_node::mpv_client", "Sending command: {}", cmd_str.trim());

        stream.write_all(cmd_str.as_bytes())?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        // Log the response
        // tracing::debug!(target: "mpv_client", "Response: {}", response.trim());

        Ok(serde_json::from_str(&response)?)
    }
}
