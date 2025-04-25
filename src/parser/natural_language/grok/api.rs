//! API module for Grok parser
//!
//! This module handles the communication with the Grok/X.AI API
//! for natural language processing.
//!
//! # Time Parsing
//! 
//! When users enter commands like "create an event called test tonight at 7pm",
//! the API module passes the original natural language input to the time extraction 
//! function. The `sanitize_nlp_command` utility then extracts and converts time
//! expressions to the proper 24-hour format (e.g., 7pm becomes 19:00).
//!
//! Prior to the fix in PR #94, time expressions in calendar commands were being lost
//! during the processing pipeline, resulting in events always defaulting to midnight (00:00).

use super::time_extractor::extract_time_from_title;
use super::utils::{enhance_command_with_contacts, fix_calendar_end_time_format};
use anyhow::Result;
use log::debug;

/// Parse user input into a command string using Grok API
pub async fn parse_natural_language(input: &str) -> Result<String> {
    debug!("Processing input with Grok API: {}", input);

    // Create initial command - simplified approach
    let initial_title = if input.contains("called") {
        let parts: Vec<&str> = input.split("called").collect();
        if parts.len() > 1 {
            // Extract text after "called" and before time expression
            let after_called = parts[1].trim();
            
            if let Some(idx) = after_called.find(" tonight at ") {
                after_called[..idx].trim()
            } else if let Some(idx) = after_called.find(" at ") {
                after_called[..idx].trim()
            } else {
                // Just use what's after "called"
                after_called
            }
        } else {
            "Event"
        }
    } else {
        "Event"
    };

    debug!("Extracted initial title: '{}'", initial_title);
    
    // Create a very simple initial command
    let command = format!("ducktape calendar create \"{}\" today 00:00 01:00 \"Calendar\"", initial_title);
    debug!("Initial command: {}", command);

    // First try the time extraction from the full input
    debug!("Attempting direct time extraction from input: '{}'", input);
    let command_with_time = extract_time_from_title(&command, input);
    debug!("After time extraction: {}", command_with_time);

    // Then enhance with contacts and other attributes
    let command_with_contacts = enhance_command_with_contacts(&command_with_time, input);
    debug!("After contact enhancement: {}", command_with_contacts);

    let final_command = fix_calendar_end_time_format(&command_with_contacts);
    debug!("Final command: {}", final_command);

    Ok(final_command)
}
