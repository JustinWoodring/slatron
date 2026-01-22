#[cfg(test)]
mod tests {
    use crate::rhai_engine::{is_safe_path, is_safe_url};

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

    #[test]
    fn test_is_safe_url() {
        // Valid URLs
        assert!(is_safe_url("http://example.com/file.jpg"));
        assert!(is_safe_url("https://example.com/file.jpg"));
        assert!(is_safe_url("HTTPS://EXAMPLE.COM/FILE.JPG"));

        // Invalid protocols
        assert!(!is_safe_url("ftp://example.com/file.txt"));
        assert!(!is_safe_url("file:///etc/passwd"));
        assert!(!is_safe_url("gopher://example.com"));

        // malicious injections
        assert!(!is_safe_url("-o /etc/shadow"));
        assert!(!is_safe_url("--output /etc/shadow"));
        assert!(!is_safe_url("-L"));

        // Plain strings
        assert!(!is_safe_url("example.com"));
    }
}
