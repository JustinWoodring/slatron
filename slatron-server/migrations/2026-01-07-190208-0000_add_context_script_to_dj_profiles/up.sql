ALTER TABLE dj_profiles ADD COLUMN context_script_id INTEGER REFERENCES scripts(id);
