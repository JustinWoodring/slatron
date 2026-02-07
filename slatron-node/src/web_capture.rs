use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Duration;

/// Maximum age for cached web captures before re-capturing
const CACHE_MAX_AGE_SECS: u64 = 300; // 5 minutes

/// Capture a web page to a video file using Xvfb + headless Chromium + ffmpeg.
///
/// This function:
/// 1. Checks for a cached capture and returns it if fresh enough
/// 2. Spawns Xvfb on a virtual display
/// 3. Launches headless Chromium in kiosk mode on that display
/// 4. Waits for the page to load
/// 5. Captures the screen with ffmpeg x11grab
/// 6. Cleans up processes
/// 7. Returns the path to the output video
pub async fn capture_web_page(
    url: &str,
    duration_secs: u32,
    resolution: (u32, u32),
) -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    let output_path = cache_dir.join(format!("{}.mp4", url_hash(url)));

    // Check cache
    if output_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&output_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = modified.elapsed() {
                    if age.as_secs() < CACHE_MAX_AGE_SECS {
                        tracing::info!("Using cached web capture for: {}", url);
                        return Ok(output_path);
                    }
                }
            }
        }
    }

    tracing::info!(
        "Capturing web page: {} ({}x{}, {}s)",
        url,
        resolution.0,
        resolution.1,
        duration_secs
    );

    let display = ":99";
    let (width, height) = resolution;

    // 1. Start Xvfb
    let mut xvfb = tokio::process::Command::new("Xvfb")
        .arg(display)
        .arg("-screen")
        .arg("0")
        .arg(format!("{}x{}x24", width, height))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| {
            anyhow!(
                "Failed to start Xvfb (is it installed?): {}. \
                 Web capture requires: Xvfb, chromium/google-chrome, ffmpeg",
                e
            )
        })?;

    // Give Xvfb time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Start Chromium
    // Try chromium first, then google-chrome, then chromium-browser
    let chrome_cmd = find_chrome_binary();
    let mut chrome = tokio::process::Command::new(&chrome_cmd)
        .env("DISPLAY", display)
        .arg("--kiosk")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .arg("--disable-software-rasterizer")
        .arg("--disable-dev-shm-usage")
        .arg(format!("--window-size={},{}", width, height))
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| {
            let _ = xvfb.start_kill();
            anyhow!("Failed to start Chrome/Chromium ({}): {}", chrome_cmd, e)
        })?;

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 3. Capture with ffmpeg
    let ffmpeg_result = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-f")
        .arg("x11grab")
        .arg("-video_size")
        .arg(format!("{}x{}", width, height))
        .arg("-i")
        .arg(format!("{}+0,0", display))
        .arg("-t")
        .arg(duration_secs.to_string())
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("ultrafast")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(output_path.to_string_lossy().to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;

    // 4. Cleanup - kill processes regardless
    let _ = chrome.start_kill();
    let _ = xvfb.start_kill();

    // Wait for processes to exit
    let _ = chrome.wait().await;
    let _ = xvfb.wait().await;

    match ffmpeg_result {
        Ok(status) if status.success() => {
            tracing::info!("Web page captured successfully: {}", output_path.display());
            Ok(output_path)
        }
        Ok(status) => Err(anyhow!(
            "ffmpeg exited with code: {}. Ensure ffmpeg is installed with x11grab support.",
            status
        )),
        Err(e) => Err(anyhow!("Failed to run ffmpeg: {}", e)),
    }
}

/// Generate a deterministic hash for a URL to use as cache filename
fn url_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

/// Get or create the web capture cache directory
fn get_cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cache_dir = PathBuf::from(home).join(".slatron/web_cache");
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    Ok(cache_dir)
}

/// Find an available Chrome/Chromium binary
fn find_chrome_binary() -> String {
    for cmd in &[
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
    ] {
        if which_exists(cmd) {
            return cmd.to_string();
        }
    }
    // Default fallback
    "chromium".to_string()
}

/// Check if a command exists in PATH
fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
