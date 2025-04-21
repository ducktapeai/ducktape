//! AppleScript integration for interacting with macOS Reminders.app
//
// This module provides functions to interact with the Reminders application via AppleScript

use super::todo_types::{TodoConfig, TodoItem, TodoError};
use super::todo_util::escape_applescript_string;
use anyhow::{Result, anyhow};
use log::{debug, error, info};
use std::process::Command;

/// Ensure Reminders.app is running
pub async fn ensure_reminders_running() -> Result<()> {
    let check_script = r#"tell application "Reminders"
        if it is not running then
            launch
            delay 1
        end if
        return "OK"
    end tell"#;
    
    let output = Command::new("osascript").arg("-e").arg(check_script).output()?;
    
    if output.status.success() { 
        Ok(()) 
    } else { 
        Err(anyhow!(TodoError::NotRunning)) 
    }
}

/// Create a single todo in Reminders.app
pub async fn create_single_todo(config: TodoConfig<'_>) -> Result<()> {
    debug!("Creating todo with config: {:?}", config);
    
    // Make sure Reminders app is running
    ensure_reminders_running().await?;
    
    let target_lists = if config.lists.is_empty() { vec!["Reminders"] } else { config.lists };

    // Format reminder time to AppleScript-friendly string if provided
    let reminder_prop = if let Some(time_str) = config.reminder_time {
        // Parse input in format "YYYY-MM-DD HH:MM"
        match chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M") {
            Ok(naive_dt) => {
                // Format as "date \"MM/dd/yyyy hh:mm:ss AM/PM\""
                let formatted = naive_dt.format("%m/%d/%Y %I:%M:%S %p").to_string();
                debug!("Setting reminder time to: {}", formatted);
                // Use the correct AppleScript syntax for setting a due date on reminders
                format!(", due date:date \"{}\"", formatted)
            }
            Err(e) => {
                error!("Invalid reminder time format: {}", e);
                String::new()
            }
        }
    } else {
        String::new()
    };

    let mut success_count = 0;
    for list in target_lists {
        // Escape all inputs to prevent command injection
        let escaped_list = escape_applescript_string(list);
        let escaped_title = escape_applescript_string(config.title);
        let escaped_notes = escape_applescript_string(config.notes.as_deref().unwrap_or(""));

        // Updated AppleScript with escaped inputs
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
            escaped_list, // search for list
            escaped_list, // create list if not found
            escaped_title,
            escaped_notes,
            reminder_prop
        );

        debug!("Executing AppleScript: {}", script);

        let output = Command::new("osascript").arg("-e").arg(&script).output()?;
        let result = String::from_utf8_lossy(&output.stdout);
        let error_output = String::from_utf8_lossy(&output.stderr);
        
        if !error_output.is_empty() {
            error!("AppleScript error: {}", error_output);
        }

        if result.contains("Success") {
            info!("Todo created in list {}: {}", list, config.title);
            success_count += 1;
        } else {
            let error_msg = result.replace("Error: ", "");
            error!("Failed to create todo in list {}: {}", list, error_msg);
        }
    }

    if success_count > 0 {
        Ok(())
    } else {
        Err(anyhow!(TodoError::General(format!(
            "Failed to create todo '{}' in any specified list",
            config.title
        ))))
    }
}

/// Get available reminder lists
pub async fn get_reminder_lists() -> Result<Vec<String>> {
    // Make sure Reminders app is running
    ensure_reminders_running().await?;
    
    let script = r#"tell application "Reminders"
    set listNames to {}
    repeat with l in lists
        copy (name of l) to the end of listNames
    end repeat
    return listNames
end tell"#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;
    if !output.status.success() {
        return Err(anyhow!(TodoError::ScriptError(String::from_utf8_lossy(&output.stderr).to_string())));
    }
    
    let lists_str = String::from_utf8_lossy(&output.stdout);
    let lists: Vec<String> = lists_str
        .trim_matches('{')
        .trim_matches('}')
        .split(", ")
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    
    debug!("Found {} reminder lists", lists.len());
    Ok(lists)
}

/// Fetch todos from a specific list or all lists
pub async fn fetch_todos(list_name: Option<&str>) -> Result<Vec<TodoItem>> {
    // Make sure Reminders app is running
    ensure_reminders_running().await?;
    
    let script = if let Some(list) = list_name {
        let escaped_list = escape_applescript_string(list);
        format!(
            r#"tell application "Reminders"
    set todoList to {}
    set listObj to first list whose name is "{0}"
    repeat with t in (reminders in listObj)
        set todoTitle to name of t
        set todoCompleted to completed of t
        set todoBody to ""
        try
            set todoBody to body of t
        end try
        set todoItem to {{title:todoTitle, notes:todoBody, completed:todoCompleted, listName:"{0}"}}
        copy todoItem to end of todoList
    end repeat
    return todoList
end tell"#,
            escaped_list
        )
    } else {
        r#"tell application "Reminders"
    set todoList to {}
    repeat with l in lists
        set listName to name of l
        repeat with t in (reminders in l)
            set todoTitle to name of t
            set todoCompleted to completed of t
            set todoBody to ""
            try
                set todoBody to body of t
            end try
            set todoItem to {title:todoTitle, notes:todoBody, completed:todoCompleted, listName:listName}
            copy todoItem to end of todoList
        end repeat
    end repeat
    return todoList
end tell"#
        .to_string()
    };

    let output = Command::new("osascript").arg("-e").arg(script).output()?;
    if !output.status.success() {
        return Err(anyhow!(TodoError::ScriptError(String::from_utf8_lossy(&output.stderr).to_string())));
    }
    
    // Parse the AppleScript output into TodoItem structs
    let todos_output = String::from_utf8_lossy(&output.stdout);
    let mut todos = Vec::new();
    
    // Very basic parsing of AppleScript output - in a real implementation 
    // you might want a more robust parser
    for line in todos_output.lines() {
        if line.contains("title:") && line.contains("notes:") {
            // Extract data from line
            let mut title = String::new();
            let mut notes = None;
            let mut completed = false;
            let mut list_name = String::new();
            
            // Super simple parsing - you might want to improve this
            if let Some(title_start) = line.find("title:") {
                if let Some(title_end) = line[title_start..].find(",") {
                    title = line[title_start + 6..title_start + title_end].trim().to_string();
                }
            }
            
            if let Some(notes_start) = line.find("notes:") {
                if let Some(notes_end) = line[notes_start..].find(",") {
                    let note_text = line[notes_start + 6..notes_start + notes_end].trim().to_string();
                    if !note_text.is_empty() {
                        notes = Some(note_text);
                    }
                }
            }
            
            if let Some(completed_start) = line.find("completed:") {
                if let Some(completed_str) = line[completed_start + 10..].split_whitespace().next() {
                    completed = completed_str == "true";
                }
            }
            
            if let Some(list_start) = line.find("listName:") {
                list_name = line[list_start + 9..].trim().to_string();
            }
            
            todos.push(TodoItem {
                title,
                notes,
                lists: vec![list_name],
                reminder_time: None, // We don't parse this in this example
                completed,
            });
        }
    }
    
    debug!("Fetched {} todos", todos.len());
    Ok(todos)
}

/// Delete a todo by title and list
pub async fn delete_todo(title: &str, list_name: Option<&str>) -> Result<()> {
    // Make sure Reminders app is running
    ensure_reminders_running().await?;
    
    let escaped_title = escape_applescript_string(title);
    
    let script = if let Some(list) = list_name {
        let escaped_list = escape_applescript_string(list);
        format!(
            r#"tell application "Reminders"
    try
        set targetList to first list whose name is "{}"
        set itemsToDelete to (reminders in targetList whose name is "{}")
        if (count of itemsToDelete) > 0 then
            delete itemsToDelete
            return "Success: Todo deleted"
        else
            return "Error: Todo not found in specified list"
        end if
    on error errMsg
        return "Error: " & errMsg
    end try
end tell"#,
            escaped_list,
            escaped_title
        )
    } else {
        format!(
            r#"tell application "Reminders"
    try
        set foundTodo to false
        repeat with l in lists
            set itemsToDelete to (reminders in l whose name is "{}")
            if (count of itemsToDelete) > 0 then
                delete itemsToDelete
                set foundTodo to true
            end if
        end repeat
        
        if foundTodo then
            return "Success: Todo deleted"
        else
            return "Error: Todo not found in any list"
        end if
    on error errMsg
        return "Error: " & errMsg
    end try
end tell"#,
            escaped_title
        )
    };

    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Success") {
        info!("Todo deleted: {}", title);
        Ok(())
    } else {
        let error_msg = result.replace("Error: ", "");
        error!("Failed to delete todo: {}", error_msg);
        
        if error_msg.contains("not found") {
            Err(anyhow!(TodoError::TodoNotFound(title.to_string())))
        } else {
            Err(anyhow!(TodoError::ScriptError(error_msg)))
        }
    }
}