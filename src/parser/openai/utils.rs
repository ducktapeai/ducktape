//! Utility functions for the OpenAI parser module
//!
//! This module contains shared utility functions used by the OpenAI parser

use crate::calendar::validate_email;
use anyhow::{Result, anyhow};
use log::debug;
use regex::Regex;

/// Helper function to clean up NLP-generated commands
/// Removes unnecessary quotes and normalizes spacing
pub fn sanitize_nlp_command(command: &str) -> String {
    // Clean up the command
    let cleaned = command
        .replace("\u{a0}", " ") // Replace non-breaking spaces
        .replace("\"\"", "\""); // Replace double quotes

    // Ensure the command starts with ducktape
    if !cleaned.starts_with("ducktape") {
        return format!("ducktape {}", cleaned);
    }

    cleaned
}

/// Sanitize user input to prevent injection or other security issues
pub fn sanitize_user_input(input: &str) -> String {
    // Filter out control characters except for newlines and tabs
    input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

/// Validate returned calendar command for security
pub fn validate_calendar_command(command: &str) -> Result<()> {
    // Check for suspicious patterns
    if command.contains("&&")
        || command.contains("|")
        || command.contains(";")
        || command.contains("`")
    {
        return Err(anyhow!("Generated command contains potentially unsafe characters"));
    }

    // Validate interval values are reasonable if present
    if let Some(interval_idx) = command.find("--interval") {
        let interval_part = &command[interval_idx + 10..];
        let re = Regex::new(r"^\s*(\d+)").unwrap();
        if let Some(caps) = re.captures(interval_part) {
            if let Some(interval_match) = caps.get(1) {
                let interval_str = interval_match.as_str();
                if let Ok(interval) = interval_str.parse::<u32>() {
                    if interval > 100 {
                        return Err(anyhow!("Unreasonably large interval value: {}", interval));
                    }
                }
            }
        }
    }

    // Validate count values are reasonable if present
    if let Some(count_idx) = command.find("--count") {
        let count_part = &command[count_idx + 7..];
        let re = Regex::new(r"^\s*(\d+)").unwrap();
        if let Some(caps) = re.captures(count_part) {
            if let Some(count_match) = caps.get(1) {
                let count_str = count_match.as_str();
                if let Ok(count) = count_str.parse::<u32>() {
                    if count > 500 {
                        return Err(anyhow!("Unreasonably large count value: {}", count));
                    }
                }
            }
        }
    }

    Ok(())
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
        let enhanced = format!("{} --zoom", command.trim());
        debug!("Added zoom flag based on input keywords: {}", enhanced);
        return enhanced;
    }

    command.to_string()
}

/// Enhance command with recurrence flags based on natural language
pub fn enhance_command_with_recurrence(command: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();

    // Check for recurring event keywords in the input but missing flags
    let has_recurring_keyword = command.to_lowercase().contains("every day")
        || command.to_lowercase().contains("every week")
        || command.to_lowercase().contains("every month")
        || command.to_lowercase().contains("every year")
        || command.to_lowercase().contains("daily")
        || command.to_lowercase().contains("weekly")
        || command.to_lowercase().contains("monthly")
        || command.to_lowercase().contains("yearly")
        || command.to_lowercase().contains("annually");

    // If recurring keywords found but no --repeat flag, add it
    if has_recurring_keyword && !command.contains("--repeat") && !command.contains("--recurring") {
        if command.contains("every day") || command.contains("daily") {
            enhanced = format!("{} --repeat daily", enhanced);
        } else if command.contains("every week") || command.contains("weekly") {
            enhanced = format!("{} --repeat weekly", enhanced);
        } else if command.contains("every month") || command.contains("monthly") {
            enhanced = format!("{} --repeat monthly", enhanced);
        } else if command.contains("every year")
            || command.contains("yearly")
            || command.contains("annually")
        {
            enhanced = format!("{} --repeat yearly", enhanced);
        }
    }

    // Look for interval patterns like "every 2 weeks" and add --interval
    let re_interval =
        Regex::new(r"every (\d+) (day|days|week|weeks|month|months|year|years)").unwrap();
    if let Some(caps) = re_interval.captures(&command.to_lowercase()) {
        if let Some(interval_str) = caps.get(1) {
            if let Ok(interval) = interval_str.as_str().parse::<u32>() {
                if interval > 0 && interval < 100 && // Reasonable limit
                   !command.contains("--interval")
                {
                    enhanced = format!("{} --interval {}", enhanced, interval);
                }
            }
        }
    }

    enhanced
}

/// Get available calendars from the system
pub async fn get_available_calendars() -> Result<Vec<String>> {
    // Execute AppleScript to get calendars
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(
            r#"tell application "Calendar"
            set calList to {}
            repeat with c in calendars
                copy (name of c) to end of calList
            end repeat
            return calList
        end tell"#,
        )
        .output()?;

    let calendars_str = String::from_utf8_lossy(&output.stdout);
    Ok(calendars_str
        .trim_matches('{')
        .trim_matches('}')
        .split(", ")
        .map(|s| s.trim_matches('"').to_string())
        .collect())
}

/// Helper function to enhance commands with proper contact and email handling
pub fn enhance_command_with_contacts(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();

    // Step 1: Extract email addresses from the input
    let email_addresses = extract_emails(input);

    // Step 2: Extract contact names
    let contact_names = extract_contact_names(input);

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
        let email_regex = Regex::new(r#"--email\s+"([^@"]+)""#).unwrap();

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

/// Helper function to extract contact names from natural language input
pub fn extract_contact_names(input: &str) -> Vec<String> {
    let mut contact_names = Vec::new();
    let input_lower = input.to_lowercase();

    // Check for different contact-related keywords
    let text_to_parse = if input_lower.contains(" with ") {
        debug!("Found 'with' keyword for contact extraction");
        input.split(" with ").nth(1)
    } else if input_lower.contains(" to ") {
        debug!("Found 'to' keyword for contact extraction");
        input.split(" to ").nth(1)
    } else if input_lower.contains("invite ") {
        debug!("Found 'invite' keyword for contact extraction");
        let parts: Vec<&str> = input.splitn(2, "invite ").collect();
        if parts.len() > 1 { Some(parts[1]) } else { None }
    } else {
        None
    };

    if let Some(after_word) = text_to_parse {
        debug!("Text to parse for contacts: '{}'", after_word);

        // Pattern to detect email addresses (simple version)
        let email_pattern = Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();

        for name_part in after_word.split(|c: char| c == ',' || c == ';' || c == '.') {
            let name_part = name_part.trim();
            if name_part.is_empty() {
                continue;
            }

            // Skip if the whole part looks like an email address
            if email_pattern.is_match(name_part) {
                debug!("Skipping email-like string: {}", name_part);
                continue;
            }

            if name_part.contains(" and ") {
                let and_parts: Vec<&str> = name_part.split(" and ").collect();
                for and_part in and_parts {
                    let final_name = refine_name(and_part);
                    if !final_name.is_empty() && !email_pattern.is_match(&final_name) {
                        contact_names.push(final_name);
                    }
                }
            } else {
                let final_name = refine_name(name_part);
                if !final_name.is_empty() && !email_pattern.is_match(&final_name) {
                    contact_names.push(final_name);
                }
            }
        }
    }

    debug!("Extracted contact names: {:?}", contact_names);
    contact_names
}

/// Helper function to refine a name by removing trailing stop words
fn refine_name(name_part: &str) -> String {
    let stop_words = ["at", "on", "tomorrow", "today", "for", "about", "regarding"];
    let mut final_name = name_part.trim().to_string();

    for word in &stop_words {
        if let Some(pos) = final_name.to_lowercase().find(&format!(" {}", word)) {
            final_name = final_name[0..pos].trim().to_string();
        }
    }

    final_name
}

/// Helper function to escape strings for AppleScript to prevent command injection
pub fn escape_applescript_string(input: &str) -> String {
    // First replace double quotes with escaped quotes for AppleScript
    let escaped = input.replace("\"", "\"\"");

    // Remove any control characters that could interfere with AppleScript execution
    escaped
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

// Enhanced email extraction with improved validation
pub fn extract_emails(input: &str) -> Vec<String> {
    // Use a more strict email regex pattern
    let re =
        Regex::new(r"\b[A-Za-z0-9._%+-]{1,64}@(?:[A-Za-z0-9-]{1,63}\.){1,125}[A-Za-z]{2,63}\b")
            .unwrap_or_else(|e| {
                debug!("Failed to create regex: {}", e);
                Regex::new(r"[^@]+@[^@]+\.[^@]+").unwrap() // Fallback to simpler pattern
            });

    let mut emails = Vec::new();

    // Split by common separators
    for part in input.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let part = part.trim();
        if part.len() > 320 {
            // Max allowed email length according to standards
            debug!("Skipping potential email due to excessive length: {}", part);
            continue;
        }

        if re.is_match(part) {
            // Additional validation to prevent injection
            if !part.contains('\'') && !part.contains('\"') && !part.contains('`') {
                emails.push(part.to_string());
            } else {
                debug!("Skipping email with potentially dangerous characters: {}", part);
            }
        }
    }

    debug!("Extracted emails: {:?}", emails);
    emails
}

/// Fix calendar end time formatting to ensure it's just a time (HH:MM) not a date-time
pub fn fix_calendar_end_time_format(command: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    debug!("Checking calendar command for end time format: {}", command);

    // Regex to match the calendar create command format with potential date in end time
    // Using raw string (r#"..."#) to avoid escaping issues
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

        // Test non-ducktape command with prefix added
        let input = "create a meeting tomorrow at 3pm";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape create a meeting tomorrow at 3pm");
    }

    #[test]
    fn test_sanitize_user_input() {
        let input = "Meeting with John\u{0000} tomorrow";
        let sanitized = sanitize_user_input(input);
        assert_eq!(sanitized, "Meeting with John tomorrow");

        let input = "Lunch\nmeeting";
        let sanitized = sanitize_user_input(input);
        assert_eq!(sanitized, "Lunch\nmeeting");
    }

    #[test]
    fn test_enhance_command_with_contacts() {
        // Test adding contacts flag
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-04-25 10:00 11:00 \"Work\"";
        let input = "Schedule a meeting with John Smith";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert!(enhanced.contains("--contacts \"John Smith\""));
    }

    #[test]
    fn test_enhance_command_with_zoom() {
        // Test adding zoom flag
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-04-25 10:00 11:00 \"Work\"";
        let input = "Schedule a zoom meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(enhanced.contains("--zoom"));

        // Test adding zoom flag for zoom keyword
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a zoom meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(enhanced.contains("--zoom"));

        // Test adding zoom flag for video call keyword
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a video call with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(enhanced.contains("--zoom"));

        // Test not adding zoom flag for non-zoom input
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a regular meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(!enhanced.contains("--zoom"));

        // Test not duplicating zoom flag
        let cmd =
            "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\" --zoom";
        let input = "Schedule a zoom meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert_eq!(enhanced.matches("--zoom").count(), 1);
    }

    #[test]
    fn test_enhance_command_with_recurrence() {
        // Test adding daily recurrence
        let input =
            "ducktape calendar create \"Daily Standup\" 2024-03-15 10:00 11:00 \"Work\" every day";
        let enhanced = enhance_command_with_recurrence(input);
        assert!(enhanced.contains("--repeat daily"));

        // Test adding weekly recurrence with interval
        let input = "ducktape calendar create \"Bi-weekly Meeting\" 2024-03-15 10:00 11:00 \"Work\" every 2 weeks";
        let enhanced = enhance_command_with_recurrence(input);
        assert!(enhanced.contains("--repeat weekly"));
        assert!(enhanced.contains("--interval 2"));

        // Test adding monthly recurrence
        let input =
            "ducktape calendar create \"Monthly Review\" 2024-03-15 10:00 11:00 \"Work\" monthly";
        let enhanced = enhance_command_with_recurrence(input);
        assert!(enhanced.contains("--repeat monthly"));

        // Test non-calendar command remains unchanged
        let input = "ducktape todo \"Buy groceries\"";
        let enhanced = enhance_command_with_recurrence(input);
        assert_eq!(input, enhanced);
    }

    #[test]
    fn test_validate_calendar_command() {
        // Test valid command
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --repeat weekly --interval 2";
        assert!(validate_calendar_command(cmd).is_ok());

        // Test command with shell injection attempt
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00; rm -rf /";
        assert!(validate_calendar_command(cmd).is_err());

        // Test unreasonable interval
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --interval 500";
        assert!(validate_calendar_command(cmd).is_err());

        // Test unreasonable count
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --count 1000";
        assert!(validate_calendar_command(cmd).is_err());
    }

    #[test]
    fn test_extract_contact_names() {
        // Test basic contact extraction with "with" keyword
        let input = "Schedule a meeting with John Smith tomorrow at 2pm";
        let contacts = extract_contact_names(input);
        assert_eq!(contacts, vec!["John Smith"]);

        // Test basic contact extraction with "invite" keyword
        let input = "create a zoom event at 10am on April 1 called Project Deadlines and invite Shaun Stuart";
        let contacts = extract_contact_names(input);
        assert_eq!(contacts, vec!["Shaun Stuart"]);

        // Test filtering out email addresses
        let input = "Schedule a meeting with john.doe@example.com tomorrow";
        let contacts = extract_contact_names(input);
        assert!(contacts.is_empty());

        // Test handling multiple names
        let input = "Schedule a meeting with John Smith and Jane Doe tomorrow";
        let contacts = extract_contact_names(input);
        assert!(contacts.contains(&"John Smith".to_string()));
        assert!(contacts.contains(&"Jane Doe".to_string()));
    }

    #[test]
    fn test_extract_emails() {
        // Test basic email extraction
        let input = "Schedule meeting with john@example.com";
        let emails = extract_emails(input);
        assert_eq!(emails, vec!["john@example.com"]);

        // Test multiple email addresses
        let input = "Send invite to john@example.com, jane@example.com";
        let emails = extract_emails(input);
        assert_eq!(emails, vec!["john@example.com", "jane@example.com"]);

        // Test filtering invalid emails
        let input = "Send to not.an.email and valid@example.com";
        let emails = extract_emails(input);
        assert_eq!(emails, vec!["valid@example.com"]);

        // Test handling injection attempts
        let input = "Send to malicious\"@example.com";
        let emails = extract_emails(input);
        assert!(emails.is_empty());
    }
}
