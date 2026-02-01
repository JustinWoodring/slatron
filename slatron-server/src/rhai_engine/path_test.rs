#[cfg(test)]
mod tests {
    use crate::rhai_engine::is_safe_path;

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path("foo.txt"));
        assert!(is_safe_path("static/foo.txt"));
        assert!(is_safe_path("foo/bar/baz.txt"));

        // Reject parent dir traversal
        assert!(!is_safe_path("../foo.txt"));
        assert!(!is_safe_path("foo/../../bar.txt"));

        // Reject absolute paths
        assert!(!is_safe_path("/etc/passwd"));
        assert!(!is_safe_path("/tmp/foo.txt"));

        // Check Windows style absolute paths if running on windows?
        // Rust's is_absolute handles platform specifics, but our test env is Linux.
        if cfg!(windows) {
            assert!(!is_safe_path("C:\\Windows\\System32"));
        }
    }
}
