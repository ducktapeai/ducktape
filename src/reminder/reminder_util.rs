//! Utility functions for reminder operations

use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, NaiveDate};
use regex::Regex;

/// Escape a string for use in AppleScript
pub fn escape_applescript_string(input: &str) -> String {
    input.replace("\"", "\\\"")
}

/// Format a time string for use in AppleScript reminders
pub fn format_reminder_time(time_str: &str) -> Result<String> {
    // Check for common formats and standardize
    // Expects input in format like "2025-04-22 15:30"
    let date_regex = Regex::new(r"^(\d{4})-(\d{1,2})-(\d{1,2}) (\d{1,2}):(\d{1,2})$").unwrap();

    if let Some(captures) = date_regex.captures(time_str) {
        let year = captures[1].parse::<i32>()?;
        let month = captures[2].parse::<u32>()?;
        let day = captures[3].parse::<u32>()?;
        let hour = captures[4].parse::<u32>()?;
        let minute = captures[5].parse::<u32>()?;

        // Validate date components
        if month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 {
            return Err(anyhow!("Invalid date or time components"));
        }

        // Format for AppleScript: MM/dd/yyyy hh:mm:ss AM/PM
        let date =
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| anyhow!("Invalid date"))?;

        // Format with specific date format required by AppleScript
        // This will give us something like: "4/22/2023 3:30:00 PM"
        let formatted = format!(
            "{}/{}/{} {}:{:02}:00 {}",
            month,
            day,
            year,
            if hour % 12 == 0 { 12 } else { hour % 12 },
            minute,
            if hour >= 12 { "PM" } else { "AM" }
        );

        Ok(formatted)
    } else {
        Err(anyhow!("Invalid time format. Expected YYYY-MM-DD HH:MM"))
    }
}

/// Resolve relative date expressions like "today", "tomorrow"
pub fn resolve_relative_date(date_str: &str) -> Result<String> {
    let today = Local::now().date_naive();

    match date_str.trim().to_lowercase().as_str() {
        "today" => Ok(format!("{}-{:02}-{:02}", today.year(), today.month(), today.day())),
        "tomorrow" => {
            let tomorrow =
                today.succ_opt().ok_or_else(|| anyhow!("Error calculating tomorrow's date"))?;
            Ok(format!("{}-{:02}-{:02}", tomorrow.year(), tomorrow.month(), tomorrow.day()))
        }
        _ => Err(anyhow!("Unknown relative date: {}", date_str)),
    }
}

/// Parse the output of the list command to extract reminder items
pub fn parse_reminder_list_output(_output: &str) -> Vec<super::ReminderItem> {
    let mut reminders = Vec::new();

    // In a real implementation, this would parse the output of the list command
    // For now, returning an empty vector

    reminders
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_escape_applescript_string() {
        assert_eq!(escape_applescript_string("Hello"), "Hello");
        assert_eq!(escape_applescript_string("Hello\"World"), "Hello\\\"World");
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
        assert_eq!(tomorrow, format!("{}-{:02}-{:02}", now.year(), now.month(), now.day() + 1));

        let today = resolve_relative_date("today").unwrap();
        assert_eq!(today, format!("{}-{:02}-{:02}", now.year(), now.month(), now.day()));
    }
}
