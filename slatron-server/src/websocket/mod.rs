use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use diesel::prelude::*;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    LoadContent {
        content_id: i32,
        path: Option<String>,
    },
    #[serde(rename = "queue_content")]
    QueueContent {
        content_id: i32,
        path: Option<String>,
    },
    #[serde(rename = "reload_schedule")]
    ReloadSchedule,
    #[serde(rename = "shutdown")]
    Shutdown,
    #[serde(rename = "inject_audio")]
    InjectAudio { url: String, mix: bool },
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
        playback_duration_secs: Option<f32>,
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
    #[serde(rename = "screenshot")]
    Screenshot { image_base64: String },
}

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    const OUTBOUND_CHANNEL_CAPACITY: usize = 256;
    const AUTH_DEADLINE: Duration = Duration::from_secs(30);
    // Window for the forwarder to flush queued messages (e.g. an auth-failure
    // response) before the socket is torn down.
    const TEARDOWN_FLUSH_GRACE: Duration = Duration::from_secs(1);

    let (mut sender, mut receiver) = socket.split();
    // Bounded: a stalled client now applies backpressure to senders instead of
    // letting the queue grow without limit.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(OUTBOUND_CHANNEL_CAPACITY);

    // Shared with the auth-deadline timer below.
    let authed = Arc::new(AtomicBool::new(false));
    let auth_deadline_hit = Arc::new(tokio::sync::Notify::new());
    // Set once authenticated so cleanup knows which node to mark offline.
    let node_id_slot: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));

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

    // Auth deadline: an unauthenticated connection must not hold a socket and
    // task open forever.
    let deadline_task = {
        let authed = Arc::clone(&authed);
        let deadline_hit = Arc::clone(&auth_deadline_hit);
        tokio::spawn(async move {
            tokio::time::sleep(AUTH_DEADLINE).await;
            if !authed.load(Ordering::SeqCst) {
                tracing::warn!(
                    "Closing node connection: not authenticated within {}s",
                    AUTH_DEADLINE.as_secs()
                );
                deadline_hit.notify_one();
            }
        })
    };

    // Bridge for external broadcasters (heartbeat monitor, API): they keep the
    // unbounded sender API stored in connected_nodes, but their messages are
    // pushed into this socket's bounded channel. On a stalled client the
    // bounded channel fills and broadcasts are dropped instead of being
    // buffered without limit.
    let (broadcast_tx, mut broadcast_rx) =
        tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let broadcast_tx_for_map = broadcast_tx.clone();
    let bounded_tx = tx.clone();
    let pump_task = tokio::spawn(async move {
        while let Some(msg) = broadcast_rx.recv().await {
            match bounded_tx.try_send(msg) {
                Ok(()) => {}
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    tracing::warn!("Node outbound channel full; dropping broadcast message");
                }
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => break,
            }
        }
    });

    // Handle incoming messages
    let authed_for_recv = Arc::clone(&authed);
    let node_id_slot_for_recv = Arc::clone(&node_id_slot);
    let broadcast_tx_for_recv = broadcast_tx_for_map;
    let mut recv_task = tokio::spawn(async move {
        let authed = authed_for_recv;
        let id_slot = node_id_slot_for_recv;
        let broadcast_tx = broadcast_tx_for_recv;
        let mut node_id: Option<i32> = None;
        let mut authenticated = false;

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
                                    authed.store(true, Ordering::SeqCst);
                                    *id_slot.lock().expect("node id slot poisoned") = Some(id);

                                    let _ = tx.send(ServerMessage::AuthResponse {
                                        success: true,
                                        message: "Authenticated successfully".to_string(),
                                        node_id: Some(id),
                                    })
                                    .await;

                                    tracing::info!("Node {} authenticated", node_name);

                                    // Register in connected_nodes
                                    {
                                        let mut nodes = state_clone.connected_nodes.write().await;
                                        nodes.insert(id, broadcast_tx.clone());
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(ServerMessage::AuthResponse {
                                        success: false,
                                        message: e,
                                        node_id: None,
                                    })
                                    .await;
                                    tracing::warn!(
                                        "Node {} failed authentication; closing connection",
                                        node_name
                                    );
                                    // End the recv loop so handle_socket
                                    // flushes the failure response and drops
                                    // the socket.
                                    break;
                                }
                            }
                        }
                        NodeMessage::Heartbeat {
                            current_content_id,
                            playback_position_secs,
                            playback_duration_secs,
                            status,
                            cpu_usage_percent,
                            memory_usage_mb,
                            errors,
                        } => {
                            if authenticated {
                                if let Some(id) = node_id {
                                    if let Err(e) = update_node_status(
                                        &state_clone,
                                        id,
                                        &status,
                                        current_content_id,
                                        playback_position_secs,
                                        playback_duration_secs,
                                        cpu_usage_percent,
                                        memory_usage_mb,
                                        if errors.is_empty() {
                                            None
                                        } else {
                                            Some(errors.join("; "))
                                        },
                                    )
                                    .await
                                    {
                                        tracing::error!("Failed to update node status: {}", e);
                                    }

                                    let _ = tx.send(ServerMessage::HeartbeatAck).await;

                                    tracing::debug!(
                                        "Node {} heartbeat: status={}, content={:?}, pos={:?}, dur={:?}, cpu={:.1}%, mem={:.1}MB",
                                        id,
                                        status,
                                        current_content_id,
                                        playback_position_secs,
                                        playback_duration_secs,
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
                                })
                                .await;
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
                        NodeMessage::Screenshot { image_base64 } => {
                            if authenticated {
                                if let Some(id) = node_id {
                                    // Determine screenshot path
                                    let ui_path = state_clone
                                        .config
                                        .server
                                        .ui_path
                                        .clone()
                                        .unwrap_or_else(|| "static".to_string());

                                    let screenshots_dir =
                                        std::path::Path::new(&ui_path).join("screenshots");
                                    if !screenshots_dir.exists() {
                                        let _ = std::fs::create_dir_all(&screenshots_dir);
                                    }

                                    let file_path =
                                        screenshots_dir.join(format!("node_{}.jpg", id));

                                    // Decode and save
                                    match general_purpose::STANDARD.decode(&image_base64) {
                                        Ok(bytes) => {
                                            if let Err(e) = std::fs::write(&file_path, &bytes) {
                                                tracing::error!("Failed to save screenshot: {}", e);
                                            } else {
                                                // tracing::debug!("Saved screenshot for node {}", id);
                                                // Update valid_screenshot boolean/timestamp?
                                                // Currently UI just needs to fetch /screenshots/node_X.jpg
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "Failed to decode screenshot base64: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Wait for either task to finish, or the auth deadline to fire
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
            pump_task.abort();
            deadline_task.abort();
        }
        _ = &mut recv_task => {
            // Recv ended: client disconnected, auth failed, or protocol error.
            // The forwarder can't exit via channel close (the broadcast bridge
            // holds a producer), so give it a brief window to flush queued
            // messages (e.g. an auth-failure response), then tear it down.
            let _ = tokio::time::timeout(TEARDOWN_FLUSH_GRACE, &mut send_task).await;
            send_task.abort();
            pump_task.abort();
            deadline_task.abort();
        }
        _ = auth_deadline_hit.notified() => {
            // Never authenticated in time: aborting the forwarder and recv
            // task drops both halves of the socket, closing the connection.
            recv_task.abort();
            send_task.abort();
            pump_task.abort();
            // deadline_task already completed; abort is a harmless no-op.
            deadline_task.abort();
        }
    }

    // Clean up: mark node as offline and remove from connected_nodes
    let node_id = *node_id_slot.lock().expect("node id slot poisoned");
    if let Some(id) = node_id {
        let _ = mark_node_offline(&state, id).await;
        // Remove from connected_nodes
        {
            let mut nodes = state.connected_nodes.write().await;
            nodes.remove(&id);
        }
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
    playback_duration_secs: Option<f32>,
    _cpu_usage_percent: f64,
    _memory_usage_mb: f64,
    _error_msg: Option<String>,
) -> Result<(), String> {
    use crate::schema::nodes::dsl as n_dsl;

    let mut conn = state
        .db
        .get()
        .map_err(|_| "Database connection error".to_string())?;

    diesel::update(n_dsl::nodes.filter(n_dsl::id.eq(node_id)))
        .set((
            n_dsl::status.eq(status),
            n_dsl::last_heartbeat.eq(Utc::now().naive_utc()),
            n_dsl::current_content_id.eq(current_content_id),
            n_dsl::playback_position_secs.eq(playback_position_secs),
            n_dsl::playback_duration_secs.eq(playback_duration_secs),
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
