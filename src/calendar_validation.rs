//! Validation functions for calendar event data.
//
// This module provides validation helpers for dates, times, emails, and script safety.

use regex::Regex;
use chrono::Datelike;

/// Validate date string has format YYYY-MM-DD
pub fn validate_date_format(date: &str) -> bool {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    if !re.is_match(date) {
        return false;
    }
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        let year = naive_date.year();
        return year >= 2000 && year <= 2100;
    }
    false
}

/// Validate time string has format HH:MM
pub fn validate_time_format(time: &str) -> bool {
    let re = Regex::new(r"^\d{1,2}:\d{2}$").unwrap();
    if !re.is_match(time) {
        return false;
    }
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
        return hours < 24 && minutes < 60;
    }
    false
}

/// Enhanced email validation to handle edge cases and improve error reporting
pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[A-Za-z0-9._%+-]{1,64}@(?>[A-Za-z0-9-]{1,63}\.){1,125}[A-Za-z]{2,63}$").unwrap();
    if !re.is_match(email) {
        return false;
    }
    if contains_dangerous_characters(email) {
        return false;
    }
    true
}

/// Check for potentially dangerous characters that could cause AppleScript injection
pub fn contains_dangerous_characters(input: &str) -> bool {
    input.contains(';')
        || input.contains('&')
        || input.contains('|')
        || input.contains('<')
        || input.contains('>')
        || input.contains('$')
}

/// Check for characters that could break AppleScript specifically
pub fn contains_dangerous_chars_for_script(input: &str) -> bool {
    input.contains('"') || input.contains('\\') || input.contains('¬')
}
