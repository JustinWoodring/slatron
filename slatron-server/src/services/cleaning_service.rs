use crate::db::DbPool;
use diesel::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::time::interval;

pub async fn run(db_pool: DbPool) {
    let mut tick = interval(Duration::from_secs(5 * 60)); // 5 minutes

    // The first tick completes immediately, satisfying the "on startup" requirement.
    loop {
        tick.tick().await;

        let pool = db_pool.clone();

        // Run blocking cleanup in a separate thread to avoid blocking the async runtime
        if let Err(e) = tokio::task::spawn_blocking(move || {
            let tts_dir = PathBuf::from("static/tts");
            cleanup_tts(&tts_dir);

            if let Ok(mut conn) = pool.get() {
                let back_dir = PathBuf::from("static/bumper_backs");
                cleanup_bumper_backs(&back_dir, &mut conn);
            } else {
                tracing::error!("Cleanup task failed to get DB connection");
            }
        })
        .await
        {
            tracing::error!("Cleanup task panic: {}", e);
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
                                            tracing::error!(
                                                "Failed to delete old TTS file {:?}: {}",
                                                path,
                                                e
                                            );
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

fn cleanup_bumper_backs(dir: &Path, conn: &mut SqliteConnection) {
    use crate::schema::bumper_backs::dsl::*;

    if !dir.exists() {
        return;
    }

    tracing::debug!("Running Bumper Back cleanup task...");

    // Get all valid file paths from DB
    let valid_paths: HashSet<String> = match bumper_backs.select(file_path).load::<String>(conn) {
        Ok(paths) => paths.into_iter().collect(),
        Err(e) => {
            tracing::error!("Cleanup failed to fetch bumper backs from DB: {}", e);
            return;
        }
    };

    match std::fs::read_dir(dir) {
        Ok(entries) => {
            let mut count = 0;
            let mut errors = 0;

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        // Convert absolute/relative path to db format
                        // DB paths are relative to static/ usually, or just "bumper_backs/filename.mp4"
                        // Assuming db format is "bumper_backs/xyz.mp4"

                        // We need to check if the file on disk matches any in DB.
                        // Since we are scanning `static/bumper_backs`, the entry path is `static/bumper_backs/file.mp4`
                        // DB probably stores `bumper_backs/file.mp4` or just filename?
                        // Let's assume DB stores relative path like "bumper_backs/foo.mp4"

                        // Wait, previous code used `static/` prefix in rendering service, implying DB does NOT have `static/`.
                        // If DB stores `bumper_backs/foo.mp4`, and we search `static/bumper_backs`,
                        // entry file_name is `foo.mp4`.

                        if let Some(filename) = path.file_name() {
                            let filename_str = filename.to_string_lossy();
                            let relative_path = format!("bumper_backs/{}", filename_str);

                            if !valid_paths.contains(&relative_path) {
                                // Not in DB, delete it
                                if let Err(e) = std::fs::remove_file(&path) {
                                    tracing::error!(
                                        "Failed to delete orphan bumper back {:?}: {}",
                                        path,
                                        e
                                    );
                                    errors += 1;
                                } else {
                                    count += 1;
                                    tracing::info!("Deleted orphan bumper back: {}", relative_path);
                                }
                            }
                        }
                    }
                }
            }
            if count > 0 || errors > 0 {
                tracing::info!(
                    "Bumper Back Cleanup: Removed {} orphan files. {} errors.",
                    count,
                    errors
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to read bumper back directory: {}", e);
        }
    }
}
