mod api;
mod auth;
mod config;
mod db;
mod models;
mod rhai_engine;
mod schema;
mod services;
mod websocket;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "slatron_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load("config.toml")?;
    tracing::info!("Loaded configuration");

    // Setup database
    let db_pool = db::create_pool(&config.database.url)?;
    db::run_migrations(&mut db_pool.get()?)?;
    tracing::info!("Database initialized");

    // Create app state
    let state = AppState {
        db: db_pool,
        config: Arc::new(config),
    };

    // Get address before moving state
    let addr = format!("{}:{}", state.config.server.host, state.config.server.port);

    // Build router
    let app = Router::new()
        // API routes
        .nest("/api", api::routes())
        // WebSocket endpoint
        .route("/ws", get(websocket::ws_handler))
        // Serve static files (React build)
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
