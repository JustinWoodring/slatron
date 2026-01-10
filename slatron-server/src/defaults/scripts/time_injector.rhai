
// Script Type: server_context
// This script runs before a DJ Generation to inject variables into the prompt context.

// Get current system time in a readable format
let now = date(); 

// We can format it manually or rely on the default string representation.
// Rhai 'date()' returns an Instant/Date object. 
// Let's assume we want a simple string for the LLM.

let time_str = now.to_string(); 

// Return a map of variables to inject
return #{
    current_time: time_str,
    time_of_day: get_time_of_day(now)
};

fn get_time_of_day(d) {
    // This is a rough approximation depending on available Date methods in the Rhai engine
    // If specific hour extraction isn't available, we might just return "unknown"
    // But typically date objects stringify to something containing the time.
    
    // For now, let's just return the full string as the LLM can parse it.
    return "See current_time";
}
