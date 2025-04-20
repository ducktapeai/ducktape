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
    // Fixed regex: removed unsupported atomic group (?>...) and replaced with (?:...)
    let re = Regex::new(r"^[A-Za-z0-9._%+-]{1,64}@[A-Za-z0-9-]{1,63}(?:\.[A-Za-z0-9-]{1,63}){1,125}\.[A-Za-z]{2,63}$").unwrap();
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

/// Validate an EventConfig for correctness and safety.
/// Returns an error if any field is invalid or unsafe for AppleScript.
pub fn validate_event_config(config: &crate::calendar::calendar_types::EventConfig) -> anyhow::Result<()> {
    use crate::calendar::calendar_validation::{validate_date_format, validate_time_format, validate_email, contains_dangerous_characters, contains_dangerous_chars_for_script};
    use crate::calendar::calendar_types::CalendarError;
    use log::debug;
    use anyhow::anyhow;

    // Validate date format (YYYY-MM-DD)
    if !validate_date_format(&config.start_date) {
        return Err(CalendarError::InvalidDateTime(format!(
            "Invalid date format: {}",
            config.start_date
        )).into());
    }

    // Validate time format (HH:MM)
    if !validate_time_format(&config.start_time) {
        return Err(CalendarError::InvalidDateTime(format!(
            "Invalid time format: {}",
            config.start_time
        )).into());
    }

    // Validate end time if specified
    if let Some(end_time) = &config.end_time {
        if !validate_time_format(end_time) {
            return Err(CalendarError::InvalidDateTime(format!(
                "Invalid end time format: {}",
                end_time
            )).into());
        }
    }

    // Process title to safely handle quotes from NLP-generated commands
    let mut sanitized_title = config.title.trim_matches('"').to_string();
    if sanitized_title.contains("\\\"") {
        sanitized_title = sanitized_title.replace("\\\"", "");
    }
    debug!("Original title: '{}', Sanitized title: '{}'", config.title, sanitized_title);
    if sanitized_title.contains(';')
        || sanitized_title.contains('&')
        || sanitized_title.contains('|')
        || sanitized_title.contains('<')
        || sanitized_title.contains('>')
        || sanitized_title.contains('$')
    {
        return Err(anyhow!("Title contains potentially dangerous characters"));
    }

    // Validate location if specified
    if let Some(location) = &config.location {
        let mut sanitized_location = location.replace("\\\"", "").replace('"', "");
        if sanitized_location.starts_with('"') && sanitized_location.ends_with('"') {
            sanitized_location = sanitized_location[1..sanitized_location.len() - 1].to_string();
        }
        if contains_dangerous_characters(&sanitized_location) {
            return Err(anyhow!("Location contains potentially dangerous characters"));
        }
    }

    // Validate description if specified
    if let Some(description) = &config.description {
        if contains_dangerous_chars_for_script(description) {
            return Err(anyhow!("Description contains potentially dangerous characters"));
        }
    }

    // Validate emails
    for email in &config.emails {
        if !validate_email(email) {
            return Err(anyhow!("Invalid email format: {}", email));
        }
    }

    // Validate timezone if specified
    if let Some(timezone) = &config.timezone {
        if timezone.len() > 50 || contains_dangerous_chars_for_script(timezone) {
            return Err(anyhow!("Invalid timezone format"));
        }
    }

    // Validate recurrence if specified
    if let Some(recurrence) = &config.recurrence {
        if let Some(end_date) = &recurrence.end_date {
            if !validate_date_format(end_date) {
                return Err(anyhow!("Invalid recurrence end date format: {}", end_date));
            }
        }
    }

    // If creating a Zoom meeting, validate needed fields
    if config.create_zoom_meeting {
        if config.end_time.is_none() {
            return Err(anyhow!("End time is required for Zoom meetings"));
        }
    }

    Ok(())
}
