ALTER TABLE schedules ADD COLUMN dj_id INTEGER REFERENCES dj_profiles(id);
