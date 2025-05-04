//! Utility functions for reminder operations

use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, NaiveDate, Timelike};
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

/// Parse natural language time expressions into a standardized format
pub fn parse_natural_language_time(time_expr: &str) -> Result<String> {
    let time_expr = time_expr.trim().to_lowercase();
    let now = Local::now();

    // Extract time information using regex patterns
    let time_pattern = Regex::new(r"(\d{1,2})(:\d{2})?\s*(am|pm)?").unwrap();
    let mut hour = None;
    let mut minute = 0;
    let mut is_pm = false;

    // Parse time component if present
    if let Some(time_captures) = time_pattern.captures(&time_expr) {
        hour = Some(time_captures[1].parse::<u32>()?);

        // Parse minutes if provided
        if let Some(min_match) = time_captures.get(2) {
            minute = min_match.as_str()[1..].parse::<u32>()?;
        }

        // Check for AM/PM
        if let Some(ampm) = time_captures.get(3) {
            is_pm = ampm.as_str() == "pm";
        }
    }

    // Convert hour to 24-hour format if needed
    if let Some(h) = hour {
        if h <= 12 && is_pm {
            hour = Some(if h == 12 { 12 } else { h + 12 });
        } else if h == 12 && !is_pm {
            hour = Some(0); // 12 AM = 0 in 24-hour format
        }
    }

    // Determine the date
    let target_date = if time_expr.contains("tomorrow") {
        now.date_naive()
            .succ_opt()
            .ok_or_else(|| anyhow!("Error calculating tomorrow's date"))?
    } else if time_expr.contains("today") {
        now.date_naive()
    } else {
        // Default to today if no specific date is mentioned
        now.date_naive()
    };

    // Format the result in the expected format: YYYY-MM-DD HH:MM
    let formatted_date =
        format!("{}-{:02}-{:02}", target_date.year(), target_date.month(), target_date.day());

    // If a time was specified, include it; otherwise, use current time
    if let Some(h) = hour {
        Ok(format!("{} {:02}:{:02}", formatted_date, h, minute))
    } else {
        Ok(format!("{} {:02}:{:02}", formatted_date, now.hour(), now.minute()))
    }
}

/// Parse the output of the list command to extract reminder items
pub fn parse_reminder_list_output(_output: &str) -> Vec<super::ReminderItem> {
    let reminders = Vec::new();

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
