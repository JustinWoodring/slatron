ALTER TABLE dj_profiles DROP COLUMN context_script_id;
ALTER TABLE dj_profiles ADD COLUMN context_script_ids TEXT;
