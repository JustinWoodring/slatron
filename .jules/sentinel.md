## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-23 - [SSRF via Scriptable Download Tool]
**Vulnerability:** The `download_file` function exposed to Rhai scripts in both `slatron-server` and `slatron-node` used `curl` without validating the URL protocol.
**Learning:** Tools like `curl` support many protocols (`file://`, `ftp://`, etc.) which can be abused for SSRF or LFI if user input is passed directly to them. Simply relying on "it downloads stuff" hides the complexity of the underlying tool's capabilities.
**Prevention:** Always whitelist allowed protocols (e.g., `http://`, `https://`) when using generic download tools or libraries, especially when input comes from a scriptable environment.

## 2025-05-24 - [Rhai Script RCE Mitigation]
**Vulnerability:** As identified in the 2025-05-23 finding, `shell_execute` allowed arbitrary commands.
**Learning:** Even "trusted" script editors shouldn't have root-equivalent access to the underlying OS. Restricting to a functional allowlist (`yt-dlp`, `ffmpeg`, `ffprobe`) balances functionality with security.
**Prevention:** Explicit allowlists are superior to blacklists or "trust". The fix validates the command against a hardcoded array before execution.
