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
            // Revised calendar command parsing:
            // Determine if "--all-day" is present in the command.
            let all_day = parts.contains(&"--all-day".to_string());
            let mut title = String::new();
            let mut date = String::new();
            let mut time = "00:00".to_string(); // default for all_day events
            let mut calendar = None;
            let mut location = None;
            let mut description = None;
            let mut email = None; // New email parameter
            
            if all_day {
                // For all-day events:
                if parts.len() < 3 {
                    println!("Usage: calendar \"<title>\" <date> [calendar-name] [--location \"<location>\"] [--description \"<description>\"] [--email \"<email>\"] --all-day");
                    return Ok(());
                }
                title = parts[1].clone();
                date = parts[2].clone();
                // Check if an optional calendar token is provided at index 3 that is not a flag.
                let mut flag_index = 3;
                if parts.len() > flag_index && !parts[flag_index].starts_with("--") {
                    calendar = Some(parts[flag_index].clone());
                    flag_index += 1;
                }
                // Process remaining flags.
                while flag_index < parts.len() {
                    match parts[flag_index].as_str() {
                        "--location" => {
                            if flag_index + 1 < parts.len() {
                                location = Some(parts[flag_index + 1].clone());
                                flag_index += 1;
                            } else {
                                println!("--location requires a value");
                                return Ok(());
                            }
                        }
                        "--description" => {
                            if flag_index + 1 < parts.len() {
                                description = Some(parts[flag_index + 1].clone());
                                flag_index += 1;
                            } else {
                                println!("--description requires a value");
                                return Ok(());
                            }
                        }
                        "--email" => {  // New flag for email
                            if flag_index + 1 < parts.len() {
                                email = Some(parts[flag_index + 1].clone());
                                flag_index += 1;
                            } else { println!("--email requires a value"); return Ok(()); }
                        }
                        _ => { }
                    }
                    flag_index += 1;
                }
            } else {
                // For timed events.
                if parts.len() < 4 {
                    println!("Usage: calendar \"<title>\" <date> <time> [calendar-name] [--location \"<location>\"] [--description \"<description>\"] [--email \"<email>\"]");
                    return Ok(());
                }
                title = parts[1].clone();
                date = parts[2].clone();
                time = parts[3].clone();
                let mut flag_index = 4;
                if parts.len() > flag_index && !parts[flag_index].starts_with("--") {
                    calendar = Some(parts[flag_index].clone());
                    flag_index += 1;
                }
                while flag_index < parts.len() {
                    match parts[flag_index].as_str() {
                        "--location" => {
                            if flag_index + 1 < parts.len() {
                                location = Some(parts[flag_index + 1].clone());
                                flag_index += 1;
                            } else {
                                println!("--location requires a value");
                                return Ok(());
                            }
                        }
                        "--description" => {
                            if flag_index + 1 < parts.len() {
                                description = Some(parts[flag_index + 1].clone());
                                flag_index += 1;
                            } else {
                                println!("--description requires a value");
                                return Ok(());
                            }
                        }
                        "--email" => {  // New flag for email
                            if flag_index + 1 < parts.len() {
                                email = Some(parts[flag_index + 1].clone());
                                flag_index += 1;
                            } else { println!("--email requires a value"); return Ok(()); }
                        }
                        _ => { }
                    }
                    flag_index += 1;
                }
            }
            calendar::create_event(&title, &date, &time, calendar.as_deref(), all_day, location, description, email)?;
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
