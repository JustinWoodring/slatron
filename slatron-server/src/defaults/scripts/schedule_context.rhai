// Script Type: server_context
// Injects Schedule Info (Current Block stats, upcoming items)

let context_str = "";

if schedule != () {
    // Current Block Info
    let block = schedule.block;
    if block != () {
        context_str += `Current Segment: ${block.name}\n`;
        
        // Calculate dynamic messaging based on time remaining??
        // For now just raw info.
        if schedule.time_remaining_minutes != () {
            context_str += `Time Remaining in Segment: ${schedule.time_remaining_minutes} minutes.\n`;
        }
    }

    // Upcoming Content
    if schedule.upcoming != () && schedule.upcoming.len() > 0 {
        context_str += "Coming Up Next:\n";
        // Show next 3 items max
        let limit = 3;
        let count = 0;
        for item in schedule.upcoming {
            if count >= limit { break; }
            context_str += `- ${item.title} (${item.content_type})\n`;
            count += 1;
        }
    }
} else {
    context_str = "No Schedule Information Available.";
}

context += context_str;
