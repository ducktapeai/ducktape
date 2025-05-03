//! Common utility functions shared across all parser implementations
//!
//! This module provides general utility functions that are used by multiple
//! parser implementations.

/// Utility functions for parsing
use anyhow::Result;
use chrono::{Local, NaiveDate, NaiveTime};
use log::debug;
use regex::Regex;
use thiserror::Error;

/// Error type for natural language parsing failures
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Could not detect intent in input")]
    IntentNotDetected,
    #[error("Missing required entity: {0}")]
    MissingEntity(&'static str),
    #[error("Failed to parse date/time: {0}")]
    DateTimeParse(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Parse a natural language input into a valid DuckTape command string.
///
/// # Arguments
/// * `input` - The natural language input string
///
/// # Returns
/// * `Ok(String)` - A valid DuckTape CLI command string
/// * `Err(ParseError)` - If parsing fails or required entities are missing
///
/// # Examples
/// ```
/// let cmd = parse_natural_language_to_command("schedule a zoom meeting with Joe tomorrow at 3pm").unwrap();
/// assert!(cmd.starts_with("ducktape calendar create"));
/// ```
pub fn parse_natural_language_to_command(input: &str) -> Result<String, ParseError> {
    let input = input.trim();
    let lower = input.to_lowercase();
    debug!("parse_natural_language_to_command: input='{}' lower='{}'", input, lower);

    // Improved note intent detection regex
    let note_re =
        Regex::new(r"(?i)(create a note|add a note|take note|note|create note|add note)").unwrap();
    if note_re.is_match(&lower) {
        debug!("Note intent detected for input: '{}'", input);
        let title = extract_note_title(input).unwrap_or_else(|| "Untitled".to_string());
        debug!("Extracted note title: '{}'", title);
        return Ok(format!("ducktape note create \"{}\"", title));
    }

    // 1. Intent Detection
    let (intent, mut command) = if lower.contains("remind me") || lower.contains("reminder") {
        ("reminder", String::from("ducktape reminder create"))
    } else if lower.contains("calendar") || lower.contains("event") || lower.contains("meeting") {
        ("calendar", String::from("ducktape calendar create"))
    } else if lower.contains("note") {
        ("note", String::from("ducktape note create"))
    } else {
        return Err(ParseError::IntentNotDetected);
    };
    debug!("Intent detected: '{}', command: '{}'", intent, command);

    // 2. Entity Extraction
    // Title
    let title = extract_title(input).unwrap_or_else(|| "Untitled".to_string());
    command.push_str(&format!(" \"{}\"", title));

    // Date/Time (very basic, can be improved)
    let (date, start_time, end_time) = extract_date_time(input).unwrap_or_else(|| {
        let today = Local::now().date_naive();
        (today.to_string(), "09:00".to_string(), "10:00".to_string())
    });
    if intent == "calendar" {
        command.push_str(&format!(" {} {} {} \"Work\"", date, start_time, end_time));
    }

    // Contacts
    if let Some(contacts) = extract_contacts(input) {
        command.push_str(&format!(" --contacts \"{}\"", contacts));
    }

    // Zoom (robust detection)
    let zoom_keywords =
        ["zoom", "video call", "video meeting", "virtual meeting", "online meeting"];
    if zoom_keywords.iter().any(|kw| lower.contains(kw)) {
        command.push_str(" --zoom");
    }

    // Recurrence (very basic)
    if lower.contains("every week") || lower.contains("weekly") {
        command.push_str(" --repeat weekly");
    } else if lower.contains("every day") || lower.contains("daily") {
        command.push_str(" --repeat daily");
    }

    Ok(command)
}

/// Extract a title from the input using simple patterns
fn extract_title(input: &str) -> Option<String> {
    let re = Regex::new(r#"called ([\w\s]+)"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }
    // Fallback: look for quoted text
    let re = Regex::new(r#"([^"]+)"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }
    // Fallback: after 'meeting', 'event', or 'reminder'
    for kw in ["meeting", "event", "reminder", "note"] {
        if let Some(idx) = input.to_lowercase().find(kw) {
            let after = &input[idx + kw.len()..];
            let words: Vec<&str> = after.split_whitespace().collect();
            if !words.is_empty() {
                return Some(words.join(" ").trim().to_string());
            }
        }
    }
    None
}

/// Extract a note title from the input using patterns like 'called', 'titled', 'about', or after 'note'
fn extract_note_title(input: &str) -> Option<String> {
    // More specific regex for "called" pattern with word boundaries
    let re = Regex::new(r#"(?i)note\s+called\s+(.+)$"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }

    // Alternative "called" pattern
    let re = Regex::new(r#"(?i)called\s+(.+)$"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }

    let re = Regex::new(r#"(?i)titled\s+(.+)$"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }

    let re = Regex::new(r#"(?i)about\s+(.+)$"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }

    // Fallback: after 'note'
    if let Some(idx) = input.to_lowercase().find("note") {
        let after = &input[idx + 4..];
        if !after.trim().is_empty() {
            return Some(after.trim().to_string());
        }
    }

    None
}

/// Extract date and time from input (very basic, can be improved)
fn extract_date_time(input: &str) -> Option<(String, String, String)> {
    // Look for 'tomorrow at HH:MM(am|pm)' or 'at HH:MM(am|pm)'
    let re = Regex::new(r#"tomorrow at (\d{1,2})(?::(\d{2}))?\s*(am|pm)?"#).unwrap();
    if let Some(caps) = re.captures(&input.to_lowercase()) {
        let hour: u32 = caps[1].parse().ok()?;
        let minute: u32 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let ampm = caps.get(3).map(|m| m.as_str());
        let mut hour = if let Some(ampm) = ampm {
            if ampm == "pm" && hour < 12 { hour + 12 } else { hour }
        } else {
            hour
        };
        let tomorrow = Local::now().date_naive() + chrono::Duration::days(1);
        return Some((
            tomorrow.to_string(),
            format!("{:02}:{:02}", hour, minute),
            format!("{:02}:{:02}", hour + 1, minute),
        ));
    }
    // Fallback: today at HH:MM
    let re = Regex::new(r#"at (\d{1,2})(?::(\d{2}))?\s*(am|pm)?"#).unwrap();
    if let Some(caps) = re.captures(&input.to_lowercase()) {
        let hour: u32 = caps[1].parse().ok()?;
        let minute: u32 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let ampm = caps.get(3).map(|m| m.as_str());
        let mut hour = if let Some(ampm) = ampm {
            if ampm == "pm" && hour < 12 { hour + 12 } else { hour }
        } else {
            hour
        };
        let today = Local::now().date_naive();
        return Some((
            today.to_string(),
            format!("{:02}:{:02}", hour, minute),
            format!("{:02}:{:02}", hour + 1, minute),
        ));
    }
    None
}

/// Extract contacts from input (very basic, can be improved)
fn extract_contacts(input: &str) -> Option<String> {
    let re = Regex::new(r#"with ([\w\s]+)"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }
    let re = Regex::new(r#"invite ([\w\s]+)"#).unwrap();
    if let Some(caps) = re.captures(input) {
        return Some(caps[1].trim().to_string());
    }
    None
}

/// Sanitize user input to prevent injection
pub fn sanitize_user_input(input: &str) -> String {
    // Filter out control characters except for newlines and tabs
    input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

/// Helper function to clean up NLP-generated commands
/// Removes unnecessary quotes and normalizes spacing
pub fn sanitize_nlp_command(command: &str) -> String {
    debug!("sanitize_nlp_command: input='{}'", command);
    let zoom_keywords =
        ["zoom", "video call", "video meeting", "virtual meeting", "online meeting"];
    let input_lower = command.to_lowercase();
    // Always try robust mapping first
    if let Ok(mut cmd) = parse_natural_language_to_command(command) {
        debug!("sanitize_nlp_command: mapped to '{}'", cmd);
        if zoom_keywords.iter().any(|kw| input_lower.contains(kw)) && !cmd.contains("--zoom") {
            cmd.push_str(" --zoom");
        }
        return cmd;
    }
    // Fallback: only prepend ducktape if not already a ducktape command
    let cleaned = command.trim();
    debug!("sanitize_nlp_command: fallback, cleaned='{}'", cleaned);
    if !cleaned.starts_with("ducktape") {
        return format!("ducktape {}", cleaned);
    }
    cleaned.to_string()
}

/// Validate calendar command for security
pub fn validate_calendar_command(command: &str) -> Result<()> {
    use anyhow::anyhow;

    // Security checks
    if command.contains("&&")
        || command.contains("|")
        || command.contains(";")
        || command.contains("`")
    {
        return Err(anyhow!("Generated command contains potentially unsafe characters"));
    }

    // Only check calendar commands
    if command.contains("calendar create") {
        // Check for reasonably sized intervals for recurring events
        if command.contains("--interval") {
            let re = regex::Regex::new(r"--interval (\d+)").unwrap();
            if let Some(caps) = re.captures(command) {
                if let Some(interval_match) = caps.get(1) {
                    if let Ok(interval) = interval_match.as_str().parse::<i32>() {
                        if interval > 100 {
                            return Err(anyhow!("Unreasonable interval value: {}", interval));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_sanitize_nlp_command() {
        let input = "create a meeting tomorrow";
        let sanitized = sanitize_nlp_command(input);
        assert!(sanitized.starts_with("ducktape"));

        let input = "ducktape calendar create \"Test Event\" 2024-05-01 10:00 11:00 \"Work\"";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, input);
    }

    #[test]
    fn test_validate_calendar_command() {
        // Test valid command
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\"";
        assert!(validate_calendar_command(cmd).is_ok());

        // Test command with unsafe characters
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" && echo hacked";
        assert!(validate_calendar_command(cmd).is_err());

        // Test command with unreasonable interval
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --interval 1000";
        assert!(validate_calendar_command(cmd).is_err());
    }

    #[test]
    fn test_parse_natural_language_to_command_calendar() {
        let cmd =
            parse_natural_language_to_command("schedule a zoom meeting with Joe tomorrow at 3pm")
                .unwrap();
        assert!(cmd.starts_with("ducktape calendar create"));
        assert!(cmd.contains("--zoom"));
        assert!(cmd.contains("--contacts"));
    }

    #[test]
    fn test_parse_natural_language_to_command_reminder() {
        let cmd =
            parse_natural_language_to_command("remind me to call Jane tomorrow at 2pm").unwrap();
        assert!(cmd.starts_with("ducktape reminder create"));
    }

    #[test]
    fn test_parse_natural_language_to_command_note() {
        let cmd = parse_natural_language_to_command("create a note about project ideas").unwrap();
        assert!(cmd.starts_with("ducktape note create"));
    }

    #[test]
    fn test_parse_natural_language_to_command_note_patterns() {
        let cmd = parse_natural_language_to_command("create a note called call Shaun").unwrap();
        assert_eq!(cmd, "ducktape note create \"call Shaun\"");
        let cmd = parse_natural_language_to_command("add a note about project ideas").unwrap();
        assert_eq!(cmd, "ducktape note create \"project ideas\"");
        let cmd = parse_natural_language_to_command("create a note titled Meeting Notes").unwrap();
        assert_eq!(cmd, "ducktape note create \"Meeting Notes\"");
    }

    #[test]
    fn test_parse_natural_language_to_command_intent_not_detected() {
        let err = parse_natural_language_to_command("just some random text").unwrap_err();
        matches!(err, ParseError::IntentNotDetected);
    }

    #[test]
    fn test_parse_natural_language_to_command_note_multiword() {
        let cmd =
            super::parse_natural_language_to_command("create a note called call Shaun Stuart")
                .unwrap();
        assert_eq!(cmd, "ducktape note create \"call Shaun Stuart\"");
    }
}
