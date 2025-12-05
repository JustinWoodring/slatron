# Slatron Quick Start Guide

This guide will help you get Slatron up and running in 5 minutes.

## Prerequisites

Make sure you have installed:
- Rust (1.75+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Node.js (18+): Download from nodejs.org
- MPV: `brew install mpv` (macOS) or `apt install mpv` (Linux)
- Diesel CLI: `cargo install diesel_cli --no-default-features --features sqlite`

## Step 1: Start the Server

```bash
cd slatron-server

# Create database directory
mkdir -p data

# Run database migrations
diesel migration run

# Start the server
cargo run --release
```

The server will start on http://localhost:8080

**Default login credentials:**
- Username: `admin`
- Password: `admin`

## Step 2: Access the Web UI

Open your browser and navigate to http://localhost:8080

You should see the login page. Log in with the default credentials.

## Step 3: Register a Node (Optional)

If you want to test with a playback node:

1. In the web UI, go to "Nodes" → "Register Node"
2. Enter a name (e.g., "test-node")
3. Copy the generated secret key
4. Edit `slatron-node/config.toml`:
   ```toml
   node_name = "test-node"
   server_url = "ws://127.0.0.1:8080/ws"
   secret_key = "paste-your-secret-key-here"
   ```
5. Start MPV with IPC enabled:
   ```bash
   mpv --input-ipc-server=/tmp/mpv-socket --idle &
   ```
6. Start the node:
   ```bash
   cd slatron-node
   cargo run --release
   ```

## Step 4: Create Your First Schedule

1. Go to "Content" → "Add Content"
   - Title: "Test Video"
   - Type: local_file
   - Path: /path/to/your/video.mp4

2. Go to "Schedules" → "Create Schedule"
   - Name: "Test Schedule"
   - Type: weekly
   - Priority: 1

3. Add a time block:
   - Day: Monday (1)
   - Start Time: 14:00
   - Duration: 60 minutes
   - Content: Select "Test Video"

4. Assign the schedule to your node (if you registered one)

## Step 5: View the Collapsed Schedule

To see what will play on a node at a given time:

```bash
curl "http://localhost:8080/api/schedules/collapsed?node_id=1&date=2025-12-08" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

(Get your JWT token from the browser's localStorage after logging in)

## Development Mode

For development with hot-reload:

**Server:**
```bash
cd slatron-server
cargo watch -x run
```

**UI:**
```bash
cd slatron-ui
npm run dev
```

The UI will run on http://localhost:3000 and proxy API requests to the server.

## Next Steps

- Explore the Scripts page to see builtin Rhai scripts
- Create overlay scripts to add station bugs
- Set up multiple nodes for different locations
- Configure user roles and permissions

## Troubleshooting

**Database errors:**
- Make sure you ran `diesel migration run`
- Check that the `data/` directory exists

**Node connection errors:**
- Verify the server URL in node config.toml
- Check that the secret key matches what was generated
- Ensure the server is running and accessible

**MPV errors:**
- Make sure MPV is running with `--input-ipc-server`
- Check that the socket path in config.toml matches MPV's socket

**UI doesn't load:**
- Run `npm run build` in slatron-ui to build the static files
- Or run `npm run dev` for development mode

## Support

For more details, see:
- Full documentation: README.md
- Design specification: slatron-design-doc.md
