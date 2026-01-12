mod api;
mod auth;
mod config;
mod db;
mod models;
mod rhai_engine;
mod schema;
mod seeding;
mod services;
mod websocket;

use anyhow::Result;
use axum::{routing::get, Router};

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::DbPool;
use crate::services::{
    ai_service::AiService, script_service::ScriptService, tts_service::TtsService,
};
use crate::websocket::ServerMessage;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<Config>,
    // Store last 100 logs per node_id
    pub node_logs: Arc<RwLock<HashMap<i32, VecDeque<crate::models::LogEntry>>>>,
    pub ai_service: Arc<AiService>,
    pub tts_service: Arc<TtsService>,
    pub script_service: Arc<ScriptService>,
    pub connected_nodes: Arc<RwLock<HashMap<i32, UnboundedSender<ServerMessage>>>>,
    // Track recent plays globally (Content IDs)
    pub recent_plays: Arc<RwLock<VecDeque<i32>>>,
}

use clap::Parser;

#[derive(Parser)]
#[command(version, author = "SLATRON AUTHORS", about = "Slatron Server\nLicensed under AGPLv3\nCreated by SLATRON AUTHORS", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Generate a default configuration template to stdout
    #[arg(long)]
    generate_config: bool,
}

fn run_onboarding() -> Result<Config> {
    use dialoguer::{theme::ColorfulTheme, Input};

    println!("Welcome to Slatron Server!");
    println!("It looks like you don't have a configuration file yet.");
    println!("Let's get you set up.\n");

    let host: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Server Host")
        .default("0.0.0.0".to_string())
        .interact_text()?;

    let port: u16 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Server Port")
        .default(8080)
        .interact_text()?;

    let db_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Database URL")
        .default("sqlite://slatron.db".to_string())
        .interact_text()?;

    // Generate random JWT secret
    let jwt_secret = uuid::Uuid::new_v4().to_string();

    let config_content = format!(
        r#"[server]
host = "{}"
port = {}

[server.https]
enabled = false
cert_path = "certs/cert.pem"
key_path = "certs/key.pem"

[database]
url = "{}"

[jwt]
secret = "{}"
expiration_hours = 24

[logging]
level = "info"
"#,
        host, port, db_url, jwt_secret
    );

    println!("\nGenerating configuration file: server-config.toml");
    std::fs::write("server-config.toml", &config_content)?;
    println!("Configuration saved successfully!");
    println!("----------------------------------------\n");

    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    if cli.generate_config {
        println!("{}", Config::default_template());
        return Ok(());
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "slatron_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    #[cfg(feature = "ml-support")]
    {
        // Extract embedded ML model
        let data_dir = std::path::Path::new("data");
        // Check if snac.onnx exists inside data dir. If not, extract.
        if !data_dir.join("snac.onnx").exists() {
            tracing::info!("Extracting embedded ML models to {:?}", data_dir);
            if !data_dir.exists() {
                std::fs::create_dir_all(data_dir)?;
            }

            let zip_data = include_bytes!(concat!(env!("OUT_DIR"), "/model_assets.zip"));
            let cursor = std::io::Cursor::new(zip_data);
            let mut archive = zip::ZipArchive::new(cursor)?;

            archive.extract(data_dir)?;
            tracing::info!("ML models extracted.");
        }
    }

    // Determine config path
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| "server-config.toml".to_string());

    // Check if config exists
    if std::fs::metadata(&config_path).is_err() {
        // If config arg was NOT explicitly passed (and we defaulted currently), AND we are in a TTY
        // Then try onboarding.
        // clap doesn't tell us easily if it was default, so we check if cli.config is None.
        if cli.config.is_none() && console::user_attended() {
            match run_onboarding() {
                Ok(_cfg) => {
                    // Update the path we are "loading from" conceptually, though we return the object directly
                    // Actually, run_onboarding saves to "server-config.toml", so let's stick with that.
                    // But we already loaded the struct, so we can just return it.
                    // However, main flow expects to load it. Let's just fall through or return.
                    // simpler: run onboarding, then update config_path to "server-config.toml"
                    // checking if it exists now.
                }
                Err(e) => {
                    eprintln!("Onboarding failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    // Check again
    let effective_config_path = if std::fs::metadata(&config_path).is_ok() {
        config_path
    } else if std::fs::metadata("server-config.toml").is_ok() {
        "server-config.toml".to_string()
    } else {
        eprintln!("Error: Configuration file '{}' not found.", config_path);
        eprintln!("Run with --generate-config to see a template.");
        std::process::exit(1);
    };

    // Load configuration
    let config = Config::load(&effective_config_path)?;
    tracing::info!("Loaded configuration from {}", effective_config_path);

    // Setup database
    let db_pool = db::create_pool(&config.database.url)?;
    db::run_migrations(&mut db_pool.get()?)?;
    // Seed default values
    seeding::seed_defaults(&db_pool)?;
    tracing::info!("Database initialized and seeded");

    // Create app state
    let state = AppState {
        db: db_pool,
        config: Arc::new(config.clone()),
        node_logs: Arc::new(RwLock::new(HashMap::new())),
        ai_service: Arc::new(AiService::new()),
        tts_service: Arc::new(TtsService::new()),
        script_service: Arc::new(ScriptService::new()),
        connected_nodes: Arc::new(RwLock::new(HashMap::new())),
        recent_plays: Arc::new(RwLock::new(VecDeque::new())),
    };

    // Spawn heartbeat monitor
    tokio::spawn(services::heartbeat_monitor::run(state.clone()));

    // Spawn TTS cleanup task
    tokio::spawn(services::tts_cleanup::run());

    // Get address before moving state
    let addr = format!("{}:{}", state.config.server.host, state.config.server.port);

    // Determine UI path
    let static_path = if let Some(path) = &state.config.server.ui_path {
        tracing::info!("Using configured UI path: {}", path);
        path.clone()
    } else {
        #[cfg(feature = "embed-ui")]
        {
            tracing::info!("Using embedded UI");
            let output_dir = std::path::Path::new("embedded_ui");
            if output_dir.exists() {
                let _ = std::fs::remove_dir_all(output_dir);
            }
            std::fs::create_dir_all(output_dir)?;

            let zip_data = include_bytes!(concat!(env!("OUT_DIR"), "/ui.zip"));
            let cursor = std::io::Cursor::new(zip_data);
            let mut archive = zip::ZipArchive::new(cursor)?;

            archive.extract(output_dir)?;
            tracing::info!("Extracted embedded UI to {:?}", output_dir);
            output_dir.to_string_lossy().to_string()
        }
        #[cfg(not(feature = "embed-ui"))]
        {
            tracing::info!("Using default static UI path: static");
            "static".to_string()
        }
    };

    // Build router
    let app = Router::new()
        // API routes
        // Pass state.clone() to allow middleware configuration
        .nest("/api", api::routes(state.clone()))
        // WebSocket endpoint
        .route("/ws", get(websocket::ws_handler))
        // Serve static files (React build) with fallback to index.html for SPA routing
        .fallback_service(
            ServeDir::new(&static_path)
                .not_found_service(ServeFile::new(format!("{}/index.html", static_path))),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener_address: std::net::SocketAddr = addr
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid host/port: {}", e))?;

    if let Some(https_config) = &config.server.https {
        if https_config.enabled {
            use axum_server::tls_rustls::RustlsConfig;

            tracing::info!("Starting server in HTTPS mode on {}", addr);

            // Validate cert paths
            if !std::path::Path::new(&https_config.cert_path).exists() {
                anyhow::bail!("Certificate file not found: {}", https_config.cert_path);
            }
            if !std::path::Path::new(&https_config.key_path).exists() {
                anyhow::bail!("Key file not found: {}", https_config.key_path);
            }

            let tls_config =
                RustlsConfig::from_pem_file(&https_config.cert_path, &https_config.key_path)
                    .await?;

            axum_server::bind_rustls(listener_address, tls_config)
                .serve(app.into_make_service())
                .await?;

            return Ok(());
        }
    }

    // Default HTTP mode
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {} (HTTP)", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
