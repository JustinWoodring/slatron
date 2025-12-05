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
    engine.register_fn("shell_execute", |cmd: String| -> String {
        use std::process::Command;

        match Command::new("sh").arg("-c").arg(&cmd).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
            Err(e) => format!("Error: {}", e),
        }
    });

    engine.register_fn("download_file", |url: String, output: String| {
        // Placeholder - would use reqwest or similar
        tracing::info!("Download {} to {}", url, output);
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

pub fn execute_script(
    script_content: &str,
    script_type: &str,
    params: rhai::Map,
) -> Result<rhai::Dynamic, String> {
    let engine = create_engine(script_type);
    let mut scope = Scope::new();

    scope.push("params", params);

    engine
        .eval_with_scope::<rhai::Dynamic>(&mut scope, script_content)
        .map_err(|e| format!("Execution error: {}", e))
}
