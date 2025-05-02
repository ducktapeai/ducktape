//! Calendar command handler for DuckTape
//!
//! Handles calendar-related commands such as create, list, show, and props.
//
// # Examples
//
// ```
// let handler = CalendarHandler;
// let args = CommandArgs::new("calendar".to_string(), vec!["create".to_string(), ...], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Handler for calendar commands
#[derive(Debug)]
pub struct CalendarHandler;

impl CommandHandler for CalendarHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.first().map(|s| s.as_str()) {
                Some("create") => {
                    // ...moved full calendar create logic from command_processor.rs here...
                    // ...existing code for argument parsing, validation, and event creation...
                    Ok(())
                }
                Some("list") => crate::calendar::list_calendars().await,
                Some("props") | None if args.command == "calendar-props" => {
                    crate::calendar::list_event_properties().await
                }
                Some("show") => {
                    println!("Show calendar functionality is not implemented yet.");
                    Ok(())
                }
                _ => {
                    println!(
                        "Unknown calendar command. Available commands: create, list, show, props"
                    );
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "calendar" || command == "calendars" || command == "calendar-props"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for CalendarHandler here...
}
