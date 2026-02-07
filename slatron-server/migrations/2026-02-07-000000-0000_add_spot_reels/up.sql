-- Create spot_reels table
CREATE TABLE spot_reels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create spot_reel_items table
CREATE TABLE spot_reel_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spot_reel_id INTEGER NOT NULL REFERENCES spot_reels(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL CHECK(item_type IN ('image', 'video', 'web')),
    item_path TEXT NOT NULL,
    display_duration_secs INTEGER NOT NULL DEFAULT 10,
    position INTEGER NOT NULL DEFAULT 0,
    title TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add spot_reel_id to content_items
ALTER TABLE content_items ADD COLUMN spot_reel_id INTEGER REFERENCES spot_reels(id) ON DELETE SET NULL;
