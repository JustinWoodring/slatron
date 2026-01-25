use crate::rhai_engine::create_engine;

#[test]
fn test_download_file_unsafe_protocol() {
    let engine = create_engine("content_loader");

    let script = r#"
        download_file("file:///etc/passwd", "test_output.txt")
    "#;

    let result = engine.eval::<bool>(script);

    if let Ok(val) = result {
         assert_eq!(val, false, "download_file should return false for file:// protocol");
    }

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

     let script = r#"
        download_file("http://example.com", "../test.txt")
    "#;
    let result = engine.eval::<bool>(script);
    if let Ok(val) = result {
        assert_eq!(val, false, "Should reject .. path");
    }

    let script = r#"
        download_file("http://example.com", "/tmp/test.txt")
    "#;
    let result = engine.eval::<bool>(script);
     if let Ok(val) = result {
        assert_eq!(val, false, "Should reject absolute path");
    }
}

#[test]
fn test_shell_execute_arbitrary_command() {
    let engine = create_engine("content_loader");

    // Attempt to run 'ls' which should now FAIL (blocked by whitelist)
    let script = r#"
        let result = shell_execute("ls", ["-la"]);
        result
    "#;

    let result = engine.eval::<rhai::Map>(script);

    match result {
        Ok(map) => {
            let code = map.get("code").expect("should have code").clone_cast::<i64>();
            let stderr = map.get("stderr").expect("should have stderr").to_string();
            // println!("Output of ls stderr: {}", stderr);

            assert_eq!(code, -1, "Should return -1 for blocked command");
            assert!(stderr.contains("Security Violation"), "Stderr should mention Security Violation");
        },
        Err(e) => panic!("Script failed: {}", e),
    }
}

#[test]
fn test_shell_execute_allowed_command() {
    let engine = create_engine("content_loader");

    // Attempt to run 'yt-dlp' which IS allowed.
    // It will likely fail to execute because it's not installed or arguments are invalid,
    // but it should NOT return the Security Violation message.
    let script = r#"
        let result = shell_execute("yt-dlp", ["--version"]);
        result
    "#;

    let result = engine.eval::<rhai::Map>(script);

    match result {
        Ok(map) => {
            let stderr = map.get("stderr").expect("should have stderr").to_string();
            // It might fail with "No such file or directory", but NOT "Security Violation"
            assert!(!stderr.contains("Security Violation"), "yt-dlp should be allowed (even if execution fails)");
        },
        Err(e) => panic!("Script failed: {}", e),
    }
}
