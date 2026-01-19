use super::*;

#[test]
fn test_shell_execute_is_removed() {
    // "content_loader" type previously had shell_execute
    let engine = create_engine("content_loader", None, None);

    // Try to execute a shell command
    let script = r#"
        shell_execute("echo vulnerable")
    "#;

    let result = engine.eval::<String>(script);

    // Verify that shell_execute is NOT available
    assert!(result.is_err(), "Security fix failed: shell_execute is still available");

    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Function not found"), "Unexpected error: {}", err_msg);
}
