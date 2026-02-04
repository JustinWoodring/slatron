## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-23 - [SSRF via Scriptable Download Tool]
**Vulnerability:** The `download_file` function exposed to Rhai scripts in both `slatron-server` and `slatron-node` used `curl` without validating the URL protocol.
**Learning:** Tools like `curl` support many protocols (`file://`, `ftp://`, etc.) which can be abused for SSRF or LFI if user input is passed directly to them. Simply relying on "it downloads stuff" hides the complexity of the underlying tool's capabilities.
**Prevention:** Always whitelist allowed protocols (e.g., `http://`, `https://`) when using generic download tools or libraries, especially when input comes from a scriptable environment.

## 2025-05-23 - [Command Injection via sh -c in Node]
**Vulnerability:** `slatron-node` exposed `shell_execute` which passed script-provided strings directly to `sh -c`. This allowed arbitrary command execution and privilege escalation on the display node.
**Learning:** Using `sh -c` with any user-influenced input is a critical risk. Even if the input is "just" arguments for a known tool, shell operators (`;`, `|`, `$()`) can be injected.
**Prevention:** Avoid invoking the shell (`sh`, `bash`, `cmd`). Execute binaries directly using `std::process::Command` (Rust) or `child_process.spawn` (Node). If a single string interface is required, parse it into `(exe, args)` manually or use a secure tokenizer, and validate the executable against a strict allowlist.
