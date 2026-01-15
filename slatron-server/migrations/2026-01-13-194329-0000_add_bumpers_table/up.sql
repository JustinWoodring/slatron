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
