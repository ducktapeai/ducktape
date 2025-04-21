//! Utility functions for the todo module
//
// This module contains helper functions used by the todo module

use anyhow::Result;
use chrono::{DateTime, Local};

/// Escape a string for use in AppleScript to prevent command injection
pub fn escape_applescript_string(input: &str) -> String {
    // First replace double quotes with escaped quotes for AppleScript
    let escaped = input.replace("\"", "\"\"");

    // Remove any control characters that could interfere with AppleScript execution
    escaped
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

/// Format a reminder time from standard format to AppleScript format
pub fn format_reminder_time(time_str: &str) -> Result<String> {
    // Parse input in format "YYYY-MM-DD HH:MM"
    match chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M") {
        Ok(naive_dt) => {
            // Format as "MM/dd/yyyy hh:mm:ss AM/PM" for AppleScript
            let formatted = naive_dt.format("%m/%d/%Y %I:%M:%S %p").to_string();
            Ok(formatted)
        }
        Err(e) => {
            Err(anyhow::anyhow!("Invalid reminder time format: {}", e))
        }
    }
}

/// Calculate a relative date (e.g., "tomorrow", "next week") into a specific date
pub fn resolve_relative_date(date_str: &str) -> Result<DateTime<Local>> {
    let now = Local::now();
    
    match date_str.to_lowercase().as_str() {
        "today" => Ok(now),
        "tomorrow" => Ok(now + chrono::Duration::days(1)),
        "next week" => Ok(now + chrono::Duration::days(7)),
        "next month" => {
            // Simple approach - just add 30 days
            // In a real implementation, you would want to handle month boundaries properly
            Ok(now + chrono::Duration::days(30))
        }
        _ => Err(anyhow::anyhow!("Unsupported relative date: {}", date_str)),
    }
}

/// Parse a list of todo items returned from AppleScript
pub fn parse_todo_list_output(output: &str) -> Vec<super::TodoItem> {
    let mut todos = Vec::new();
    
    // This is a placeholder for a more sophisticated parser
    // In a production system, you would want to use a proper parser for AppleScript output
    
    todos
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_escape_applescript_string() {
        assert_eq!(escape_applescript_string("Hello"), "Hello");
        assert_eq!(escape_applescript_string("Hello\"World"), "Hello\"\"World");
        assert_eq!(escape_applescript_string("Line 1\nLine 2"), "Line 1\nLine 2");
        
        // Test with control characters
        assert_eq!(escape_applescript_string("Test\u{0007}"), "Test");
    }
    
    #[test]
    fn test_format_reminder_time() {
        let result = format_reminder_time("2025-04-22 15:30").unwrap();
        // Note: The exact format might depend on the locale, so be careful with this test
        assert!(result.contains("04/22/2025"));
        assert!(result.contains("03:30:00") || result.contains("3:30:00"));
    }
    
    #[test]
    fn test_resolve_relative_date() {
        let now = Local::now();
        
        let tomorrow = resolve_relative_date("tomorrow").unwrap();
        assert_eq!(tomorrow.date_naive(), (now + chrono::Duration::days(1)).date_naive());
        
        let next_week = resolve_relative_date("next week").unwrap();
        assert_eq!(next_week.date_naive(), (now + chrono::Duration::days(7)).date_naive());
    }
}