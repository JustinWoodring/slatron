#[cfg(test)]
mod tests {
    use crate::rhai_engine::create_engine;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_download_file_file_protocol_vulnerability() {
        let dir = tempdir().unwrap();
        let secret_path = dir.path().join("secret.txt");
        // Use a subdirectory for output to ensure we are not writing to root,
        // but `download_file` takes a path.
        // We will use the temp dir for output as well.
        let output_path = dir.path().join("vulnerability_test_output.txt");

        // Create a "secret" file
        let mut file = File::create(&secret_path).unwrap();
        writeln!(file, "This is a secret").unwrap();

        // Construct a file:// URL
        let file_url = format!("file://{}", secret_path.to_str().unwrap());
        let output_str = output_path.to_str().unwrap();

        // Setup engine
        let engine = create_engine("content_loader");

        // Run script to "download" the secret file
        let script = format!(
            r#"download_file("{}", "{}")"#,
            file_url, output_str
        );

        let result = engine.eval::<bool>(&script);

        // Now we expect failure (return false)
        assert!(!result.unwrap_or(false), "Vulnerability fixed: download_file should fail for file:// URL");

        // Verify file was NOT created
        assert!(!output_path.exists());
    }

    #[test]
    fn test_download_file_http_allowed() {
         // This test might be flaky if we rely on external network.
         // We'll skip this for now.
    }
}
