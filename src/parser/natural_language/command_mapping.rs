//! Command mapping module for natural language processing
//!
//! This module provides utilities for mapping natural language verbs and phrases
//! to their corresponding Ducktape commands.

use log::debug;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Maps common natural language verbs to their corresponding command actions
pub static COMMAND_VERB_MAPPING: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    // Calendar related commands
    map.insert("schedule", "calendar create");
    map.insert("create", "calendar create");
    map.insert("add", "calendar create");
    map.insert("new", "calendar create");
    map.insert("setup", "calendar create");
    map.insert("organize", "calendar create");

    // Meeting/event specific mappings
    map.insert("meeting", "calendar create");
    map.insert("appointment", "calendar create");
    map.insert("event", "calendar create");

    // Reminder related commands
    map.insert("remind", "reminder create");
    map.insert("remember", "reminder create");
    map.insert("reminder", "reminder create");

    // Other command mappings can be added here
    map
});

/// Normalizes an input command by mapping natural language verbs to proper Ducktape commands
///
/// This function transforms commands like "create a meeting" into "calendar create" commands
/// by looking up the first verb in the COMMAND_VERB_MAPPING.
///
/// # Arguments
///
/// * `input` - The input command string to normalize
///
/// # Returns
///
/// The normalized command string with the appropriate Ducktape command
///
/// # Examples
///
/// ```
/// let input = "create a zoom meeting at 9am";
/// let normalized = normalize_command(input);
/// assert_eq!(normalized, "calendar create a zoom meeting at 9am");
/// ```
pub fn normalize_command(input: &str) -> String {
    // First check if the input already starts with any known command
    // to avoid double prefixing
    if input.starts_with("calendar create")
        || input.starts_with("calendar")
        || input.starts_with("reminder")
        || input.starts_with("note")
        || input.starts_with("todo")
    {
        return input.to_string();
    }

    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return input.to_string();
    }

    debug!("Normalizing command: {}", input);

    // Check if the first word is a known verb
    if let Some(command) = COMMAND_VERB_MAPPING.get(words[0]) {
        let result = format!("{} {}", command, words[1..].join(" "));
        debug!("Normalized command verb '{}' to '{}'", words[0], result);
        return result;
    }

    // Check for context clues in the input
    let input_lower = input.to_lowercase();

    // Check for meeting or event related keywords
    if input_lower.contains("meeting")
        || input_lower.contains("event")
        || input_lower.contains("appointment")
        || input_lower.contains("zoom")
    {
        // If it has meeting keywords but no recognized command, default to calendar create
        debug!("Input contains meeting-related keywords, defaulting to calendar create");
        return format!("calendar create {}", input);
    }

    // Check for reminder related keywords
    if input_lower.contains("remind me")
        || input_lower.contains("reminder")
        || input_lower.contains("todo")
        || (input_lower.contains("remind") && !input_lower.starts_with("remind"))
    {
        // If it has reminder keywords but no recognized command, default to reminder create
        debug!("Input contains reminder-related keywords, defaulting to reminder create");
        return format!("reminder create {}", input);
    }

    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_command_verb() {
        // Test basic verb mapping
        assert_eq!(
            normalize_command("create a meeting at 3pm"),
            "calendar create a meeting at 3pm"
        );
        assert_eq!(
            normalize_command("schedule zoom call at 9am"),
            "calendar create zoom call at 9am"
        );
        assert_eq!(
            normalize_command("remind me to call Sarah"),
            "reminder create me to call Sarah"
        );
    }

    #[test]
    fn test_normalize_with_meeting_keywords() {
        // Test mapping based on keywords
        assert_eq!(
            normalize_command("zoom meeting with Team at 2pm"),
            "calendar create zoom meeting with Team at 2pm"
        );
    }

    #[test]
    fn test_normalize_with_reminder_keywords() {
        // Test mapping based on reminder keywords
        assert_eq!(
            normalize_command("remind me to buy milk"),
            "reminder create remind me to buy milk"
        );
        assert_eq!(
            normalize_command("please remind me to call John tomorrow"),
            "reminder create please remind me to call John tomorrow"
        );
        assert_eq!(
            normalize_command("set a reminder to finish the report"),
            "reminder create set a reminder to finish the report"
        );
    }

    #[test]
    fn test_normalize_unchanged() {
        // Test input that should remain unchanged
        assert_eq!(
            normalize_command("calendar create \"Meeting\" today 14:00 15:00"),
            "calendar create \"Meeting\" today 14:00 15:00"
        );

        assert_eq!(
            normalize_command("reminder create \"Call mom\" --remind tomorrow"),
            "reminder create \"Call mom\" --remind tomorrow"
        );
    }
}
