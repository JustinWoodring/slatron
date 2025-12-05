mod config;
mod heartbeat;
mod mpv_client;
mod rhai_engine;
mod schedule;
mod websocket_client;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::schedule::ScheduleCache;
use crate::websocket_client::WebSocketClient;

#[derive(Clone)]
pub struct NodeState {
    pub config: Arc<Config>,
    pub schedule_cache: Arc<RwLock<ScheduleCache>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "slatron_node=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load("config.toml")?;
    tracing::info!("Loaded configuration for node: {}", config.node_name);

    // Create node state
    let state = NodeState {
        config: Arc::new(config),
        schedule_cache: Arc::new(RwLock::new(ScheduleCache::new())),
    };

    // Start WebSocket client
    let mut ws_client = WebSocketClient::new(state.clone());
    ws_client.connect_and_run().await?;

    Ok(())
}
