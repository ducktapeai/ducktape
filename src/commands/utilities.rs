use anyhow::Result;
use crate::commands::{CommandArgs, CommandExecutor};
use std::future::Future;
use std::pin::Pin;
use crate::{file_search, state};

pub struct UtilitiesCommand;

impl CommandExecutor for UtilitiesCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.command.as_str() {
                "search" => search_files(args),
                "cleanup" => cleanup_storage(),
                "exit" => {
                    // Exit the application
                    std::process::exit(0);
                }
                _ => {
                    println!("Unknown utility command");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "search" | "cleanup" | "exit")
    }
}

fn search_files(args: CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Usage: search <path> <pattern>");
        return Ok(());
    }
    
    file_search::search(&args.args[0], &args.args[1])
}

fn cleanup_storage() -> Result<()> {
    println!("Cleaning up old items...");
    state::StateManager::new()?.cleanup_old_items()?;
    
    println!("Compacting storage files...");
    state::StateManager::new()?.vacuum()?;
    
    println!("Cleanup complete!");
    Ok(())
}