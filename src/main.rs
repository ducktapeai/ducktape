mod file_search;
mod calendar;

use anyhow::Result;
use env_logger::Env;
use log::{error, info};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

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

        for c in input.chars() {
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
        "help" => {
            println!("Available commands:");
            println!("  search <path> <pattern> - Search for files");
            println!("  calendar \"<title>\" <date> <time> [calendar-name] - Create calendar event");
            println!("  calendars - List available calendars");
            println!("  calendar-props - List available calendar event properties");
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

fn handle_calendar_command(args: CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Usage: calendar \"<title>\" <date> [time] [calendar-name] [--location \"<location>\"] [--description \"<description>\"] [--email \"<email>\"] [--all-day]");
        return Ok(());
    }

    let all_day = args.flags.contains_key("--all-day");
    let mut config = calendar::EventConfig::new(
        &args.args[0],
        &args.args[1],
        if all_day { "00:00" } else { args.args.get(2).map_or("00:00", String::as_str) }
    );

    config.all_day = all_day;
    
    // Set calendar if provided
    if !all_day && args.args.len() > 3 || all_day && args.args.len() > 2 {
        config.calendar = Some(&args.args[if all_day { 2 } else { 3 }]);
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

    calendar::create_event(config)
}
