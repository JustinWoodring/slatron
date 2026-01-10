
// Script Type: server_context
// This script runs before a DJ Generation to inject variables into the prompt context.

// 1. Get current system time formatted
let time_str = get_date_time("%I:%M %p"); // e.g. "04:30 PM"
let full_date = get_date_time("%A, %B %d, %Y"); // e.g. "Monday, January 01, 2024"

// Return a map of variables to inject
context += `The current time is ${time_str} on ${full_date}.`;
