// Auto Station Bumpers Global Script
// Automatically injects station idents and transitions based on time and content

fn on_load() {
    // Play station ident at the top of every hour
    if is_top_of_hour() {
        inject_bumper("Station Ident");
    }
}

fn on_unload() {
    // Play transition between content during daytime hours
    let hour = get_current_hour();

    // Only play transitions between 6 AM and 10 PM
    if hour >= 6 && hour < 22 {
        inject_bumper("Swoosh Transition");
    }
}
