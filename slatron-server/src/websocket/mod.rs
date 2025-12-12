use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use chrono::Utc;
use diesel::prelude::*;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::sync::Arc;
// use tokio::sync::RwLock;

// Server → Node messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "auth_response")]
    AuthResponse {
        success: bool,
        message: String,
        node_id: Option<i32>,
    },
    #[serde(rename = "schedule_updated")]
    ScheduleUpdated { timestamp: String },
    #[serde(rename = "command")]
    Command { command: NodeCommand },
    #[serde(rename = "heartbeat_ack")]
    HeartbeatAck,
}

// ...

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum NodeCommand {
    #[serde(rename = "play")]
    Play,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "stop")]
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
        playback_position_secs: Option<f32>,
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
    #[serde(rename = "log")]
    Log {
        level: String,
        message: String,
        target: String,
        timestamp: String,
    },
}

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

    let mut node_id: Option<i32> = None;
    let mut authenticated = false;

    // Clone state for use in the async blocks
    let state_clone = state.clone();

    // Spawn a task to forward messages from the channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(node_msg) = serde_json::from_str::<NodeMessage>(&text) {
                    match node_msg {
                        NodeMessage::Authenticate {
                            node_name,
                            secret_key,
                        } => {
                            // Authenticate node
                            let auth_result =
                                authenticate_node(&state_clone, &node_name, &secret_key).await;

                            match auth_result {
                                Ok(id) => {
                                    node_id = Some(id);
                                    authenticated = true;

                                    let _ = tx.send(ServerMessage::AuthResponse {
                                        success: true,
                                        message: "Authenticated successfully".to_string(),
                                        node_id: Some(id),
                                    });

                                    tracing::info!("Node {} authenticated", node_name);
                                }
                                Err(e) => {
                                    let _ = tx.send(ServerMessage::AuthResponse {
                                        success: false,
                                        message: e,
                                        node_id: None,
                                    });
                                }
                            }
                        }
                        NodeMessage::Heartbeat {
                            current_content_id,
                            playback_position_secs,
                            status,
                            cpu_usage_percent,
                            memory_usage_mb,
                            errors: _,
                        } => {
                            if authenticated {
                                if let Some(id) = node_id {
                                    if let Err(e) = update_node_status(
                                        &state_clone,
                                        id,
                                        &status,
                                        current_content_id,
                                        playback_position_secs,
                                    )
                                    .await
                                    {
                                        tracing::error!("Failed to update node status: {}", e);
                                    }

                                    let _ = tx.send(ServerMessage::HeartbeatAck);

                                    tracing::debug!(
                                        "Node {} heartbeat: status={}, content={:?}, pos={:?}, cpu={:.1}%, mem={:.1}MB",
                                        id,
                                        status,
                                        current_content_id,
                                        playback_position_secs,
                                        cpu_usage_percent,
                                        memory_usage_mb
                                    );
                                }
                            }
                        }
                        NodeMessage::RequestSchedule => {
                            if authenticated && node_id.is_some() {
                                // Send schedule update notification
                                let _ = tx.send(ServerMessage::ScheduleUpdated {
                                    timestamp: Utc::now().to_rfc3339(),
                                });
                            }
                        }
                        NodeMessage::ReportPaths { available_paths } => {
                            if authenticated {
                                if let Some(id) = node_id {
                                    let _ =
                                        update_node_paths(&state_clone, id, &available_paths).await;
                                }
                            }
                        }
                        NodeMessage::ContentError { content_id, error } => {
                            if authenticated {
                                tracing::error!(
                                    "Node {:?} content error: content_id={}, error={}",
                                    node_id,
                                    content_id,
                                    error
                                );
                            }
                        }
                        NodeMessage::Log {
                            level,
                            message,
                            target,
                            timestamp,
                        } => {
                            if authenticated {
                                if let Some(id) = node_id {
                                    let mut logs = state_clone.node_logs.write().await;
                                    let queue = logs.entry(id).or_default();
                                    if queue.len() >= 100 {
                                        queue.pop_front();
                                    }
                                    queue.push_back(crate::models::LogEntry {
                                        level,
                                        message,
                                        target,
                                        timestamp,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    // Clean up: mark node as offline
    if let Some(id) = node_id {
        let _ = mark_node_offline(&state, id).await;
        tracing::info!("Node {} disconnected", id);
    }
}

async fn authenticate_node(
    state: &AppState,
    node_name: &str,
    secret_key: &str,
) -> Result<i32, String> {
    use crate::schema::nodes::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    let node = dsl::nodes
        .filter(dsl::name.eq(node_name))
        .filter(dsl::secret_key.eq(secret_key))
        .select(crate::models::Node::as_select())
        .first(&mut conn)
        .map_err(|_| "Invalid credentials".to_string())?;

    // Update node status to online
    diesel::update(dsl::nodes.filter(dsl::id.eq(node.id.expect("Node ID missing"))))
        .set((
            dsl::status.eq("online"),
            dsl::last_heartbeat.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)
        .map_err(|_| "Failed to update node status".to_string())?;

    Ok(node.id.expect("Node ID missing"))
}

async fn update_node_status(
    state: &AppState,
    node_id: i32,
    status: &str,
    current_content_id: Option<i32>,
    playback_position_secs: Option<f32>,
) -> Result<(), String> {
    use crate::schema::nodes::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    diesel::update(dsl::nodes.filter(dsl::id.eq(node_id)))
        .set((
            dsl::status.eq(status),
            dsl::last_heartbeat.eq(Utc::now().naive_utc()),
            dsl::current_content_id.eq(current_content_id),
            dsl::playback_position_secs.eq(playback_position_secs),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update node status: {}", e))?;

    Ok(())
}

async fn update_node_paths(state: &AppState, node_id: i32, paths: &[String]) -> Result<(), String> {
    use crate::schema::nodes::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    let paths_json =
        serde_json::to_string(paths).map_err(|_| "Failed to serialize paths".to_string())?;

    diesel::update(dsl::nodes.filter(dsl::id.eq(node_id)))
        .set(dsl::available_paths.eq(paths_json))
        .execute(&mut conn)
        .map_err(|_| "Failed to update node paths".to_string())?;

    Ok(())
}

async fn mark_node_offline(state: &AppState, node_id: i32) -> Result<(), String> {
    use crate::schema::nodes::dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    diesel::update(dsl::nodes.filter(dsl::id.eq(node_id)))
        .set(dsl::status.eq("offline"))
        .execute(&mut conn)
        .map_err(|_| "Failed to mark node offline".to_string())?;

    Ok(())
}
