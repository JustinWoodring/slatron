-- Create new table with updated check constraint
CREATE TABLE scripts_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  script_type TEXT NOT NULL CHECK (script_type IN ('content_loader', 'overlay', 'global', 'transformer', 'adapter')),
  script_content TEXT NOT NULL,
  parameters_schema TEXT,
  is_builtin BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Copy data
INSERT INTO scripts_new SELECT * FROM scripts;

-- Drop old table
DROP TABLE scripts;

-- Rename new table
ALTER TABLE scripts_new RENAME TO scripts;
