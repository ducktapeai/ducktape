use crate::commands::CommandArgs;
use crate::commands::CommandExecutor;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

pub struct HelpCommand;

impl CommandExecutor for HelpCommand {
    fn execute(&self, _args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move { print_help() })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "help"
    }
}

pub fn print_help() -> Result<()> {
    println!("DuckTape - Your AI-Powered Command Line Productivity Duck 🦆");
    println!("\nUsage:");
    println!("  1. Natural Language: Just type what you want to do");
    println!("  2. Command Mode: ducktape <command> [options]");

    println!("\nNatural Language Examples:");
    println!("  \"create a meeting with John tomorrow at 2pm\"");
    println!("  \"add a todo to buy groceries next Monday\"");
    println!("  \"make a note about the project requirements\"");
    println!("  \"schedule kids dentist appointment on March 15th at 10am\"");
    println!("  \"create weekly team meeting every Tuesday at 10am\"");

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
    println!("\nRecurrence Options:");
    println!("  --repeat <daily|weekly|monthly|yearly>   Set recurrence frequency");
    println!("  --recurring <daily|weekly|monthly|yearly> Alternative to --repeat");
    println!("  --interval <number>                      Set interval (e.g., every 2 weeks)");
    println!("  --until <YYYY-MM-DD>                     Set end date for recurrence");
    println!("  --count <number>                         Set number of occurrences");
    println!("  --days <0,1,2...>                        Set days of week (0=Sun, 1=Mon, etc.)");

    println!("\nContact Groups:");
    println!("  ducktape contacts                       List all contact groups");
    println!("  ducktape contacts add <id> <name> <contact1,contact2,...> [description]");
    println!("  ducktape contacts show <id>             Show details for a contact group");
    println!("  ducktape contacts remove <id>           Remove a contact group");
    println!("  ducktape calendar create \"Meeting\" 2023-06-15 14:00 15:00 --group \"team\"");
    println!("  (Use --group with calendar create to add contacts from a group)");

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

    println!("\nEvent Search:");
    println!("  ducktape search-events \"<query>\" [--calendar \"<calendar>\"]");
    println!("  Example: ducktape search-events \"Lakers basketball\" --calendar \"Sports\"");

    println!("\nApplication Control:");
    println!("  exit, quit                      Exit the application");
    println!("  Ctrl+C                          Interrupt current operation");
    println!("  Ctrl+D                          Exit the application");

    println!("\nTips:");
    println!("  - Dates can be in YYYY-MM-DD format");
    println!("  - Times should be in 24-hour format (HH:MM)");
    println!("  - Use quotes around titles and text with spaces");
    println!("  - Recurring events: ducktape calendar create \"Weekly Meeting\" 2024-05-01 10:00 11:00 --repeat weekly");
    println!("  - Contact groups save time by letting you quickly add the same contacts to multiple events");

    Ok(())
}
