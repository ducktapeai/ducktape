//! Common utility functions shared across all parser implementations
//!
//! This module provides general utility functions that are used by multiple
//! parser implementations.

/// Utility functions for parsing
use anyhow::Result;
use log::debug;

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
    // Normalize spacing and trim
    let cleaned = command.trim();

    // Check if this is a calendar creation command from NLP that needs special handling
    if cleaned.contains("create a") && cleaned.contains("calendar") {
        let mut title = "Untitled Event";

        // Extract the title between "create a" and the next keyword
        if let Some(title_text) = cleaned.split("create a").nth(1) {
            let after = title_text.trim();
            let end =
                after.find(|c: char| c == ' ' || c == '"' || c == '\'').unwrap_or(after.len());
            title = after[..end].trim();
        }

        // Compose a basic calendar create command (date/time parsing is handled elsewhere)
        return format!("ducktape calendar create \"{}\" today 00:00 01:00 \"Calendar\"", title);
    }

    // Ensure the command starts with ducktape
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
}
