use rhai::{Engine, Scope};

pub fn create_engine(script_type: &str, mpv: Option<std::sync::Arc<crate::mpv_client::MpvClient>>) -> Engine {
    let mut engine = Engine::new();

    // Register functions based on script type
    match script_type {
        "content_loader" => {
            register_content_loader_functions(&mut engine);
        }
        "overlay" => {
             if let Some(mpv) = mpv {
                 register_overlay_functions(&mut engine, mpv);
             } else {
                 tracing::warn!("MPV client not provided for overlay script engine");
             }
        }
        "global" => {
            if let Some(mpv) = mpv {
                register_global_functions(&mut engine, mpv);
            } else {
                tracing::warn!("MPV client not provided for global script engine");
            }
        }
        "transformer" => {
            register_transformer_functions(&mut engine);
            // Allow transformers to download files and exec shell (for dynamic content fetching)
            register_content_loader_functions(&mut engine);
        }
        _ => {}
    }

    engine.on_print(|x| {
        tracing::info!("[SCRIPT] {}", x);
    });

    engine
}

fn register_content_loader_functions(engine: &mut Engine) {
    engine.register_fn("shell_execute", |cmd: String| -> String {
        use std::process::Command;

        match Command::new("sh").arg("-c").arg(&cmd).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => format!("Error: {}", e),
        }
    });

    engine.register_fn("download_file", |url: String, output: String| -> String {
        tracing::info!(target: "slatron_node::rhai", "Downloading {} to {}", url, output);
        use std::process::Command;

        // Manually expand ~ to HOME since Command doesn't run via shell
        let expanded_output = if output.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                output.replacen("~", &home, 1)
            } else {
                output
            }
        } else {
             output
        };
        
        // Use curl for simple synchronous download
        let status_res = Command::new("curl")
            .arg("-L") // Follow redirects
            .arg("-o")
            .arg(&expanded_output)
            .arg(&url)
            .status();

        match status_res {
            Ok(status) => {
                if status.success() {
                    tracing::info!(target: "slatron_node::rhai", "Download successful: {}", expanded_output);
                } else {
                    tracing::error!(target: "slatron_node::rhai", "Download failed with status: {}", status);
                }
            }
            Err(e) => {
                 tracing::error!(target: "slatron_node::rhai", "Failed to execute curl: {}", e);
            }
        }
        
        expanded_output
    });

    engine.register_fn("get_env", |key: String| -> String {
        std::env::var(&key).unwrap_or_default()
    });

    engine.register_fn("capture_website", |url: String, output: String| -> String {
        // This expects the `screenshot.js` tool to be in a specific location or configurable
        // For now we assume `tools/screenshot.js` relative to current dir or absolute path
        use std::process::Command;

        // Find screenshot script path
        let script_path = if std::path::Path::new("slatron-node/tools/screenshot.js").exists() {
             "slatron-node/tools/screenshot.js"
        } else if std::path::Path::new("tools/screenshot.js").exists() {
             "tools/screenshot.js"
        } else {
             // Fallback or error
             "screenshot.js"
        };

        // Expand output path
        let expanded_output = if output.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                output.replacen("~", &home, 1)
            } else {
                output
            }
        } else {
             output
        };

        tracing::info!(target: "slatron_node::rhai", "Capturing website {} to {}", url, expanded_output);

        let output_res = Command::new("node")
            .arg(script_path)
            .arg(&url)
            .arg(&expanded_output)
            .output();

        match output_res {
            Ok(output_data) => {
                if output_data.status.success() {
                    tracing::info!(target: "slatron_node::rhai", "Screenshot success");
                    expanded_output
                } else {
                     let err = String::from_utf8_lossy(&output_data.stderr);
                     tracing::error!(target: "slatron_node::rhai", "Screenshot failed: {}", err);
                     format!("Error: {}", err)
                }
            }
            Err(e) => {
                 tracing::error!(target: "slatron_node::rhai", "Failed to execute node script: {}", e);
                 format!("Error: {}", e)
            }
        }
    });
}

fn register_overlay_functions(
    engine: &mut Engine,
    mpv: std::sync::Arc<crate::mpv_client::MpvClient>,
) {
    let mpv_clone = mpv.clone();
    engine.register_fn(
        "mpv_overlay",
        move |path: String, x: i64, y: i64, opacity: f64| {
            // TODO: MPV overlay-add command
            // We use overlay-add <id> <x> <y> <file> <offset> <fmt> <w> <h> <stride-w> <stride-h>
            // Actually, simpler usage via `video-add` might be better for images, but overlay-add is for OSD?
            // "overlay-add" is for OSD overlays.
            // Signature: overlay-add <id> <x> <y> <file> <offset> <fmt> <w> <h> <stride-w> <stride-h>
            // But MPV client has add_overlay
            if let Err(e) = mpv_clone.add_overlay(&path, x as i32, y as i32, opacity) {
                tracing::error!(target: "slatron_node::rhai", "mpv_overlay failed: {}", e);
            }
        },
    );

    let mpv_clone = mpv.clone();
    engine.register_fn(
        "mpv_text",
        move |text: String, _x: i64, _y: i64, _size: i64, _color: String| {
            // Using OSD overlay for text is complex (requires rendering text to image or using ASS/OSD commands).
            // MPV doesn't have a direct "draw text at x,y" command via IPC easily without ASS.
            // A common workaround is `osd-overlay` with data.
            // `overlay-add` expects a file or raw memory.

            // Alternative: `show-text` but that's fleeting.
            // Best bet: use `osd-overlay` command.
            // id=1 for text overlay?
            // command: ["osd-overlay", <id>, "none", <text>, <x>, <y>, <align>, <style>]
            // This is actually not standard MPV IPC command, it depends on scripts usually.

            // Standard MPV has `osd-msg` or `show-text`.
            // But for persistent text, we might need a custom script or use `overlay-add` with generated image.
            // Let's assume for now we use `show-text` with long duration or just log it as not fully supported.

            // Or use `script-message-to osc show-message ...`

            // Let's try to construct an ASS subtitle string and set it? No.

            // Re-reading `mpv_client.rs`... it only has `add_overlay`.
            // Let's implement `mpv_text` as a best-effort logging or `show-text` for now.
            // Or maybe the user expects `osd-overlay` which is available in some mpv builds?
            // Let's use `show-text` with 5 seconds duration.
            let cmd = serde_json::json!({
                "command": ["show-text", text, 5000, 1] // text, duration(ms), level
            });
            if let Err(e) = mpv_clone.send_command(cmd) {
                tracing::error!(target: "slatron_node::rhai", "mpv_text failed: {}", e);
            }
        },
    );

    let mpv_clone = mpv.clone();
    engine.register_fn("mpv_remove_overlay", move |id: i64| {
        let cmd = serde_json::json!({
            "command": ["overlay-remove", id]
        });
        if let Err(e) = mpv_clone.send_command(cmd) {
            tracing::error!(target: "slatron_node::rhai", "mpv_remove_overlay failed: {}", e);
        }
    });

    // These need actual implementation or MPV queries
    engine.register_fn("get_video_width", || -> i64 { 1920 });
    engine.register_fn("get_video_height", || -> i64 { 1080 });
}

fn register_global_functions(
    engine: &mut Engine,
    mpv: std::sync::Arc<crate::mpv_client::MpvClient>,
) {
    let mpv_clone = mpv.clone();
    engine.register_fn("mpv_set_loop", move |enabled: bool| {
        // loop-file
        let val = if enabled { "inf" } else { "no" };
        let cmd = serde_json::json!({
             "command": ["set_property", "loop-file", val]
        });
        if let Err(e) = mpv_clone.send_command(cmd) {
            tracing::error!(target: "slatron_node::rhai", "mpv_set_loop failed: {}", e);
        }
    });

    let mpv_clone = mpv.clone();
    engine.register_fn("get_content_duration", move || -> f64 {
        match mpv_clone.get_duration() {
            Ok(d) => d,
            Err(_) => 0.0,
        }
    });

    engine.register_fn("get_block_duration", || -> f64 { 0.0 });

    let mpv_clone = mpv.clone();
    engine.register_fn("get_playback_position", move || -> f64 {
        match mpv_clone.get_position() {
            Ok(p) => p,
            Err(_) => 0.0,
        }
    });

    let mpv_clone = mpv.clone();
    engine.register_fn("mpv_play", move |path: String| {
        if let Err(e) = mpv_clone.play(&path, None, None) {
            tracing::error!(target: "slatron_node::rhai", "mpv_play failed: {}", e);
        }
    });
}

fn register_transformer_functions(engine: &mut Engine) {
    // Transformer scripts will interact with a "settings" map passed in the scope.
    // We provide helper functions to make it cleaner.

    // settings.loop = true
    engine.register_fn("set_loop", |ctx: &mut rhai::Map, enabled: bool| {
        ctx.insert("loop".into(), rhai::Dynamic::from(enabled));
    });

    // settings.volume = 50
    engine.register_fn("set_volume", |ctx: &mut rhai::Map, volume: i64| {
        ctx.insert("volume".into(), rhai::Dynamic::from(volume));
    });

    // settings.start_time = 10.0
    engine.register_fn("set_start_time", |ctx: &mut rhai::Map, seconds: f64| {
        ctx.insert("start_time".into(), rhai::Dynamic::from(seconds));
    });

    // settings.end_time = 20.0
    engine.register_fn("set_end_time", |ctx: &mut rhai::Map, seconds: f64| {
        ctx.insert("end_time".into(), rhai::Dynamic::from(seconds));
    });

    // settings.path = "..."
    engine.register_fn("set_path", |ctx: &mut rhai::Map, path: String| {
        ctx.insert("path".into(), rhai::Dynamic::from(path));
    });
}

pub fn execute_script_function(
    script_content: &str,
    fn_name: &str,
    settings: &mut rhai::Map,
    args: rhai::Map,
    mpv: std::sync::Arc<crate::mpv_client::MpvClient>,
) -> Result<(), String> {
    let mut engine = create_engine("transformer", Some(mpv.clone()));

    // Register mpv_send
    let mpv_clone = mpv.clone();
    engine.register_fn("mpv_send", move |cmd_map: rhai::Map| {
        // Convert rhai map to json
        let dynamic_map = rhai::Dynamic::from(cmd_map);
        match rhai::serde::from_dynamic::<serde_json::Value>(&dynamic_map) {
            Ok(json_val) => {
                 if let Err(e) = mpv_clone.send_command(json_val) {
                     tracing::error!(target: "slatron_node::rhai", "mpv_send failed: {}", e);
                 }
            }
            Err(e) => {
                tracing::error!(target: "slatron_node::rhai", "mpv_send serialization error: {}", e);
            }
        }
    });

    let mut scope = Scope::new();
    scope.push("args", args);

    // Only compile if we can cache it? For now, re-compile is okay as scripts are small.
    // Optimization: Cache AST in NodeState using script_id if perf issues arise.

    // Log script preview (only if transform/major entry point?)
    let preview: String = script_content.chars().take(200).collect();
    tracing::info!(target: "slatron_node::rhai", "Compiling script [fn={}] (len={}): {}...", fn_name, script_content.len(), preview);

    let ast = engine
        .compile(script_content)
        .map_err(|e| format!("Compilation error: {}", e))?;

    // Debug: List functions in AST
    // Note: Rhai 1.x AST iter_functions might need specific import or be different,
    // but typically it offers a way to inspect.
    // Actually, `ScriptFnMetadata` is available.
    // Let's just try to rely on current logging first, but if I can iterate, valuable.
    // AST doesn't easily expose functions in public API without iterating.
    // I'll skip listing functions for now to avoid compilation error if API mismatch.

    // Push settings to scope to allow global access (fn on_load() using 'settings' global)
    scope.push("settings", settings.clone());

    engine
        .run_ast_with_scope(&mut scope, &ast)
        .map_err(|e| format!("Execution error: {}", e))?;

    // Special handling for "transform": legacy global variables
    if fn_name == "transform" {
        let mut legacy_found = false;
        if let Some(val) = scope.get_value::<bool>("loop") {
            tracing::info!(target: "slatron_node::rhai", "Legacy global detected: loop = {}", val);
            settings.insert("loop".into(), rhai::Dynamic::from(val));
            legacy_found = true;
        }
        if let Some(val) = scope.get_value::<i64>("volume") {
            tracing::info!(target: "slatron_node::rhai", "Legacy global detected: volume = {}", val);
            settings.insert("volume".into(), rhai::Dynamic::from(val));
            legacy_found = true;
        }
        if let Some(val) = scope.get_value::<f64>("start_time") {
            tracing::info!(target: "slatron_node::rhai", "Legacy global detected: start_time = {}", val);
            settings.insert("start_time".into(), rhai::Dynamic::from(val));
            legacy_found = true;
        }
        if let Some(val) = scope.get_value::<f64>("end_time") {
            tracing::info!(target: "slatron_node::rhai", "Legacy global detected: end_time = {}", val);
            settings.insert("end_time".into(), rhai::Dynamic::from(val));
            legacy_found = true;
        }

        // Pass empty settings map context
        let script_settings = rhai::Map::new();
        let result = engine.call_fn::<rhai::Map>(&mut scope, &ast, fn_name, (script_settings,));

        match result {
            Ok(returned_settings) => {
                tracing::info!(target: "slatron_node::rhai", "Script returned {} settings", returned_settings.len());
                for (k, v) in returned_settings {
                    tracing::info!(target: "slatron_node::rhai", "  -> Setting override: {} = {:?}", k, v);
                    settings.insert(k, v);
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("not found") {
                    if legacy_found {
                        tracing::info!(target: "slatron_node::rhai", "No 'transform' function found. Used legacy global variables.");
                    } else {
                        tracing::warn!(target: "slatron_node::rhai", "No 'transform' function AND no legacy variables found. Script may be doing nothing.");
                    }
                } else {
                    tracing::error!(target: "slatron_node::rhai", "Script 'transform' call failed: {}", e);
                }
            }
        }
    } else {
        // Generic function call (on_load, on_unload)
        // Pass a copy of the settings so the script can read them
        tracing::info!(target: "slatron_node::rhai", "Attempting to call script function: {}", fn_name);

        let settings_copy = settings.clone();
        let result = engine.call_fn::<()>(&mut scope, &ast, fn_name, (settings_copy,));

        match result {
            Ok(_) => {
                tracing::info!(target: "slatron_node::rhai", "Successfully executed '{}' (1 arg)", fn_name);
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("not found") {
                    // Try calling with 0 args
                    tracing::info!(target: "slatron_node::rhai", "Function '{}' with 1 arg not found, trying 0 args...", fn_name);
                    let retry = engine.call_fn::<()>(&mut scope, &ast, fn_name, ());
                    match retry {
                        Ok(_) => {
                            tracing::info!(target: "slatron_node::rhai", "Successfully executed '{}' (0 args)", fn_name);
                        }
                        Err(retry_e) => {
                            let retry_err = retry_e.to_string();
                            if !retry_err.contains("not found") {
                                tracing::error!(target: "slatron_node::rhai", "Script function '{}' failed: {}", fn_name, retry_err);
                            } else {
                                // Really not found
                                tracing::info!(target: "slatron_node::rhai", "Script function '{}' not found (checked 0 and 1 args).", fn_name);
                            }
                        }
                    }
                } else {
                    tracing::error!(target: "slatron_node::rhai", "Script function '{}' call failed: {}", fn_name, e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
