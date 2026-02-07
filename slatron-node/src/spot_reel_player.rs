use crate::NodeState;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize, Debug, Clone)]
pub struct SpotReelResponse {
    pub id: Option<i32>,
    pub title: String,
    pub items: Vec<SpotReelItemResponse>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpotReelItemResponse {
    pub id: Option<i32>,
    pub item_type: String,
    pub item_path: String,
    pub display_duration_secs: i32,
    pub position: i32,
    pub title: Option<String>,
}

/// Play a spot reel in a loop until the cancellation token is triggered.
/// Fetches reel items from the server, then cycles through them.
pub async fn play_spot_reel(
    state: &NodeState,
    reel_id: i32,
    cancel: CancellationToken,
) -> Result<()> {
    // Fetch reel data from server
    let server_url = &state.config.server_url;
    let base_url = server_url
        .replace("ws://", "http://")
        .replace("wss://", "https://");
    let api_url = base_url.split("/ws").next().unwrap_or(&base_url);

    let url = format!("{}/api/spot-reels/{}", api_url, reel_id);
    let client = reqwest::Client::new();

    let reel: SpotReelResponse = client
        .get(&url)
        .send()
        .await?
        .json()
        .await
        .map_err(|e| anyhow!("Failed to fetch spot reel {}: {}", reel_id, e))?;

    if reel.items.is_empty() {
        tracing::warn!("Spot reel '{}' has no items, nothing to play", reel.title);
        // Just wait until cancelled
        cancel.cancelled().await;
        return Ok(());
    }

    tracing::info!(
        "Starting spot reel '{}' with {} items",
        reel.title,
        reel.items.len()
    );

    // Sort items by position
    let mut items = reel.items.clone();
    items.sort_by_key(|i| i.position);

    // Loop through items until cancelled
    loop {
        for item in &items {
            if cancel.is_cancelled() {
                tracing::info!("Spot reel '{}' cancelled", reel.title);
                return Ok(());
            }

            let item_title = item
                .title
                .as_deref()
                .unwrap_or(&item.item_path);

            tracing::info!(
                "Spot reel item: {} (type: {}, duration: {}s)",
                item_title,
                item.item_type,
                item.display_duration_secs
            );

            match item.item_type.as_str() {
                "image" => {
                    play_image_item(state, item, &cancel).await?;
                }
                "video" => {
                    play_video_item(state, item, &cancel).await?;
                }
                "web" => {
                    play_web_item(state, item, &cancel, api_url).await?;
                }
                other => {
                    tracing::warn!("Unknown spot reel item type: {}", other);
                }
            }

            if cancel.is_cancelled() {
                return Ok(());
            }
        }
    }
}

/// Play an image item - load in MPV and wait for display_duration_secs
async fn play_image_item(
    state: &NodeState,
    item: &SpotReelItemResponse,
    cancel: &CancellationToken,
) -> Result<()> {
    // MPV can display images as stills
    state.mpv.play(&item.item_path, None, Some(false))?;

    // Wait for display duration or cancellation
    let duration = Duration::from_secs(item.display_duration_secs as u64);
    tokio::select! {
        _ = tokio::time::sleep(duration) => {},
        _ = cancel.cancelled() => {},
    }

    Ok(())
}

/// Play a video item - load in MPV and wait for completion or display_duration_secs
async fn play_video_item(
    state: &NodeState,
    item: &SpotReelItemResponse,
    cancel: &CancellationToken,
) -> Result<()> {
    state.mpv.play(&item.item_path, None, Some(false))?;

    let duration = Duration::from_secs(item.display_duration_secs as u64);

    // Poll MPV idle state or wait for duration cap
    let poll_interval = Duration::from_millis(500);
    let start = tokio::time::Instant::now();

    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }

        if start.elapsed() >= duration {
            break;
        }

        // Check if MPV finished playing (went idle)
        if let Ok(true) = state.mpv.is_idle() {
            break;
        }

        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {},
            _ = cancel.cancelled() => { return Ok(()); },
        }
    }

    Ok(())
}

/// Play a web page item - capture to video and play
async fn play_web_item(
    state: &NodeState,
    item: &SpotReelItemResponse,
    cancel: &CancellationToken,
    _api_url: &str,
) -> Result<()> {
    // Try to capture the web page to a video file
    match crate::web_capture::capture_web_page(
        &item.item_path,
        item.display_duration_secs as u32,
        (1920, 1080),
    )
    .await
    {
        Ok(video_path) => {
            let path_str = video_path.to_string_lossy().to_string();
            state.mpv.play(&path_str, None, Some(false))?;

            let duration = Duration::from_secs(item.display_duration_secs as u64);
            tokio::select! {
                _ = tokio::time::sleep(duration) => {},
                _ = cancel.cancelled() => {},
            }
        }
        Err(e) => {
            tracing::error!("Failed to capture web page '{}': {}", item.item_path, e);
            tracing::info!("Skipping web item, waiting display duration before next item");

            // Still wait the duration to maintain timing
            let duration = Duration::from_secs(item.display_duration_secs as u64);
            tokio::select! {
                _ = tokio::time::sleep(duration) => {},
                _ = cancel.cancelled() => {},
            }
        }
    }

    Ok(())
}
