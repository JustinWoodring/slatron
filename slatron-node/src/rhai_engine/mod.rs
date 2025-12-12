use rhai::{Engine, Scope};

pub fn create_engine(script_type: &str) -> Engine {
    let mut engine = Engine::new();

    // Register functions based on script type
    match script_type {
        "content_loader" => {
            register_content_loader_functions(&mut engine);
        }
        "overlay" => {
            register_overlay_functions(&mut engine);
        }
        "global" => {
            register_global_functions(&mut engine);
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
}

fn register_overlay_functions(engine: &mut Engine) {
    engine.register_fn(
        "mpv_overlay",
        |_path: String, _x: i64, _y: i64, _opacity: f64| {
            // TODO: Send to MPV
        },
    );

    engine.register_fn(
        "mpv_text",
        |_text: String, _x: i64, _y: i64, _size: i64, _color: String| {
            // TODO: Send to MPV
        },
    );

    engine.register_fn("mpv_remove_overlay", |_id: i64| {
        // TODO: Send to MPV
    });

    engine.register_fn("get_video_width", || -> i64 { 1920 });

    engine.register_fn("get_video_height", || -> i64 { 1080 });
}

fn register_global_functions(engine: &mut Engine) {
    engine.register_fn("mpv_set_loop", |_enabled: bool| {
        // TODO: Send to MPV
    });

    engine.register_fn("get_content_duration", || -> f64 { 0.0 });

    engine.register_fn("get_block_duration", || -> f64 { 0.0 });

    engine.register_fn("get_playback_position", || -> f64 { 0.0 });

    engine.register_fn("mpv_play", |_path: String| {
        // TODO: Send to MPV
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
}

pub fn execute_script_function(
    script_content: &str,
    fn_name: &str,
    settings: &mut rhai::Map,
    args: rhai::Map,
    mpv: std::sync::Arc<crate::mpv_client::MpvClient>,
) -> Result<(), String> {
    let mut engine = create_engine("transformer");

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
