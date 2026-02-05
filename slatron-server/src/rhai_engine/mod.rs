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
        _ => {}
    }

    engine.on_print(|x| {
        tracing::info!("[SCRIPT] {}", x);
    });

    engine.on_debug(|x, src, pos| {
        let src_str = src.unwrap_or("unknown");
        tracing::info!("[SCRIPT DEBUG] {} @ {:?}: {}", src_str, pos, x);
    });

    // Override parse_json with serde_json for better compatibility (e.g. unicode escapes)
    engine.register_fn("parse_json", |json: String| -> rhai::Dynamic {
        match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(v) => rhai::serde::to_dynamic(&v).unwrap_or(rhai::Dynamic::UNIT),
            Err(e) => {
                tracing::error!("JSON Parse Error: {}", e);
                // Return empty map on error to prevent crash, or maybe rethrow?
                // Rhai parse_json usually throws.
                // Let's print error and return format error string as Map for now, or just empty map.
                // Better yet, throw exception if possible? But closure returns Dynamic.
                // For now, logging and returning empty map is safer than crash.
                rhai::Dynamic::UNIT
            }
        }
    });

    engine
}

fn register_content_loader_functions(engine: &mut Engine) {
    // Execute shell command
    engine.register_fn(
        "shell_execute",
        |cmd: String, args: Vec<rhai::Dynamic>| -> rhai::Map { run_shell_execute(cmd, args) },
    );
    // Overload for single argument
    engine.register_fn("shell_execute", |cmd: String| -> rhai::Map {
        run_shell_execute(cmd, vec![])
    });

    // Helper to download file via curl/wget (simplified)
    engine.register_fn(
        "download_file",
        |url: String, output_path: String| -> bool {
            if !is_safe_path(&output_path) {
                tracing::error!(
                    "Security Alert: Script attempted to download file to unsafe path: {}",
                    output_path
                );
                return false;
            }

            // Security Check: Validate Protocol
            if !url.starts_with("http://") && !url.starts_with("https://") {
                tracing::error!(
                    "Security Alert: Script attempted to download using unsafe protocol: {}",
                    url
                );
                return false;
            }

            let status = std::process::Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&output_path)
                .arg(&url)
                .status();

            match status {
                Ok(s) => {
                    if s.success() {
                        tracing::info!("Download successful: {} -> {}", url, output_path);
                        true
                    } else {
                        tracing::error!(
                            "Download failed: {} -> {}, exit code: {:?}",
                            url,
                            output_path,
                            s.code()
                        );
                        false
                    }
                }
                Err(e) => {
                    tracing::error!("Download command failed to start: {}", e);
                    false
                }
            }
        },
    );

    engine.register_fn("get_env", |key: String| -> String {
        std::env::var(&key).unwrap_or_default()
    });

    engine.register_fn("to_json", |v: rhai::Dynamic| -> String {
        serde_json::to_string(&v).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    });
}

fn register_overlay_functions(engine: &mut Engine) {
    engine.register_fn(
        "mpv_overlay",
        |_path: String, _x: i64, _y: i64, _opacity: f64| {
            // Placeholder
        },
    );

    engine.register_fn(
        "mpv_text",
        |_text: String, _x: i64, _y: i64, _size: i64, _color: String| {
            // Placeholder
        },
    );

    engine.register_fn("mpv_remove_overlay", |_id: i64| {
        // Placeholder
    });

    engine.register_fn("get_video_width", || -> i64 { 1920 });

    engine.register_fn("get_video_height", || -> i64 { 1080 });
}

fn register_global_functions(engine: &mut Engine) {
    engine.register_fn("mpv_set_loop", |_enabled: bool| {
        // Placeholder
    });

    engine.register_fn("get_content_duration", || -> f64 { 0.0 });

    engine.register_fn("get_block_duration", || -> f64 { 0.0 });

    engine.register_fn("get_playback_position", || -> f64 { 0.0 });

    engine.register_fn("mpv_play", |_path: String| {
        // Placeholder
    });

    engine.register_fn("to_json", |v: rhai::Dynamic| -> String {
        serde_json::to_string(&v).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    });
}

fn run_shell_execute(cmd: String, args: Vec<rhai::Dynamic>) -> rhai::Map {
    // Security: Allowlist check
    let allowed_commands = vec!["yt-dlp", "ffmpeg", "ffprobe"];
    if !allowed_commands.contains(&cmd.as_str()) {
        let err_msg = format!("Security Alert: Command not allowed: {}", cmd);
        tracing::error!("{}", err_msg);
        let mut map = rhai::Map::new();
        map.insert("code".into(), (-1 as i64).into());
        map.insert("stdout".into(), "".into());
        map.insert("stderr".into(), err_msg.into());
        return map;
    }

    // Security: Argument check for yt-dlp
    if cmd == "yt-dlp" {
        for arg in &args {
            let s = arg.to_string();
            if s.starts_with("--exec") {
                let err_msg = format!("Security Alert: Argument not allowed for yt-dlp: {}", s);
                tracing::error!("{}", err_msg);
                let mut map = rhai::Map::new();
                map.insert("code".into(), (-1 as i64).into());
                map.insert("stdout".into(), "".into());
                map.insert("stderr".into(), err_msg.into());
                return map;
            }
        }
    }

    let mut command = std::process::Command::new(&cmd);

    let mut args_str = String::new();
    for arg in args {
        let s = arg.to_string();
        args_str.push_str(&format!("{} ", s));
        command.arg(s);
    }

    tracing::info!("Executing Shell: {} {}", cmd, args_str);

    let mut map = rhai::Map::new();

    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().unwrap_or(-1) as i64;

            if !stdout.is_empty() {
                tracing::debug!("Shell Stdout: {}", stdout);
            }
            if !stderr.is_empty() {
                tracing::warn!("Shell Stderr: {}", stderr);
            }

            if !output.status.success() {
                tracing::error!("Shell Failed ({}): {}", code, stderr);
            }

            map.insert("code".into(), code.into());
            map.insert("stdout".into(), stdout.into());
            map.insert("stderr".into(), stderr.into());
        }
        Err(e) => {
            let err_msg = format!("Execution failed: {}", e);
            tracing::error!("Shell Spawn Failed: {}", err_msg);

            map.insert("code".into(), (-1 as i64).into());
            map.insert("stdout".into(), "".into());
            map.insert("stderr".into(), err_msg.into());
        }
    }
    map
}

pub fn validate_script(script_content: &str, script_type: &str) -> Vec<String> {
    let engine = create_engine(script_type);

    match engine.compile(script_content) {
        Ok(_) => vec![],
        Err(e) => vec![format!("Compilation error: {}", e)],
    }
}

pub(crate) fn is_safe_path(path_str: &str) -> bool {
    let path = std::path::Path::new(path_str);
    // Prevent directory traversal
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return false;
    }
    // Prevent absolute paths to avoid writing to system directories
    if path.is_absolute() {
        return false;
    }
    true
}

pub fn execute_script(
    script_content: &str,
    script_type: &str,
    params: rhai::Map,
    settings: std::collections::HashMap<String, String>,
) -> Result<(rhai::Dynamic, Vec<String>), String> {
    let mut engine = create_engine(script_type);
    let mut scope = Scope::new();

    // Add params to scope
    let params_dynamic: rhai::Dynamic = params.clone().into();
    scope.push("params", params_dynamic.clone());

    // Inject Settings
    let mut settings_map = rhai::Map::new();
    for (k, v) in settings {
        settings_map.insert(k.into(), v.into());
    }
    scope.push("settings", settings_map);

    // Capture MPV commands
    let commands = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let commands_clone = commands.clone();

    engine.register_fn("mpv_send", move |cmd: String| {
        if let Ok(mut cmds) = commands_clone.lock() {
            cmds.push(cmd);
        }
    });

    // Compile AST first
    let ast = engine
        .compile(script_content)
        .map_err(|e| format!("Compilation error: {}", e))?;

    // Run top-level statements (registers functions)
    let mut result = engine
        .eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast)
        .map_err(|e| format!("Top-level execution error: {}", e))?;

    // If content_loader, try to call load_content()
    if script_type == "content_loader" {
        // Inspect AST for load_content function
        let has_load_content = ast.iter_functions().find(|f| f.name == "load_content");

        if let Some(f) = has_load_content {
            let call_result = if f.params.len() == 1 {
                tracing::info!("Executing 'load_content(params)'");
                engine.call_fn::<rhai::Dynamic>(&mut scope, &ast, "load_content", (params_dynamic,))
            } else {
                tracing::info!("Executing 'load_content()'");
                engine.call_fn::<rhai::Dynamic>(&mut scope, &ast, "load_content", ())
            };

            match call_result {
                Ok(r) => {
                    result = r;
                }
                Err(e) => {
                    tracing::error!("Entry point execution failed: {}", e);
                    return Err(format!("Entry point error: {}", e));
                }
            }
        } else {
            // Fallback to top-level result
            tracing::debug!("Entry point 'load_content' not found, using top-level result");
        }
    }

    let cmds = commands.lock().map(|c| c.clone()).unwrap_or_default();
    Ok((result, cmds))
}

#[cfg(test)]
mod path_test;

#[cfg(test)]
mod security_test;
