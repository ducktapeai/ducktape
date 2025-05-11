//! Validation functions for reminder operations
//
// This module provides validation for todo/reminder operations

use super::reminder_types::ReminderError;
use anyhow::{Result, anyhow};
use log::warn;

/// Validate a reminder title
pub fn validate_title(title: &str) -> Result<()> {
    if title.is_empty() {
        return Err(anyhow!(ReminderError::InvalidInput(
            "Reminder title cannot be empty".to_string()
        )));
    }

    if title.len() > 250 {
        warn!("Reminder title is very long: {} characters", title.len());
    }

    Ok(())
}

/// Validate reminder time format
pub fn validate_reminder_time(time: &str) -> Result<()> {
    // Check if the time string is in a valid format
    // Simple check for ISO-like format: YYYY-MM-DD HH:MM
    if !time.contains('-') || !time.contains(':') || time.len() < 10 {
        return Err(anyhow!(ReminderError::InvalidInput(format!(
            "Invalid reminder time format: '{}'. Expected format: YYYY-MM-DD HH:MM",
            time
        ))));
    }

    // Parse and validate hour and minute
    let parts: Vec<&str> = time.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(anyhow!(ReminderError::InvalidInput(format!(
            "Invalid reminder time format: '{}'. Expected format: YYYY-MM-DD HH:MM",
            time
        ))));
    }
    let time_part = parts[1];
    let hm: Vec<&str> = time_part.split(':').collect();
    if hm.len() != 2 {
        return Err(anyhow!(ReminderError::InvalidInput(format!(
            "Invalid time component: '{}'. Expected HH:MM",
            time_part
        ))));
    }
    let hour: u32 = hm[0]
        .parse()
        .map_err(|_| anyhow!(ReminderError::InvalidInput(format!("Invalid hour: '{}'", hm[0]))))?;
    let minute: u32 = hm[1].parse().map_err(|_| {
        anyhow!(ReminderError::InvalidInput(format!("Invalid minute: '{}'", hm[1])))
    })?;
    if hour >= 24 {
        return Err(anyhow!(ReminderError::InvalidInput(format!(
            "Hour out of range: {} (must be 0-23)",
            hour
        ))));
    }
    if minute >= 60 {
        return Err(anyhow!(ReminderError::InvalidInput(format!(
            "Minute out of range: {} (must be 0-59)",
            minute
        ))));
    }

    Ok(())
}

/// Validate a reminder list name
pub fn validate_list_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!(ReminderError::InvalidInput("List name cannot be empty".to_string())));
    }

    // Check if list exists or can be created
    // In practice, we might want to check against the actual lists in the system

    Ok(())
}

/// Validate reminder configuration before creating
pub fn validate_reminder_config(config: &super::ReminderConfig<'_>) -> Result<()> {
    // Validate title
    validate_title(config.title)?;

    // Validate reminder time if provided
    if let Some(time_str) = config.reminder_time {
        validate_reminder_time(time_str)?;
    }

    // Validate list names
    for list in &config.lists {
        validate_list_name(list)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_title() {
        assert!(validate_title("Buy groceries").is_ok());
        assert!(validate_title("").is_err());
        assert!(validate_title("    ").is_err());

        // Test title with max length
        let long_title = "a".repeat(256);
        assert!(validate_title(&long_title).is_err());
    }

    #[test]
    fn test_validate_reminder_time() {
        assert!(validate_reminder_time("2025-04-15 14:30").is_ok());
        assert!(validate_reminder_time("2025-04-15 24:30").is_err()); // invalid hour
        assert!(validate_reminder_time("2025-04-15 14:60").is_err()); // invalid minute
        assert!(validate_reminder_time("not a date").is_err());
        assert!(validate_reminder_time("2025/04/15 14:30").is_err()); // wrong format
    }

    #[test]
    fn test_validate_list_name() {
        assert!(validate_list_name("Work").is_ok());
        assert!(validate_list_name("Personal Tasks").is_ok());
        assert!(validate_list_name("").is_err());
        assert!(validate_list_name("Invalid?Name").is_err());
        assert!(validate_list_name("Invalid/Name").is_err());
    }
}
