use anyhow::Result;
use crate::commands::{CommandArgs, CommandExecutor};
use std::future::Future;
use std::pin::Pin;
use crate::notes;

pub struct NotesCommand;

impl CommandExecutor for NotesCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.command.as_str() {
                "note" => create_note(args),
                "notes" => list_notes(),
                _ => {
                    println!("Unknown notes command");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "note" | "notes")
    }
}

fn create_note(args: CommandArgs) -> Result<()> {
    if args.args.is_empty() {
        println!("Usage: note \"<title>\" --content \"<content>\" [--folder \"<folder>\"]");
        return Ok(());
    }
    
    let content = args
        .flags
        .get("--content")
        .and_then(|c| c.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");
    
    let mut config = notes::NoteConfig::new(&args.args[0], content);
    
    if let Some(folder) = args.flags.get("--folder") {
        config.folder = folder.as_deref();
    }
    
    notes::create_note(config)
}

fn list_notes() -> Result<()> {
    notes::list_notes()
}