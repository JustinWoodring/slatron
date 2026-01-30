## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-23 - [SSRF via Scriptable Download Tool]
**Vulnerability:** The `download_file` function exposed to Rhai scripts in both `slatron-server` and `slatron-node` used `curl` without validating the URL protocol.
**Learning:** Tools like `curl` support many protocols (`file://`, `ftp://`, etc.) which can be abused for SSRF or LFI if user input is passed directly to them. Simply relying on "it downloads stuff" hides the complexity of the underlying tool's capabilities.
**Prevention:** Always whitelist allowed protocols (e.g., `http://`, `https://`) when using generic download tools or libraries, especially when input comes from a scriptable environment.

## 2025-10-26 - [Argument Injection in Shell Command Wrappers]
**Vulnerability:** The `download_file` function constructed a `curl` command by passing the user-supplied URL directly as an argument. A URL starting with `-` could be interpreted as a flag (e.g., `-o/etc/passwd`).
**Learning:** Even when avoiding `sh -c` and using `Command::new()`, argument injection is possible if the underlying program parses arguments flexibly (like `curl` or `tar`).
**Prevention:** Validate inputs that are passed as arguments to external commands to ensure they don't look like flags (e.g., start with `-`), or use `--` delimiter if supported by the tool.

## 2025-10-26 - [Restricting Scriptable Shell Execution]
**Vulnerability:** `shell_execute` allowed arbitrary command execution.
**Learning:** "Trusted users" is a weak defense for RCE. Allowing arbitrary execution makes the application a foothold for attackers.
**Prevention:** Implemented an allowlist (`yt-dlp`, `ffmpeg`, `ffprobe`) to restrict the attack surface while maintaining functionality.
