-- Remove spot_reel_id from content_items
-- SQLite doesn't support DROP COLUMN before 3.35.0, so we recreate the table
CREATE TABLE content_items_backup AS SELECT
    id, title, description, content_type, content_path, adapter_id,
    duration_minutes, tags, node_accessibility, created_at, updated_at,
    transformer_scripts, is_dj_accessible
FROM content_items;

DROP TABLE content_items;

CREATE TABLE content_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    content_type TEXT NOT NULL,
    content_path TEXT NOT NULL,
    adapter_id INTEGER REFERENCES scripts(id) ON DELETE SET NULL,
    duration_minutes INTEGER,
    tags TEXT,
    node_accessibility TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    transformer_scripts TEXT,
    is_dj_accessible BOOLEAN NOT NULL DEFAULT 0
);

INSERT INTO content_items SELECT * FROM content_items_backup;
DROP TABLE content_items_backup;

-- Drop spot_reel_items and spot_reels
DROP TABLE IF EXISTS spot_reel_items;
DROP TABLE IF EXISTS spot_reels;
