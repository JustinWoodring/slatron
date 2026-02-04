use super::*;

#[test]
fn test_shell_execute_valid() {
    let engine = create_engine("content_loader", None, None);
    let script = r#"shell_execute("echo hello")"#;
    let result = engine.eval::<String>(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "hello");
}

#[test]
fn test_shell_execute_quotes() {
    let engine = create_engine("content_loader", None, None);
    let script = r#"shell_execute("echo 'hello world'")"#;
    let result = engine.eval::<String>(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "hello world");
}

#[test]
fn test_shell_execute_forbidden_command() {
    let engine = create_engine("content_loader", None, None);
    // ls is not in allowlist
    let script = r#"shell_execute("ls -la")"#;
    let result = engine.eval::<String>(script);

    // It should either return an error or an error string (depending on implementation)
    // Current impl returns string "Error: ..."
    // Future impl might throw exception or return error string.
    // Let's assume it returns a string starting with "Security Alert" or similar.
    if let Ok(val) = result {
        assert!(
            val.contains("Error") || val.contains("not allowed") || val.contains("Security Alert")
        );
    }
}

#[test]
fn test_shell_execute_injection_mitigated() {
    let engine = create_engine("content_loader", None, None);
    // Attempt injection: echo hello; echo injected
    let script = r#"shell_execute("echo hello; echo injected")"#;
    let result = engine.eval::<String>(script);

    if let Ok(val) = result {
        // If vulnerable (sh -c), output is "hello\ninjected"
        // If secured (args), output is "hello; echo injected"
        assert!(
            !val.contains("\ninjected"),
            "Vulnerability detected: Command injection succeeded"
        );
        assert!(
            val.contains("hello; echo injected"),
            "Should treat semicolons as arguments"
        );
    }
}
