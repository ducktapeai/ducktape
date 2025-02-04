use anyhow::{anyhow, Result};
use std::process::Command;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct TodoConfig<'a> {
    pub title: &'a str,
    pub notes: Option<String>,
    // New field for tagging a todo to different lists
    pub lists: Vec<&'a str>,
    // New field for setting a reminder time for the todo item
    pub reminder_time: Option<&'a str>,
}

impl<'a> TodoConfig<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title, notes: None, lists: Vec::new(), reminder_time: None }
    }
}

pub fn create_todo(mut config: TodoConfig) -> Result<()> {
    let target_lists = if config.lists.is_empty() {
        vec!["Reminders"]
    } else {
        config.lists
    };
    // Format reminder time to AppleScript-friendly string if provided
    let reminder_prop = if let Some(time_str) = config.reminder_time {
        // Parse input in format "YYYY-MM-DD HH:MM"
        match chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M") {
            Ok(naive_dt) => {
                // Format as "MM/dd/yyyy hh:mm:ss AM/PM"
                let formatted = naive_dt.format("%m/%d/%Y %I:%M:%S %p").to_string();
                format!(", remind me date:date \"{}\"", formatted)
            },
            Err(e) => {
                println!("Invalid reminder time format: {}", e);
                String::new()
            }
        }
    } else {
        String::new()
    };
    let mut success_count = 0;
    for list in target_lists {
        // Updated AppleScript to locate or create the target list
        let script = format!(
r#"tell application "Reminders"
    try
        set remLists to lists whose name is "{}"
        if (count of remLists) > 0 then
            set targetList to item 1 of remLists
        else
            set targetList to make new list with properties {{name:"{}"}}
        end if
        set newTodo to make new reminder in targetList with properties {{name:"{}", body:"{}"{} }}
        return "Success: Todo created"
    on error errMsg
        return "Error: " & errMsg
    end try
end tell"#,
            list, // search for list
            list, // create list if not found
            config.title,
            config.notes.as_deref().unwrap_or(""),
            reminder_prop
        );
        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()?;
        let result = String::from_utf8_lossy(&output.stdout);
        if result.contains("Success") {
            println!("Todo created in list {}: {}", list, config.title);
            success_count += 1;
        } else {
            println!("Failed to create todo in list {}: {}", list, config.title);
        }
    }
    if success_count > 0 {
        Ok(())
    } else {
        Err(anyhow!("Failed to create todo in any specified list"))
    }
}

pub fn list_todos() -> Result<()> {
    // List to‑do items; for demonstration we're listing reminders as todos.
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
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;
        
    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Error") {
        Err(anyhow::anyhow!("Failed to list todos: {}", result))
    } else {
        println!("To-do items:");
        for todo in result.trim_matches(&['{', '}'][..]).split(", ") {
            println!("  - {}", todo.trim_matches('"'));
        }
        Ok(())
    }
}
