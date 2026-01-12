use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::time::interval;

pub async fn run() {
    let mut tick = interval(Duration::from_secs(5 * 60)); // 5 minutes

    // The first tick completes immediately, satisfying the "on startup" requirement.
    loop {
        tick.tick().await;

        // Run blocking cleanup in a separate thread to avoid blocking the async runtime
        if let Err(e) = tokio::task::spawn_blocking(move || {
            let tts_dir = PathBuf::from("static/tts");
            cleanup_tts(&tts_dir);
        }).await {
            tracing::error!("TTS Cleanup task panic: {}", e);
        }
    }
}

fn cleanup_tts(dir: &PathBuf) {
    // Check if directory exists
    if !dir.exists() {
        return;
    }

    tracing::debug!("Running TTS cleanup task...");

    match std::fs::read_dir(dir) {
        Ok(entries) => {
            let mut count = 0;
            let mut errors = 0;
            let now = SystemTime::now();
            let threshold = Duration::from_secs(15 * 60); // 15 minutes old

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        // Check extension to be safe (wav, pcm, mp3)
                        if let Some(ext) = path.extension() {
                             let ext_str = ext.to_string_lossy();
                             if ext_str != "wav" && ext_str != "pcm" && ext_str != "mp3" {
                                 continue;
                             }
                        }

                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(age) = now.duration_since(modified) {
                                    if age > threshold {
                                        if let Err(e) = std::fs::remove_file(&path) {
                                            tracing::error!("Failed to delete old TTS file {:?}: {}", path, e);
                                            errors += 1;
                                        } else {
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if count > 0 || errors > 0 {
                tracing::info!("TTS Cleanup: Removed {} files. {} errors.", count, errors);
            }
        }
        Err(e) => {
            tracing::error!("Failed to read TTS directory for cleanup: {}", e);
        }
    }
}
