mod file_search;
mod calendar;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use anyhow::Result;
use env_logger::Env;
use log::info;  // Add this line

fn main() -> Result<()> {
    // Initialize logger with default settings
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .init();
    
    info!("Starting DuckTape Terminal");

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
            if parts.len() < 3 {
                println!("Usage:");
                println!("  For timed events:");
                println!("    calendar \"<title>\" <date> <time> [calendar-name] [--location \"<location>\"] [--description \"<description>\"]");
                println!("    Example: calendar \"Team Meeting\" 2024-02-20 14:30 \"Work\" --location \"Conference Room\" --description \"Discuss project updates\"");
                println!();
                println!("  For all-day events:");
                println!("    calendar \"<title>\" <date> [calendar-name] [--location \"<location>\"] [--description \"<description>\"] --all-day");
                println!("    Example: calendar \"Company Holiday\" 2024-02-20 \"Work\" --all-day --description \"Holiday celebration\"");
                return Ok(());
            }

            let mut title = String::new();
            let mut date = String::new();
            let mut time = "00:00".to_string();
            let mut calendar = None;
            let mut all_day = false;
            let mut location = None;
            let mut description = None;

            let mut i = 1;
            while i < parts.len() {
                if i == 1 {
                    title = parts[i].clone();
                } else if i == 2 {
                    date = parts[i].clone();
                } else {
                    match parts[i].as_str() {
                        "--all-day" => {
                            all_day = true;
                        }
                        "--location" => {
                            if i + 1 < parts.len() {
                                location = Some(parts[i + 1].to_string());
                                i += 1;
                            } else {
                                println!("--location requires a value");
                                return Ok(());
                            }
                        }
                        "--description" => {
                            if i + 1 < parts.len() {
                                description = Some(parts[i + 1].to_string());
                                i += 1;
                            } else {
                                println!("--description requires a value");
                                return Ok(());
                            }
                        }
                        _ => {
                            if time == "00:00" && !all_day {
                                time = parts[i].clone();
                            } else {
                                calendar = Some(parts[i].clone());
                            }
                        }
                    }
                }
                i += 1;
            }

            calendar::create_event(&title, &date, &time, calendar.as_deref(), all_day, location, description)?;
        }
        "calendars" => {
            calendar::list_calendars()?;
        }
        "calendar-props" => {
            calendar::list_event_properties()?;
        }
        "help" => {
            println!("Available commands:");
            println!("  search <path> <pattern> - Search for files");
            println!("  calendar \"<title>\" <date> <time> [calendar-name] - Create calendar event");
            println!("  calendars - List available calendars");
            println!("  calendar-props - List available calendar event properties");
            println!("  help - Show this help");
            println!("  exit - Exit the application");
        }
        "exit" => std::process::exit(0),
        _ => println!("Unknown command. Type 'help' for available commands."),
    }
    Ok(())
}
