-- Delete existing scripts and settings to ensure clean slate for seeding code
DELETE FROM scripts;
DELETE FROM global_settings;

-- Add unique constraint to scripts name
CREATE UNIQUE INDEX idx_scripts_name_unique ON scripts(name);
