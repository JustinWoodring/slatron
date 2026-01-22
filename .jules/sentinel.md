## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-24 - [Node Script RCE Vulnerability]
**Vulnerability:** The `slatron-node` client exposed `shell_execute` (sh -c) to `transformer` scripts, which are synced from the server and executed automatically.
**Learning:** This extended the server-side RCE risk to all edge devices (displays), turning them into potential botnet nodes if an editor account is compromised.
**Prevention:** Removed `shell_execute` from `slatron-node`. Dangerous capabilities should not be exposed to remotely synced scripts on edge devices.
