mod file_search;
mod calendar;
mod todo;
mod state;  // Add local state module

use anyhow::Result;
use env_logger::Env;
use log::{error, info};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

// Remove ducktape imports since we're using local modules
// use ducktape::state;
// use ducktape::todo::TodoConfig;

/// Command line arguments structure
#[derive(Debug)]
struct CommandArgs {
    command: String,
    args: Vec<String>,
    flags: std::collections::HashMap<String, Option<String>>,
}

impl CommandArgs {
    fn parse(input: &str) -> Result<Self> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    if let Some('\\') = chars.peek() {
                        // Found an escaped quote
                        chars.next(); // Skip the backslash
                        current.push('"');
                    } else {
                        in_quotes = !in_quotes;
                        if !in_quotes && !current.is_empty() {
                            parts.push(current.clone());
                            current.clear();
                        }
                    }
                }
                // Skip escaped backslashes before quotes
                '\\' if chars.peek() == Some(&'"') => {
                    continue;
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
            return Err(anyhow::anyhow!("No command provided"));
        }

        let command = parts.remove(0);
        let mut args = Vec::new();
        let mut flags = std::collections::HashMap::new();
        let mut i = 0;

        while i < parts.len() {
            if parts[i].starts_with("--") {
                let flag = parts[i].clone();
                if i + 1 < parts.len() && !parts[i + 1].starts_with("--") {
                    flags.insert(flag, Some(parts[i + 1].clone()));
                    i += 1;
                } else {
                    flags.insert(flag, None);
                }
            } else {
                args.push(parts[i].clone());
            }
            i += 1;
        }

        Ok(CommandArgs { command, args, flags })
    }
}

fn main() -> Result<()> {
    // Initialize logging with custom format
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use chrono::Local;
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    info!("Starting DuckTape Terminal");

    let mut rl = DefaultEditor::new()?;
    println!("Welcome to DuckTape Terminal! Type 'help' for commands.");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if let Err(err) = process_command(&line) {
                    error!("Failed to process command: {:?}", err);
                }
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
    let args = CommandArgs::parse(command)?;
    
    match args.command.as_str() {
        "search" => {
            if args.args.len() < 2 {
                println!("Usage: search <path> <pattern>");
                return Ok(());
            }
            file_search::search(&args.args[0], &args.args[1])?;
            Ok(())
        }
        "calendar" => handle_calendar_command(args),
        "calendars" => calendar::list_calendars(),
        "calendar-props" => calendar::list_event_properties(),
        "todo" => handle_todo_command(args),
        "list-todos" => {
            let todos = state::load_todos()?;
            println!("Stored Todo Items:");
            for item in todos {
                println!("  - {} [{}]", item.title, 
                    item.reminder_time.as_deref().unwrap_or("No reminder"));
            }
            Ok(())
        },
        "help" => {
            println!("Available commands:");
            println!("  search <path> <pattern> - Search for files");
            println!("  calendar \"<title>\" <date> <time> [calendar-name...] - Create calendar event");
            println!("  calendars - List available calendars");
            println!("  calendar-props - List available calendar event properties");
            println!("  Options:");
            println!("    --all-day                  Create an all-day event");
            println!("    --location \"<location>\"    Set event location");
            println!("    --description \"<desc>\"     Set event description");
            println!("    --email \"<email>\"         Add attendee");
            println!("    --reminder <minutes>       Set reminder (minutes before event)");
            println!("  help - Show this help");
            println!("  exit - Exit the application");
            Ok(())
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => {
            println!("Unknown command. Type 'help' for available commands.");
            Ok(())
        }
    }
}

// Modify the todo handler to save state after creating a todo
fn handle_todo_command(args: CommandArgs) -> Result<()> {
    if args.args.is_empty() {
        println!("Usage: todo \"<task title>\" [--notes \"<task notes>\"] [--lists \"<list1>,<list2>,...\"] [--reminder-time \"YYYY-MM-DD HH:MM\"]");
        return Ok(());
    }
    let mut config = todo::TodoConfig::new(&args.args[0]);
    if let Some(notes) = args.flags.get("--notes") {
        config.notes = notes.clone();
    }
    if let Some(lists) = args.flags.get("--lists") {
        let list_names: Vec<&str> = lists
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        config.lists = list_names;
    }
    if let Some(reminder) = args.flags.get("--reminder-time") {
        config.reminder_time = reminder.as_deref();
    }
    todo::create_todo(config)?;
    
    // Save todo state after creation
    let mut todos = state::load_todos().unwrap_or_else(|_| vec![]);
    todos.push(state::TodoItem {
        title: args.args[0].clone(),
        notes: args.flags.get("--notes").and_then(|n| n.clone()),
        lists: args.flags.get("--lists")
                     .map(|l| l.as_deref().unwrap_or("").split(',').map(|s| s.trim().to_owned()).collect())
                     .unwrap_or(vec!["Reminders".to_owned()]),
        reminder_time: args.flags.get("--reminder-time").and_then(|r| r.clone()),
    });
    state::save_todos(&todos)?;
    
    Ok(())
}

fn handle_calendar_command(args: CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Usage: calendar \"<title>\" <date> [time] [calendar-name...] [--location \"<location>\"] [--description \"<description>\"] [--email \"<email>\"] [--all-day]");
        return Ok(());
    }

    let all_day = args.flags.contains_key("--all-day");
    let mut config = calendar::EventConfig::new(
        &args.args[0],
        &args.args[1],
        if all_day { "00:00" } else { args.args.get(2).map_or("00:00", String::as_str) }
    );

    config.all_day = all_day;
    
    // Set calendars if provided
    if !all_day && args.args.len() > 3 || all_day && args.args.len() > 2 {
        let calendar_index = if all_day { 2 } else { 3 };
        config.calendars = args.args[calendar_index..].iter()
            .map(String::as_str)
            .collect();
    }

    // Set optional properties from flags
    if let Some(loc) = args.flags.get("--location") {
        config.location = loc.clone();
    }
    if let Some(desc) = args.flags.get("--description") {
        config.description = desc.clone();
    }
    if let Some(email) = args.flags.get("--email") {
        config.email = email.clone();
    }

    // Set reminder if provided (in minutes)
    if let Some(reminder) = args.flags.get("--reminder") {
        if let Some(minutes_str) = reminder {
            config.reminder = Some(minutes_str.parse().map_err(|_| {
                anyhow::anyhow!("Invalid reminder duration: must be a number of minutes")
            })?);
        }
    }

    calendar::create_event(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_command_args_parse_basic() {
        let input = "calendar \"Test Event\" 2024-02-21 14:30";
        let args = CommandArgs::parse(input).unwrap();
        assert_eq!(args.command, "calendar");
        assert_eq!(args.args.len(), 3);
        assert_eq!(args.args[0], "Test Event");
        assert_eq!(args.flags.len(), 0);
    }

    #[test]
    fn test_command_args_parse_with_flags() {
        let input = "calendar \"Test Event\" 2024-02-21 --all-day --location \"Test Location\"";
        let args = CommandArgs::parse(input).unwrap();
        assert_eq!(args.command, "calendar");
        assert_eq!(args.args[0], "Test Event");
        assert!(args.flags.contains_key("--all-day"));
        assert_eq!(args.flags.get("--location").unwrap().as_ref().unwrap(), "Test Location");
    }

    #[test]
    fn test_command_args_parse_empty() {
        let input = "";
        let result = CommandArgs::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_args_parse_quoted_strings() {
        let input = r#"calendar "Meeting with \"quotes\"" 2024-02-21"#;
        let args = CommandArgs::parse(input).unwrap();
        assert_eq!(args.args[0], r#"Meeting with "quotes""#, 
            "\nExpected: Meeting with \"quotes\"\nGot: {}", args.args[0]);
        
        // Add more test cases
        let input2 = r#"calendar "Meeting \"quoted\" text" 2024-02-21"#;
        let args2 = CommandArgs::parse(input2).unwrap();
        assert_eq!(args2.args[0], r#"Meeting "quoted" text"#);
    }
}
