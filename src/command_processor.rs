use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

/// Command line arguments structure
#[derive(Debug, Clone)]
pub struct CommandArgs {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, Option<String>>,
}

impl CommandArgs {
    pub fn parse(input: &str) -> Result<Self> {
        // Normalize input by replacing non-breaking spaces and multiple spaces with a single space
        let normalized_input =
            input.replace('\u{a0}', " ").split_whitespace().collect::<Vec<_>>().join(" ");

        debug!("Normalized input: {}", normalized_input);

        // Handle exit commands
        if normalized_input.eq_ignore_ascii_case("exit")
            || normalized_input.eq_ignore_ascii_case("quit")
            || normalized_input.eq_ignore_ascii_case("ducktape exit")
            || normalized_input.eq_ignore_ascii_case("ducktape quit")
        {
            return Ok(CommandArgs {
                command: "exit".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        // Special case for help commands
        if normalized_input.eq_ignore_ascii_case("help")
            || normalized_input.eq_ignore_ascii_case("ducktape help")
            || normalized_input.eq_ignore_ascii_case("ducktape --help")
            || normalized_input.eq_ignore_ascii_case("ducktape -h")
            || normalized_input.eq_ignore_ascii_case("--h")
            || normalized_input.eq_ignore_ascii_case("-h")
            || normalized_input.eq_ignore_ascii_case("ducktape --h")
        {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        // Special case for version commands
        if normalized_input.eq_ignore_ascii_case("version")
            || normalized_input.eq_ignore_ascii_case("ducktape version")
            || normalized_input.eq_ignore_ascii_case("ducktape --version")
            || normalized_input.eq_ignore_ascii_case("ducktape -v")
            || normalized_input.eq_ignore_ascii_case("--version")
            || normalized_input.eq_ignore_ascii_case("-v")
        {
            return Ok(CommandArgs {
                command: "version".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        // Special case for calendars command
        if normalized_input.eq_ignore_ascii_case("calendars")
            || normalized_input.eq_ignore_ascii_case("ducktape calendars")
        {
            return Ok(CommandArgs {
                command: "calendars".to_string(),
                args: vec![],
                flags: HashMap::new(),
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

        if parts.is_empty() {
            return Err(anyhow!("No command provided"));
        }

        debug!("Parsed parts after normalization: {:?}", parts);

        // Special case for help command
        if parts.len() == 1
            && (parts[0].eq_ignore_ascii_case("--help") || parts[0].eq_ignore_ascii_case("-h"))
        {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        // Check for and remove "ducktape" prefix, being more lenient with case and whitespace
        let first_part = parts[0].trim();
        if !first_part.eq_ignore_ascii_case("ducktape") {
            // If the first word is a valid command on its own (like "calendars", "calendar", etc.)
            // then assume we're missing the ducktape prefix and proceed
            if first_part.eq_ignore_ascii_case("calendars")
                || first_part.eq_ignore_ascii_case("calendar")
                || first_part.eq_ignore_ascii_case("todo")
                || first_part.eq_ignore_ascii_case("note")
                || first_part.eq_ignore_ascii_case("notes")
                || first_part.eq_ignore_ascii_case("config")
                || first_part.eq_ignore_ascii_case("version")
                || first_part.eq_ignore_ascii_case("help")
            {
                // Don't remove first part, just proceed
                debug!("First part '{}' is a valid command, keeping it", first_part);
            } else {
                debug!("First part '{}' does not match 'ducktape'", first_part);
                return Err(anyhow!("Commands must start with 'ducktape'"));
            }
        } else {
            // Remove "ducktape" prefix
            parts.remove(0);
        }

        if parts.is_empty() {
            return Err(anyhow!("No command provided after 'ducktape'"));
        }

        let command = parts.remove(0).to_lowercase();
        let mut args = Vec::new();
        let mut flags = HashMap::new();
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

        Ok(CommandArgs { command, args, flags })
    }
}

// Command handler trait for handling commands
pub trait CommandHandler: Debug + Send + Sync {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>>;
    fn can_handle(&self, command: &str) -> bool;
}

// Calendar handler
#[derive(Debug)]
pub struct CalendarHandler;

impl CommandHandler for CalendarHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.get(0).map(|s| s.as_str()) {
                Some("create") => {
                    if args.args.len() < 5 {
                        println!("Not enough arguments for calendar create command");
                        println!(
                            "Usage: ducktape calendar create <title> <date> <start_time> <end_time> [calendar]"
                        );
                        return Ok(());
                    }

                    let title = &args.args[1];
                    let date = &args.args[2];
                    let start_time = &args.args[3];
                    let end_time = &args.args[4];
                    let calendar = args.args.get(5).cloned();

                    // Extract location and description from flags if present
                    let location = args.flags.get("--location").cloned().flatten();
                    let description = args.flags.get("--notes").cloned().flatten();
                    let emails = args.flags.get("--attendees").cloned().flatten();
                    let contacts = args.flags.get("--contacts").cloned().flatten();

                    // Create event config and pass to calendar module
                    let mut config = crate::calendar::EventConfig::new(title, date, start_time);
                    config.end_time = Some(end_time.clone());

                    if let Some(cal) = calendar {
                        config.calendars = vec![cal];
                    }

                    config.location = location;
                    config.description = description;

                    // Check for --zoom flag and set create_zoom_meeting property
                    if args.flags.contains_key("--zoom") {
                        info!("Zoom flag detected, creating event with Zoom meeting");
                        config.create_zoom_meeting = true;
                    }

                    if let Some(attendees_str) = emails {
                        config.emails = attendees_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|email| crate::calendar::validate_email(email))
                            .collect();
                    }

                    // If contacts are specified, use create_event_with_contacts
                    if let Some(contacts_str) = contacts {
                        // Split contacts string by commas and convert to string slices
                        let contact_names: Vec<&str> = contacts_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();

                        if !contact_names.is_empty() {
                            info!("Creating event with contacts: {:?}", contact_names);
                            return crate::calendar::create_event_with_contacts(
                                config,
                                &contact_names,
                            )
                            .await;
                        }
                    }

                    crate::calendar::create_event(config).await
                }
                Some("list") => crate::calendar::list_calendars().await,
                Some("show") => {
                    // TODO: Implement show calendar functionality
                    println!("Show calendar functionality is not implemented yet.");
                    Ok(())
                }
                _ => {
                    println!("Unknown calendar command. Available commands: create, list, show");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "calendar" || command == "calendars"
    }
}

// Todo handler
#[derive(Debug)]
pub struct TodoHandler;

impl CommandHandler for TodoHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.get(0).map(|s| s.as_str()) {
                Some("create") | Some("add") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for todo create command");
                        println!("Usage: ducktape todo create <title> [list1] [list2] ...");
                        return Ok(());
                    }

                    let title = &args.args[1];

                    // Create a new TodoConfig with the title
                    let mut config = crate::todo::TodoConfig::new(title);

                    // Set lists if provided in arguments (args[2] and beyond are list names)
                    if args.args.len() > 2 {
                        let list_names: Vec<&str> =
                            args.args[2..].iter().map(|s| s.as_str()).collect();
                        config.lists = list_names;
                    }

                    // Set reminder time if provided via --remind flag
                    if let Some(Some(reminder_time)) = args.flags.get("--remind") {
                        config.reminder_time = Some(reminder_time);
                    }

                    // Set notes if provided via --notes flag
                    if let Some(Some(notes)) = args.flags.get("--notes") {
                        config.notes = Some(notes.clone());
                    }

                    // Note: create_todo is synchronous, so don't await it
                    match crate::todo::create_todo(config) {
                        Ok(_) => {
                            println!("Todo '{}' created successfully", title);
                            Ok(())
                        }
                        Err(e) => {
                            println!("Failed to create todo: {}", e);
                            Err(anyhow!("Failed to create todo: {}", e))
                        }
                    }
                }
                Some("list") => {
                    // Implementation for listing todos would go here
                    println!("Listing todos... (not implemented yet)");
                    Ok(())
                }
                Some("delete") => {
                    // Implementation for deleting todos would go here
                    println!("Deleting todo... (not implemented yet)");
                    Ok(())
                }
                _ => {
                    println!("Unknown todo command. Available commands: create/add, list, delete");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "todo" || command == "todos"
    }
}

// Notes handler
#[derive(Debug)]
pub struct NotesHandler;

impl CommandHandler for NotesHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.get(0).map(|s| s.as_str()) {
                Some("create") | Some("add") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for note create command");
                        println!("Usage: ducktape note create <title> [content] [folder]");
                        return Ok(());
                    }

                    let title = &args.args[1];
                    let content = args.args.get(2).cloned().unwrap_or_default();
                    let folder = args.args.get(3).cloned();

                    // Create note config and pass to notes module
                    let config = crate::notes::NoteConfig {
                        title,
                        content: &content,
                        folder: folder.as_deref(),
                    };

                    crate::notes::create_note(config)
                }
                Some("list") => {
                    // TODO: Implement list notes functionality
                    println!("List notes functionality is not implemented yet.");
                    Ok(())
                }
                _ => {
                    println!("Unknown notes command. Available commands: create/add, list");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "note" || command == "notes"
    }
}

// Config handler
#[derive(Debug)]
pub struct ConfigHandler;

impl CommandHandler for ConfigHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.get(0).map(|s| s.as_str()) {
                Some("set") => {
                    if args.args.len() < 3 {
                        println!("Not enough arguments for config set command");
                        println!("Usage: ducktape config set <key> <value>");
                        return Ok(());
                    }

                    let key = &args.args[1];
                    let value = &args.args[2];

                    // Load config
                    let mut config = crate::config::Config::load()?;

                    // Update config based on key
                    match key.as_str() {
                        "calendar.default" => {
                            config.calendar.default_calendar = Some(value.clone());
                        }
                        "calendar.reminder" => {
                            if let Ok(minutes) = value.parse::<i32>() {
                                config.calendar.default_reminder_minutes = Some(minutes);
                            } else {
                                println!("Invalid reminder minutes value: {}", value);
                                return Ok(());
                            }
                        }
                        "calendar.duration" => {
                            if let Ok(minutes) = value.parse::<i32>() {
                                config.calendar.default_duration_minutes = Some(minutes);
                            } else {
                                println!("Invalid duration minutes value: {}", value);
                                return Ok(());
                            }
                        }
                        "todo.default_list" => {
                            config.todo.default_list = Some(value.clone());
                        }
                        "notes.default_folder" => {
                            config.notes.default_folder = Some(value.clone());
                        }
                        "language_model.provider" => match value.to_lowercase().as_str() {
                            "openai" => {
                                config.language_model.provider = crate::config::LLMProvider::OpenAI;
                            }
                            "grok" => {
                                config.language_model.provider = crate::config::LLMProvider::Grok;
                            }
                            "deepseek" => {
                                config.language_model.provider =
                                    crate::config::LLMProvider::DeepSeek;
                            }
                            _ => {
                                println!("Invalid language model provider: {}", value);
                                println!("Valid options are: openai, grok, deepseek");
                                return Ok(());
                            }
                        },
                        _ => {
                            println!("Unknown config key: {}", key);
                            return Ok(());
                        }
                    }

                    // Save updated config
                    config.save()?;
                    println!("Config updated: {} = {}", key, value);
                    Ok(())
                }
                Some("get") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for config get command");
                        println!("Usage: ducktape config get <key>");
                        return Ok(());
                    }

                    let key = &args.args[1];
                    let config = crate::config::Config::load()?;

                    // Get config value based on key
                    match key.as_str() {
                        "calendar.default" => {
                            println!(
                                "calendar.default = {}",
                                config
                                    .calendar
                                    .default_calendar
                                    .unwrap_or_else(|| "Not set".to_string())
                            );
                        }
                        "calendar.reminder" => {
                            println!(
                                "calendar.reminder = {}",
                                config
                                    .calendar
                                    .default_reminder_minutes
                                    .map_or_else(|| "Not set".to_string(), |m| m.to_string())
                            );
                        }
                        "calendar.duration" => {
                            println!(
                                "calendar.duration = {}",
                                config
                                    .calendar
                                    .default_duration_minutes
                                    .map_or_else(|| "Not set".to_string(), |m| m.to_string())
                            );
                        }
                        "todo.default_list" => {
                            println!(
                                "todo.default_list = {}",
                                config.todo.default_list.unwrap_or_else(|| "Not set".to_string())
                            );
                        }
                        "notes.default_folder" => {
                            println!(
                                "notes.default_folder = {}",
                                config
                                    .notes
                                    .default_folder
                                    .unwrap_or_else(|| "Not set".to_string())
                            );
                        }
                        "language_model.provider" => {
                            let provider = match config.language_model.provider {
                                crate::config::LLMProvider::OpenAI => "openai",
                                crate::config::LLMProvider::Grok => "grok",
                                crate::config::LLMProvider::DeepSeek => "deepseek",
                            };
                            println!("language_model.provider = {}", provider);
                        }
                        "all" => {
                            println!("Current Configuration:");
                            println!("======================");
                            println!(
                                "calendar.default = {}",
                                config
                                    .calendar
                                    .default_calendar
                                    .unwrap_or_else(|| "Not set".to_string())
                            );
                            println!(
                                "calendar.reminder = {}",
                                config
                                    .calendar
                                    .default_reminder_minutes
                                    .map_or_else(|| "Not set".to_string(), |m| m.to_string())
                            );
                            println!(
                                "calendar.duration = {}",
                                config
                                    .calendar
                                    .default_duration_minutes
                                    .map_or_else(|| "Not set".to_string(), |m| m.to_string())
                            );
                            println!(
                                "todo.default_list = {}",
                                config.todo.default_list.unwrap_or_else(|| "Not set".to_string())
                            );
                            println!(
                                "notes.default_folder = {}",
                                config
                                    .notes
                                    .default_folder
                                    .unwrap_or_else(|| "Not set".to_string())
                            );
                            let provider = match config.language_model.provider {
                                crate::config::LLMProvider::OpenAI => "openai",
                                crate::config::LLMProvider::Grok => "grok",
                                crate::config::LLMProvider::DeepSeek => "deepseek",
                            };
                            println!("language_model.provider = {}", provider);
                        }
                        _ => {
                            println!("Unknown config key: {}", key);
                        }
                    }
                    Ok(())
                }
                _ => {
                    println!("Unknown config command. Available commands: set, get");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "config"
    }
}

// Utilities handler
#[derive(Debug)]
pub struct UtilitiesHandler;

impl CommandHandler for UtilitiesHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.get(0).map(|s| s.as_str()) {
                Some("date") => {
                    println!("Current date: {}", chrono::Local::now().format("%Y-%m-%d"));
                    Ok(())
                }
                Some("time") => {
                    println!("Current time: {}", chrono::Local::now().format("%H:%M:%S"));
                    Ok(())
                }
                Some("datetime") => {
                    println!(
                        "Current date and time: {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                    );
                    Ok(())
                }
                _ => {
                    println!("Unknown utility command. Available commands: date, time, datetime");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "utility" || command == "utils"
    }
}

// Contact groups handler
#[derive(Debug)]
pub struct ContactGroupsHandler;

impl CommandHandler for ContactGroupsHandler {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.args.get(0).map(|s| s.as_str()) {
                Some("create") => {
                    if args.args.len() < 3 {
                        println!("Not enough arguments for contact group create command");
                        println!("Usage: ducktape contacts create <group_name> <emails...>");
                        return Ok(());
                    }

                    let group_name = &args.args[1];
                    let emails: Vec<String> = args.args.iter().skip(2).cloned().collect();

                    if emails.is_empty() {
                        println!("No email addresses provided");
                        return Ok(());
                    }

                    // Validate email addresses
                    for email in &emails {
                        if !crate::calendar::validate_email(email) {
                            println!("Invalid email address: {}", email);
                            return Ok(());
                        }
                    }

                    // Create contact group
                    let result = crate::contact_groups::create_group(group_name, &emails);
                    match result {
                        Ok(_) => {
                            println!(
                                "Created contact group '{}' with {} members",
                                group_name,
                                emails.len()
                            );
                        }
                        Err(e) => {
                            println!("Failed to create contact group: {}", e);
                        }
                    }
                    Ok(())
                }
                Some("list") => {
                    match crate::contact_groups::list_groups() {
                        Ok(groups) => {
                            if groups.is_empty() {
                                println!("No contact groups found");
                            } else {
                                println!("Available contact groups:");
                                for group in groups {
                                    println!("  - {}", group);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to list contact groups: {}", e);
                        }
                    }
                    Ok(())
                }
                Some("show") => {
                    if args.args.len() < 2 {
                        println!("Not enough arguments for contact group show command");
                        println!("Usage: ducktape contacts show <group_name>");
                        return Ok(());
                    }

                    let group_name = &args.args[1];
                    match crate::contact_groups::get_group(group_name) {
                        Ok(Some(members)) => {
                            println!("Members of contact group '{}':", group_name);
                            for member in members {
                                println!("  - {}", member);
                            }
                        }
                        Ok(None) => {
                            println!("Contact group '{}' not found", group_name);
                        }
                        Err(e) => {
                            println!("Failed to show contact group: {}", e);
                        }
                    }
                    Ok(())
                }
                _ => {
                    println!("Unknown contacts command. Available commands: create, list, show");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "contacts" || command == "contact"
    }
}

// Version handler
#[derive(Debug)]
pub struct VersionHandler;

impl CommandHandler for VersionHandler {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("DuckTape v{}", VERSION);
            println!(
                "A tool for interacting with Apple Calendar, Notes, and Reminders via the command line."
            );
            println!("© 2024-2025 DuckTape Team");
            Ok(())
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "version" || command == "--version" || command == "-v"
    }
}

// Help handler
#[derive(Debug)]
pub struct HelpHandler;

impl CommandHandler for HelpHandler {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            print_help()?;
            Ok(())
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "help" || command == "--help" || command == "-h"
    }
}

// Print help information
pub fn print_help() -> Result<()> {
    println!("DuckTape - A tool for interacting with Apple Calendar, Notes, and Reminders");
    println!();
    println!("USAGE:");
    println!("  ducktape [COMMAND] [SUBCOMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  calendar  Manage calendar events");
    println!("  todo      Manage todo items");
    println!("  notes     Manage notes");
    println!("  config    Manage configuration");
    println!("  contacts  Manage contact groups");
    println!("  utils     Utility commands");
    println!("  help      Show this help message");
    println!("  version   Show version information");
    println!();
    println!("For more information on a specific command, run:");
    println!("  ducktape [COMMAND] --help");
    println!();
    println!("EXAMPLES:");
    println!("  ducktape calendar create \"Meeting with Team\" 2025-04-15 10:00 11:00");
    println!("  ducktape todo add \"Buy groceries\" tomorrow 18:00");
    println!("  ducktape notes create \"Meeting Notes\" \"Points discussed in the meeting\"");
    println!("  ducktape config set calendar.default \"Personal\"");
    Ok(())
}

// Command processor that manages handlers and executes commands
#[derive(Debug)]
pub struct CommandProcessor {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn CommandHandler>> = vec![
            Box::new(HelpHandler),
            Box::new(CalendarHandler),
            Box::new(TodoHandler),
            Box::new(NotesHandler),
            Box::new(ConfigHandler),
            Box::new(UtilitiesHandler),
            Box::new(ContactGroupsHandler),
            Box::new(VersionHandler),
        ];

        Self { handlers }
    }

    pub async fn execute(&self, args: CommandArgs) -> Result<()> {
        debug!("Attempting to execute command: {}", args.command);
        let command_name = args.command.clone(); // Clone the command name for logging
        let args_debug = format!("{:?}", args.args); // Format args for debug logging

        for handler in &self.handlers {
            if handler.can_handle(&command_name) {
                info!("Executing command '{}' with arguments: {}", command_name, args_debug);
                match handler.execute(args).await {
                    Ok(()) => {
                        debug!("Command '{}' executed successfully", command_name);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Failed to execute command '{}': {:?}", command_name, e);
                        return Err(e);
                    }
                }
            }
        }

        warn!("Unrecognized command: {}", command_name);
        println!("Unrecognized command. Type 'help' for a list of available commands.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_args_parse() {
        // Test basic command
        let args =
            CommandArgs::parse("ducktape calendar create \"Test Event\" 2023-01-01 09:00 10:00")
                .unwrap();
        assert_eq!(args.command, "calendar");
        assert_eq!(args.args, vec!["create", "Test Event", "2023-01-01", "09:00", "10:00"]);
        assert!(args.flags.is_empty());

        // Test with flags
        let args = CommandArgs::parse(
            "ducktape calendar create \"Test Event\" 2023-01-01 09:00 10:00 --location \"Office\"",
        )
        .unwrap();
        assert_eq!(args.command, "calendar");
        assert_eq!(args.args, vec!["create", "Test Event", "2023-01-01", "09:00", "10:00"]);
        assert_eq!(args.flags.get("--location").unwrap().as_ref().unwrap(), "Office");

        // Test help command
        let args = CommandArgs::parse("ducktape help").unwrap();
        assert_eq!(args.command, "help");
        assert!(args.args.is_empty());
        assert!(args.flags.is_empty());

        // Test version command
        let args = CommandArgs::parse("ducktape version").unwrap();
        assert_eq!(args.command, "version");
        assert!(args.args.is_empty());
        assert!(args.flags.is_empty());
    }
}
