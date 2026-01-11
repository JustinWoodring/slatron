use base64::{engine::general_purpose, Engine as _};
use std::env;
use std::fs;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{interval, sleep};

use crate::websocket_client::NodeMessage;
use crate::NodeState;

pub struct ScreenshotManager {
    state: NodeState,
    sender: UnboundedSender<NodeMessage>,
}

impl ScreenshotManager {
    pub fn new(state: NodeState, sender: UnboundedSender<NodeMessage>) -> Self {
        Self { state, sender }
    }

    pub async fn start(self) {
        // Run every 60 seconds
        let mut tick = interval(Duration::from_secs(60));
        let temp_dir = env::temp_dir();
        let screenshot_path = temp_dir.join("slatron_node_screenshot.jpg");

        loop {
            tick.tick().await;

            // Only take screenshot if online and playing (or just online? Node might be idle but showing something?)
            // If idle, MPV shows nothing (black screen or logo if configured).
            // Let's just try to take it if MPV process is alive.

            if let Ok(_mpv_running) = self.state.mpv.is_idle() {
                // is_idle returns true if idle (no file loaded).
                // Even if idle, we can capture the idle screen.
            } else {
                // MPV might be dead/not connected
                continue;
            }

            // Take screenshot
            // We use a random filename to avoid collisions or caching?
            // Actually reusing same path is fine since we read it immediately.
            let path_str = screenshot_path.to_string_lossy().to_string();

            // Clean up old file if exists
            if screenshot_path.exists() {
                let _ = fs::remove_file(&screenshot_path);
            }

            if let Err(e) = self.state.mpv.screenshot(&path_str) {
                tracing::warn!("Failed to request screenshot: {}", e);
                continue;
            }

            // Wait for file to be written (MPV takes a moment)
            // Poll for file existence for up to 2 seconds
            let mut file_exists = false;
            for _ in 0..20 {
                if screenshot_path.exists() {
                    file_exists = true;
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }

            if file_exists {
                // Read file
                match fs::read(&screenshot_path) {
                    Ok(bytes) => {
                        let b64 = general_purpose::STANDARD.encode(&bytes);
                        let msg = NodeMessage::Screenshot { image_base64: b64 };
                        if let Err(e) = self.sender.send(msg) {
                            tracing::error!("Failed to send screenshot message: {}", e);
                            break; // Channel closed
                        } else {
                            tracing::info!("Sent status screenshot");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to read screenshot file: {}", e);
                    }
                }
            } else {
                tracing::warn!("Screenshot file was not created in time.");
            }
        }
    }
}
