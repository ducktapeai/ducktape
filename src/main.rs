mod file_search;
mod calendar;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use anyhow::Result;

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    println!("Welcome to DuckTape Terminal! Type 'help' for commands.");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                process_command(&line)?;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn process_command(command: &str) -> Result<()> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in command.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                if !in_quotes && !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    if parts.is_empty() {
        return Ok(());
    }

    match parts[0].as_str() {
        "search" => {
            if parts.len() < 3 {
                println!("Usage: search <path> <pattern>");
                return Ok(());
            }
            file_search::search(&parts[1], &parts[2])?;
        }
        "calendar" => {
            if parts.len() < 4 {
                println!("Usage: calendar \"<title>\" <date> <time> [calendar-name]");
                println!("Example: calendar \"Meeting\" 2025-02-20 14:30 shaun.stuart@hashicorp.com");
                return Ok(());
            }
            let calendar = if parts.len() > 4 { Some(parts[4].as_str()) } else { None };
            calendar::create_event(&parts[1], &parts[2], &parts[3], calendar)?;
        }
        "calendars" => {
            calendar::list_calendars()?;
        }
        "help" => {
            println!("Available commands:");
            println!("  search <path> <pattern> - Search for files");
            println!("  calendar \"<title>\" <date> <time> [calendar-name] - Create calendar event");
            println!("  calendars - List available calendars");
            println!("  help - Show this help");
            println!("  exit - Exit the application");
        }
        "exit" => std::process::exit(0),
        _ => println!("Unknown command. Type 'help' for available commands."),
    }
    Ok(())
}
