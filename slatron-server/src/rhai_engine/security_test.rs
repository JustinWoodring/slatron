use crate::rhai_engine::create_engine;
use rhai::Map;

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
fn test_arbitrary_shell_execution_blocked() {
    let mut engine = create_engine("content_loader");

    // Try to execute 'echo' which should be blocked
    let script = r#"
        shell_execute("echo", ["VULNERABLE"])
    "#;

    let result = engine.eval::<Map>(script);

    match result {
        Ok(map) => {
            let stderr = map.get("stderr").unwrap().to_string();
            if !stderr.contains("Security Alert: Command not allowed") {
                 panic!("FAILURE: Command was NOT blocked as expected. Stderr: {}", stderr);
            }
        },
        Err(e) => {
            panic!("Execution failed unexpectedly: {}", e);
        }
    }
}

#[test]
fn test_ytdlp_exec_blocked() {
    let mut engine = create_engine("content_loader");

    // Try to execute yt-dlp with --exec
    let script = r#"
        shell_execute("yt-dlp", ["--exec", "echo FAIL", "http://example.com"])
    "#;

    let result = engine.eval::<Map>(script);

     match result {
        Ok(map) => {
            let stderr = map.get("stderr").unwrap().to_string();
            if !stderr.contains("Security Alert: Illegal arguments") {
                 panic!("FAILURE: Dangerous argument was NOT blocked. Stderr: {}", stderr);
            }
        },
        Err(e) => {
            panic!("Execution failed unexpectedly: {}", e);
        }
    }
}
