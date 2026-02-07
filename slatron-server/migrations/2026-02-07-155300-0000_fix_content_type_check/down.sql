-- Revert: recreate content_items with the original CHECK constraint (without 'spot_reel')
-- First delete any spot_reel content items since they won't pass the old constraint
DELETE FROM content_items WHERE content_type = 'spot_reel';

CREATE TABLE content_items_old (
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
    transformer_scripts TEXT,
    is_dj_accessible BOOLEAN NOT NULL DEFAULT 0,
    spot_reel_id INTEGER REFERENCES spot_reels(id) ON DELETE SET NULL,
    FOREIGN KEY (adapter_id) REFERENCES scripts(id) ON DELETE SET NULL
);

INSERT INTO content_items_old (id, title, description, content_type, content_path, adapter_id, duration_minutes, tags, node_accessibility, created_at, updated_at, transformer_scripts, is_dj_accessible, spot_reel_id)
    SELECT id, title, description, content_type, content_path, adapter_id, duration_minutes, tags, node_accessibility, created_at, updated_at, transformer_scripts, is_dj_accessible, spot_reel_id
    FROM content_items;

DROP TABLE content_items;

ALTER TABLE content_items_old RENAME TO content_items;
