## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-23 - [SSRF via Scriptable Download Tool]
**Vulnerability:** The `download_file` function exposed to Rhai scripts in both `slatron-server` and `slatron-node` used `curl` without validating the URL protocol.
**Learning:** Tools like `curl` support many protocols (`file://`, `ftp://`, etc.) which can be abused for SSRF or LFI if user input is passed directly to them. Simply relying on "it downloads stuff" hides the complexity of the underlying tool's capabilities.
**Prevention:** Always whitelist allowed protocols (e.g., `http://`, `https://`) when using generic download tools or libraries, especially when input comes from a scriptable environment.

## 2025-05-24 - [Unauthenticated Node Schedule Exposure]
**Vulnerability:** The `/api/nodes/:id/schedule` endpoint was public, exposing full Rhai script source code and file paths to unauthenticated users.
**Learning:** `slatron-node` uses a WebSocket for commands but standard HTTP for fetching schedules. The HTTP client was not sharing the authentication context (Secret Key) used by the WebSocket.
**Prevention:** Ensure all API endpoints used by the Node client validate the `X-Node-Secret` header or use a shared session token if possible. Added `X-Node-Secret` check to `get_node_schedule`.
