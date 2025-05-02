//! Exit command handler for DuckTape
//!
//! Handles exit and quit commands.
//
// # Examples
//
// ```
// let handler = ExitHandler;
// let args = CommandArgs::new("exit".to_string(), vec![], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct ExitHandler;

impl CommandHandler for ExitHandler {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            println!("Exiting DuckTape...");
            std::process::exit(0);
        })
    }
    fn can_handle(&self, command: &str) -> bool {
        command == "exit" || command == "quit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for ExitHandler here...
}
