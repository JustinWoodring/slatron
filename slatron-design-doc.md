# Slatron - TV Scheduling Software Design Document

**Version:** 1.0  
**Date:** December 5, 2025  
**Document Purpose:** Comprehensive technical specification for AI-assisted implementation

---

## Executive Summary

Slatron is a distributed TV scheduling system with a centralized server, distributed playback nodes, and web-based administration. The system uses React for UI, Rust for both server and nodes, Rhai for scripting, and MPV for playback.

### Core Architecture
- **Server**: Rust + Axum/Actix + Diesel + SQLite
- **Node**: Rust + MPV (IPC socket)
- **UI**: React + TypeScript + Tailwind + Zustand
- **Communication**: WebSockets (server↔node), REST API (UI↔server)
- **Scripting**: Rhai (embedded Rust scripting)

### Key Features
1. Multi-day schedule grid with drag-and-drop (15-min increments, zoom to 1-min)
2. Layered scheduling with priority-based override
3. Content library with node-specific path awareness
4. Extensible plugin system via Rhai scripts
5. Real-time node monitoring and control
6. Role-based permissions (admin/editor/viewer)

---

## 1. System Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Scheduling Server                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Web UI     │  │  REST API    │  │  WebSocket   │      │
│  │  (React)     │──│  (Axum)      │──│   Server     │      │
│  └──────────────┘  └──────────────┘  └──────┬───────┘      │
│                          │                   │               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            Core Services & Rhai Engine              │  │
│  └───────────────────────┬───────────────────────────────┘  │
│                          │                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │          Database (Diesel + SQLite)                  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ WebSocket (JSON messages)
                          │
        ┌─────────────────┴──────────────────┬──────────────┐
        │                                    │              │
   ┌────▼─────┐                        ┌────▼─────┐   ┌────▼─────┐
   │  Node 1  │                        │  Node 2  │   │  Node N  │
   │          │                        │          │   │          │
   │ Runtime  │                        │ Runtime  │   │ Runtime  │
   │    ↓     │                        │    ↓     │   │    ↓     │
   │   MPV    │                        │   MPV    │   │   MPV    │
   └──────────┘                        └──────────┘   └──────────┘
```

---

## 2. Database Schema

All tables use Diesel ORM with SQLite as default (swappable via Diesel adapters).

### 2.1 Users Table
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL CHECK(role IN ('admin', 'editor', 'viewer')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 2.2 Nodes Table
```sql
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) UNIQUE NOT NULL,
    secret_key VARCHAR(512) NOT NULL,
    ip_address VARCHAR(45),
    status VARCHAR(50) NOT NULL DEFAULT 'offline' CHECK(status IN ('online', 'offline', 'error')),
    last_heartbeat TIMESTAMP,
    available_paths TEXT, -- JSON array
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 2.3 Schedules Table
```sql
CREATE TABLE schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    schedule_type VARCHAR(50) NOT NULL CHECK(schedule_type IN ('weekly', 'one_off')),
    priority INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 2.4 Schedule Blocks Table
```sql
CREATE TABLE schedule_blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id INTEGER NOT NULL,
    content_id INTEGER,
    day_of_week INTEGER CHECK(day_of_week BETWEEN 0 AND 6), -- 0=Sunday for weekly
    specific_date DATE, -- For one-off schedules
    start_time TIME NOT NULL, -- HH:MM:SS
    duration_minutes INTEGER NOT NULL,
    script_id INTEGER, -- Optional per-block script
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (schedule_id) REFERENCES schedules(id) ON DELETE CASCADE,
    FOREIGN KEY (content_id) REFERENCES content_items(id) ON DELETE SET NULL,
    FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE SET NULL
);
```

### 2.5 Content Items Table
```sql
CREATE TABLE content_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    content_type VARCHAR(50) NOT NULL CHECK(content_type IN ('local_file', 'remote_url', 'adapter')),
    content_path VARCHAR(1024) NOT NULL,
    adapter_id INTEGER,
    duration_minutes INTEGER,
    tags TEXT, -- JSON array
    node_accessibility TEXT, -- JSON: {"node_id": bool}
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (adapter_id) REFERENCES scripts(id) ON DELETE SET NULL
);
```

### 2.6 Scripts Table
```sql
CREATE TABLE scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    script_type VARCHAR(50) NOT NULL CHECK(script_type IN ('content_loader', 'overlay', 'global')),
    script_content TEXT NOT NULL,
    parameters_schema TEXT, -- JSON schema
    is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 2.7 Node Schedule Assignments Table
```sql
CREATE TABLE node_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER NOT NULL,
    schedule_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (schedule_id) REFERENCES schedules(id) ON DELETE CASCADE,
    UNIQUE(node_id, schedule_id)
);
```

### 2.8 Permissions Table
```sql
CREATE TABLE permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    resource_type VARCHAR(50) NOT NULL CHECK(resource_type IN ('node', 'schedule')),
    resource_id INTEGER NOT NULL,
    permission_level VARCHAR(50) NOT NULL CHECK(permission_level IN ('read', 'write', 'admin')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

### 2.9 Global Settings Table
```sql
CREATE TABLE global_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key VARCHAR(255) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

## 3. WebSocket Protocol

### 3.1 Server → Node Messages

```typescript
type ServerMessage = 
  | { type: "auth_response", success: boolean, message: string }
  | { type: "schedule_updated", timestamp: string }
  | { type: "command", command: NodeCommand }
  | { type: "heartbeat_ack" };

type NodeCommand =
  | { action: "play" }
  | { action: "pause" }
  | { action: "stop" }
  | { action: "seek", position_secs: number }
  | { action: "load_content", content_id: number }
  | { action: "reload_schedule" }
  | { action: "shutdown" };
```

### 3.2 Node → Server Messages

```typescript
type NodeMessage =
  | { type: "authenticate", node_name: string, secret_key: string }
  | { 
      type: "heartbeat", 
      current_content_id: number | null,
      playback_position_secs: number | null,
      status: "playing" | "paused" | "stopped" | "error",
      cpu_usage_percent: number,
      memory_usage_mb: number,
      errors: string[]
    }
  | { type: "request_schedule" }
  | { type: "report_paths", available_paths: string[] }
  | { type: "content_error", content_id: number, error: string };
```

---

## 4. REST API Endpoints

### 4.1 Authentication
```
POST /api/auth/login
Body: { username, password }
Response: { token, user: { id, username, role } }

POST /api/auth/logout
Headers: Authorization: Bearer <token>
```

### 4.2 Schedules
```
GET    /api/schedules
POST   /api/schedules
PUT    /api/schedules/:id
DELETE /api/schedules/:id

GET    /api/schedules/:id/blocks
POST   /api/schedules/:id/blocks
PUT    /api/schedules/:id/blocks/:block_id
DELETE /api/schedules/:id/blocks/:block_id

GET    /api/schedules/collapsed?node_id=1&date=2025-12-05
```

### 4.3 Content
```
GET    /api/content
POST   /api/content
PUT    /api/content/:id
DELETE /api/content/:id
```

### 4.4 Nodes
```
GET    /api/nodes
POST   /api/nodes
DELETE /api/nodes/:id
POST   /api/nodes/:id/command
```

### 4.5 Scripts
```
GET    /api/scripts
POST   /api/scripts
PUT    /api/scripts/:id
DELETE /api/scripts/:id
POST   /api/scripts/:id/validate
```

### 4.6 Users & Permissions
```
GET    /api/users
POST   /api/users
PUT    /api/users/:id
DELETE /api/users/:id

GET    /api/permissions
POST   /api/permissions
DELETE /api/permissions/:id
```

---

## 5. Scheduling Algorithm

### 5.1 Collapsed Schedule Calculation

```rust
// Pseudocode for schedule collapse
fn calculate_collapsed_schedule(node_id, date) {
    // 1. Get all schedules assigned to node
    let schedules = get_node_schedules(node_id);
    
    // 2. Sort by priority (descending)
    schedules.sort_by(|a, b| b.priority.cmp(&a.priority));
    
    // 3. Create 1440-minute timeline (24 hours)
    let mut timeline = vec![None; 1440];
    
    // 4. For each schedule (high priority first)
    for schedule in schedules {
        let blocks = get_blocks_for_date(schedule, date);
        
        for block in blocks {
            let start_min = block.start_time.minutes_since_midnight();
            let end_min = start_min + block.duration_minutes;
            
            // Fill timeline slots
            for minute in start_min..end_min {
                if timeline[minute].is_none() || 
                   timeline[minute].priority < schedule.priority {
                    timeline[minute] = Some(BlockInfo {
                        content_id: block.content_id,
                        script_id: block.script_id,
                        priority: schedule.priority,
                        schedule_name: schedule.name
                    });
                }
            }
        }
    }
    
    // 5. Collapse adjacent identical blocks
    collapse_timeline(timeline)
}
```

### 5.2 Time Block Rules

**Snapping:**
- UI allows free drag/drop placement
- Blocks snap to 15-minute increments on drop/resize
- When zoomed to 1-minute, can place at 1-minute boundaries.

**Content Duration Handling:**
- Shorter than block: Loop, bumper, or script-defined behavior
- Longer than block: Hard cut at boundary
- Zero content (gap): Show default "no signal" pattern

---

## 6. Rhai Plugin System

### 6.1 Content Loader Scripts

**Purpose:** Download or prepare content before playback

**Available Functions:**
```javascript
shell_execute(cmd: String) -> String
download_file(url: String, output: String) -> ()
get_env(key: String) -> String
```

**Expected Return:** String (path to playable file)

**Example - yt-dlp Wrapper:**
```rust
let url = params["url"];
let output_dir = params["output_dir"];
let quality = params["quality"]; // "best", "720p", "480p"

let cmd = `yt-dlp -f "bestvideo[height<=${quality}]+bestaudio" -o "${output_dir}/%(title)s.%(ext)s" "${url}"`;
shell_execute(cmd);

// Return path (simplified)
`${output_dir}/video.mp4`
```

### 6.2 Overlay Scripts

**Purpose:** Add visual elements via MPV overlays

**Available Functions:**
```javascript
mpv_overlay(image_path: String, x: i64, y: i64, opacity: f64) -> ()
mpv_text(text: String, x: i64, y: i64, size: i64, color: String) -> ()
mpv_remove_overlay(id: i64) -> ()
get_video_width() -> i64
get_video_height() -> i64
```

**Example - Station Bug:**
```rust
let logo_path = params["logo_path"];
let position = params["position"]; // "top-left", "top-right", etc.
let opacity = params["opacity"];

let coords = if position == "top-right" {
    #{x: get_video_width() - 110, y: 10}
} else {
    #{x: 10, y: 10}
};

mpv_overlay(logo_path, coords.x, coords.y, opacity);
```

### 6.3 Global Scripts

**Purpose:** Control playback behavior

**Available Functions:**
```javascript
mpv_set_loop(enabled: bool) -> ()
get_content_duration() -> f64
get_block_duration() -> f64
get_playback_position() -> f64
```

**Example - Auto-Loop Short Content:**
```rust
let content_duration = get_content_duration();
let block_duration = get_block_duration();

if content_duration < block_duration {
    mpv_set_loop(true);
}
```

### 6.4 Parameters Schema

Scripts expose UI configuration via JSON Schema:

```json
{
  "type": "object",
  "properties": {
    "logo_path": {
      "type": "string",
      "title": "Logo File Path"
    },
    "position": {
      "type": "string",
      "title": "Position",
      "enum": ["top-left", "top-right", "bottom-left", "bottom-right"]
    },
    "opacity": {
      "type": "number",
      "title": "Opacity",
      "minimum": 0,
      "maximum": 1,
      "default": 0.8
    }
  },
  "required": ["logo_path"]
}
```

---

## 7. Server Implementation Details

### 7.1 Project Structure
```
slatron-server/
├── Cargo.toml
├── migrations/
│   └── 2025_01_01_000000_initial/
│       ├── up.sql
│       └── builtin_scripts.sql
├── src/
│   ├── main.rs
│   ├── api/ (REST endpoints)
│   ├── models/ (Diesel models)
│   ├── services/ (Business logic)
│   ├── websocket/ (WS handler)
│   ├── rhai/ (Script engine)
│   └── auth/ (JWT & permissions)
└── static/ (React build output)
```

### 7.2 Key Dependencies
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
diesel = { version = "2.1", features = ["sqlite", "r2d2", "chrono"] }
tokio-tungstenite = "0.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonwebtoken = "9.2"
bcrypt = "0.15"
rhai = "1.16"
chrono = { version = "0.4", features = ["serde"] }
```

### 7.3 Schedule Service Core Logic

```rust
// Simplified collapse algorithm
pub fn calculate_collapsed_schedule(
    &self,
    node_id: i32,
    date: NaiveDate,
) -> Result<Vec<CollapsedBlock>> {
    let schedules = self.get_node_schedules(node_id)?;
    schedules.sort_by(|a, b| b.priority.cmp(&a.priority));
    
    let mut timeline: Vec<Option<(i32, ScheduleBlock)>> = vec![None; 1440];
    
    for schedule in schedules {
        let blocks = match schedule.schedule_type {
            ScheduleType::Weekly => {
                let dow = date.weekday().num_days_from_sunday() as i32;
                self.get_weekly_blocks(schedule.id, dow)?
            }
            ScheduleType::OneOff => {
                self.get_oneoff_blocks(schedule.id, date)?
            }
        };
        
        for block in blocks {
            let start = block.start_time.num_seconds_from_midnight() / 60;
            let end = start + block.duration_minutes as u32;
            
            for minute in start..end.min(1440) {
                let slot = &mut timeline[minute as usize];
                if slot.is_none() || slot.as_ref().unwrap().0 < schedule.priority {
                    *slot = Some((schedule.priority, block.clone()));
                }
            }
        }
    }
    
    // Collapse adjacent identical slots
    self.collapse_timeline(timeline)
}
```

---

## 8. Node Implementation Details

### 8.1 Project Structure
```
slatron-node/
├── Cargo.toml
├── config.toml
├── src/
│   ├── main.rs
│   ├── websocket/ (WS client)
│   ├── schedule/ (Cache & executor)
│   ├── mpv/ (IPC client)
│   ├── rhai/ (Script engine)
│   └── heartbeat.rs
└── cache/
    └── schedule.json
```

### 8.2 Key Dependencies
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rhai = "1.16"
sysinfo = "0.30"
```

### 8.3 WebSocket Client with Reconnection

```rust
pub async fn connect_and_run(&mut self) -> Result<()> {
    let mut delay = Duration::from_secs(5);
    let max_delay = Duration::from_secs(300);
    
    loop {
        match self.connect().await {
            Ok(ws) => {
                delay = Duration::from_secs(5); // Reset
                if let Err(e) = self.handle_connection(ws).await {
                    error!("Connection error: {}", e);
                }
            }
            Err(e) => error!("Failed to connect: {}", e),
        }
        
        sleep(delay).await;
        delay = (delay * 2).min(max_delay); // Exponential backoff
    }
}
```

### 8.4 MPV IPC Client

```rust
pub struct MpvClient {
    socket_path: String,
}

impl MpvClient {
    pub async fn play(&self, path: &str) -> Result<()> {
        self.send_command(json!({
            "command": ["loadfile", path, "replace"]
        })).await
    }
    
    pub async fn add_overlay(&self, path: &str, x: i32, y: i32, opacity: f64) -> Result<()> {
        self.send_command(json!({
            "command": ["overlay-add", 0, x, y, path, opacity]
        })).await
    }
    
    async fn send_command(&self, cmd: Value) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        let cmd_str = format!("{}\n", serde_json::to_string(&cmd)?);
        stream.write_all(cmd_str.as_bytes())?;
        
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        Ok(serde_json::from_str(&response)?)
    }
}
```

---

## 9. Web UI Implementation

### 9.1 Project Structure
```
slatron-ui/
├── package.json
├── src/
│   ├── components/
│   │   ├── ScheduleGrid/
│   │   │   ├── ScheduleGrid.tsx
│   │   │   ├── ScheduleBlock.tsx
│   │   │   └── TimeAxis.tsx
│   │   ├── ContentLibrary/
│   │   ├── NodeDashboard/
│   │   └── Common/
│   ├── stores/ (Zustand)
│   ├── api/
│   └── pages/
└── public/
```

### 9.2 Key Dependencies
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "zustand": "^4.4.7",
    "@tanstack/react-query": "^5.14.0",
    "react-router-dom": "^6.20.0",
    "axios": "^1.6.2",
    "date-fns": "^2.30.0",
    "react-beautiful-dnd": "^13.1.1"
  },
  "devDependencies": {
    "typescript": "^5.3.3",
    "tailwindcss": "^3.3.6",
    "vite": "^5.0.7"
  }
}
```

### 9.3 Schedule Grid Component (Simplified)

```typescript
export function ScheduleGrid({ scheduleId, startDate, numDays, zoomLevel }) {
  const { blocks, createBlock, updateBlock } = useScheduleStore();
  
  const minutesPerSlot = { hour: 60, '15min': 15, '5min': 5, '1min': 1 }[zoomLevel];
  const slotHeight = 60;
  
  return (
    <div className="flex flex-col">
      {/* Day headers */}
      <div className="flex">
        {Array.from({ length: numDays }).map((_, i) => (
          <div key={i} className="flex-1 text-center">
            {format(addDays(startDate, i), 'EEE MMM d')}
          </div>
        ))}
      </div>
      
      {/* Grid */}
      <div className="flex overflow-auto">
        <TimeAxis zoomLevel={zoomLevel} />
        {Array.from({ length: numDays }).map((_, dayIndex) => (
          <div key={dayIndex} className="flex-1 relative"
               onDrop={(e) => handleDrop(e, dayIndex)}>
            {blocks.filter(b => matchesDay(b, dayIndex)).map(block => (
              <ScheduleBlock 
                key={block.id}
                block={block}
                slotHeight={slotHeight}
                onResize={handleResize}
                onMove={handleMove}
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
```

### 9.4 Zustand Store Pattern

```typescript
export const useScheduleStore = create<ScheduleStore>((set, get) => ({
  schedules: [],
  blocks: [],
  
  fetchBlocks: async (scheduleId) => {
    const res = await apiClient.get(`/api/schedules/${scheduleId}/blocks`);
    set({ blocks: res.data.blocks });
  },
  
  createBlock: async (scheduleId, data) => {
    const res = await apiClient.post(`/api/schedules/${scheduleId}/blocks`, data);
    set({ blocks: [...get().blocks, res.data] });
  },
  
  updateBlock: async (scheduleId, blockId, data) => {
    await apiClient.put(`/api/schedules/${scheduleId}/blocks/${blockId}`, data);
    set({ 
      blocks: get().blocks.map(b => 
        b.id === blockId ? { ...b, ...data } : b
      ) 
    });
  },
}));
```

---

## 10. Error Handling

### 10.1 Content Loading Failures

**Node Behavior:**
1. Attempt to load content via adapter script
2. On failure:
   - Log error locally
   - Report to server via WebSocket
   - Execute fallback behavior (configurable):
     - `skip`: Move to next content
     - `retry`: Retry N times with delay
     - `error_screen`: Show error message
     - `previous`: Continue previous content

**Configuration (global_settings):**
```sql
INSERT INTO global_settings (key, value) VALUES
('content_error_behavior', 'skip'),
('content_error_retry_attempts', '3'),
('content_error_retry_delay_secs', '5');
```

### 10.2 Network Failures

**Node Disconnection:**
- Node detects disconnect
- Continues playing cached schedule
- Reconnects with exponential backoff (5s → 300s max)
- If offline > 24 hours: show warning overlay

**Server Disconnection:**
- Server marks node offline after 3 missed heartbeats (15s)
- UI shows red "offline" indicator
- On reconnect: node re-authenticates and syncs

### 10.3 MPV IPC Failures

**Behavior:**
- Detect IPC socket error
- Attempt to restart MPV (3 attempts, 10s delay)
- If all attempts fail:
  - Report critical error to server
  - Set node status to "error"
  - Admin can manually restart via command

---

## 11. Deployment

### 11.1 Server Deployment

**Binary:**
```bash
cargo build --release
./target/release/slatron-server --config config.toml
```

**Docker:**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-0
COPY --from=builder /app/target/release/slatron-server /usr/local/bin/
EXPOSE 8080
CMD ["slatron-server"]
```

### 11.2 Node Deployment

**config.toml:**
```toml
node_name = "studio-a"
server_url = "ws://192.168.1.100:8080"
secret_key = "node-secret-from-server"
heartbeat_interval_secs = 5
schedule_poll_interval_secs = 60
mpv_socket_path = "/tmp/mpv-socket"
```

**Systemd Service:**
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

---

## 12. Authentication & Authorization

### 12.1 User Roles

- **Admin**: Full access to everything
- **Editor**: Can edit content/schedules (if granted permission), read-only nodes
- **Viewer**: Read-only access to assigned resources

### 12.2 Permissions

**Resource Types:** `node`, `schedule`  
**Levels:** `read`, `write`, `admin`

**Example:**
- User has `write` on Schedule 1 → can edit blocks
- User has `read` on Node 1 → can view status, cannot send commands

### 12.3 JWT Tokens

- Username/password → JWT token
- Token contains: user_id, username, role
- Expires after 24 hours
- All API requests require `Authorization: Bearer <token>` header

### 12.4 Node Authentication

- Nodes use secret key (UUID generated on creation)
- Validated via WebSocket auth message
- No JWT tokens for nodes

---

## 13. Testing Strategy

### 13.1 Unit Tests

**Server:**
- Schedule collapse algorithm
- Permission checks
- JWT generation/validation

**Node:**
- Reconnection logic
- Schedule cache management
- MPV command serialization

### 13.2 Integration Tests

- Full API CRUD operations
- WebSocket message flows
- Authentication/authorization

### 13.3 Manual Test Checklist

- [ ] Create weekly/one-off schedules
- [ ] Drag-and-drop blocks
- [ ] Verify collapsed schedule
- [ ] Test node connection/heartbeat
- [ ] Send play/pause/stop commands
- [ ] Upload and execute Rhai scripts
- [ ] Test permission enforcement

---

## 14. Builtin Scripts

The following scripts are embedded in the server binary and seeded on first run:

### 14.1 yt-dlp Video Downloader
```rust
let url = params["url"];
let output_dir = params["output_dir"];
let quality = params["quality"];

let cmd = `yt-dlp -f "bestvideo[height<=${quality}]+bestaudio" -o "${output_dir}/%(title)s.%(ext)s" "${url}"`;
shell_execute(cmd);
`${output_dir}/video.mp4`
```

### 14.2 Station Bug Overlay
```rust
let logo_path = params["logo_path"];
let position = params["position"];
let opacity = params["opacity"];

let coords = match position {
    "top-left" => #{x: 10, y: 10},
    "top-right" => #{x: -110, y: 10},
    "bottom-left" => #{x: 10, y: -110},
    _ => #{x: -110, y: -110}
};

mpv_overlay(logo_path, coords.x, coords.y, opacity);
```

### 14.3 Loop Content Script
```rust
let content_duration = get_content_duration();
let block_duration = get_block_duration();

if content_duration < block_duration {
    mpv_set_loop(true);
}
```

### 14.4 No Signal Pattern
```rust
mpv_play("/builtin/test-pattern.mp4");
mpv_set_loop(true);
```

---

## 15. Configuration Reference

### 15.1 Server config.toml
```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite://data/slatron.db"

[jwt]
secret = "your-secret-key"
expiration_hours = 24

[logging]
level = "info"
```

### 15.2 Node config.toml
```toml
node_name = "studio-a"
server_url = "ws://192.168.1.100:8080"
secret_key = "node-secret-key"
heartbeat_interval_secs = 5
schedule_poll_interval_secs = 60
mpv_socket_path = "/tmp/mpv-socket"
offline_mode_warning_hours = 24
```

---

## 16. Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Setup Rust server project with Axum + Diesel
- [ ] Implement database schema and migrations
- [ ] Create REST API endpoints
- [ ] Implement WebSocket server
- [ ] Add JWT authentication

### Phase 2: Scheduling Engine
- [ ] Implement schedule collapse algorithm
- [ ] Create schedule CRUD operations
- [ ] Add node schedule assignments
- [ ] Implement collapsed schedule API

### Phase 3: Node Runtime
- [ ] Setup Rust node project
- [ ] Implement WebSocket client with reconnection
- [ ] Create MPV IPC client
- [ ] Implement schedule cache
- [ ] Create playback executor

### Phase 4: Rhai Integration
- [ ] Setup Rhai engine on server
- [ ] Setup Rhai engine on node
- [ ] Register MPV control functions
- [ ] Create builtin scripts
- [ ] Implement script validation API

### Phase 5: Web UI
- [ ] Setup React + Vite project
- [ ] Implement authentication pages
- [ ] Create schedule grid component
- [ ] Implement drag-and-drop
- [ ] Create content library UI
- [ ] Create node dashboard
- [ ] Add script editor

### Phase 6: Permissions & Polish
- [ ] Implement permission system
- [ ] Add role-based UI hiding
- [ ] Test error handling
- [ ] Add logging and monitoring
- [ ] Write documentation

---

## Appendix A: Database Seeds

```sql
-- Default admin user (password: admin)
INSERT INTO users (username, password_hash, role) VALUES
('admin', '$2b$12$KIXxkf.9f8D2E8LyH8Q8NeH8hYqH8.8H8Q8NeH8', 'admin');

-- Global settings
INSERT INTO global_settings (key, value, description) VALUES
('content_error_behavior', 'skip', 'skip | retry | error_screen | previous'),
('content_error_retry_attempts', '3', 'Retry attempts on content error'),
('content_error_retry_delay_secs', '5', 'Delay between retries'),
('node_heartbeat_timeout_secs', '15', 'Mark node offline after N seconds'),
('default_transition_type', 'cut', 'cut | fade');
```

---

## Appendix B: MPV IPC Reference

**Commands Used:**
- `loadfile <path> replace` - Load and play file
- `set_property pause <bool>` - Pause/unpause
- `stop` - Stop playback
- `seek <seconds> absolute` - Seek to position
- `overlay-add <id> <x> <y> <file> <opacity>` - Add overlay
- `get_property time-pos` - Get current position
- `get_property duration` - Get content duration

**Socket Location:**
- Default: `/tmp/mpv-socket`
- Configurable in node config

---

## Appendix C: API Response Examples

**GET /api/schedules/collapsed?node_id=1&date=2025-12-05**
```json
{
  "blocks": [
    {
      "start_time": "14:00:00",
      "duration_minutes": 120,
      "content": {
        "id": 42,
        "title": "Movie",
        "content_path": "/media/movie.mp4"
      },
      "script_id": null,
      "priority": 2,
      "schedule_name": "Prime Time"
    }
  ]
}
```

---

**END OF DOCUMENT**
