-- Seed Global Settings
INSERT INTO global_settings (key, value, description, updated_at)
VALUES 
('station_name', 'Slatron TV', 'The name of the station.', CURRENT_TIMESTAMP),
('station_bug_enabled', 'false', 'Enable or disable the station bug overlay.', CURRENT_TIMESTAMP),
('station_bug_image', '/path/to/bug.png', 'Path to the station bug image.', CURRENT_TIMESTAMP);

-- Seed Station Bug Script
INSERT INTO scripts (name, description, script_type, script_content, parameters_schema, is_builtin, created_at, updated_at)
VALUES 
('Station Bug', 'Displays the station bug based on global settings.', 'overlay', 
'// Station Bug Script
// Reads "station_bug_enabled" and "station_bug_image" from global settings.

fn main() {
    print("Running Station Bug Script");
    
    let enabled = settings["station_bug_enabled"];
    if enabled != "true" {
        print("Station bug disabled");
        return;
    }

    let image = settings["station_bug_image"];
    let name = settings["station_name"];
    
    // Example: send MPV command to show text (simulating bug)
    // In real usage, you might use overlay-add or similar
    let msg = "Station: " + name;
    mpv_send("show-text \"" + msg + "\" 5000");
    
    print("Displayed station bug: " + msg);
}', 
'{}', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
