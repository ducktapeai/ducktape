use crate::commands::{CommandArgs, CommandExecutor};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Command handler for the "version" command.
/// 
/// This command displays the current version of the DuckTape application
/// as defined in Cargo.toml.
pub struct VersionCommand;

impl CommandExecutor for VersionCommand {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move { print_version() })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "version"
    }
}

/// Prints the current version of the DuckTape application.
///
/// # Returns
/// 
/// A `Result<()>` indicating success or failure.
fn print_version() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!("DuckTape version {}", version);
    Ok(())
}
