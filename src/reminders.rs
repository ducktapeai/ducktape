use anyhow::Result;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ReminderConfig<'a> {
    pub title: &'a str,
    pub days: i32,
    pub time: Option<&'a str>,
}

impl<'a> ReminderConfig<'a> {
    #[allow(dead_code)]
    pub fn new(title: &'a str) -> Self {
        Self { title, days: 0, time: None }
    }
}

#[allow(dead_code)]
pub fn create_reminder(config: ReminderConfig) -> Result<()> {
    // Build properties for AppleScript; note that AppleScript requires a proper date format.
    let mut properties = format!("name:\"{}\"", config.title);
    if let Some(date_str) = config.time {
        properties.push_str(&format!(", remind me date:date \"{}\"", date_str));
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

    let output = std::process::Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        println!("Reminder created: {}", config.title);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to create reminder: {}", result))
    }
}

#[allow(dead_code)]
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

    let output = std::process::Command::new("osascript").arg("-e").arg(script).output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Error") {
        Err(anyhow::anyhow!("Failed to list reminders: {}", result))
    } else {
        println!("Reminders:");
        for reminder in result.trim_matches(&['{', '}'][..]).split(", ") {
            println!("  - {}", reminder.trim_matches('"'));
        }
        Ok(())
    }
}
