INSERT INTO scripts (name, description, script_type, script_content, parameters_schema, is_builtin, created_at, updated_at)
VALUES 
('Loop Content', 'Enables looping for this content.', 'transformer', 
'// Loop Content
fn transform(settings) {
    set_loop(settings, true);
    return settings;
}', 
'{}', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

('Mute Audio', 'Mutes audio playback.', 'transformer', 
'// Mute Audio
fn transform(settings) {
    set_volume(settings, 0);
    return settings;
}', 
'{}', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

('Start at 10s', 'Starts playback at 10 seconds.', 'transformer', 
'// Start at 10s
fn transform(settings) {
    set_start_time(settings, 10.0);
    return settings;
}', 
'{}', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
