use std::sync::Arc;
use crate::mpv_client::MpvClient;
use crate::rhai_engine::create_engine;

// We can't easily mock MpvClient without traits because it's a struct with internal logic.
// However, we can test that the engine compiles and registers functions correctly.

#[test]
fn test_engine_creation() {
    // We can pass None for MPV in tests if we want to just test basic engine creation
    // But to test overlay/global, we need an MPV client.
    // MpvClient::new takes a socket path string. It doesn't connect on creation, only on send.
    let mpv = Arc::new(MpvClient::new("/tmp/test_socket".to_string()));

    let engine = create_engine("transformer", Some(mpv.clone()), None);
    // Check if basic functions exist by compiling a simple script
    assert!(engine.compile("set_loop(true)").is_ok());
}

#[test]
fn test_overlay_functions_registration() {
    let mpv = Arc::new(MpvClient::new("/tmp/test_socket".to_string()));
    let engine = create_engine("overlay", Some(mpv), None);

    // Check if mpv_overlay exists
    assert!(engine.compile("mpv_overlay(\"test.png\", 0, 0, 1.0)").is_ok());
    assert!(engine.compile("mpv_text(\"Hello\", 10, 10, 24, \"#FFFFFF\")").is_ok());
}

#[test]
fn test_global_functions_registration() {
    let mpv = Arc::new(MpvClient::new("/tmp/test_socket".to_string()));
    let engine = create_engine("global", Some(mpv), None);

    // Check if mpv_set_loop exists
    assert!(engine.compile("mpv_set_loop(true)").is_ok());
    assert!(engine.compile("mpv_play(\"video.mp4\")").is_ok());
}
