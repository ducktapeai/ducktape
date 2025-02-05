mod calendar;
mod file_search;
mod notes;
mod state;
mod todo;
mod openai_parser;  // Keep only this one, remove ai_parser

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

        // Special case for help command
        if parts.len() == 1 && (parts[0] == "--help" || parts[0] == "-h") {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: std::collections::HashMap::new(),
            });
        }

        // Check for and remove "ducktape" prefix
        if parts[0] != "ducktape" {
            return Err(anyhow::anyhow!("Commands must start with 'ducktape'"));
        }
        parts.remove(0); // Remove "ducktape"

        if parts.is_empty() {
            return Err(anyhow::anyhow!("No command provided after 'ducktape'"));
        }

        // Check if the first argument after "ducktape" is a help flag
        if parts[0] == "--help" || parts[0] == "-h" {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: std::collections::HashMap::new(),
            });
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

        Ok(CommandArgs {
            command,
            args,
            flags,
        })
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

    // Duck with tape emoji combination
    let prompt = "🦆 "; // Duck with paperclip (representing tape)
                        // Alternative combinations:
                        // let prompt = "🦆🤐 ";  // Duck with zipper mouth (looks like tape)
                        // let prompt = "🦆⌇ ";   // Duck with tape-like symbol
                        // ASCII art alternative:
                        // let prompt = "<=|] ";  // Duck with tape mark

    loop {
        let readline = rl.readline(prompt);
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
    // If the command doesn't start with "ducktape", treat it as natural language
    if !command.trim().starts_with("ducktape") {
        let runtime = tokio::runtime::Runtime::new()?;
        let ducktape_command = runtime.block_on(crate::openai_parser::parse_natural_language(command))?;
        println!("🦆 Interpreting as: {}", ducktape_command);  // Show the interpreted command
        return process_command(&ducktape_command);
    }

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
                println!(
                    "  - {} [{}]",
                    item.title,
                    item.reminder_time.as_deref().unwrap_or("No reminder")
                );
            }
            Ok(())
        }
        "list-events" => {
            let events = state::load_events()?;
            println!("Stored Calendar Events:");
            for event in events {
                println!(
                    "  - {} [{}]",
                    event.title,
                    if event.all_day {
                        "All day"
                    } else {
                        &event.time
                    }
                );
                println!("    Date: {}", event.date);
                println!("    Calendars: {}", event.calendars.join(", "));
                if let Some(loc) = event.location {
                    println!("    Location: {}", loc);
                }
                if let Some(desc) = event.description {
                    println!("    Description: {}", desc);
                }
                if let Some(email) = event.email {
                    println!("    Attendee: {}", email);
                }
                if let Some(reminder) = event.reminder {
                    println!("    Reminder: {} minutes before", reminder);
                }
                println!(); // Empty line between events
            }
            Ok(())
        }
        "note" => {
            if args.args.is_empty() {
                println!("Usage: note \"<title>\" --content \"<content>\" [--folder \"<folder>\"]");
                return Ok(());
            }
            let content = args.flags.get("--content")
                .and_then(|c| c.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");
            let mut config = notes::NoteConfig::new(&args.args[0], content);
            if let Some(folder) = args.flags.get("--folder") {
                config.folder = folder.as_deref();
            }
            notes::create_note(config)
        },
        "notes" => notes::list_notes(),
        "help" => {
            print_help()
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => {
            println!("Unknown command. Type 'ducktape --help' for available commands.");
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

    // Create new todo item and save using StateManager
    let todo_item = state::TodoItem {
        title: args.args[0].clone(),
        notes: args.flags.get("--notes").and_then(|n| n.clone()),
        lists: args.flags.get("--lists")
                     .map(|l| l.as_deref().unwrap_or("").split(',').map(|s| s.trim().to_owned()).collect())
                     .unwrap_or(vec!["Reminders".to_owned()]),
        reminder_time: args.flags.get("--reminder-time").and_then(|r| r.clone()),
    };

    // Use StateManager to save the todo
    state::StateManager::new()?.add(todo_item)?;
    
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
        if all_day {
            "00:00"
        } else {
            args.args.get(2).map_or("00:00", String::as_str)
        },
    );

    config.all_day = all_day;

    // Set calendars if provided
    if !all_day && args.args.len() > 3 || all_day && args.args.len() > 2 {
        let calendar_index = if all_day { 2 } else { 3 };
        config.calendars = args.args[calendar_index..]
            .iter()
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

fn print_help() -> Result<()> {
    println!("DuckTape - Your AI-Powered Command Line Productivity Duck 🦆");
    println!("\nDescription:");
    println!("  A unified CLI for Apple Calendar, Reminders, and Notes with natural language support");
    println!("  Just type what you want to do - DuckTape's AI will understand!");
    println!("\nNatural Language Examples:");
    println!("  \"schedule a meeting with John tomorrow at 2pm\"");
    println!("  \"remind me to buy groceries next Monday morning\"");
    println!("  \"take notes about the project meeting\"");
    println!("  \"add a todo about calling the bank\"");
    println!("\nOr use traditional commands:");
    println!("  ducktape [command] [options]");
    println!("  ducktape --help | -h");
    println!("\nCommand Groups:");
    println!("  Calendar:");
    println!("    ducktape calendar \"<title>\" <date> <time> [calendar-name...] - Create event");
    println!("    ducktape calendars - List available calendars");
    println!("    ducktape list-events - Show all calendar events");
    println!("\n  Todo & Reminders:");
    println!("    ducktape todo \"<title>\" - Create a todo item");
    println!("    ducktape list-todos - List all stored todos");
    println!("\n  Notes:");
    println!("    ducktape note \"<title>\" --content \"<content>\" [--folder \"<folder>\"]");
    println!("    ducktape notes - List all notes");
    println!("\n  Utility:");
    println!("    ducktape search <path> <pattern> - Search for files");
    println!("    ducktape calendar-props - List available calendar properties");
    println!("\nOptions by Command Type:");
    println!("  Calendar Options:");
    println!("    --all-day                  Create an all-day event");
    println!("    --location \"<location>\"    Set event location");
    println!("    --description \"<desc>\"     Set event description");
    println!("    --email \"<email>\"         Add attendee");
    println!("    --reminder <minutes>       Set reminder (minutes before event)");
    println!("\n  Todo Options:");
    println!("    --notes \"<notes>\"         Add notes to the todo");
    println!("    --lists \"<list1,list2>\"   Add to specific lists");
    println!("    --reminder-time \"YYYY-MM-DD HH:MM\"  Set reminder time");
    println!("\n  Note Options:");
    println!("    --content \"<content>\"     Set note content");
    println!("    --folder \"<folder>\"       Specify note folder");
    println!("\nGeneral Commands:");
    println!("  ducktape --help (or -h) - Show this help");
    println!("  ducktape exit - Exit the application");
    println!("\nAI Features:");
    println!("  - Natural language command processing");
    println!("  - Smart date/time understanding (\"tomorrow\", \"next Monday\")");
    println!("  - Context-aware command generation");
    println!("  - Automatic calendar/list selection");
    println!("\nEnvironment Setup:");
    println!("  Export your OpenAI API key:");
    println!("  export OPENAI_API_KEY='your-api-key-here'");
    println!("\nState Files:");
    println!("  ~/.ducktape/todos.json - Todo items");
    println!("  ~/.ducktape/events.json - Calendar events");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_command_args_parse_basic() {
        let input = "ducktape calendar \"Test Event\" 2024-02-21 14:30";
        let args = CommandArgs::parse(input).unwrap();
        assert_eq!(args.command, "calendar");
        assert_eq!(args.args.len(), 3);
        assert_eq!(args.args[0], "Test Event");
        assert_eq!(args.flags.len(), 0);
    }

    #[test]
    fn test_command_args_parse_with_flags() {
        let input = "ducktape calendar \"Test Event\" 2024-02-21 --all-day --location \"Test Location\"";
        let args = CommandArgs::parse(input).unwrap();
        assert_eq!(args.command, "calendar");
        assert_eq!(args.args[0], "Test Event");
        assert!(args.flags.contains_key("--all-day"));
        assert_eq!(
            args.flags.get("--location").unwrap().as_ref().unwrap(),
            "Test Location"
        );
    }

    #[test]
    fn test_command_args_parse_empty() {
        let input = "";
        let result = CommandArgs::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_args_parse_quoted_strings() {
        let input = r#"ducktape calendar "Meeting with \"quotes\"" 2024-02-21"#;
        let args = CommandArgs::parse(input).unwrap();
        assert_eq!(
            args.args[0], r#"Meeting with "quotes""#,
            "\nExpected: Meeting with \"quotes\"\nGot: {}",
            args.args[0]
        );

        // Add more test cases
        let input2 = r#"ducktape calendar "Meeting \"quoted\" text" 2024-02-21"#;
        let args2 = CommandArgs::parse(input2).unwrap();
        assert_eq!(args2.args[0], r#"Meeting "quoted" text"#);
    }
}
