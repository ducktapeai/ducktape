//! Reminder command handler for DuckTape
//!
//! Handles reminder-related commands such as create, add, list, and delete.
//
// # Examples
//
// ```
// let handler = ReminderHandler;
// let args = CommandArgs::new("reminder".to_string(), vec!["create".to_string(), ...], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::{Result, anyhow};
use log::debug;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct ReminderHandler;

impl CommandHandler for ReminderHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.first().map(|s| s.as_str()) {
                Some("create") | Some("add") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for reminder create command");
                        println!("Usage: ducktape reminder create <title> [list1] [list2] ...");
                        return Ok(());
                    }
                    let title = &args.args[1];
                    let mut config = crate::reminder::ReminderConfig::new(title);
                    if args.args.len() > 2 {
                        let list_names: Vec<&str> = args.args[2..]
                            .iter()
                            .map(|s| s.as_str())
                            .filter(|s| !s.starts_with("--"))
                            .collect();
                        if !list_names.is_empty() {
                            config.lists = list_names;
                        }
                    }
                    let reminder_time = if let Some(Some(time)) = args.flags.get("remind") {
                        debug!("Found reminder time in flags HashMap: {}", time);
                        Some(time.as_str().to_string())
                    } else if let Some(remind_idx) =
                        args.args.iter().position(|arg| arg == "--remind")
                    {
                        if remind_idx + 1 < args.args.len() {
                            let time = &args.args[remind_idx + 1];
                            debug!("Found reminder time as arg: {}", time);
                            Some(time.trim_matches('"').trim_matches('\'').to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(time_str) = &reminder_time {
                        debug!("Setting reminder time: {}", time_str);
                        config.reminder_time = Some(time_str);
                    }
                    let notes = if let Some(Some(note_text)) = args.flags.get("notes") {
                        Some(note_text.clone())
                    } else if let Some(notes_idx) =
                        args.args.iter().position(|arg| arg == "--notes")
                    {
                        if notes_idx + 1 < args.args.len() {
                            Some(args.args[notes_idx + 1].clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(note_text) = notes {
                        debug!("Setting notes: {}", note_text);
                        config.notes =
                            Some(note_text.trim_matches('"').trim_matches('\'').to_string());
                    }
                    debug!("Final reminder config: {:?}", config);
                    match crate::reminder::create_reminder(config).await {
                        Ok(_) => {
                            println!("Reminder '{}' created successfully", title);
                            Ok(())
                        }
                        Err(e) => {
                            println!("Failed to create reminder: {}", e);
                            Err(anyhow!("Failed to create reminder: {}", e))
                        }
                    }
                }
                Some("list") => {
                    println!("Listing reminders... (not implemented yet)");
                    Ok(())
                }
                Some("delete") => {
                    println!("Deleting reminder... (not implemented yet)");
                    Ok(())
                }
                _ => {
                    println!(
                        "Unknown reminder command. Available commands: create/add, list, delete"
                    );
                    Ok(())
                }
            }
        })
    }
    fn can_handle(&self, command: &str) -> bool {
        command == "reminder" || command == "reminders"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for ReminderHandler here...
}
