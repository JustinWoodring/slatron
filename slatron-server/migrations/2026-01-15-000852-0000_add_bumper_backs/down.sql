-- Remove bumper_back_id column from bumpers
-- SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
CREATE TABLE bumpers_backup AS SELECT
    id, name, bumper_type, description, is_template,
    template_content, rendered_path, duration_ms, is_builtin,
    created_at, updated_at, last_rendered_at
FROM bumpers;

DROP TABLE bumpers;

CREATE TABLE bumpers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    bumper_type TEXT NOT NULL,
    description TEXT,
    is_template BOOLEAN NOT NULL DEFAULT 0,
    template_content TEXT,
    rendered_path TEXT,
    duration_ms INTEGER,
    is_builtin BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_rendered_at TIMESTAMP
);

INSERT INTO bumpers SELECT
    id, name, bumper_type, description, is_template,
    template_content, rendered_path, duration_ms, is_builtin,
    created_at, updated_at, last_rendered_at
FROM bumpers_backup;

DROP TABLE bumpers_backup;

-- Drop bumper_backs table
DROP TABLE bumper_backs;
