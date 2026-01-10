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
        let mut current_content_id = *self.state.current_content_id.read().await;

        let is_idle = self.state.mpv.is_idle().unwrap_or(false);

        if is_idle {
            // If MPV is idle, we are definitely NOT playing anything.
            if current_content_id.is_some() {
                tracing::info!("MPV is idle, clearing content ID");
                *self.state.current_content_id.write().await = None;
                current_content_id = None;
            }
        }
        // If not idle, double check what is ACTUALLY playing to catch Queued content
        else if let Ok(path) = self.state.mpv.get_path() {
            // Path from MPV might differ from cache path logic (e.g. absolute vs relative)
            // We need a way to map MPV Path back to ID.
            // We have content_cache: Map<ID, ServerContentItem>.
            // Let's iterate values. (Slow? Cache is small, likely fine for now).
            let cache = self.state.content_cache.read().await;

            // Check if current_id logic matches MPV path.
            let matches_current = if let Some(id) = current_content_id {
                cache
                    .get(&id)
                    .map(|c| c.content_path == path)
                    .unwrap_or(false)
            } else {
                false
            };

            if !matches_current {
                // MPV is playing something else. Find ID.
                // This handles the transition from Song A -> Song B (Queued)
                let found_id = cache.iter().find_map(|(id, item)| {
                    // Try exact match
                    if item.content_path == path {
                        return Some(*id);
                    }
                    // Try filename match fallback (if paths differ by absolute/relative)
                    if std::path::Path::new(&item.content_path).file_name()
                        == std::path::Path::new(&path).file_name()
                    {
                        return Some(*id);
                    }
                    None
                });

                if let Some(id) = found_id {
                    // Update State!
                    // tracing::info!("Resolved Path {} to ID {}", path, id);
                    *self.state.current_content_id.write().await = Some(id);
                    current_content_id = Some(id);
                } else {
                    tracing::warn!(
                        "DEBUG: Content Path Mismatch. MPV: '{}'. Cache has {} items.",
                        path,
                        cache.len()
                    );
                    // Optional: Log first few cache items to debug
                    /* for (_, item) in cache.iter().take(3) {
                         tracing::warn!("Cache Item: '{}'", item.content_path);
                    } */
                }
            }
        } else {
            // MPV reports no path? Maybe idle?
            // If we thought we were playing, but MPV says "no path", we stopped.
            if current_content_id.is_some() {
                *self.state.current_content_id.write().await = None;
                current_content_id = None;
            }
        }

        // Get position from MPV if playing
        let playback_position_secs = if current_content_id.is_some() {
            self.state.mpv.get_position().ok().map(|p| p as f32)
        } else {
            None
        };

        let playback_duration_secs = if current_content_id.is_some() {
            self.state.mpv.get_duration().ok().map(|d| d as f32)
        } else {
            None
        };

        let status = "online".to_string();

        NodeMessage::Heartbeat {
            current_content_id,
            playback_position_secs,
            playback_duration_secs,
            status,
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_usage,
            errors: vec![],
        }
    }
}
