-- AI Providers
CREATE TABLE ai_providers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  provider_type TEXT NOT NULL, -- 'openai', 'ollama', 'gemini'
  api_key TEXT,
  endpoint_url TEXT,
  model_name TEXT,
  is_active BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- DJ Profiles
CREATE TABLE dj_profiles (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  personality_prompt TEXT NOT NULL,
  voice_config_json TEXT NOT NULL, -- JSON blob for TTS config
  context_depth INTEGER NOT NULL DEFAULT 5,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- DJ Memories
CREATE TABLE dj_memories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  dj_id INTEGER NOT NULL REFERENCES dj_profiles(id) ON DELETE CASCADE,
  memory_type TEXT NOT NULL, -- 'track_intro', 'life_event', 'user_interaction'
  content TEXT NOT NULL,
  importance_score INTEGER NOT NULL DEFAULT 1,
  happened_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Alter content_items
ALTER TABLE content_items ADD COLUMN is_dj_accessible BOOLEAN NOT NULL DEFAULT 0;
