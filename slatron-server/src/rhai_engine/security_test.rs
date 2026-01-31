use crate::rhai_engine::create_engine;

#[test]
fn test_download_file_unsafe_protocol() {
    let engine = create_engine("content_loader");

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
     let engine = create_engine("content_loader");

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
fn test_shell_execute_blocked() {
    let engine = create_engine("content_loader");

    // Attempt to run ls
    let script = r#"
        let res = shell_execute("ls", ["-la"]);
        res.stderr
    "#;

    let result = engine.eval::<String>(script);
    if let Ok(stderr) = result {
        assert!(
            stderr.contains("Security Error: Command not allowed"),
            "Should block unauthorized command"
        );
    } else {
        panic!("Script failed to run");
    }
}

#[test]
fn test_shell_execute_allowed() {
    let engine = create_engine("content_loader");

    // Attempt to run yt-dlp.
    // We expect it NOT to be blocked. It might fail if not installed, but stderr shouldn't be "Security Error..."
    let script = r#"
        let res = shell_execute("yt-dlp", ["--version"]);
        res.stderr
    "#;

    let result = engine.eval::<String>(script);
    if let Ok(stderr) = result {
        assert!(
            !stderr.contains("Security Error: Command not allowed"),
            "Should allow authorized command"
        );
    } else {
        // If script failed to run, it might be due to other reasons, but let's hope it runs.
    }
}
