## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-23 - [SSRF via Scriptable Download Tool]
**Vulnerability:** The `download_file` function exposed to Rhai scripts in both `slatron-server` and `slatron-node` used `curl` without validating the URL protocol.
**Learning:** Tools like `curl` support many protocols (`file://`, `ftp://`, etc.) which can be abused for SSRF or LFI if user input is passed directly to them. Simply relying on "it downloads stuff" hides the complexity of the underlying tool's capabilities.
**Prevention:** Always whitelist allowed protocols (e.g., `http://`, `https://`) when using generic download tools or libraries, especially when input comes from a scriptable environment.

## 2025-10-26 - [Argument Injection in Allowed Tools]
**Vulnerability:** Even when restricting `shell_execute` to safe tools like `yt-dlp`, argument injection via flags like `--exec` can still allow arbitrary code execution.
**Learning:** Allowlisting binaries is insufficient if the binaries themselves have features that execute commands. Deep inspection of arguments is required for complex tools.
**Prevention:** When allowing execution of powerful tools, one must deny-list known dangerous flags (e.g., `--exec`, `-I` for `yt-dlp`) or, preferably, allow-list only specific safe flags.
