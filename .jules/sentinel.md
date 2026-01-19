## 2025-05-23 - [Rhai Script RCE Capability]
**Vulnerability:** The `slatron-server` Rhai engine exposes a `shell_execute` function to `content_loader` and `transformer` scripts, which executes arbitrary shell commands using `sh -c`.
**Learning:** This architectural decision allows any user with `editor` privileges (who can create/update scripts) to execute arbitrary code on the server. While access is restricted to editors, this bypasses the typical "application admin vs server admin" boundary.
**Prevention:** In the future, this should be restricted to a specific allowlist of commands or sandboxed entirely. For now, it is a known risk inherent to the design of the system which uses external tools like `curl`, `ffmpeg`, etc. driven by user scripts.

## 2025-05-24 - [Unrestricted Shell Execution in Nodes]
**Vulnerability:** `slatron-node` exposed `shell_execute` to scripts synced from the server, allowing compromised servers (or editors) to execute arbitrary commands on all display nodes (RCE).
**Learning:** Even though nodes are "clients", they execute logic pushed from the server. Trusted execution environments must be strictly defined. The presence of `shell_execute` in the node's engine was a critical oversight.
**Prevention:** Removed `shell_execute` from `slatron-node`. Added a regression test (`rhai_engine/security_test.rs`) to prevent re-introduction.
