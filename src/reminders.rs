use anyhow::{Result, anyhow};
use std::process::Command;

#[derive(Debug)]
pub struct ReminderConfig<'a> {
    pub title: &'a str,
    pub remind_date: Option<&'a str>, // e.g. "2025-02-21 14:30"
    pub notes: Option<String>,
}

impl<'a> ReminderConfig<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title, remind_date: None, notes: None }
    }
}

pub fn create_reminder(config: ReminderConfig) -> Result<()> {
    // Build properties for AppleScript; note that AppleScript requires a proper date format.
    let mut properties = format!("name:\"{}\"", config.title);
    if let Some(date_str) = config.remind_date {
        properties.push_str(&format!(", remind me date:date \"{}\"", date_str));
    }
    if let Some(notes) = config.notes {
        properties.push_str(&format!(", body:\"{}\"", notes));
    }

    let script = format!(
        r#"tell application "Reminders"
            try
                set newReminder to make new reminder with properties {{{}}}
                return "Success: Reminder created"
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
        properties
    );

    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        println!("Reminder created: {}", config.title);
        Ok(())
    } else {
        Err(anyhow!("Failed to create reminder: {}", result))
    }
}

pub fn list_reminders() -> Result<()> {
    let script = r#"tell application "Reminders"
        try
            set output to {}
            repeat with r in reminders
                copy name of r to end of output
            end repeat
            return output
        on error errMsg
            return "Error: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Error") {
        Err(anyhow!("Failed to list reminders: {}", result))
    } else {
        println!("Reminders:");
        for reminder in result.trim_matches(&['{', '}'][..]).split(", ") {
            println!("  - {}", reminder.trim_matches('"'));
        }
        Ok(())
    }
}
