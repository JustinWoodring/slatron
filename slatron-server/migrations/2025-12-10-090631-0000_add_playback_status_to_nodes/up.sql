ALTER TABLE nodes ADD COLUMN current_content_id INTEGER REFERENCES content_items(id);
ALTER TABLE nodes ADD COLUMN playback_position_secs FLOAT;
