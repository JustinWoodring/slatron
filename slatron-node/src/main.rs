mod config;
mod heartbeat;
mod mpv_client;
mod playback;
mod rhai_engine;
mod schedule;
mod screenshot;
mod websocket_client;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveTime};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::config::Config;

use crate::mpv_client::MpvClient;
use crate::schedule::ScheduleCache;
use crate::websocket_client::WebSocketClient;

#[derive(Clone)]
pub struct NodeState {
    pub config: Arc<Config>,
    pub schedule_cache: Arc<RwLock<ScheduleCache>>,
    pub node_id: Arc<RwLock<Option<i32>>>,
    pub mpv: Arc<MpvClient>,
    pub mpv_process: Arc<Mutex<Option<Child>>>,
    pub mpv_voice: Arc<MpvClient>,
    pub mpv_process_voice: Arc<Mutex<Option<Child>>>,
    pub log_sender: Arc<Mutex<Option<UnboundedSender<crate::websocket_client::NodeMessage>>>>,
    pub script_cache: Arc<RwLock<HashMap<i32, String>>>,
    pub script_name_cache: Arc<RwLock<HashMap<String, i32>>>,
    pub content_cache: Arc<RwLock<HashMap<i32, ServerContentItem>>>, // To lookup transformer_scripts
    pub current_content_id: Arc<RwLock<Option<i32>>>,
    pub global_settings: Arc<RwLock<HashMap<String, String>>>,
    pub active_scripts: Arc<RwLock<Vec<(String, rhai::Map)>>>,
    pub active_settings: Arc<RwLock<rhai::Map>>,
    pub schedule_update_notify: Arc<tokio::sync::Notify>,
}

// Log Visitor to extract message
struct LogVisitor {
    message: String,
}

impl LogVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl Visit for LogVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

// Log Layer
struct WebSocketLogLayer {
    sender: Arc<Mutex<Option<UnboundedSender<crate::websocket_client::NodeMessage>>>>,
}

impl<S> Layer<S> for WebSocketLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Ok(guard) = self.sender.lock() {
            if let Some(sender) = guard.as_ref() {
                let mut visitor = LogVisitor::new();
                event.record(&mut visitor);

                let log_msg = crate::websocket_client::NodeMessage::Log {
                    level: event.metadata().level().to_string(),
                    message: visitor.message,
                    target: event.metadata().target().to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                let _ = sender.send(log_msg);
            }
        }
    }
}

#[derive(Deserialize)]
struct ServerScheduleResponse {
    // schedule: Option<serde_json::Value>,
    blocks: Vec<ServerScheduleBlock>,
    content: Vec<ServerContentItem>,
    scripts: Vec<ServerScript>,
}

#[derive(Deserialize)]
struct ServerScheduleBlock {
    // id: i32,
    // schedule_id: i32,
    content_id: Option<i32>,
    day_of_week: Option<i32>,
    specific_date: Option<NaiveDate>,
    start_time: NaiveTime,
    duration_minutes: i32,
    script_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct ServerContentItem {
    pub id: i32,
    pub content_path: String,
    pub transformer_scripts: Option<String>,
}

#[derive(Deserialize)]
pub struct ServerScript {
    pub id: i32,
    pub name: String,
    pub script_content: String,
    pub script_type: String,
}

use clap::Parser;

#[derive(Parser)]
#[command(version, author = "SLATRON AUTHORS", about = "Slatron Node\nLicensed under AGPLv3\nCreated by SLATRON AUTHORS", long_about = None)]
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

    println!("Welcome to Slatron Node!");
    println!("It looks like you don't have a configuration file yet.");
    println!("Let's get you set up to connect to your Slatron Server.\n");

    let node_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Node Name")
        .default("Local".to_string())
        .interact_text()?;

    let server_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Slatron Server URL (WebSocket)")
        .default("ws://127.0.0.1:8080/ws".to_string())
        .interact_text()?;

    // Warn user about Secret Key
    println!("\n[!] You need a Secret Key from your Slatron Server to connect.");
    println!("    You can find this when creating a new Node in the Server Dashboard.\n");

    let secret_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Node Secret Key")
        .interact_text()?;

    let config_content = format!(
        r#"node_name = "{}"
server_url = "{}"
secret_key = "{}"
heartbeat_interval_secs = 5
schedule_poll_interval_secs = 60
mpv_socket_path = "/tmp/mpv-socket"
offline_mode_warning_hours = 24
"#,
        node_name, server_url, secret_key
    );

    println!("\nGenerating configuration file: node-config.toml");
    std::fs::write("node-config.toml", &config_content)?;
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

    // Create log sender shared state
    let log_sender = Arc::new(Mutex::new(None));
    let log_layer_sender = log_sender.clone();

    // Initialize tracing with custom layer
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "slatron_node=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(WebSocketLogLayer {
            sender: log_layer_sender,
        })
        .init();

    // Determine config path
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| "node-config.toml".to_string());

    // Check if config exists
    if std::fs::metadata(&config_path).is_err() {
        // If config arg was NOT explicitly passed (and we defaulted currently), AND we are in a TTY
        // Then try onboarding.
        if cli.config.is_none() && console::user_attended() {
            match run_onboarding() {
                Ok(_cfg) => {
                    // fall through, logic below picks up file or we could use cfg directly but
                    // main flow loads from file path.
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
    } else if std::fs::metadata("node-config.toml").is_ok() {
        "node-config.toml".to_string()
    } else {
        eprintln!("Error: Configuration file '{}' not found.", config_path);
        eprintln!("Run with --generate-config to see a template.");
        std::process::exit(1);
    };

    // Load configuration
    let config = Config::load(&effective_config_path)?;
    tracing::info!("Loaded configuration for node: {}", config.node_name);

    // Spawn MPV (Main)
    tracing::info!("Spawning MPV (Main)...");
    let mut mpv_child = match crate::mpv_client::spawn_mpv(&config.mpv_socket_path) {
        Ok(child) => {
            tracing::info!("MPV (Main) spawned successfully");
            Some(child)
        }
        Err(e) => {
            tracing::error!(
                "Failed to spawn MPV (Main): {}. Continuing without managed process (assuming manual start).",
                e
            );
            None
        }
    };

    // Spawn MPV (Voice)
    tracing::info!("Spawning MPV (Voice)...");
    let mut mpv_voice_child = match crate::mpv_client::spawn_mpv(&config.voice_mpv_socket_path) {
        Ok(child) => {
            tracing::info!("MPV (Voice) spawned successfully");
            Some(child)
        }
        Err(e) => {
            tracing::error!(
                "Failed to spawn MPV (Voice): {}. Voice injection might fail.",
                e
            );
            None
        }
    };

    // Capture output (Main)
    if let Some(child) = mpv_child.as_mut() {
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        tracing::info!(target: "slatron_node::mpv_main", "{}", l);
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        tracing::error!(target: "slatron_node::mpv_main", "{}", l);
                    }
                }
            });
        }
    }

    // Capture output (Voice)
    if let Some(child) = mpv_voice_child.as_mut() {
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        tracing::info!(target: "slatron_node::mpv_voice", "{}", l);
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        tracing::error!(target: "slatron_node::mpv_voice", "{}", l);
                    }
                }
            });
        }
    }

    // Create node state
    let state = NodeState {
        config: Arc::new(config.clone()),
        schedule_cache: Arc::new(RwLock::new(ScheduleCache::new())),
        node_id: Arc::new(RwLock::new(None)),
        mpv: Arc::new(MpvClient::new(config.mpv_socket_path.clone())),
        mpv_process: Arc::new(Mutex::new(mpv_child)),
        mpv_voice: Arc::new(MpvClient::new(config.voice_mpv_socket_path.clone())),
        mpv_process_voice: Arc::new(Mutex::new(mpv_voice_child)),
        log_sender: log_sender,
        script_cache: Arc::new(RwLock::new(HashMap::new())),
        script_name_cache: Arc::new(RwLock::new(HashMap::new())),
        content_cache: Arc::new(RwLock::new(HashMap::new())),
        current_content_id: Arc::new(RwLock::new(None)),
        global_settings: Arc::new(RwLock::new(HashMap::new())),
        active_scripts: Arc::new(RwLock::new(Vec::new())),
        active_settings: Arc::new(RwLock::new(rhai::Map::new())),
        schedule_update_notify: Arc::new(tokio::sync::Notify::new()),
    };

    // Start WebSocket client
    let mut ws_client = WebSocketClient::new(state.clone());

    // We don't use the result of spawn, no need for var assignment
    tokio::spawn(async move {
        if let Err(e) = ws_client.connect_and_run().await {
            tracing::error!("WebSocket client error: {}", e);
        }
    });

    // Start schedule poller
    let state_clone_poll = state.clone();
    tokio::spawn(async move {
        poll_schedule(state_clone_poll).await;
    });

    // Start playback loop with shutdown signal handling
    tokio::select! {
        _ = playback_loop(state.clone()) => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
    }

    // Kill MPV process if managed
    if let Ok(mut child_lock) = state.mpv_process.lock() {
        if let Some(mut child) = child_lock.take() {
            tracing::info!("Killing managed MPV process...");
            let _ = child.kill();
            let _ = child.wait(); // Prevent zombie process
        }
    }

    Ok(())
}

async fn poll_schedule(state: NodeState) {
    let client = reqwest::Client::new();
    let poll_interval_secs = state.config.schedule_poll_interval_secs;

    // 1. Wait for Node ID (Connection) with fast polling
    // 1. Wait for Node ID (Connection) with fast polling
    let node_id = loop {
        if let Some(id) = *state.node_id.read().await {
            break Some(id);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };

    // 2. Initial Fetch Immediately upon connection
    if let Some(id) = node_id {
        tracing::info!(
            "Node connected with ID: {}, fetching initial schedule...",
            id
        );
        fetch_and_update_schedule(&client, &state, id).await;
    }

    // 3. Aligned Polling Loop
    loop {
        // Calculate sleep time to align with next interval boundary
        let now = chrono::Utc::now();
        let current_secs = now.timestamp() as u64; // Unix timestamp in seconds

        let interval = poll_interval_secs;

        // Next boundary is the smallest multiple of `interval` strictly greater than `current_secs`
        // Exception: If we just fetched (which might be slightly past boundary due to execution time),
        // we want the NEXT boundary.
        // Logic: next_boundary = (current_secs / interval + 1) * interval
        let next_boundary = (current_secs / interval + 1) * interval;

        let wait_secs = next_boundary.saturating_sub(current_secs);

        // Handle sub-second alignment for precision (optional but nice)
        let now_ns = now.timestamp_subsec_nanos();

        // Total wait: Wait full seconds + remaining nanoseconds to align to second 0
        let wait_duration = Duration::from_secs(wait_secs) - Duration::from_nanos(now_ns as u64);

        // Safety: If calculation is wonky or negative (shouldn't be with saturating_sub), default to interval
        let sleep_duration = if wait_duration.as_millis() > 0 {
            wait_duration
        } else {
            Duration::from_secs(interval)
        };

        // tracing::debug!("Sleeping for {:?} until next aligned poll", sleep_duration);
        tokio::select! {
            _ = tokio::time::sleep(sleep_duration) => {},
            _ = state.schedule_update_notify.notified() => {
                 tracing::info!("Forced schedule update triggered");
            }
        }

        // Re-read node_id in case it changed (unlikely but safe)
        if let Some(id) = *state.node_id.read().await {
            fetch_and_update_schedule(&client, &state, id).await;
        } else {
            // If lost connection/id, go back to wait loop? Or just wait.
        }
    }
}

async fn fetch_and_update_schedule(client: &reqwest::Client, state: &NodeState, node_id: i32) {
    // Convert WS URL to HTTP URL
    let http_base = state
        .config
        .server_url
        .replace("ws://", "http://")
        .replace("/ws", "");

    // Fetch Schedule
    let url = format!("{}/api/nodes/{}/schedule", http_base, node_id);
    let today = chrono::Utc::now().date_naive();

    if let Ok(res) = client.get(&url).send().await {
        if res.status().is_success() {
            if let Ok(response) = res.json::<ServerScheduleResponse>().await {
                // Update cache
                let mut cache = state.schedule_cache.write().await;
                let mut script_cache = state.script_cache.write().await;
                let mut script_name_cache = state.script_name_cache.write().await;
                let mut content_cache = state.content_cache.write().await;

                let mut content_map = HashMap::new();
                for item in response.content {
                    content_map.insert(item.id, item.content_path.clone());
                    content_cache.insert(item.id, item);
                }

                for script in response.scripts {
                    script_cache.insert(script.id, script.script_content);
                    script_name_cache.insert(script.name, script.id);
                }

                let mut blocks_by_date: HashMap<NaiveDate, Vec<crate::schedule::ScheduleBlock>> =
                    HashMap::new();

                for server_block in response.blocks {
                    let content_path = server_block
                        .content_id
                        .and_then(|cid| content_map.get(&cid).cloned());

                    let block = crate::schedule::ScheduleBlock {
                        start_time: server_block.start_time,
                        duration_minutes: server_block.duration_minutes,
                        content_id: server_block.content_id,
                        content_path,
                        script_id: server_block.script_id,
                    };

                    if let Some(date) = server_block.specific_date {
                        blocks_by_date.entry(date).or_default().push(block.clone());
                    } else if let Some(dow) = server_block.day_of_week {
                        let today_dow = today.weekday().number_from_monday() as i32;
                        let today_dow_ui_index = today_dow - 1; // 0=Mon

                        if dow == today_dow_ui_index {
                            blocks_by_date.entry(today).or_default().push(block.clone());
                        }
                    }
                }

                for (date, blocks) in blocks_by_date {
                    cache.update(date, blocks);
                }
                tracing::info!("Schedule updated from server");
            }
        }
    }

    // Fetch Global Settings
    // Fetch Global Settings
    let settings_url = format!("{}/api/settings", http_base);
    if let Ok(res) = client.get(&settings_url).send().await {
        if res.status().is_success() {
            #[derive(Deserialize)]
            struct SettingItem {
                key: String,
                value: String,
            }
            if let Ok(settings_list) = res.json::<Vec<SettingItem>>().await {
                let mut settings_guard = state.global_settings.write().await;
                for s in settings_list {
                    settings_guard.insert(s.key, s.value);
                }
                tracing::info!("Global settings updated");
            }
        }
    }
}

async fn playback_loop(state: NodeState) {
    let mut last_content_id: Option<i32> = None;
    let loop_interval = Duration::from_secs(1);

    loop {
        tokio::time::sleep(loop_interval).await;

        let now = chrono::Utc::now();
        let today = now.date_naive();
        let time = now.time();

        let block_opt = {
            let cache = state.schedule_cache.read().await;
            cache.get_current_block(today, time).cloned()
        };

        if let Some(block) = block_opt {
            if block.content_id != last_content_id {
                if block.content_id.is_none() {
                    tracing::info!("Status: Entering DJ Block (Waiting for Dynamic Content)");
                } else {
                    tracing::info!("Content changed to {:?}", block.content_id);
                }
                last_content_id = block.content_id;

                if let Some(content_id) = block.content_id {
                    // Pass the block's content path (which might be None, play_content resolves it)
                    if let Err(e) =
                        crate::playback::play_content(&state, content_id, block.content_path).await
                    {
                        tracing::error!("Failed to play content: {}", e);
                    }
                } else {
                    // Content ID is None but there is a block? Stop.
                    crate::playback::stop_playback(&state).await;
                }
            }
        } else {
            // Nothing scheduled
            if last_content_id.is_some() {
                tracing::info!("Schedule ended, stopping playback");
                crate::playback::stop_playback(&state).await;
                last_content_id = None;
            }
        }
    }
}
