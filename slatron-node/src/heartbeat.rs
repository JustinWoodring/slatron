use std::time::Duration;
use sysinfo::System;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::interval;

use crate::websocket_client::NodeMessage;
use crate::NodeState;

pub struct HeartbeatManager {
    state: NodeState,
    sender: UnboundedSender<NodeMessage>,
}

impl HeartbeatManager {
    pub fn new(state: NodeState, sender: UnboundedSender<NodeMessage>) -> Self {
        Self { state, sender }
    }

    pub async fn start(self) {
        let interval_secs = self.state.config.heartbeat_interval_secs;
        let mut tick = interval(Duration::from_secs(interval_secs));

        loop {
            tick.tick().await;

            let heartbeat = self.collect_heartbeat_data().await;

            if self.sender.send(heartbeat).is_err() {
                tracing::error!("Failed to send heartbeat");
                break;
            }
        }
    }

    async fn collect_heartbeat_data(&self) -> NodeMessage {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Get CPU and memory usage (sysinfo 0.30 API)
        // Calculate average CPU usage across all cores
        let cpu_usage = if !sys.cpus().is_empty() {
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
        } else {
            0.0
        } as f64;

        let memory_usage = (sys.used_memory() as f64) / (1024.0 * 1024.0); // Convert to MB

        // Get playback status
        let current_content_id = *self.state.current_content_id.read().await;

        // Get position from MPV if playing
        let playback_position_secs = if current_content_id.is_some() {
            self.state.mpv.get_position().ok().map(|p| p as f32)
        } else {
            None
        };

        let status = "online".to_string();

        NodeMessage::Heartbeat {
            current_content_id,
            playback_position_secs,
            status,
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_usage,
            errors: vec![],
        }
    }
}
