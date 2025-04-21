//! Validation functions for todo operations
//
// This module provides validation for todo/reminder operations

use anyhow::Result;
use chrono::NaiveDateTime;
use log::error;
use regex::Regex;

/// Validate a todo title
pub fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow::anyhow!("Todo title cannot be empty"));
    }
    
    if title.len() > 255 {
        return Err(anyhow::anyhow!("Todo title cannot exceed 255 characters"));
    }
    
    Ok(())
}

/// Validate reminder time format (YYYY-MM-DD HH:MM)
pub fn validate_reminder_time(time_str: &str) -> Result<()> {
    match NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M") {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Invalid reminder time format: {}", e);
            Err(anyhow::anyhow!(
                "Invalid reminder time format. Expected format: YYYY-MM-DD HH:MM"
            ))
        }
    }
}

/// Validate a reminder list name
pub fn validate_list_name(list_name: &str) -> Result<()> {
    if list_name.trim().is_empty() {
        return Err(anyhow::anyhow!("List name cannot be empty"));
    }
    
    // Check for invalid characters in list name
    let invalid_chars = r#"[/\:*?"<>|]"#;
    let re = Regex::new(invalid_chars).unwrap();
    if re.is_match(list_name) {
        return Err(anyhow::anyhow!(
            "List name contains invalid characters. Avoid: / \\ : * ? \" < > |"
        ));
    }
    
    Ok(())
}

/// Validate todo configuration before creating
pub fn validate_todo_config<'a>(config: &super::TodoConfig<'a>) -> Result<()> {
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