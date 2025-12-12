# Slatron 📺

**A robust, distributed TV scheduling and digital signage system.**

Slatron allows you to manage content, schedule broadcasts, and control playback nodes across your network from a centralized, web-based dashboard. It's built for reliability using Rust and modern web technologies.

<img width="1460" height="1017" alt="image" src="https://github.com/user-attachments/assets/8ff8ed0e-2c4f-41bc-a05b-94cc5d04ec15" />


---

## 🌟 Key Features

*   **Centralized Management**: Manage multiple playback nodes (TVs, screens) from a single server.
*   **Flexible Scheduling**: Drag-and-drop schedule grid with layered priorities and interrupt scheduling.
*   **Role-Based Access Control (RBAC)**: Secure your station with `Admin`, `Editor`, and `Viewer` roles.
*   **Real-time Monitoring**: Live status updates from nodes via WebSockets.
*   **Scripting Engine**: Use Rhai scripts for dynamic content loading, overlays, and playback logic.
*   **Resilient Playback**: Local caching on nodes ensures playback continues even if the network goes down.
*   **Content Library**: Organize video files and manage valid paths per node.

---

## 🚀 Quick Start

### Prerequisites
*   **Rust** (1.75+)
*   **Node.js** (18+)
*   **MPV** (Must be installed on playback nodes)
*   **SQLite** (For the server database)

### 1. Setting up the Server (`slatron-server`)

The server manages the database, API, and serving the web interface. It will automatically initialize the database and run migrations on startup.

```bash
cd slatron-server

# 1. Setup Database Directory
mkdir -p data

# 2. Build & Run (Embedding the UI for ease of use)
# This requires `npm` to be in your path, as it builds the UI automatically.
cargo run --release --features embed-ui -- --generate-config > config.toml
# Edit config.toml if needed, then:
cargo run --release --features embed-ui
```

The server will start on `http://0.0.0.0:8080` (or your configured port).
**Default Login**: `admin` / `admin`

### 2. Setting up a Node (`slatron-node`)

The node runs on the computer connected to the display (e.g., a Raspberry Pi or mini PC).

```bash
cd slatron-node

# 1. Generate a config template
cargo run --release -- --generate-config > config.toml

# 2. Edit config.toml
# - Set `server_url` to your server's WebSocket URL (e.g., ws://192.168.1.10:8080/ws)
# - You will need a `secret_key`. Log into the Web UI, go to "Nodes", create a node, and copy its secret.

# 3. Run the node
cargo run --release
```

**Note**: Ensure `mpv` is installed and available in the system PATH. Slatron will launch and control it automatically.

---

## ⚙️ Configuration

Both components support CLI arguments for configuration management.

### Command Line Interface
- **Generate Template**: `slatron-server --generate-config` (Prints to stdout)
- **Specify File**: `slatron-server --config my_config.toml`
- **Default**: Looks for `config.toml` in the current directory.

### Server Configuration (`config.toml`)

```toml
[server]
host = "0.0.0.0"
port = 8080
# Optional: Path to serve UI files from. 
# If omitted and compiled with `embed-ui`, it uses the internal zip.
# ui_path = "./static"

[server.https]
enabled = false
# cert_path = "/path/to/cert.pem"
# key_path = "/path/to/key.pem"

[database]
url = "sqlite://data/slatron.db"

[jwt]
secret = "CHANGE_THIS_IN_PRODUCTION"
expiration_hours = 24
```

### Node Configuration (`config.toml`)
```toml
node_name = "Lobby Display"
server_url = "ws://localhost:8080/ws"
secret_key = "PASTE_SECRET_FROM_UI"
heartbeat_interval_secs = 5
mpv_socket_path = "/tmp/mpv-socket"
```

---

## 📦 Deployment & Frontend Embedding

### Single Binary Deployment
To simplify deployment, you can embed the frontend files directly into the server binary.

1.  Enable the `embed-ui` feature during compilation:
    ```bash
    cargo build --release --bin slatron-server --features embed-ui
    ```
    *This triggers a build script that runs `npm run build` in `slatron-ui` and zips the result.*

2.  Run the binary anywhere. It will automatically extract the UI to an `embedded_ui` directory at startup and serve it.
    *   No need to copy a `static/` folder manually!

### HTTPS Support
Secure your server by enabling HTTPS in `config.toml`. You will need a certificate and private key (e.g., from Let's Encrypt).

---

## 🛠 Architecture

### Components
*   **slatron-server** (Rust/Axum): The brain. Handles database, API, Auth, and WebSockets.
*   **slatron-node** (Rust): The player. Connects to server, downloads content/schedules, and controls MPV via IPC.
*   **slatron-ui** (React/Vite): The face. communicating via REST API.

### Scripting (Rhai)
Slatron uses the Rhai scripting language for safety and flexibility.
*   **Content Loaders**: Custom logic to fetch/prepare content (e.g., download from YouTube).
*   **Global Scripts**: Run logic based on playback events (e.g., "If content < 10s, loop it").

### Heartbeat Monitoring
Nodes send heartbeats every 5 seconds. If the server doesn't hear from a node for **30 seconds**, it automatically marks it as `Offline` in the dashboard.

---

## Scripting API Reference (Rhai)

Scripts in Slatron allow you to customize playback behavior and interactivity.

### Lifecycle Hooks
These functions are called automatically by the node during the playback lifecycle.

*   `fn transform(settings)` -> `Map`
    *   **When**: Called *before* playback starts.
    *   **Purpose**: Update settings (volume, loop, start/end time) based on custom logic.
    *   **Returns**: A Map of settings to override.
*   `fn on_load(settings)`
    *   **When**: Called immediately *after* playback starts.
    *   **Purpose**: Apply overlays, start auxiliary processes, or log events.
*   `fn on_unload(settings)`
    *   **When**: Called *before* the content changes or stops.
    *   **Purpose**: Cleanup (e.g., remove overlays).

### Core Functions

#### Playback Control
*   `set_volume(ctx, volume: int)`: Set playback volume (0-100).
*   `set_loop(ctx, enabled: bool)`: Enable/Disable looping.
*   `set_start_time(ctx, seconds: float)`: Start playback from offset.
*   `set_end_time(ctx, seconds: float)`: Stop playback at offset.
*   `mpv_send(command_map)`: Send a raw JSON command to MPV (Node-side only).

#### System & Content
*   `download_file(url, output_path)`: Downloads a file. Supports `~` expansion on nodes.
*   `shell_execute(command)`: Executes a shell command and returns output.
*   `get_env(key)`: Returns value of an environment variable.

#### Overlays
*   `mpv_overlay(path, x, y, opacity)`: Display an image overlay.
*   `mpv_text(text, x, y, size, color)`: Display text overlay.
*   `mpv_remove_overlay(id)`: Remove an overlay.
*   `get_video_width()` / `get_video_height()`: Get resolution of current content.

---

## 📄 License

Slatron is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**.
See the [LICENSE](LICENSE) file for details.

---
**Created by Justin Woodring &SLATRON AUTHORS**
