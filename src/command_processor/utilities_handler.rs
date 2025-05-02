//! Utilities command handler for DuckTape
//!
//! Handles utility commands such as date, time, and datetime.
//
// # Examples
//
// ```
// let handler = UtilitiesHandler;
// let args = CommandArgs::new("utils".to_string(), vec!["date".to_string(), ...], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct UtilitiesHandler;

impl CommandHandler for UtilitiesHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.first().map(|s| s.as_str()) {
                Some("date") => {
                    println!("Current date: {}", chrono::Local::now().format("%Y-%m-%d"));
                    Ok(())
                }
                Some("time") => {
                    println!("Current time: {}", chrono::Local::now().format("%H:%M:%S"));
                    Ok(())
                }
                Some("datetime") => {
                    println!(
                        "Current date and time: {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                    );
                    Ok(())
                }
                _ => {
                    println!("Unknown utility command. Available commands: date, time, datetime");
                    Ok(())
                }
            }
        })
    }
    fn can_handle(&self, command: &str) -> bool {
        command == "utility" || command == "utils"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for UtilitiesHandler here...
}
