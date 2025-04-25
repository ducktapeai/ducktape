//! Time extraction module for natural language parsing
//!
//! This module provides functionality to extract time expressions from event titles
//! in natural language processing.

use chrono::Local;
use log::debug;
use regex::Regex;

/// Extract time expressions from event titles and update command accordingly
///
/// This function looks for time patterns like "tonight at 7pm" in event titles that were
/// incorrectly parsed by the language model, extracts the time information, and fixes
/// the command to use the correct time.
///
/// # Arguments
///
/// * `command` - The calendar command to process
/// * `_input` - The original user input (unused but kept for API consistency)
///
/// # Returns
///
/// A string containing the corrected command with proper time values
pub fn extract_time_from_title(command: &str, _input: &str) -> String {
    // Skip if not a calendar command
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let cmd_parts: Vec<&str> = command.split_whitespace().collect();
    if cmd_parts.len() < 4 {
        return command.to_string();
    }

    // Extract the title from the command using regex
    let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+"#).unwrap();
    let caps = match re.captures(command) {
        Some(c) => c,
        None => return command.to_string(),
    };

    let title = caps.get(1).map_or("", |m| m.as_str());

    // Look for time expressions in the title
    let time_patterns = [
        (r"tonight at (\d{1,2})(:\d{2})?\s*(am|pm)", "today"),
        (r"today at (\d{1,2})(:\d{2})?\s*(am|pm)", "today"),
        (r"tomorrow at (\d{1,2})(:\d{2})?\s*(am|pm)", "tomorrow"),
        (r"this evening at (\d{1,2})(:\d{2})?\s*(am|pm)", "today"),
    ];

    for (pattern, date_text) in &time_patterns {
        let time_re = Regex::new(pattern).unwrap();
        if let Some(time_caps) = time_re.captures(title) {
            // Extract hour and am/pm
            let hour_str = time_caps.get(1).map_or("", |m| m.as_str());
            let minute_str = time_caps.get(2).map_or(":00", |m| m.as_str());
            let am_pm = time_caps.get(3).map_or("", |m| m.as_str()).to_lowercase();

            // Calculate 24-hour format
            let hour: u32 = hour_str.parse().unwrap_or(0);
            let hour_24 = match (hour, am_pm.as_str()) {
                (12, "am") => 0,
                (h, "am") => h,
                (12, "pm") => 12,
                (h, "pm") => h + 12,
                _ => hour,
            };

            // Format the start time
            let start_time = format!("{:02}{}", hour_24, minute_str);

            // Calculate end time (1 hour later by default)
            let end_time = format!("{:02}{}", (hour_24 + 1) % 24, minute_str);

            // Create a new fixed command
            let date_today = if *date_text == "today" {
                Local::now().format("%Y-%m-%d").to_string()
            } else {
                (Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string()
            };

            // Clean up title by removing the time expression
            let clean_title =
                time_re.replace(title, "").trim().trim_end_matches(" and").trim().to_string();

            // Check if there's any title left after cleaning
            let final_title = if clean_title.is_empty() {
                "Event".to_string() // Default title
            } else {
                clean_title
            };

            debug!("Extracted time expression from title: '{}'", title);
            debug!("Fixed time: {}:{}, date: {}", hour_24, minute_str, date_today);

            // Extract everything after the date, time, and calendar name
            let cmd_suffix = if let Some(pos) = command.find(" \"Calendar\"") {
                let pos_after_calendar = pos + " \"Calendar\"".len();
                if pos_after_calendar < command.len() { &command[pos_after_calendar..] } else { "" }
            } else {
                ""
            };

            // Construct the new command
            let fixed_command = format!(
                r#"ducktape calendar create "{}" {} {} {} "Calendar"{}"#,
                final_title, date_today, start_time, end_time, cmd_suffix
            );

            debug!("Fixed command with time from title: {}", fixed_command);
            return fixed_command;
        }
    }

    // No time expression found in title
    command.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_time_from_title() {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let tomorrow = (Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

        // Test "tonight at 7pm" in title
        let command =
            r#"ducktape calendar create "test tonight at 7pm" today 00:00 01:00 "Calendar""#;
        let fixed = extract_time_from_title(command, "");
        assert!(fixed.contains("test"));
        assert!(fixed.contains(&today));
        assert!(fixed.contains("19:00"));
        assert!(fixed.contains("20:00"));

        // Test "tomorrow at 9am" in title
        let command =
            r#"ducktape calendar create "Meeting tomorrow at 9am" today 00:00 01:00 "Calendar""#;
        let fixed = extract_time_from_title(command, "");
        assert!(fixed.contains("Meeting"));
        assert!(fixed.contains(&tomorrow));
        assert!(fixed.contains("09:00"));
        assert!(fixed.contains("10:00"));

        // Test "today at 3:30pm" in title (with minutes)
        let command =
            r#"ducktape calendar create "Call today at 3:30pm" today 00:00 01:00 "Calendar""#;
        let fixed = extract_time_from_title(command, "");
        assert!(fixed.contains("Call"));
        assert!(fixed.contains(&today));
        assert!(fixed.contains("15:30"));
        assert!(fixed.contains("16:30"));
    }
}
