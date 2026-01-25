+++
title = "Getting Started"
description = "Installation and basic configuration of Slatron Server and Node."
weight = 1
+++

## Prerequisites

### Required
*   **Rust** (1.75+)
*   **Node.js** (18+) - For building the UI
*   **SQLite** - For the server database

### Required on Playback Nodes
*   **MPV** - Media player with IPC support
*   **yt-dlp** - For YouTube content support (MPV integration)

### Required on Server (for Bumpers)
*   **MLT Framework** (`melt` command)
*   **FFmpeg** (`ffprobe` command)

## Installation

### 1. Setting up the Server (`slatron-server`)

The server manages the database, API, and serves the web interface.

```bash
cd slatron-server
mkdir -p data
cargo run --release --features embed-ui -- --generate-config > config.toml
cargo run --release --features embed-ui
```

### 2. Setting up a Node (`slatron-node`)

The node runs on the computer connected to the display.

```bash
cd slatron-node
cargo run --release -- --generate-config > config.toml
# Edit config.toml with your server_url and secret_key
cargo run --release
```
