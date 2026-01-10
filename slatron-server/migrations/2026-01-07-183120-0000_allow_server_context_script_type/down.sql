PRAGMA foreign_keys=off;

-- Revert to previous constraint (without 'server_context')
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

-- Copy data (Warning: Data with script_type='server_context' will fail to insert and be lost if not handled, but for down migration we assume loss or error is acceptable/expected)
INSERT INTO scripts_new SELECT * FROM scripts WHERE script_type != 'server_context';

-- Drop old table
DROP TABLE scripts;

-- Rename new table
ALTER TABLE scripts_new RENAME TO scripts;

PRAGMA foreign_keys=on;
