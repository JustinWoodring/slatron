-- Revert to original check constraint
CREATE TABLE scripts_old (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  script_type TEXT NOT NULL CHECK (script_type IN ('content_loader', 'overlay', 'global')),
  script_content TEXT NOT NULL,
  parameters_schema TEXT,
  is_builtin BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Copy data (excluding transformer types which aren't in original constraint)
INSERT INTO scripts_old SELECT * FROM scripts WHERE script_type IN ('content_loader', 'overlay', 'global');

-- Drop current table
DROP TABLE scripts;

-- Rename old table
ALTER TABLE scripts_old RENAME TO scripts;
