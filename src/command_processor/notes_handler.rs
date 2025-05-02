//! Notes command handler for DuckTape
//!
//! Handles notes-related commands such as create, add, list, folders, delete, and search.
//
// # Examples
//
// ```
// let handler = NotesHandler;
// let args = CommandArgs::new("notes".to_string(), vec!["create".to_string(), ...], ...);
// handler.execute(args).await?;
// ```

use super::{CommandArgs, CommandHandler};
use anyhow::{Result, anyhow};
use log::debug;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct NotesHandler;

impl CommandHandler for NotesHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.first().map(|s| s.as_str()) {
                Some("create") | Some("add") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for note create command");
                        println!(
                            "Usage: ducktape note create <title> [content] [--folder <folder_name>]"
                        );
                        return Ok(());
                    }
                    let mut title_parts = Vec::new();
                    let mut i = 1;
                    while i < args.args.len() && !args.args[i].starts_with("--") {
                        title_parts.push(args.args[i].trim_matches('"'));
                        i += 1;
                    }
                    let title = if title_parts.len() > 1 && !args.args[1].contains(' ') {
                        title_parts.join(" ")
                    } else {
                        args.args[1].trim_matches('"').to_string()
                    };
                    let content = if let Some(Some(content_val)) = args.flags.get("content") {
                        content_val.trim_matches('"')
                    } else if args.args.len() > 2 && !args.args[2].starts_with("--") {
                        args.args[2].trim_matches('"')
                    } else {
                        ""
                    };
                    let folder = args.flags.get("folder").and_then(|f| f.as_deref());
                    debug!(
                        "Creating note: title='{}', content_length={}, folder={:?}",
                        title,
                        content.len(),
                        folder
                    );
                    let config = crate::notes::NoteConfig { title: &title, content, folder };
                    match crate::notes::create_note(config).await {
                        Ok(_) => {
                            println!("Note created successfully: {}", title);
                            Ok(())
                        }
                        Err(e) => {
                            println!("Failed to create note: {}", e);
                            Err(anyhow!("Failed to create note: {}", e))
                        }
                    }
                }
                Some("list") => match crate::notes::list_notes().await {
                    Ok(notes) => {
                        if notes.is_empty() {
                            println!("No notes found");
                        } else {
                            println!("Notes:");
                            for note in notes {
                                println!("  - {} (in folder: {})", note.title, note.folder);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        println!("Failed to list notes: {}", e);
                        Err(e)
                    }
                },
                Some("folders") => match crate::notes::get_note_folders().await {
                    Ok(folders) => {
                        if folders.is_empty() {
                            println!("No note folders found");
                        } else {
                            println!("Note folders:");
                            for folder in folders {
                                println!("  - {}", folder);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        println!("Failed to get note folders: {}", e);
                        Err(e)
                    }
                },
                Some("delete") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for note delete command");
                        println!("Usage: ducktape note delete <title> [--folder <folder_name>]");
                        return Ok(());
                    }
                    let mut title_parts = Vec::new();
                    let mut i = 1;
                    while i < args.args.len() && !args.args[i].starts_with("--") {
                        title_parts.push(args.args[i].trim_matches('"'));
                        i += 1;
                    }
                    let title = if title_parts.len() > 1 && !args.args[1].contains(' ') {
                        title_parts.join(" ")
                    } else {
                        args.args[1].trim_matches('"').to_string()
                    };
                    let folder = args.flags.get("folder").and_then(|f| f.as_deref());
                    match crate::notes::delete_note(&title, folder).await {
                        Ok(_) => {
                            println!("Note deleted successfully: {}", title);
                            Ok(())
                        }
                        Err(e) => {
                            println!("Failed to delete note: {}", e);
                            Err(e)
                        }
                    }
                }
                Some("search") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for note search command");
                        println!("Usage: ducktape note search <keyword>");
                        return Ok(());
                    }
                    let mut keyword_parts = Vec::new();
                    let mut i = 1;
                    while i < args.args.len() && !args.args[i].starts_with("--") {
                        keyword_parts.push(args.args[i].trim_matches('"'));
                        i += 1;
                    }
                    let keyword = if keyword_parts.len() > 1 && !args.args[1].contains(' ') {
                        keyword_parts.join(" ")
                    } else {
                        args.args[1].trim_matches('"').to_string()
                    };
                    match crate::notes::search_notes(&keyword).await {
                        Ok(notes) => {
                            if notes.is_empty() {
                                println!("No notes found matching '{}'", keyword);
                            } else {
                                println!("Notes matching '{}':", keyword);
                                for note in notes {
                                    println!("  - {} (in folder: {})", note.title, note.folder);
                                }
                            }
                            Ok(())
                        }
                        Err(e) => {
                            println!("Failed to search notes: {}", e);
                            Err(e)
                        }
                    }
                }
                _ => {
                    println!(
                        "Unknown notes command. Available commands: create/add, list, folders, delete, search"
                    );
                    Ok(())
                }
            }
        })
    }
    fn can_handle(&self, command: &str) -> bool {
        command == "note" || command == "notes"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // ...add unit tests for NotesHandler here...
}
