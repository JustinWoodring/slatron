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

    engine
}

fn register_content_loader_functions(engine: &mut Engine) {
    // Execute shell command
    engine.register_fn(
        "shell_execute",
        |cmd: String, args: Vec<rhai::Dynamic>| -> String { run_shell_execute(cmd, args) },
    );
    // Overload for single argument
    engine.register_fn("shell_execute", |cmd: String| -> String {
        run_shell_execute(cmd, vec![])
    });

    // Helper to download file via curl/wget (simplified)
    engine.register_fn(
        "download_file",
        |url: String, output_path: String| -> bool {
            let status = std::process::Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&output_path)
                .arg(&url)
                .status();

            match status {
                Ok(s) => s.success(),
                Err(_) => false,
            }
        },
    );

    engine.register_fn("get_env", |key: String| -> String {
        std::env::var(&key).unwrap_or_default()
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
}

fn run_shell_execute(cmd: String, args: Vec<rhai::Dynamic>) -> String {
    let mut command = std::process::Command::new(cmd);

    for arg in args {
        command.arg(arg.to_string());
    }

    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() {
                stdout.to_string()
            } else {
                format!("Error: {}\nStderr: {}", stdout, stderr)
            }
        }
        Err(e) => format!("Execution failed: {}", e),
    }
}

pub fn validate_script(script_content: &str, script_type: &str) -> Vec<String> {
    let engine = create_engine(script_type);

    match engine.compile(script_content) {
        Ok(_) => vec![],
        Err(e) => vec![format!("Compilation error: {}", e)],
    }
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
    scope.push("params", params);

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

    match engine.eval_with_scope::<rhai::Dynamic>(&mut scope, script_content) {
        Ok(result) => {
            let cmds = commands.lock().map(|c| c.clone()).unwrap_or_default();
            Ok((result, cmds))
        }
        Err(e) => Err(format!("Execution error: {}", e)),
    }
}
