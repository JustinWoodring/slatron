DELETE FROM scripts WHERE is_builtin = TRUE;
DELETE FROM global_settings;
DELETE FROM users WHERE username = 'admin';
