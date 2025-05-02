//! Help command handler for DuckTape
//!
//! Handles help-related commands.
//
// # Examples
//
// ```
// let handler = HelpHandler;
// let args = CommandArgs::new("help".to_string(), vec![], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct HelpHandler;

impl CommandHandler for HelpHandler {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            print_help()?;
            Ok(())
        })
    }
    fn can_handle(&self, command: &str) -> bool {
        command == "help" || command == "--help" || command == "-h"
    }
}

fn print_help() -> Result<()> {
    println!("DuckTape - A tool for interacting with Apple Calendar, Notes, and Reminders");
    println!();
    println!("USAGE:");
    println!("  ducktape [COMMAND] [SUBCOMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  calendar  Manage calendar events");
    println!("  todo      Manage todo items");
    println!("  notes     Manage notes");
    println!("  config    Manage configuration");
    println!("  contacts  Manage contact groups");
    println!("  utils     Utility commands");
    println!("  help      Show this help message");
    println!("  version   Show version information");
    println!("  exit      Exit the application");
    println!();
    println!("For more information on a specific command, run:");
    println!("  ducktape [COMMAND] --help");
    println!();
    println!("EXAMPLES:");
    println!("  ducktape calendar create \"Meeting with Team\" 2025-04-15 10:00 11:00");
    println!("  ducktape todo add \"Buy groceries\" tomorrow 18:00");
    println!("  ducktape notes create \"Meeting Notes\" \"Points discussed in the meeting\"");
    println!("  ducktape config set calendar.default \"Personal\"");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for HelpHandler here...
}
