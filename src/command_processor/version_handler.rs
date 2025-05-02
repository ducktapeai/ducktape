//! Version command handler for DuckTape
//!
//! Handles version-related commands.
//
// # Examples
//
// ```
// let handler = VersionHandler;
// let args = CommandArgs::new("version".to_string(), vec![], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct VersionHandler;

impl CommandHandler for VersionHandler {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("DuckTape v{}", VERSION);
            println!(
                "A tool for interacting with Apple Calendar, Notes, and Reminders via the command line."
            );
            println!("© 2024-2025 DuckTape Team");
            Ok(())
        })
    }
    fn can_handle(&self, command: &str) -> bool {
        command == "version" || command == "--version" || command == "-v"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for VersionHandler here...
}
