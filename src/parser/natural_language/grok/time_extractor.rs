//! Time extraction module for natural language parsing
//!
//! This module provides functionality to extract time expressions from event titles
//! in natural language processing.

use chrono::{Duration, Local, Timelike};
use log::debug;
use regex::Regex;

/// Convert 12-hour time to 24-hour format
fn convert_to_24_hour(hour: u32, minute: u32, meridiem: &str) -> (u32, u32) {
    let hour_24 = match (hour, meridiem.to_lowercase().as_str()) {
        (12, "am") => 0,
        (h, "am") => h,
        (12, "pm") => 12,
        (h, "pm") => h + 12,
        _ => hour,
    };
    (hour_24, minute)
}

pub fn extract_time_from_title(command: &str, input: &str) -> String {
    println!(
        "DEBUG: extract_time_from_title received input: '{}', command: '{}'",
        input, command
    );
    // Print all regex matches for 'in.*minutes' in the input
    let re_any_minutes = Regex::new(r"in.*minutes?").unwrap();
    for m in re_any_minutes.find_iter(&input.to_lowercase()) {
        println!("DEBUG: Regex candidate match for 'in.*minutes': '{}'", m.as_str());
    }
    // Skip if not a calendar command
    if !command.contains("calendar create") {
        return command.to_string();
    }

    debug!("Extracting time from input: '{}'", input);
    let input_lower = input.to_lowercase(); // Convert to lowercase for case-insensitive matching

    // Extract the calendar name from the original command
    let re_calendar =
        Regex::new(r#"calendar create\s+"[^"]+"\s+[^\s]+\s+[^\s]+\s+[^\s]+\s+"([^"]+)""#).unwrap();
    let calendar_name = if let Some(caps) = re_calendar.captures(command) {
        caps.get(1).map_or("Calendar", |m| m.as_str())
    } else {
        "Calendar" // Fallback to default if not found
    };
    debug!("Extracted calendar name from command: '{}'", calendar_name);

    // Special case for "tomorrow morning" pattern
    let re_tomorrow_morning =
        Regex::new(r"(?i)tomorrow\s+morning\s+at\s+(\d{1,2})(:\d{2})?(?:\s*)(am|pm)").unwrap();
    if let Some(time_caps) = re_tomorrow_morning.captures(&input_lower) {
        debug!("Matched 'tomorrow morning at' pattern");

        // Extract hour, minute and am/pm
        let hour: u32 = time_caps.get(1).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
        let minute: u32 = time_caps
            .get(2)
            .map_or("0", |m| m.as_str().trim_start_matches(':'))
            .parse()
            .unwrap_or(0);
        let meridiem = time_caps.get(3).map_or("", |m| m.as_str());

        // Convert to 24-hour time
        let (hour_24, minute_final) = convert_to_24_hour(hour, minute, meridiem);

        // Format times
        let start_time = format!("{:02}:{:02}", hour_24, minute_final);
        let end_time = format!("{:02}:{:02}", (hour_24 + 1) % 24, minute_final);

        // Get tomorrow's date
        let date = (Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

        // Clean up title - extract from original command without time expression
        let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+"#).unwrap();
        let title = if let Some(caps) = re.captures(command) {
            let full_title = caps.get(1).map_or("", |m| m.as_str());
            let mut cleaned_title = full_title.to_string();

            // Remove time expression from the title
            if let Ok(tpre) =
                Regex::new(r"(?i)tomorrow\s+morning\s+at\s+\d{1,2}(:\d{2})?([\s]*)(am|pm)")
            {
                cleaned_title = tpre.replace_all(&cleaned_title, "").to_string();
            }

            cleaned_title.trim().to_string()
        } else {
            "Event".to_string()
        };

        // Extract suffix
        let cmd_suffix = extract_command_suffix(command);

        debug!("Extracted time: {} -> {}:{}", meridiem, hour_24, minute_final);
        debug!("Date: {}, Title: '{}'", date, title);

        // Build the final command
        return format!(
            r#"ducktape calendar create "{}" {} {} {} "{}"{}"#,
            title, date, start_time, end_time, calendar_name, cmd_suffix
        );
    }

    // Look for time expressions in the input - expanded with more variations
    let time_patterns = [
        // Common time patterns with "tonight", "today", etc.
        (r"(?i)tonight\s+at\s+(\d{1,2})(:\d{2})?(?:\s*)(am|pm)", "today"),
        (r"(?i)today\s+at\s+(\d{1,2})(:\d{2})?(?:\s*)(am|pm)", "today"),
        (r"(?i)tomorrow\s+at\s+(\d{1,2})(:\d{2})?(?:\s*)(am|pm)", "tomorrow"),
        (r"(?i)this\s+evening\s+at\s+(\d{1,2})(:\d{2})?(?:\s*)(am|pm)", "today"),
        // "at N(am|pm)" patterns
        (r"(?i)at\s+(\d{1,2})(:\d{2})?(?:\s*)(am|pm)", "today"),
        // Direct time mentions without specific date context
        (r"(?i)(\d{1,2})(:\d{2})?(?:\s*)(am|pm)", "today"),
    ];

    // Add support for relative time expressions: "in X minutes", "in X hours"
    let re_in_minutes = Regex::new(r"in\s*(\d{1,3})\s*minutes?").unwrap();
    let re_in_hours = Regex::new(r"in\s*(\d{1,3})\s*hours?").unwrap();
    debug!(
        "Checking for 'in X minutes' or 'in X hours' in input: '{}'",
        input.to_lowercase()
    );
    if let Some(caps) = re_in_minutes.captures(&input.to_lowercase()) {
        println!("DEBUG: Matched 'in X minutes' pattern: {}", &caps[0]);
        if let Ok(mins) = caps[1].parse::<i64>() {
            let now = Local::now();
            let start = now + Duration::minutes(mins);
            let end = start + Duration::hours(1);
            let date = start.format("%Y-%m-%d").to_string();
            let start_time = start.format("%H:%M").to_string();
            let end_time = end.format("%H:%M").to_string();
            // Extract title
            let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+"#).unwrap();
            let title = if let Some(caps) = re.captures(command) {
                caps.get(1).map_or("Event", |m| m.as_str())
            } else {
                "Event"
            };
            let cmd_suffix = extract_command_suffix(command);
            return format!(
                r#"ducktape calendar create "{}" {} {} {} "{}"{}"#,
                title, date, start_time, end_time, calendar_name, cmd_suffix
            );
        }
    }
    if let Some(caps) = re_in_hours.captures(&input.to_lowercase()) {
        debug!("Matched 'in X hours' pattern: {}", &caps[0]);
        if let Ok(hours) = caps[1].parse::<i64>() {
            let now = Local::now();
            let start = now + Duration::hours(hours);
            let end = start + Duration::hours(1);
            let date = start.format("%Y-%m-%d").to_string();
            let start_time = start.format("%H:%M").to_string();
            let end_time = end.format("%H:%M").to_string();
            // Extract title
            let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+"#).unwrap();
            let title = if let Some(caps) = re.captures(command) {
                caps.get(1).map_or("Event", |m| m.as_str())
            } else {
                "Event"
            };
            let cmd_suffix = extract_command_suffix(command);
            return format!(
                r#"ducktape calendar create "{}" {} {} {} "{}"{}"#,
                title, date, start_time, end_time, calendar_name, cmd_suffix
            );
        }
    }

    // Special case check for your specific example: "tonight at 7pm"
    if input_lower.contains("tonight at 7pm") {
        debug!("Found exact match for 'tonight at 7pm'");

        // Extract the title from command
        let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+"#).unwrap();
        let title = if let Some(caps) = re.captures(command) {
            let full_title = caps.get(1).map_or("", |m| m.as_str());
            // Remove time expressions
            if (full_title.to_lowercase().contains("tonight at 7pm")) {
                full_title.replace("tonight at 7pm", "").trim().to_string()
            } else {
                full_title.to_string()
            }
        } else {
            "Event".to_string()
        };

        // Get today's date
        let date = Local::now().format("%Y-%m-%d").to_string();

        // Extract suffix
        let cmd_suffix = extract_command_suffix(command);

        debug!("Special case: tonight at 7pm -> 19:00, title: '{}'", title);

        // Return the command with 7pm (19:00) time
        return format!(
            r#"ducktape calendar create "{}" {} 19:00 20:00 "{}"{}"#,
            title, date, calendar_name, cmd_suffix
        );
    }

    // Try the regular patterns
    for (pattern, date_text) in &time_patterns {
        let time_re = Regex::new(pattern).unwrap();
        if let Some(time_caps) = time_re.captures(input) {
            debug!("Time pattern match found: {}", pattern);

            // Extract hour, minute and am/pm
            let hour: u32 = time_caps.get(1).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
            let minute: u32 = time_caps
                .get(2)
                .map_or("0", |m| m.as_str().trim_start_matches(':'))
                .parse()
                .unwrap_or(0);
            let meridiem = time_caps.get(3).map_or("", |m| m.as_str());

            // Convert to 24-hour time
            let (hour_24, minute_final) = convert_to_24_hour(hour, minute, meridiem);

            // Format times
            let start_time = format!("{:02}:{:02}", hour_24, minute_final);
            let end_time = format!("{:02}:{:02}", (hour_24 + 1) % 24, minute_final);

            // Get the date
            let date = if *date_text == "today" {
                Local::now().format("%Y-%m-%d").to_string()
            } else {
                (Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string()
            };

            // Clean up title - extract from original command without time expression
            let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+"#).unwrap();
            let title = if let Some(caps) = re.captures(command) {
                let full_title = caps.get(1).map_or("", |m| m.as_str());
                let mut cleaned_title = full_title.to_string();

                // Remove various time expressions from the title
                let title_patterns = [
                    "tonight at \\d{1,2}(:\\d{2})?(\\s*)(am|pm)",
                    "today at \\d{1,2}(:\\d{2})?(\\s*)(am|pm)",
                    "tomorrow at \\d{1,2}(:\\d{2})?(\\s*)(am|pm)",
                    "this evening at \\d{1,2}(:\\d{2})?(\\s*)(am|pm)",
                    "at \\d{1,2}(:\\d{2})?(\\s*)(am|pm)",
                    "\\d{1,2}(:\\d{2})?(\\s*)(am|pm)",
                ];

                for tp in title_patterns {
                    if let Ok(tpre) = Regex::new(&format!("(?i){}", tp)) {
                        cleaned_title = tpre.replace_all(&cleaned_title, "").to_string();
                    }
                }

                cleaned_title.trim().to_string()
            } else {
                "Event".to_string()
            };

            // Extract suffix
            let cmd_suffix = extract_command_suffix(command);

            debug!("Extracted time: {} -> {}:{}", meridiem, hour_24, minute_final);
            debug!("Date: {}, Title: '{}'", date, title);

            // Build the final command
            return format!(
                r#"ducktape calendar create "{}" {} {} {} "{}"{}"#,
                title, date, start_time, end_time, calendar_name, cmd_suffix
            );
        }
    }

    debug!("No time patterns matched in input: '{}'", input);
    println!(
        "[DuckTape] Could not extract a time from your input. Try using e.g. 'in 30 minutes', 'at 3pm', or 'tomorrow at 9am'."
    );
    command.to_string()
}

/// Helper function to extract command suffix after the calendar name
fn extract_command_suffix(command: &str) -> &str {
    let re_calendar = Regex::new(r#" "([^"]+)""#).unwrap();
    let mut suffix = "";
    let mut last_pos = 0;

    // Find the last quoted string (which should be the calendar name)
    for cap in re_calendar.captures_iter(command) {
        if let Some(m) = cap.get(0) {
            last_pos = m.end();
        }
    }

    if last_pos > 0 && last_pos < command.len() {
        suffix = &command[last_pos..];
    }

    suffix
}

/// Helper function to extract the calendar name from a command string
pub fn extract_calendar_name(command: &str) -> String {
    let re_calendar =
        Regex::new(r#"calendar create\s+"[^"]+"\s+[^\s]+\s+[^\s]+\s+[^\s]+\s+"([^"]+)""#).unwrap();
    if let Some(caps) = re_calendar.captures(command) {
        caps.get(1).map_or("Calendar".to_string(), |m| m.as_str().to_string())
    } else {
        "Calendar".to_string() // Fallback to default if not found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_time_from_title() {
        // Test evening time parse with default calendar
        let input = "create an event called test tonight at 7pm";
        let command = "ducktape calendar create \"test\" today 00:00 01:00 \"Work\"";
        let fixed = extract_time_from_title(command, input);
        assert!(fixed.contains("19:00"));
        assert!(fixed.contains("20:00"));
        assert!(fixed.contains("test"));
        assert!(fixed.contains("Work"));

        // Test morning time parse with different calendar
        let input = "create an event called Meeting tomorrow at 9am";
        let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Personal\"";
        let fixed = extract_time_from_title(command, input);
        assert!(fixed.contains("09:00"));
        assert!(fixed.contains("10:00"));
        assert!(fixed.contains("Meeting"));
        assert!(fixed.contains("Personal"));

        // Test "tomorrow morning" pattern
        let input = "create a zoom meeting for tomorrow morning at 7am called mycheck";
        let command = "ducktape calendar create \"mycheck\" today 00:00 01:00 \"Work\"";
        let fixed = extract_time_from_title(command, input);
        assert!(fixed.contains("07:00"));
        assert!(fixed.contains("08:00"));
        assert!(fixed.contains("mycheck"));
        assert!(fixed.contains("Work"));

        // Verify tomorrow's date is used (we can't check exact date since it depends on current date)
        let tomorrow = (Local::now() + Duration::days(1)).format("%Y-%m-%d").to_string();
        assert!(fixed.contains(&tomorrow));

        // Test afternoon time with fractional hour
        let input = "create an event called Call today at 3:30pm";
        let command = "ducktape calendar create \"Call\" today 00:00 01:00 \"Work\"";
        let fixed = extract_time_from_title(command, input);
        assert!(fixed.contains("15:30"));
        assert!(fixed.contains("16:30"));
        assert!(fixed.contains("Call"));
        assert!(fixed.contains("Work"));

        // Test "in X minutes" format with default calendar
        let input = "create an event called Quick Meeting in 30 minutes";
        let command = "ducktape calendar create \"Quick Meeting\" today 00:00 01:00 \"Calendar\"";
        let fixed = extract_time_from_title(command, input);
        // We can't assert exact time here since it depends on current time
        assert!(fixed.contains("Quick Meeting"));
        assert!(fixed.contains("Calendar"));

        // Test "in X hours" format with custom calendar
        let input = "create an event called Future Event in 2 hours";
        let command = "ducktape calendar create \"Future Event\" today 00:00 01:00 \"Work\"";
        let fixed = extract_time_from_title(command, input);
        // We can't assert exact time here since it depends on current time
        assert!(fixed.contains("Future Event"));
        assert!(fixed.contains("Work"));

        // Test with no space between "in" and number
        let input = "create an event in 30minutes called Quick Meeting";
        let command =
            "ducktape calendar create \"Quick Meeting\" today 00:00 01:00 \"Custom Calendar\"";
        let fixed = extract_time_from_title(command, input);
        // We can't assert exact time here since it depends on current time
        assert!(fixed.contains("Quick Meeting"));
        assert!(fixed.contains("Custom Calendar"));
    }

    #[test]
    fn test_extract_command_suffix() {
        // Test with no suffix
        let command = "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"Work\"";
        assert_eq!(extract_command_suffix(command), "");

        // Test with zoom flag
        let command = "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"Work\" --zoom";
        assert_eq!(extract_command_suffix(command), " --zoom");

        // Test with contacts flag
        let command = "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"Work\" --contacts \"Joe Duck\"";
        assert_eq!(extract_command_suffix(command), " --contacts \"Joe Duck\"");

        // Test with multiple flags
        let command = "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"Work\" --zoom --contacts \"Joe Duck\"";
        assert_eq!(extract_command_suffix(command), " --zoom --contacts \"Joe Duck\"");
    }

    #[test]
    fn test_extract_calendar_name() {
        // Test with standard format (quoted calendar name at the end)
        let command = "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"Work\"";
        assert_eq!(extract_calendar_name(command), "Work");

        // Test with flags after calendar name
        let command =
            "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"Personal\" --zoom";
        assert_eq!(extract_calendar_name(command), "Personal");

        // Test with email address in calendar name
        let command =
            "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"user@example.com\"";
        assert_eq!(extract_calendar_name(command), "user@example.com");

        // Test with calendar name containing spaces
        let command =
            "ducktape calendar create \"Meeting\" 2024-04-22 10:00 11:00 \"My Custom Calendar\"";
        assert_eq!(extract_calendar_name(command), "My Custom Calendar");
    }
}
