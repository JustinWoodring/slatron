use std::time::Duration;
use sysinfo::{ProcessExt, System, SystemExt};
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

        // Get CPU and memory usage
        let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;
        let memory_usage = (sys.used_memory() as f64) / (1024.0 * 1024.0); // Convert to MB

        NodeMessage::Heartbeat {
            current_content_id: None, // TODO: Get from playback state
            playback_position_secs: None, // TODO: Get from MPV
            status: "stopped".to_string(), // TODO: Get actual status
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_usage,
            errors: vec![],
        }
    }
}
