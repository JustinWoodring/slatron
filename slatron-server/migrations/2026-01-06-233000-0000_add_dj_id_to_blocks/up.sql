ALTER TABLE schedule_blocks ADD COLUMN dj_id INTEGER REFERENCES dj_profiles(id);
