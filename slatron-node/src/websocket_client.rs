use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::heartbeat::HeartbeatManager;
use crate::NodeState;

// Server → Node messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "auth_response")]
    AuthResponse { success: bool, message: String },
    #[serde(rename = "schedule_updated")]
    ScheduleUpdated { timestamp: String },
    #[serde(rename = "command")]
    Command { command: NodeCommand },
    #[serde(rename = "heartbeat_ack")]
    HeartbeatAck,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum NodeCommand {
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "seek")]
    Seek { position_secs: f64 },
    #[serde(rename = "load_content")]
    LoadContent { content_id: i32 },
    #[serde(rename = "reload_schedule")]
    ReloadSchedule,
    #[serde(rename = "shutdown")]
    Shutdown,
}

// Node → Server messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeMessage {
    #[serde(rename = "authenticate")]
    Authenticate {
        node_name: String,
        secret_key: String,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        current_content_id: Option<i32>,
        playback_position_secs: Option<f64>,
        status: String,
        cpu_usage_percent: f64,
        memory_usage_mb: f64,
        errors: Vec<String>,
    },
    #[serde(rename = "request_schedule")]
    RequestSchedule,
    #[serde(rename = "report_paths")]
    ReportPaths { available_paths: Vec<String> },
    #[serde(rename = "content_error")]
    ContentError { content_id: i32, error: String },
}

pub struct WebSocketClient {
    state: NodeState,
}

impl WebSocketClient {
    pub fn new(state: NodeState) -> Self {
        Self { state }
    }

    pub async fn connect_and_run(&mut self) -> Result<()> {
        let mut delay = Duration::from_secs(5);
        let max_delay = Duration::from_secs(300);

        loop {
            match self.connect().await {
                Ok(()) => {
                    delay = Duration::from_secs(5); // Reset delay on successful connection
                }
                Err(e) => {
                    tracing::error!("Connection error: {}", e);
                }
            }

            tracing::info!("Reconnecting in {:?}...", delay);
            sleep(delay).await;
            delay = (delay * 2).min(max_delay); // Exponential backoff
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let url = &self.state.config.server_url;
        tracing::info!("Connecting to server: {}", url);

        let (ws_stream, _) = connect_async(url).await?;
        tracing::info!("WebSocket connection established");

        let (mut write, mut read) = ws_stream.split();

        // Send authentication message
        let auth_msg = NodeMessage::Authenticate {
            node_name: self.state.config.node_name.clone(),
            secret_key: self.state.config.secret_key.clone(),
        };

        let auth_json = serde_json::to_string(&auth_msg)?;
        write.send(Message::Text(auth_json)).await?;

        // Wait for auth response
        if let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(ServerMessage::AuthResponse { success, message }) =
                serde_json::from_str(&text)
            {
                if !success {
                    tracing::error!("Authentication failed: {}", message);
                    return Err(anyhow::anyhow!("Authentication failed"));
                }
                tracing::info!("Authenticated successfully");
            }
        }

        // Start heartbeat task
        let heartbeat_tx = {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<NodeMessage>();
            let mut write_clone = write.clone();

            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = write_clone.send(Message::Text(json)).await;
                    }
                }
            });

            tx
        };

        let heartbeat_manager = HeartbeatManager::new(
            self.state.clone(),
            heartbeat_tx,
        );

        tokio::spawn(async move {
            heartbeat_manager.start().await;
        });

        // Handle incoming messages
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                        self.handle_server_message(server_msg).await?;
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("Server closed connection");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn handle_server_message(&self, msg: ServerMessage) -> Result<()> {
        match msg {
            ServerMessage::AuthResponse { success, message } => {
                tracing::info!("Auth response: {} - {}", success, message);
            }
            ServerMessage::ScheduleUpdated { timestamp } => {
                tracing::info!("Schedule updated at {}", timestamp);
                // TODO: Reload schedule from server
            }
            ServerMessage::Command { command } => {
                tracing::info!("Received command: {:?}", command);
                self.handle_command(command).await?;
            }
            ServerMessage::HeartbeatAck => {
                // Heartbeat acknowledged
            }
        }

        Ok(())
    }

    async fn handle_command(&self, command: NodeCommand) -> Result<()> {
        match command {
            NodeCommand::Play => {
                tracing::info!("Command: Play");
                // TODO: Send play command to MPV
            }
            NodeCommand::Pause => {
                tracing::info!("Command: Pause");
                // TODO: Send pause command to MPV
            }
            NodeCommand::Stop => {
                tracing::info!("Command: Stop");
                // TODO: Send stop command to MPV
            }
            NodeCommand::Seek { position_secs } => {
                tracing::info!("Command: Seek to {}", position_secs);
                // TODO: Send seek command to MPV
            }
            NodeCommand::LoadContent { content_id } => {
                tracing::info!("Command: Load content {}", content_id);
                // TODO: Load content from schedule cache
            }
            NodeCommand::ReloadSchedule => {
                tracing::info!("Command: Reload schedule");
                // TODO: Reload schedule from server
            }
            NodeCommand::Shutdown => {
                tracing::info!("Command: Shutdown");
                std::process::exit(0);
            }
        }

        Ok(())
    }
}
