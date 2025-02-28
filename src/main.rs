mod calendar;
mod config;
mod deepseek_parser;
mod file_search;
mod grok_parser;
mod notes;
mod openai_parser;
mod state;
mod todo;
mod event_search; // Import the event_search module

use anyhow::Result;
use config::{Config, LLMProvider};
use env_logger::Env;
use log::{debug, error, info};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::future::Future;
use std::pin::Pin;
use tokio;

/// Command line arguments structure
#[derive(Debug)]
struct CommandArgs {
    command: String,
    args: Vec<String>,
    flags: std::collections::HashMap<String, Option<String>>,
}

impl CommandArgs {
    fn parse(input: &str) -> Result<Self> {
        // Normalize input by replacing non-breaking spaces and multiple spaces with a single space
        let normalized_input = input
            .replace('\u{a0}', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        debug!("Normalized input: {}", normalized_input);

        // Special case for help commands
        if normalized_input.eq_ignore_ascii_case("help") || 
           normalized_input.eq_ignore_ascii_case("ducktape help") ||
           normalized_input.eq_ignore_ascii_case("ducktape --help") ||
           normalized_input.eq_ignore_ascii_case("ducktape -h") ||
           normalized_input.eq_ignore_ascii_case("ducktape --h") {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: std::collections::HashMap::new(),
            });
        }

        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = normalized_input.chars().peekable();
        let mut escaped = false;

        while let Some(c) = chars.next() {
            match c {
                '\\' if !escaped => {
                    escaped = true;
                }
                '"' if !escaped => {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                ' ' if !in_quotes && !escaped => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    if escaped && c != '"' {
                        current.push('\\');
                    }
                    current.push(c);
                    escaped = false;
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {  // Fix isEmpty to is_empty
            return Err(anyhow::anyhow!("No command provided"));
        }

        debug!("Parsed parts after normalization: {:?}", parts);

        // Special case for help command
        if parts.len() == 1 && (parts[0].eq_ignore_ascii_case("--help") || parts[0].eq_ignore_ascii_case("-h")) {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: std::collections::HashMap::new(),
            });
        }

        // Check for and remove "ducktape" prefix, being more lenient with case and whitespace
        let first_part = parts[0].trim();
        if !first_part.eq_ignore_ascii_case("ducktape") {
            debug!("First part '{}' does not match 'ducktape'", first_part);
            return Err(anyhow::anyhow!("Commands must start with 'ducktape'"));
        }
        parts.remove(0); // Remove "ducktape"

        if parts.is_empty() {
            return Err(anyhow::anyhow!("No command provided after 'ducktape'"));
        }

        let command = parts.remove(0).to_lowercase();
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

        debug!("Parsed command: {:?}, args: {:?}, flags: {:?}", command, args, flags);

        Ok(CommandArgs {
            command,
            args,
            flags,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    debug!("Loaded configuration: {:?}", config);

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
                if let Err(err) = process_command(&line).await {  // Add .await here
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

// Fix recursive async function with boxing
fn process_command(command: &str) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
    Box::pin(async move {
        // Case-insensitive check for ducktape prefix
        if !command.trim().to_lowercase().starts_with("ducktape") {
            let config = Config::load()?;
            
            let response = match config.language_model.provider {
                LLMProvider::OpenAI => crate::openai_parser::parse_natural_language(command).await,
                LLMProvider::Grok => crate::grok_parser::parse_natural_language(command).await,
                LLMProvider::DeepSeek => crate::deepseek_parser::parse_natural_language(command).await,
            };

            match response {
                Ok(parsed_command) => {
                    if parsed_command.to_lowercase().contains("please provide") {
                        println!("{}", parsed_command);
                        let mut rl = rustyline::DefaultEditor::new()?;
                        let additional = rl.readline(">> Additional details: ")?;
                        let combined = format!("{} {}", command, additional);
                        
                        let new_response = match config.language_model.provider {
                            LLMProvider::OpenAI => crate::openai_parser::parse_natural_language(&combined).await?,
                            LLMProvider::Grok => crate::grok_parser::parse_natural_language(&combined).await?,
                            LLMProvider::DeepSeek => crate::deepseek_parser::parse_natural_language(&combined).await?,
                        };
                        
                        println!("{}", new_response);
                        process_command(&new_response).await
                    } else {
                        println!("{}", parsed_command);
                        process_command(&parsed_command).await
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            let args = CommandArgs::parse(command)?;
            match args.command.as_str() {
                "config" => {
                    if args.args.is_empty() {
                        println!("Usage: ducktape config <setting> <value>");
                        println!("Available settings:");
                        println!("  llm - Set the language model provider");
                        println!("\nExample: ducktape config llm grok");
                        return Ok(());
                    }
                    
                    match args.args[0].as_str() {
                        "show" => {
                            let config = Config::load()?;
                            println!("\nCurrent Configuration:");
                            println!("  Language Model Provider: {:?}", config.language_model.provider);
                            println!("\nCalendar Settings:");
                            println!("  Default Calendar: {}", config.calendar.default_calendar.as_deref().unwrap_or("None"));
                            println!("  Default Reminder: {} minutes", config.calendar.default_reminder_minutes.unwrap_or(15));
                            println!("  Default Duration: {} minutes", config.calendar.default_duration_minutes.unwrap_or(60));
                            println!("\nTodo Settings:");
                            println!("  Default List: {}", config.todo.default_list.as_deref().unwrap_or("None"));
                            println!("  Default Reminder: {}", if config.todo.default_reminder { "Enabled" } else { "Disabled" });
                            println!("\nNotes Settings:");
                            println!("  Default Folder: {}", config.notes.default_folder.as_deref().unwrap_or("None"));
                            return Ok(());
                        },
                        "llm" => {
                            if args.args.len() < 2 {
                                println!("Usage: ducktape config llm <provider>");
                                println!("Available providers: openai, grok, deepseek");
                                println!("\nExample: ducktape config llm grok");
                                return Ok(());
                            }
                            
                            let provider = match args.args[1].to_lowercase().as_str() {
                                "openai" => LLMProvider::OpenAI,
                                "grok" => LLMProvider::Grok,
                                "deepseek" => LLMProvider::DeepSeek,
                                _ => {
                                    println!("Error: Invalid provider '{}'", args.args[1]);
                                    println!("Available providers: openai, grok, deepseek");
                                    println!("\nExample: ducktape config llm grok");
                                    return Ok(());
                                }
                            };
                            
                            let mut config = Config::load()?;
                            config.language_model.provider = provider;
                            config.save()?;
                            println!("Language model provider updated to: {}", args.args[1]);
                            return Ok(());
                        }
                        _ => {
                            println!("Unknown config setting '{}'. Available settings:", args.args[0]);
                            println!("  llm - Set the language model provider");
                            println!("  show - Display current configuration");
                            println!("\nExample: ducktape config llm grok");
                            return Ok(());
                        }
                    }
                }
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
                            "  - {}",    // Fix: Remove extra format parameter
                            event.title  // Add the event title here
                        );
                        println!(
                            "    Time: {}",
                            if event.all_day {
                                "All day".to_string()
                            } else {
                                event.time.clone()
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
                        println!(
                            "Usage: note \"<title>\" --content \"<content>\" [--folder \"<folder>\"]"
                        );
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
                "notes" => notes::list_notes(),
                "delete-event" => {
                    if args.args.len() < 1 {
                        println!("Usage: delete-event \"<title>\"");
                        return Ok(());
                    }
                    calendar::delete_event(
                        &args.args[0],
                        args.args.get(1).map(|s| s.as_str()).unwrap_or(""),
                    )?;
                    // Also remove from state
                    let mut events = state::load_events()?;
                    events.retain(|e| e.title != args.args[0]);
                    state::StateManager::new()?.save(&events)?;
                    Ok(())
                }
                "cleanup" => {
                    println!("Cleaning up old items...");
                    state::StateManager::new()?.cleanup_old_items()?;
                    println!("Compacting storage files...");
                    state::StateManager::new()?.vacuum()?;
                    println!("Cleanup complete!");
                    Ok(())
                }
                "search-events" | "event-search" => {
                    if args.args.is_empty() {
                        println!("Usage: ducktape search-events <query> [--calendar <calendar>]");
                        println!("Example: ducktape search-events \"Springboks rugby\"");
                        return Ok(());
                    }
                    
                    let query = args.args.join(" ");
                    let calendar = args
                        .flags
                        .get("--calendar")
                        .and_then(|c| c.as_ref())
                        .map(|s| s.as_str());
                    
                    println!("Searching for events: {}", query);
                    return event_search::search_events(&query, calendar).await;
                },
                "help" => print_help(),
                "exit" => {
                    std::process::exit(0);
                }
                _ => {
                    println!("Unknown command. Type 'ducktape --help' for available commands.");
                    return Ok(());
                }
            }
        }
    })
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
        lists: args
            .flags
            .get("--lists")
            .map(|l| {
                l.as_deref()
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .collect()
            })
            .unwrap_or(vec!["Reminders".to_owned()]),
        reminder_time: args.flags.get("--reminder-time").and_then(|r| r.clone()),
    };

    // Use StateManager to save the todo
    state::StateManager::new()?.add(todo_item)?;

    Ok(())
}

fn handle_calendar_command(args: CommandArgs) -> Result<()> {
    match args.args.get(0).map(|s| s.to_lowercase()).as_deref() {
        Some("create") => {
            // Require at least: "create" + title + date + start_time + end_time = 5 args
            if args.args.len() < 5 {
                println!("Usage: ducktape calendar create \"<title>\" <date> <start_time> <end_time> [calendar]");
                println!("Example: ducktape calendar create \"Meeting\" 2024-02-07 09:00 10:00 \"Work\"");
                return Ok(());
            }

            let title = args.args[1].trim_matches('"').to_string();
            let date = args.args[2].trim().to_string();
            let start_time = args.args[3].trim().to_string();
            let end_time = args.args[4].trim().to_string();
            
            let mut config = calendar::EventConfig::new(&title, &date, &start_time);
            config.end_time = Some(&end_time);
            
            // Set calendar if provided, trimming any quotes
            if let Some(calendar) = args.args.get(5) {
                let calendar = calendar.trim_matches('"');
                debug!("Using calendar: {}", calendar);
                config.calendars = vec![calendar];
            }
            
            // Handle email addresses - split on commas and trim whitespace and quotes
            if let Some(emails) = args.flags.get("--email") {
                if let Some(email_str) = emails {
                    let emails: Vec<String> = email_str
                        .trim_matches('"')
                        .split(',')
                        .map(|e| e.trim().to_string())
                        .filter(|e| !e.is_empty())
                        .collect();
                    debug!("Parsed email addresses: {:?}", emails);
                    config.emails = emails;
                }
            }

            // Handle contact names if provided
            if let Some(contacts) = args.flags.get("--contacts") {
                if let Some(contact_str) = contacts {
                    let contact_names: Vec<&str> = contact_str
                        .trim_matches('"')
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();
                    
                    if !contact_names.is_empty() {
                        debug!("Looking up contacts: {:?}", contact_names);
                        return calendar::create_event_with_contacts(config, &contact_names);
                    }
                }
            }
            
            calendar::create_event(config)
        }
        Some("delete") => {
            if args.args.len() < 2 {
                println!("Usage: calendar delete <title>");
                return Ok(());
            }
            let title = &args.args[1];
            calendar::delete_event(title, args.args.get(2).map(|s| s.as_str()).unwrap_or(""))?;
            let mut events = state::load_events()?;
            events.retain(|e| e.title != args.args[1]);
            state::StateManager::new()?.save(&events)?;
            Ok(())
        }
        _ => {
            println!("Unknown calendar command. Use 'calendar create' or 'calendar delete'.");
            Ok(())
        }
    }
}

fn print_help() -> Result<()> {
    println!("DuckTape - Your AI-Powered Command Line Productivity Duck 🦆");
    println!("\nUsage:");
    println!("  1. Natural Language: Just type what you want to do");
    println!("  2. Command Mode: ducktape <command> [options]");
    
    println!("\nNatural Language Examples:");
    println!("  \"create a meeting with John tomorrow at 2pm\"");
    println!("  \"add a todo to buy groceries next Monday\"");
    println!("  \"make a note about the project requirements\"");
    println!("  \"schedule kids dentist appointment on March 15th at 10am\"");
    
    println!("\nConfiguration:");
    println!("  ducktape config llm <provider>   Switch language model provider");
    println!("  ducktape config show             Show current settings");
    println!("Available Providers:");
    println!("  - openai    (default, requires OPENAI_API_KEY)");
    println!("  - grok      (requires XAI_API_KEY)");
    println!("  - deepseek  (requires DEEPSEEK_API_KEY)");
    
    println!("\nCalendar Commands:");
    println!("  ducktape calendar create \"<title>\" <date> <start> <end> [calendar]");
    println!("  ducktape calendar delete \"<title>\"");
    println!("  ducktape calendars               List available calendars");
    println!("  ducktape calendar-props          Show calendar properties");
    
    println!("\nTodo Commands:");
    println!("  ducktape todo \"<title>\" [--notes \"<notes>\"] [--lists \"list1,list2\"]");
    println!("  ducktape list-todos              Show all todos");
    
    println!("\nNotes Commands:");
    println!("  ducktape note \"<title>\" --content \"<content>\" [--folder \"<folder>\"]");
    println!("  ducktape notes                   List all notes");
    
    println!("\nUtility Commands:");
    println!("  ducktape list-events            Show all calendar events");
    println!("  ducktape cleanup                Remove old items and compact storage");
    println!("  ducktape config show            Display current configuration");
    println!("  ducktape help                   Show this help message");
    
    println!("
Event Search:");
    println!("  ducktape search-events \"<query>\" [--calendar \"<calendar>\"]");
    println!("  Example: ducktape search-events \"Lakers basketball\" --calendar \"Sports\"");
    
    println!("\nTips:");
    println!("  - Dates can be in YYYY-MM-DD format");
    println!("  - Times should be in 24-hour format (HH:MM)");
    println!("  - Use quotes around titles and text with spaces");
    println!("  - Set your preferred calendar with: ducktape config calendar <name>");

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
        let input =
            "ducktape calendar \"Test Event\" 2024-02-21 --all-day --location \"Test Location\"";
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
        let inputs = [
            (
                r#"ducktape calendar "Meeting with \"quotes\"" 2024-02-21"#,
                r#"Meeting with "quotes""#,
            ),
            (
                r#"ducktape calendar "Meeting \"quoted\" text" 2024-02-21"#,
                r#"Meeting "quoted" text"#,
            ),
            (
                r#"ducktape calendar "Simple meeting" 2024-02-21"#,
                "Simple meeting",
            ),
        ];

        for (input, expected) in inputs {
            let args = CommandArgs::parse(input).unwrap();
            assert_eq!(
                args.args[0], expected,
                "\nInput: {}\nExpected: {}\nGot: {}\n",
                input, expected, args.args[0]
            );
        }
    }
}
