use crate::NodeState;
use anyhow::{anyhow, Result};

pub async fn play_content(
    state: &NodeState,
    content_id: i32,
    path_override: Option<String>,
) -> Result<()> {
    // 1. Resolve Content Path
    let content_path = if let Some(p) = path_override {
        p
    } else {
        let cache = state.content_cache.read().await;
        if let Some(item) = cache.get(&content_id) {
            item.content_path.clone()
        } else {
            return Err(anyhow!("Content ID {} not found in cache", content_id));
        }
    };

    tracing::info!(
        "Preparing to play content: {} (ID: {})",
        content_path,
        content_id
    );

    // 2. Unload Previous Scripts
    unload_active_scripts(state).await;

    // 3. Prepare New Settings & Scripts
    let mut settings = rhai::Map::new();
    let mut current_scripts_to_run: Vec<(String, rhai::Map)> = Vec::new();

    // Inject Global Settings
    {
        let global_guard = state.global_settings.read().await;
        for (k, v) in global_guard.iter() {
            settings.insert(k.clone().into(), v.clone().into());
        }
    }

    // Collect Global Scripts
    {
        let global_guard = state.global_settings.read().await;
        let mut global_script_ids = Vec::new();

        if let Some(json_str) = global_guard.get("global_active_scripts") {
            if let Ok(names) = serde_json::from_str::<Vec<String>>(json_str) {
                let name_cache = state.script_name_cache.read().await;
                for name in names {
                    if let Some(id) = name_cache.get(&name) {
                        global_script_ids.push(*id);
                    }
                }
            } else if let Ok(ids) = serde_json::from_str::<Vec<i32>>(json_str) {
                global_script_ids = ids;
            } else if let Ok(id) = json_str.parse::<i32>() {
                global_script_ids.push(id);
            }
        }

        if !global_script_ids.is_empty() {
            let script_cache = state.script_cache.read().await;
            for script_id in global_script_ids {
                if let Some(script_content) = script_cache.get(&script_id) {
                    current_scripts_to_run.push((script_content.clone(), rhai::Map::new()));
                }
            }
        }
    }

    // Collect Local Transformer Scripts
    {
        let content_cache = state.content_cache.read().await;
        if let Some(content) = content_cache.get(&content_id) {
            if let Some(transformers_json) = &content.transformer_scripts {
                if let Ok(entries) =
                    serde_json::from_str::<Vec<serde_json::Value>>(transformers_json)
                {
                    let script_cache = state.script_cache.read().await;
                    for entry in entries {
                        let mut script_id = None;
                        let mut args = rhai::Map::new();

                        if let Some(id) = entry.as_i64() {
                            script_id = Some(id as i32);
                        } else if let Some(obj) = entry.as_object() {
                            if let Some(id_val) = obj.get("id").or(obj.get("script_id")) {
                                if let Some(id) = id_val.as_i64() {
                                    script_id = Some(id as i32);
                                }
                            }
                            if let Some(args_val) = obj.get("args") {
                                if let Some(args_obj) = args_val.as_object() {
                                    for (k, v) in args_obj {
                                        if let Some(s) = v.as_str() {
                                            args.insert(k.clone().into(), s.into());
                                        } else if let Some(n) = v.as_i64() {
                                            args.insert(k.clone().into(), n.into());
                                        } else if let Some(b) = v.as_bool() {
                                            args.insert(k.clone().into(), b.into());
                                        } else if let Some(f) = v.as_f64() {
                                            args.insert(k.clone().into(), f.into());
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(id) = script_id {
                            if let Some(script_content) = script_cache.get(&id) {
                                current_scripts_to_run.push((script_content.clone(), args));
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Exec 'transform'
    tracing::info!(
        "Executing transform for {} scripts",
        current_scripts_to_run.len()
    );
    for (content, args) in &current_scripts_to_run {
        if let Err(e) = crate::rhai_engine::execute_script_function(
            content,
            "transform",
            &mut settings,
            args.clone(),
            state.mpv.clone(),
            Some(state.bumper_queue.clone()),
        ) {
            tracing::error!("Failed to execute transform: {}", e);
        }
    }

    // 5. Play MPV
    tracing::info!("Final playback settings: {:?}", settings);

    let mut loop_enabled = None;
    if let Some(loop_val) = settings.get("loop") {
        if let Ok(enabled) = loop_val.as_bool() {
            loop_enabled = Some(enabled);
        }
    }

    if let Some(vol_val) = settings.get("volume") {
        if let Ok(vol) = vol_val.as_float() {
            let _ = state.mpv.set_volume(vol);
        } else if let Ok(vol) = vol_val.as_int() {
            let _ = state.mpv.set_volume(vol as f64);
        }
    }

    let mut start_secs = None;
    if let Some(start_time) = settings.get("start_time") {
        if let Ok(secs) = start_time.as_float() {
            start_secs = Some(secs);
        } else if let Ok(secs) = start_time.as_int() {
            start_secs = Some(secs as f64);
        }
    }

    // Pass start_secs to mpv.play
    state.mpv.play(&content_path, start_secs, loop_enabled)?;

    // Update Current Content ID
    *state.current_content_id.write().await = Some(content_id);

    // 6. Exec 'on_load'
    for (content, args) in &current_scripts_to_run {
        let mut settings_for_load = settings.clone();
        if let Err(e) = crate::rhai_engine::execute_script_function(
            content,
            "on_load",
            &mut settings_for_load,
            args.clone(),
            state.mpv.clone(),
            Some(state.bumper_queue.clone()),
        ) {
            tracing::error!("Failed to execute on_load: {}", e);
        }
    }

    // 7. Store Active State
    *state.active_scripts.write().await = current_scripts_to_run;
    *state.active_settings.write().await = settings;

    Ok(())
}

pub async fn stop_playback(state: &NodeState) {
    unload_active_scripts(state).await;

    if let Err(e) = state.mpv.stop() {
        tracing::error!("Failed to stop playback: {}", e);
    }

    *state.current_content_id.write().await = None;
}

async fn unload_active_scripts(state: &NodeState) {
    let mut active_scripts = state.active_scripts.write().await;
    if !active_scripts.is_empty() {
        tracing::info!("Unloading {} previous scripts", active_scripts.len());
        let settings_guard = state.active_settings.read().await;
        // Make sure we iterate with settings, but we need to pass a Mutable ref to settings copy
        let mut settings_for_unload = settings_guard.clone();
        drop(settings_guard);

        for (content, args) in active_scripts.iter() {
            let args_clone = args.clone();
            if let Err(e) = crate::rhai_engine::execute_script_function(
                content,
                "on_unload",
                &mut settings_for_unload,
                args_clone,
                state.mpv.clone(),
                Some(state.bumper_queue.clone()),
            ) {
                tracing::error!("Failed to execute on_unload: {}", e);
            }
        }
        active_scripts.clear();
    }
    // Clear settings
    *state.active_settings.write().await = rhai::Map::new();
}

/// Check for queued bumpers and play them
pub async fn play_queued_bumpers(state: &NodeState) -> Result<()> {
    loop {
        let bumper_name_or_id = {
            let mut queue = state.bumper_queue.write().await;
            queue.pop_front()
        };

        if let Some(bumper_name_or_id) = bumper_name_or_id {
            tracing::info!("Playing bumper: {}", bumper_name_or_id);

            // Fetch bumper info from server
            let server_url = &state.config.server_url;
            let base_url = server_url
                .replace("ws://", "http://")
                .replace("wss://", "https://");
            let api_url = base_url.split("/ws").next().unwrap_or(&base_url);

            // Try to fetch bumper by name first, then by ID
            let bumper_info_url = if bumper_name_or_id.parse::<i32>().is_ok() {
                format!("{}/api/bumpers/{}", api_url, bumper_name_or_id)
            } else {
                // If it's a name, we need to fetch all bumpers and find by name
                format!("{}/api/bumpers", api_url)
            };

            let client = reqwest::Client::new();
            match client.get(&bumper_info_url).send().await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            // Extract bumper info
                            let bumper = if json.is_array() {
                                // Array response - find by name
                                json.as_array()
                                    .and_then(|arr| {
                                        arr.iter().find(|b| {
                                            b.get("name")
                                                .and_then(|n| n.as_str())
                                                .map(|n| n == bumper_name_or_id)
                                                .unwrap_or(false)
                                        })
                                    })
                                    .cloned()
                            } else {
                                // Single bumper response
                                Some(json)
                            };

                            if let Some(bumper) = bumper {
                                if let Some(rendered_path) =
                                    bumper.get("rendered_path").and_then(|p| p.as_str())
                                {
                                    let bumper_url = format!("{}/{}", api_url, rendered_path);

                                    // Download bumper to cache
                                    let cache_dir = std::path::PathBuf::from(
                                        std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
                                    )
                                    .join(".slatron/bumper_cache");

                                    if !cache_dir.exists() {
                                        std::fs::create_dir_all(&cache_dir)?;
                                    }

                                    let file_name =
                                        rendered_path.split('/').last().unwrap_or("bumper.mp4");
                                    let local_path = cache_dir.join(file_name);

                                    // Only download if not already cached
                                    if !local_path.exists() {
                                        tracing::info!("Downloading bumper: {}", bumper_url);
                                        match client.get(&bumper_url).send().await {
                                            Ok(resp) if resp.status().is_success() => {
                                                let bytes = resp.bytes().await?;
                                                std::fs::write(&local_path, bytes)?;
                                            }
                                            Ok(resp) => {
                                                tracing::error!(
                                                    "Failed to download bumper: HTTP {}",
                                                    resp.status()
                                                );
                                                continue;
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to download bumper: {}", e);
                                                continue;
                                            }
                                        }
                                    }

                                    // Play bumper via main MPV
                                    tracing::info!("Playing bumper from: {}", local_path.display());

                                    // Capture current state for resume
                                    let mut resume_path = None;
                                    let mut resume_pos = None;

                                    if let Ok(false) = state.mpv.is_idle() {
                                        if let Ok(path) = state.mpv.get_path() {
                                            resume_path = Some(path);
                                            if let Ok(pos) = state.mpv.get_position() {
                                                resume_pos = Some(pos);
                                            }
                                        }
                                    }

                                    if let Err(e) =
                                        state
                                            .mpv
                                            .play(&local_path.to_string_lossy(), None, Some(false))
                                    {
                                        tracing::error!("Failed to play bumper: {}", e);
                                    } else {
                                        // Wait for bumper to finish
                                        if let Some(duration) =
                                            bumper.get("duration_ms").and_then(|d| d.as_i64())
                                        {
                                            let sleep_duration =
                                                std::time::Duration::from_millis(duration as u64);
                                            tokio::time::sleep(sleep_duration).await;
                                        } else {
                                            // Default wait time if duration not available
                                            tokio::time::sleep(std::time::Duration::from_secs(3))
                                                .await;
                                        }

                                        // Resume previous content if applicable
                                        if let Some(path) = resume_path {
                                            tracing::info!(
                                                "Resuming content: {} at {:.2}s",
                                                path,
                                                resume_pos.unwrap_or(0.0)
                                            );
                                            if let Err(e) =
                                                state.mpv.play(&path, resume_pos, Some(false))
                                            {
                                                // Assuming loop disabled for resume or should check loop status too?
                                                // For now, simple resume
                                                tracing::error!("Failed to resume content: {}", e);
                                            }
                                        }
                                    }
                                } else {
                                    tracing::warn!(
                                        "Bumper '{}' has no rendered path",
                                        bumper_name_or_id
                                    );
                                }
                            } else {
                                tracing::warn!("Bumper '{}' not found", bumper_name_or_id);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse bumper response: {}", e);
                        }
                    }
                }
                Ok(response) => {
                    tracing::error!("Failed to fetch bumper: HTTP {}", response.status());
                }
                Err(e) => {
                    tracing::error!("Failed to fetch bumper: {}", e);
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}
