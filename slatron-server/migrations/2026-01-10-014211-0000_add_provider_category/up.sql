-- Add provider_category column
ALTER TABLE ai_providers ADD COLUMN provider_category TEXT NOT NULL DEFAULT 'llm';

-- Migrate existing data based on provider_type
UPDATE ai_providers SET provider_category = 'tts' WHERE provider_type IN ('orpheus', 'google', 'gemini-tts', 'elevenlabs');
-- Everything else defaults to 'llm' from the column default, effectively covering 'openai', 'ollama', 'lmstudio', etc.
