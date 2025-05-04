//! Utility functions for Grok parser implementation
//!
//! This module provides helper functions for the Grok parser,
//! including command enhancement and sanitization.

use anyhow::Result;
use log::debug;
use regex::Regex;

/// Clean up NLP-generated commands by removing unnecessary quotes and normalizing spacing
///
/// This function handles the conversion of natural language commands into structured `ducktape`
/// commands. It has special handling for time expressions in event creation commands:
///
/// # Time Parsing
///
/// When creating calendar events with time specifications like "tonight at 7pm", this function
/// extracts and converts these expressions to proper 24-hour format times. For example:
/// - "tonight at 7pm" → 19:00
/// - "today at 3:30pm" → 15:30
/// - "tomorrow at 9am" → 09:00
///
/// # Examples
///
/// ```
/// let input = "create an event called Meeting tonight at 7pm";
/// let result = sanitize_nlp_command(input);
/// // result will be: "ducktape calendar create "Meeting" 2025-04-26 19:00 20:00 "Calendar""
/// ```
pub fn sanitize_nlp_command(command: &str) -> String {
    // Ensure the command starts with ducktape
    if !command.starts_with("ducktape") {
        // Check for event creation patterns
        let is_event_creation = command.contains("create an event")
            || command.contains("schedule a")
            || command.contains("create event")
            || command.contains("schedule event")
            || command.contains("create a meeting")
            || command.contains("schedule meeting")
            || command.contains("create a zoom meeting") // Added this pattern for Zoom meetings
            || command.contains("schedule a zoom meeting"); // Added this pattern for Zoom meetings

        if is_event_creation {
            debug!("Converting event creation command to calendar command: {}", command);

            // Extract event title if possible (keep existing logic)
            let mut title = "Event";
            if command.contains(" called ") {
                let parts: Vec<&str> = command.split(" called ").collect();
                if parts.len() > 1 {
                    let title_part = parts[1];
                    let end_markers = [" at ", " on ", " for ", " with ", " and "];
                    let mut end_pos = title_part.len();
                    for marker in &end_markers {
                        if let Some(pos) = title_part.find(marker) {
                            if pos < end_pos {
                                end_pos = pos;
                            }
                        }
                    }
                    title = &title_part[..end_pos];
                }
            }

            // Get default calendar from config
            let default_calendar = match crate::config::Config::load() {
                Ok(config) => {
                    config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string())
                }
                Err(_) => "Calendar".to_string(), // Fallback to "Calendar" if config can't be loaded
            };

            debug!("Using default calendar from config: {}", default_calendar);

            // Build initial command with the default calendar from config
            let initial_command = format!(
                "ducktape calendar create \"{}\" today 00:00 01:00 \"{}\"",
                title, default_calendar
            );

            // Always delegate to extract_time_from_title for robust time parsing
            return crate::parser::natural_language::grok::time_extractor::extract_time_from_title(
                &initial_command,
                command,
            );
        }
        // For other commands, just prefix with ducktape
        return format!("ducktape {}", command);
    }
    // Basic sanitization to fix common issues with NLP-generated commands
    command
        .replace("\u{a0}", " ") // Replace non-breaking spaces
        .replace("\"\"", "\"") // Replace double quotes
        .to_string()
}

/// Enhance command with recurrence information
pub fn enhance_recurrence_command(command: &str) -> String {
    // If not a calendar command, return unchanged
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();

    // Handle "every day/week/month/year" and variants
    if command.contains(" every day") || command.contains(" daily") {
        if !enhanced.contains("--repeat") {
            enhanced = enhanced.trim().to_string() + " --repeat daily";
        }
    } else if command.contains(" every week") || command.contains(" weekly") {
        if !enhanced.contains("--repeat") {
            enhanced = enhanced.trim().to_string() + " --repeat weekly";
        }
    } else if command.contains(" every month") || command.contains(" monthly") {
        if !enhanced.contains("--repeat") {
            enhanced = enhanced.trim().to_string() + " --repeat monthly";
        }
    } else if command.contains(" every year")
        || command.contains(" yearly")
        || command.contains(" annually")
    {
        if !enhanced.contains("--repeat") {
            enhanced = enhanced.trim().to_string() + " --repeat yearly";
        }
    }

    // Handle "every X days/weeks/months/years" with regex
    let re_interval = Regex::new(r"every (\d+) (day|week|month|year)s?").unwrap();
    if let Some(caps) = re_interval.captures(command) {
        let interval = caps.get(1).map_or("", |m| m.as_str());
        let unit = caps.get(2).map_or("", |m| m.as_str());

        if !interval.is_empty() && !unit.is_empty() {
            // Add frequency if not already present
            if !enhanced.contains("--repeat") {
                enhanced = match unit {
                    "day" => enhanced.trim().to_string() + " --repeat daily",
                    "week" => enhanced.trim().to_string() + " --repeat weekly",
                    "month" => enhanced.trim().to_string() + " --repeat monthly",
                    "year" => enhanced.trim().to_string() + " --repeat yearly",
                    _ => enhanced,
                };
            }

            // Add interval if not already present
            if !enhanced.contains("--interval") {
                enhanced = enhanced.trim().to_string() + &format!(" --interval {}", interval);
            }
        }
    }

    enhanced
}

/// Add Zoom meeting flag when zoom-related keywords are detected
pub fn enhance_command_with_zoom(command: &str, input: &str) -> String {
    // If not a calendar command or already has zoom flag, return unchanged
    if !command.contains("calendar create") || command.contains("--zoom") {
        return command.to_string();
    }

    let input_lower = input.to_lowercase();
    let zoom_keywords = [
        "zoom",
        "video call",
        "video meeting",
        "virtual meeting",
        "online meeting",
        "teams meeting",
        "google meet",
    ];

    if zoom_keywords.iter().any(|&keyword| input_lower.contains(keyword)) {
        let enhanced = command.trim().to_string() + " --zoom";
        debug!("Added zoom flag based on input keywords: {}", enhanced);
        return enhanced;
    }

    command.to_string()
}

/// Enhance command with proper contact and email handling
pub fn enhance_command_with_contacts(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();

    // Step 1: Extract email addresses from the input
    let email_addresses = extract_email_addresses(input);

    // Step 2: Extract contact names using the shared utility function
    let contact_names = crate::parser::natural_language::utils::extract_contact_names(input);

    debug!("Email addresses extracted: {:?}", email_addresses);
    debug!("Contact names extracted: {:?}", contact_names);

    // Step 3: Handle email addresses if they're not already in the command
    if !email_addresses.is_empty() && !enhanced.contains("--email") {
        let escaped_emails = email_addresses.join(",").replace("\"", "\\\"");
        debug!("Adding emails to command: {}", escaped_emails);
        enhanced = format!(r#"{} --email "{}""#, enhanced, escaped_emails);
    }

    // Step 4: Clean up any incorrectly placed contact names in email flags
    if enhanced.contains("--email") {
        // Pattern: --email "Name Without @ Symbol"
        let email_regex = regex::Regex::new(r#"--email\s+"([^@"]+)""#).unwrap();

        if let Some(caps) = email_regex.captures(&enhanced) {
            if let Some(email_match) = caps.get(1) {
                let email_value = email_match.as_str();
                if !email_value.contains('@') {
                    debug!("Removing incorrectly formatted email: {}", email_value);
                    enhanced = email_regex.replace(&enhanced, "").to_string().trim().to_string();
                }
            }
        }

        // Remove specific contact names from email flags
        for name in &contact_names {
            let quoted_name = format!("--email \"{}\"", name);
            if enhanced.contains(&quoted_name) {
                debug!("Removing name '{}' from email flag", name);
                enhanced = enhanced.replace(&quoted_name, "").trim().to_string();
            }
        }
    }

    // Step 5: Add contact names if not already in the command
    if !contact_names.is_empty() && !enhanced.contains("--contacts") {
        let escaped_contacts = contact_names.join(",").replace("\"", "\\\"");
        debug!("Adding contacts to command: {}", escaped_contacts);
        enhanced = format!(r#"{} --contacts "{}""#, enhanced, escaped_contacts);
    }

    enhanced
}

/// Extract email addresses from natural language input
fn extract_email_addresses(input: &str) -> Vec<String> {
    // Email regex pattern
    let email_regex = Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();

    let mut emails = Vec::new();

    for cap in email_regex.captures_iter(input) {
        let email = cap.get(0).unwrap().as_str().to_string();
        if crate::calendar::validate_email(&email) {
            debug!("Extracted email: {}", email);
            emails.push(email);
        } else {
            debug!("Found invalid email: {}", email);
        }
    }

    emails
}

/// Fix calendar end time formatting to ensure it's just a time (HH:MM) not a date-time
pub fn fix_calendar_end_time_format(command: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    debug!("Checking calendar command for end time format: {}", command);

    // Regex to match the calendar create command format with potential date in end time
    let re = Regex::new(r#"calendar create\s+"([^"]+)"\s+(\d{4}-\d{2}-\d{2})\s+(\d{1,2}:\d{2})\s+(\d{4}-\d{2}-\d{2}\s+)?(\d{1,2}:\d{2})"#).unwrap();

    if let Some(caps) = re.captures(command) {
        // If we have a match, construct the corrected command with proper end time format
        let title = caps.get(1).map_or("", |m| m.as_str());
        let date = caps.get(2).map_or("", |m| m.as_str());
        let start_time = caps.get(3).map_or("", |m| m.as_str());
        let end_time = caps.get(5).map_or("", |m| m.as_str());

        // Check if there was a date part before the end time that needs to be removed
        if caps.get(4).is_some() {
            debug!("Found end time with date, removing date part");

            // Extract the part after the end time (flags, etc.)
            let after_end_time = if let Some(end_pos) = command.find(end_time) {
                &command[end_pos + end_time.len()..]
            } else {
                ""
            };

            let fixed_command = format!(
                r#"ducktape calendar create "{}" {} {} {} {}"#,
                title,
                date,
                start_time,
                end_time,
                after_end_time.trim()
            );

            debug!("Fixed command: {}", fixed_command);
            return fixed_command;
        }
    }

    command.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_nlp_command() {
        // Test handling of non-breaking spaces
        let input = "ducktape\u{a0}calendar create \"Meeting\"";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape calendar create \"Meeting\"");

        // Test handling of double quotes
        let input = "ducktape calendar create \"\"Meeting\"\"";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape calendar create \"Meeting\"");

        // Test natural language event creation command
        let input = "create an event called test tonight at 10pm";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.starts_with("ducktape calendar create"));
        // Title extraction might vary based on implementation details
        assert!(sanitized.contains("test") || sanitized.contains("Event"));
        assert!(sanitized.contains("22:00")); // 10pm should be converted to 22:00

        // Test another event creation pattern
        let input = "schedule a meeting with Joe tomorrow at 9am";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.starts_with("ducktape calendar create"));
        assert!(sanitized.contains("09:00")); // 9am should be converted to 09:00

        // Test zoom meeting creation pattern
        let input = "create a zoom meeting tomorrow at 8am called Important Review";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.starts_with("ducktape calendar create"));
        assert!(sanitized.contains("Important Review"));
        assert!(sanitized.contains("08:00")); // 8am should be converted to 08:00

        // Test another zoom meeting pattern
        let input = "schedule a zoom meeting with the team at 3pm";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.starts_with("ducktape calendar create"));
        assert!(sanitized.contains("15:00")); // 3pm should be converted to 15:00

        // Test specific case that caused issues: tonight at 7pm
        let input = "create an event called test tonight at 7pm";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.contains("19:00")); // 7pm should be converted to 19:00
        assert!(sanitized.contains("20:00")); // End time should be 8pm/20:00

        // Test afternoon time
        let input = "create an event called Meeting today at 3:30pm";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.contains("15:30")); // 3:30pm should be converted to 15:30

        // Test non-calendar command
        let input = "not a ducktape command";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape not a ducktape command");
    }

    #[test]
    fn test_enhance_recurrence_command() {
        // Test adding recurrence
        let input =
            "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\" every week";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--repeat weekly"));

        // Test adding interval
        let input = "ducktape calendar create \"Bi-weekly Meeting\" 2024-03-15 10:00 11:00 \"Work\" every 2 weeks";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--interval 2"));

        // Test non-calendar command (should remain unchanged)
        let input = "ducktape todo \"Buy groceries\"";
        let enhanced = enhance_recurrence_command(input);
        assert_eq!(input, enhanced);
    }

    #[test]
    fn test_enhance_command_with_zoom() {
        // Test adding zoom flag for zoom keyword
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a zoom meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(enhanced.contains("--zoom"));

        // Test not adding zoom flag for non-zoom input
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a regular meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(!enhanced.contains("--zoom"));
    }

    #[test]
    fn test_enhance_command_with_contacts() {
        // Test adding contacts with "with" pattern
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a meeting with Joe Smith";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert!(enhanced.contains("--contacts \"Joe Smith\""));

        // Test adding contacts with "invite" pattern
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a meeting and invite Jane Doe";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert!(enhanced.contains("--contacts \"Jane Doe\""));

        // Test adding contacts with "and invite" pattern (new pattern we fixed)
        let cmd = "ducktape calendar create \"TestEvent\" today 00:00 01:00 \"Calendar\"";
        let input = "create an event called TestEvent tonight at 10pm and invite Shaun Stuart";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert!(enhanced.contains("--contacts \"Shaun Stuart\""));
    }

    #[test]
    fn test_fix_calendar_end_time_format() {
        // Test fixing end time with date
        let command =
            "ducktape calendar create \"Team Meeting\" 2025-04-22 23:00 2025-04-22 00:00 \"Work\"";
        let fixed = fix_calendar_end_time_format(command);
        assert_eq!(
            fixed,
            "ducktape calendar create \"Team Meeting\" 2025-04-22 23:00 00:00 \"Work\""
        );

        // Test command that's already correct
        let command = "ducktape calendar create \"Team Meeting\" 2025-04-22 23:00 00:00 \"Work\"";
        let fixed = fix_calendar_end_time_format(command);
        assert_eq!(fixed, command);
    }
}
