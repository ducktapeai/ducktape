use crate::commands::{CommandArgs, CommandExecutor};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

pub struct VersionCommand;

impl CommandExecutor for VersionCommand {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move { print_version() })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "version"
    }
}

fn print_version() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!("DuckTape version {}", version);
    Ok(())
}