pub mod calendar;
pub mod calendar_legacy;
pub mod config;
pub mod deepseek_parser;
pub mod deepseek_reasoning;
pub mod event_search;
pub mod file_search;
pub mod grok_parser;
pub mod notes;
pub mod openai_parser;
pub mod reminders;
pub mod state;
pub mod todo;

use anyhow::Result;
use log::{debug, info};

/// Handle commands for the library interface
pub async fn handle_command(command_line: &str) -> Result<()> {
    // Clean up the command line (trim spaces, etc.)
    let command = command_line.trim();
    if command.is_empty() {
        return Ok(());
    }
    
    // Process natural language search events command
    if command.to_lowercase().contains("search") && 
       (command.to_lowercase().contains("event") || command.to_lowercase().contains("game") || 
        command.to_lowercase().contains("match") || command.to_lowercase().contains("concert")) {
        debug!("Detected event search command: {}", command);
        
        // Extract the query by removing common phrases
        let query = command
            .to_lowercase()
            .replace("search for", "")
            .replace("search", "")
            .replace("events", "")
            .replace("event", "")
            .replace("find", "")
            .replace("when is", "")
            .replace("when are", "")
            .replace("when does", "")
            .replace("when do", "")
            .trim()
            .to_string();
            
        if query.is_empty() {
            println!("Please specify what kind of events to search for.");
            return Ok(());
        }
        
        info!("Searching for events with query: {}", query);
        return event_search::search_events(&query, None).await;
    }

    // For other commands, pass through to the main executable
    debug!("Processing command: {}", command);
    let parts: Vec<&str> = command.split_whitespace().collect();
    
    if parts.is_empty() {
        return Ok(());
    }
    
    // Handle based on first part of command
    match parts[0].to_lowercase().as_str() {
        "ducktape" if parts.len() > 1 => {
            match parts[1].to_lowercase().as_str() {
                "search-events" => {
                    if parts.len() < 3 {
                        println!("Usage: ducktape search-events <query> [--calendar <calendar>]");
                        return Ok(());
                    }
                    
                    // Extract query (all remaining parts joined)
                    let query_parts: Vec<&str> = parts[2..].iter()
                        .filter(|&p| !p.starts_with("--"))
                        .filter(|&p| {
                            let prev_idx = parts.iter().position(|&x| x == *p).unwrap_or(0);
                            prev_idx == 0 || !parts[prev_idx - 1].starts_with("--")
                        })
                        .cloned()
                        .collect();
                    
                    let query = query_parts.join(" ");
                    debug!("Executing event search query: {}", query);
                    
                    // Extract optional calendar parameter if provided with --calendar flag
                    let mut calendar = None;
                    for i in 0..parts.len()-1 {
                        if parts[i] == "--calendar" {
                            calendar = Some(parts[i+1]);
                            break;
                        }
                    }
                    
                    return event_search::search_events(&query, calendar).await;
                },
                "calendar" => {
                    // Simple handling for calendar commands
                    if parts.len() < 3 {
                        println!("Supported commands: ducktape calendar create, ducktape calendar delete, ducktape calendar set-default");
                        return Ok(());
                    }
                    
                    if parts[2].to_lowercase() == "create" {
                        // For calendar create, use the event_search's command parser
                        return event_search::create_calendar_event_from_command(command).await;
                    }
                },
                _ => {
                    debug!("Command not implemented in library mode: {}", command);
                }
            }
        },
        _ => {
            debug!("Unknown command: {}", command);
        }
    }
    
    Ok(())
}

// Re-export commonly used types
pub use config::Config;
pub use state::{CalendarItem, TodoItem};
