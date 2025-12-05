-- Users Table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL CHECK(role IN ('admin', 'editor', 'viewer')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Nodes Table
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) UNIQUE NOT NULL,
    secret_key VARCHAR(512) NOT NULL,
    ip_address VARCHAR(45),
    status VARCHAR(50) NOT NULL DEFAULT 'offline' CHECK(status IN ('online', 'offline', 'error')),
    last_heartbeat TIMESTAMP,
    available_paths TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Schedules Table
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

-- Scripts Table
CREATE TABLE scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    script_type VARCHAR(50) NOT NULL CHECK(script_type IN ('content_loader', 'overlay', 'global')),
    script_content TEXT NOT NULL,
    parameters_schema TEXT,
    is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Content Items Table
CREATE TABLE content_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    content_type VARCHAR(50) NOT NULL CHECK(content_type IN ('local_file', 'remote_url', 'adapter')),
    content_path VARCHAR(1024) NOT NULL,
    adapter_id INTEGER,
    duration_minutes INTEGER,
    tags TEXT,
    node_accessibility TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (adapter_id) REFERENCES scripts(id) ON DELETE SET NULL
);

-- Schedule Blocks Table
CREATE TABLE schedule_blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id INTEGER NOT NULL,
    content_id INTEGER,
    day_of_week INTEGER CHECK(day_of_week BETWEEN 0 AND 6),
    specific_date DATE,
    start_time TIME NOT NULL,
    duration_minutes INTEGER NOT NULL,
    script_id INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (schedule_id) REFERENCES schedules(id) ON DELETE CASCADE,
    FOREIGN KEY (content_id) REFERENCES content_items(id) ON DELETE SET NULL,
    FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE SET NULL
);

-- Node Schedule Assignments Table
CREATE TABLE node_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER NOT NULL,
    schedule_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (schedule_id) REFERENCES schedules(id) ON DELETE CASCADE,
    UNIQUE(node_id, schedule_id)
);

-- Permissions Table
CREATE TABLE permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    resource_type VARCHAR(50) NOT NULL CHECK(resource_type IN ('node', 'schedule')),
    resource_id INTEGER NOT NULL,
    permission_level VARCHAR(50) NOT NULL CHECK(permission_level IN ('read', 'write', 'admin')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Global Settings Table
CREATE TABLE global_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key VARCHAR(255) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_nodes_status ON nodes(status);
CREATE INDEX idx_schedules_priority ON schedules(priority);
CREATE INDEX idx_schedule_blocks_schedule_id ON schedule_blocks(schedule_id);
CREATE INDEX idx_schedule_blocks_day_of_week ON schedule_blocks(day_of_week);
CREATE INDEX idx_schedule_blocks_specific_date ON schedule_blocks(specific_date);
CREATE INDEX idx_node_schedules_node_id ON node_schedules(node_id);
CREATE INDEX idx_permissions_user_id ON permissions(user_id);
CREATE INDEX idx_global_settings_key ON global_settings(key);
