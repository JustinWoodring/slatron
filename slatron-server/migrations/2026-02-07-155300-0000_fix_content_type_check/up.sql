-- SQLite does not support altering CHECK constraints, so we must recreate the table.
-- This migration adds 'spot_reel' to the content_type CHECK constraint.

-- 1. Create new table with updated CHECK constraint (includes all current columns)
CREATE TABLE content_items_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    content_type VARCHAR(50) NOT NULL CHECK(content_type IN ('local_file', 'remote_url', 'adapter', 'spot_reel')),
    content_path VARCHAR(1024) NOT NULL,
    adapter_id INTEGER,
    duration_minutes INTEGER,
    tags TEXT,
    node_accessibility TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    transformer_scripts TEXT,
    is_dj_accessible BOOLEAN NOT NULL DEFAULT 0,
    spot_reel_id INTEGER REFERENCES spot_reels(id) ON DELETE SET NULL,
    FOREIGN KEY (adapter_id) REFERENCES scripts(id) ON DELETE SET NULL
);

-- 2. Copy data
INSERT INTO content_items_new (id, title, description, content_type, content_path, adapter_id, duration_minutes, tags, node_accessibility, created_at, updated_at, transformer_scripts, is_dj_accessible, spot_reel_id)
    SELECT id, title, description, content_type, content_path, adapter_id, duration_minutes, tags, node_accessibility, created_at, updated_at, transformer_scripts, is_dj_accessible, spot_reel_id
    FROM content_items;

-- 3. Drop old table
DROP TABLE content_items;

-- 4. Rename new table
ALTER TABLE content_items_new RENAME TO content_items;
