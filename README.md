# Slatron - TV Scheduling Software

A distributed TV scheduling system with centralized server, distributed playback nodes, and web-based administration.

## Architecture

### Components

1. **slatron-server**: Rust-based server with REST API and WebSocket support
2. **slatron-node**: Rust-based playback node that controls MPV
3. **slatron-ui**: React-based web administration interface

### Technology Stack

- **Server**: Rust + Axum + Diesel + SQLite
- **Node**: Rust + MPV (IPC)
- **UI**: React + TypeScript + Tailwind + Zustand
- **Scripting**: Rhai embedded scripting language
- **Communication**: WebSockets (server↔node), REST API (UI↔server)

## Features

- Multi-day schedule grid with drag-and-drop
- Layered scheduling with priority-based override
- Content library with node-specific path awareness
- Extensible plugin system via Rhai scripts
- Real-time node monitoring and control
- Role-based permissions (admin/editor/viewer)

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Node.js 18 or later
- MPV media player (for playback nodes)
- SQLite 3

### Server Setup

```bash
cd slatron-server

# Install Diesel CLI
cargo install diesel_cli --no-default-features --features sqlite

# Create database directory
mkdir -p data

# Run migrations
diesel migration run

# Build and run
cargo build --release
./target/release/slatron-server
```

The server will start on `http://localhost:8080`

Default credentials:
- Username: `admin`
- Password: `admin`

### Node Setup

```bash
cd slatron-node

# Edit config.toml with your node settings
cp config.toml config.local.toml
# Edit config.local.toml with your server URL and secret key

# Build and run
cargo build --release
./target/release/slatron-node --config config.local.toml
```

**Important**: You need to register the node through the web UI first to get the secret key.

### UI Setup

```bash
cd slatron-ui

# Install dependencies
npm install

# Development mode
npm run dev

# Production build (outputs to ../slatron-server/static)
npm run build
```

In development, the UI runs on `http://localhost:3000` and proxies API requests to the server.

## Configuration

### Server (config.toml)

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite://data/slatron.db"

[jwt]
secret = "your-secret-key-change-in-production"
expiration_hours = 24

[logging]
level = "info"
```

### Node (config.toml)

```toml
node_name = "studio-a"
server_url = "ws://127.0.0.1:8080/ws"
secret_key = "your-secret-key-from-server"
heartbeat_interval_secs = 5
schedule_poll_interval_secs = 60
mpv_socket_path = "/tmp/mpv-socket"
offline_mode_warning_hours = 24
```

## MPV Setup

For nodes to control playback, MPV must be running with IPC enabled:

```bash
mpv --input-ipc-server=/tmp/mpv-socket --idle
```

## API Documentation

The REST API is documented in the design document. Key endpoints:

- `POST /api/auth/login` - Authentication
- `GET /api/schedules` - List schedules
- `GET /api/content` - List content items
- `GET /api/nodes` - List nodes
- `GET /api/scripts` - List scripts
- `GET /api/schedules/collapsed?node_id=1&date=2025-12-05` - Get collapsed schedule

## WebSocket Protocol

Nodes connect via WebSocket to `/ws` and exchange JSON messages:

### Node → Server
- `authenticate` - Initial authentication
- `heartbeat` - Status updates every 5 seconds
- `request_schedule` - Request schedule reload
- `report_paths` - Report available content paths
- `content_error` - Report playback errors

### Server → Node
- `auth_response` - Authentication result
- `schedule_updated` - Notify of schedule changes
- `command` - Playback commands (play, pause, stop, seek, etc.)
- `heartbeat_ack` - Heartbeat acknowledgment

## Rhai Scripting

Slatron supports three types of Rhai scripts:

### 1. Content Loaders
Download or prepare content before playback.

```rust
let url = params["url"];
let output_dir = params["output_dir"];
let cmd = `yt-dlp -o "${output_dir}/%(title)s.%(ext)s" "${url}"`;
shell_execute(cmd);
`${output_dir}/video.mp4`
```

### 2. Overlay Scripts
Add visual elements via MPV overlays.

```rust
let logo_path = params["logo_path"];
let x = get_video_width() - 110;
let y = 10;
mpv_overlay(logo_path, x, y, 0.8);
```

### 3. Global Scripts
Control playback behavior.

```rust
let content_duration = get_content_duration();
let block_duration = get_block_duration();
if content_duration < block_duration {
    mpv_set_loop(true);
}
```

## Deployment

### Docker (Server)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY slatron-server .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-0
COPY --from=builder /app/target/release/slatron-server /usr/local/bin/
EXPOSE 8080
CMD ["slatron-server"]
```

### Systemd (Node)

```ini
[Unit]
Description=Slatron Node
After=network.target

[Service]
ExecStart=/usr/local/bin/slatron-node --config /etc/slatron/config.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Development

### Project Structure

```
slatron/
├── slatron-server/       # Rust server
│   ├── src/
│   │   ├── api/          # REST API endpoints
│   │   ├── auth/         # JWT authentication
│   │   ├── models/       # Diesel models
│   │   ├── services/     # Business logic
│   │   ├── websocket/    # WebSocket handler
│   │   └── rhai_engine/  # Rhai scripting
│   ├── migrations/       # Database migrations
│   └── static/           # React build output
├── slatron-node/         # Rust node
│   └── src/
│       ├── mpv_client.rs # MPV IPC client
│       ├── websocket_client.rs
│       ├── schedule.rs   # Schedule cache
│       └── heartbeat.rs  # Heartbeat manager
└── slatron-ui/           # React UI
    └── src/
        ├── pages/        # Route pages
        ├── components/   # React components
        ├── stores/       # Zustand stores
        └── api/          # API client
```

### Building from Source

```bash
# Server
cd slatron-server
cargo build --release

# Node
cd slatron-node
cargo build --release

# UI
cd slatron-ui
npm install
npm run build
```

## License

See design document for full specification.

## Support

For issues and feature requests, please refer to the design document (slatron-design-doc.md).
