-- Create bumper_backs table for base video files
CREATE TABLE bumper_backs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    file_path TEXT NOT NULL,
    duration_ms INTEGER,
    is_builtin BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add bumper_back_id to bumpers table
ALTER TABLE bumpers ADD COLUMN bumper_back_id INTEGER REFERENCES bumper_backs(id);
