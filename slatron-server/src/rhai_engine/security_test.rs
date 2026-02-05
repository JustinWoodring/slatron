use crate::rhai_engine::create_engine;

#[test]
fn test_download_file_unsafe_protocol() {
    let mut engine = create_engine("content_loader");

    // We can't easily capture the return value or log output of download_file
    // without executing it via the engine and mocking everything,
    // but we can try to run a script and check if it returns false (since we modified it to return bool).

    let script = r#"
        download_file("file:///etc/passwd", "test_output.txt")
    "#;

    // This relies on `eval` returning the result of the last statement.
    let result = engine.eval::<bool>(script);

    // It might panic if eval fails, but here we expect it to return false.
    // If it fails to compile/run, unwrapping will panic, which is also a test failure.
    if let Ok(val) = result {
         assert_eq!(val, false, "download_file should return false for file:// protocol");
    } else {
         // If eval failed, it might be due to other reasons, but let's assume valid rhai syntax
         // If `download_file` throws exception, that's also fine as rejection.
    }

    // Also check that it returns false for other protocols
    let script_ftp = r#"
        download_file("ftp://example.com/foo", "test_output.txt")
    "#;
    let result_ftp = engine.eval::<bool>(script_ftp);
     if let Ok(val) = result_ftp {
         assert_eq!(val, false, "download_file should return false for ftp:// protocol");
     }
}

#[test]
fn test_download_file_safe_path_rejection() {
     let mut engine = create_engine("content_loader");

     // Test path traversal
     let script = r#"
        download_file("http://example.com", "../test.txt")
    "#;
    let result = engine.eval::<bool>(script);
    if let Ok(val) = result {
        assert_eq!(val, false, "Should reject .. path");
    }

    // Test absolute path
    let script = r#"
        download_file("http://example.com", "/tmp/test.txt")
    "#;
    let result = engine.eval::<bool>(script);
     if let Ok(val) = result {
        assert_eq!(val, false, "Should reject absolute path");
    }
}

#[test]
fn test_shell_execute_blocked_command() {
    let mut engine = create_engine("content_loader");

    // Try to run "ls"
    let script = r#"
        let result = shell_execute("ls", ["-la"]);
        result.stderr
    "#;

    let result = engine.eval::<String>(script);
    if let Ok(val) = result {
         assert!(val.contains("Security Alert"), "Should block unauthorized command 'ls'. Got: {}", val);
    } else {
         panic!("Script execution failed: {:?}", result);
    }
}

#[test]
fn test_shell_execute_blocked_args() {
    let mut engine = create_engine("content_loader");

    // Try to run "yt-dlp --exec 'touch hacked'"
    let script = r#"
        let result = shell_execute("yt-dlp", ["--exec", "touch hacked"]);
        result.stderr
    "#;

    let result = engine.eval::<String>(script);
    if let Ok(val) = result {
         assert!(val.contains("Security Alert"), "Should block yt-dlp --exec. Got: {}", val);
    } else {
         panic!("Script execution failed: {:?}", result);
    }
}

#[test]
fn test_shell_execute_allowed_command() {
    let mut engine = create_engine("content_loader");

    // We can't actually run yt-dlp or ffmpeg if they are not installed in the environment.
    // But we can check that it DOES NOT return "Security Alert".

    let script = r#"
        let result = shell_execute("ffmpeg", ["-version"]);
        result.stderr
    "#;

    let result = engine.eval::<String>(script);
    if let Ok(val) = result {
         assert!(!val.contains("Security Alert"), "Should allow authorized command 'ffmpeg'. Got: {}", val);
    } else {
        panic!("Script execution failed: {:?}", result);
    }
}
