DELETE FROM global_settings WHERE key IN ('station_name', 'station_bug_enabled', 'station_bug_image');
DELETE FROM scripts WHERE name = 'Station Bug' AND is_builtin = true;
