use anyhow::Result;
use crate::commands::{CommandArgs, CommandExecutor};
use std::future::Future;
use std::pin::Pin;
use crate::calendar;
use crate::state;
use log::debug;
use std::str::FromStr;
use std::path::Path;

pub struct CalendarCommand;

impl CommandExecutor for CalendarCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.command.as_str() {
                "calendars" => calendar::list_calendars().await,
                "calendar-props" => calendar::list_event_properties().await,
                "list-events" => list_events(),
                "delete-event" => delete_event(args).await,
                "calendar" => {
                    match args.args.get(0).map(|s| s.to_lowercase()).as_deref() {
                        Some("create") => create_calendar_event(args).await,
                        Some("delete") => delete_calendar_event(args).await,
                        Some("set-default") => set_default_calendar(args),
                        Some("import") => import_calendar_events(args).await,
                        _ => {
                            println!("Unknown calendar command. Use 'calendar create', 'calendar delete', 'calendar set-default', or 'calendar import'");
                            Ok(())
                        }
                    }
                },
                _ => {
                    println!("Unknown calendar command");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "calendar" | "calendars" | "calendar-props" | "delete-event" | "list-events")
    }
}

fn list_events() -> Result<()> {
    let events = state::load_events()?;
    println!("Stored Calendar Events:");
    for event in events {
        println!(
            "  - {}",
            event.title
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

async fn delete_event(args: CommandArgs) -> Result<()> {
    if args.args.len() < 1 {
        println!("Usage: delete-event \"<title>\"");
        return Ok(());
    }
    
    // Add await to properly handle the async function
    calendar::delete_event(
        &args.args[0],
        args.args.get(1).map(|s| s.as_str()).unwrap_or(""),
    ).await?;
    
    // Also remove from state
    let mut events = state::load_events()?;
    events.retain(|e| e.title != args.args[0]);
    state::StateManager::new()?.save(&events)?;
    Ok(())
}

async fn create_calendar_event(args: CommandArgs) -> Result<()> {
    // Require at least: "create" + title + date + start_time + end_time = 5 args
    if args.args.len() < 5 {
        println!("Usage: ducktape calendar create \"<title>\" <date> <start_time> <end_time> [calendar]");
        println!("Example: ducktape calendar create \"Meeting\" 2024-02-07 09:00 10:00 \"Work\"");
        println!("\nRecurrence options:");
        println!("  --repeat <daily|weekly|monthly|yearly>   Set recurrence frequency");
        println!("  --recurring <daily|weekly|monthly|yearly> Alternative to --repeat");
        println!("  --interval <number>                      Set interval (e.g., every 2 weeks)");
        println!("  --until <YYYY-MM-DD>                     Set end date for recurrence");
        println!("  --count <number>                         Set number of occurrences");
        println!("  --days <0,1,2...>                        Set days of week (0=Sun, 1=Mon, etc.)");
        println!("\nZoom options:");
        println!("  --zoom                                   Create a Zoom meeting for this event");
        println!("\nContact options:");
        println!("  --contacts \"<name1,name2>\"               Add contacts by name");
        println!("  --group \"<group_id>\"                     Add contacts from a predefined group");
        return Ok(());
    }
    
    // Safely sanitize title input
    let title = sanitize_input(&args.args[1], false);
    
    // Validate date format
    let date = args.args[2].trim();
    if !calendar::validate_date_format(date) {
        println!("Invalid date format. Please use YYYY-MM-DD format.");
        return Ok(());
    }
    
    // Validate time format
    let start_time = args.args[3].trim();
    if !calendar::validate_time_format(start_time) {
        println!("Invalid start time format. Please use HH:MM format (24-hour).");
        return Ok(());
    }
    
    // Fix the compile error with if let Some(end)
    let end_time = args.args[4].trim();
    if !calendar::validate_time_format(end_time) {
        println!("Invalid end time format. Please use HH:MM format (24-hour).");
        return Ok(());
    }
    
    let mut config = calendar::EventConfig::new(&title, date, start_time);
    config.end_time = Some(end_time.to_string());
    
    // Set calendar if provided, trimming any quotes and sanitizing
    if let Some(calendar_name) = args.args.get(5) {
        let sanitized_calendar = sanitize_input(calendar_name, true);
        debug!("Using calendar: {}", sanitized_calendar);
        // Store the calendar name as an owned String
        config.calendars = vec![sanitized_calendar];
    }
    
    // Handle email addresses - split on commas, trim whitespace and quotes, validate format
    if let Some(emails) = args.flags.get("--email") {
        if let Some(email_str) = emails {
            let email_str = sanitize_input(email_str, false);
            let emails: Vec<String> = email_str
                .split(',')
                .map(|e| e.trim().to_string())
                .filter(|e| !e.is_empty() && validate_email_format(e))
                .collect();
                
            if emails.len() > 0 {
                debug!("Parsed valid email addresses: {:?}", emails);
                config.emails = emails;
            } else {
                debug!("No valid email addresses found in input");
            }
        }
    }
    
    // Handle location flag
    if let Some(location) = args.flags.get("--location") {
        if let Some(loc) = location {
            let location = sanitize_input(loc, false);
            config.location = Some(location);
        }
    }
    
    // Handle description/notes flag
    if let Some(description) = args.flags.get("--notes") {
        if let Some(desc) = description {
            config.description = Some(sanitize_input(desc, true));
        }
    }
    
    // Handle reminder flag
    if let Some(reminder) = args.flags.get("--reminder") {
        if let Some(mins) = reminder {
            if let Ok(minutes) = mins.trim().parse::<i32>() {
                // Cap reminder minutes to reasonable values
                if minutes > 0 && minutes < 10080 { // Max 1 week in minutes
                    config.reminder = Some(minutes);
                } else {
                    println!("Warning: Reminder minutes should be between 1 and 10080 (1 week). Using default.");
                }
            }
        }
    }
    
    // Handle timezone flag
    if let Some(timezone) = args.flags.get("--timezone") {
        if let Some(tz) = timezone {
            let sanitized_tz = sanitize_input(tz, true);
            // Validate timezone
            match chrono_tz::Tz::from_str(&sanitized_tz) {
                Ok(_) => config.timezone = Some(sanitized_tz),
                Err(_) => {
                    println!("Warning: Invalid timezone '{}'. Using system default.", sanitized_tz);
                }
            }
        }
    }
    
    // Handle all-day flag
    if args.flags.contains_key("--all-day") {
        config.all_day = true;
    }
    
    // Handle Zoom meeting flag
    if args.flags.contains_key("--zoom") {
        config.create_zoom_meeting = true;
        debug!("Zoom meeting will be created for this event");
    }
    
    // Handle recurrence flags - support both --repeat and --recurring
    let repeat_flag = args.flags.get("--repeat")
        .or_else(|| args.flags.get("--recurring"));
    
    if let Some(repeat) = repeat_flag {
        if let Some(frequency_str) = repeat {
            match calendar::RecurrenceFrequency::from_str(frequency_str) {
                Ok(frequency) => {
                    // Create recurrence pattern
                    let mut recurrence = calendar::RecurrencePattern::new(frequency);
                    
                    // Handle interval
                    if let Some(interval) = args.flags.get("--interval") {
                        if let Some(interval_str) = interval {
                            if let Ok(interval_val) = interval_str.parse::<u32>() {
                                // Validate interval is reasonable
                                if interval_val > 0 && interval_val <= 365 {
                                    recurrence = recurrence.with_interval(interval_val);
                                } else {
                                    println!("Warning: Interval should be between 1 and 365. Using default of 1.");
                                }
                            }
                        }
                    }
                    
                    // Handle until date
                    if let Some(until) = args.flags.get("--until") {
                        if let Some(until_str) = until {
                            // Validate date format
                            if calendar::validate_date_format(until_str) {
                                recurrence = recurrence.with_end_date(until_str);
                            } else {
                                println!("Warning: Invalid end date format. Please use YYYY-MM-DD format.");
                            }
                        }
                    }
                    
                    // Handle count
                    if let Some(count) = args.flags.get("--count") {
                        if let Some(count_str) = count {
                            if let Ok(count_val) = count_str.parse::<u32>() {
                                // Validate count is reasonable
                                if count_val > 0 && count_val <= 500 {
                                    recurrence = recurrence.with_count(count_val);
                                } else {
                                    println!("Warning: Count should be between 1 and 500. Using default of no limit.");
                                }
                            }
                        }
                    }
                    
                    // Handle days (for weekly recurrence)
                    if let Some(days) = args.flags.get("--days") {
                        if let Some(days_str) = days {
                            let days: Vec<u8> = days_str
                                .split(',')
                                .filter_map(|d| {
                                    let result = d.trim().parse::<u8>();
                                    match result {
                                        Ok(val) if val <= 6 => Some(val), // 0-6 are valid day values
                                        _ => {
                                            println!("Warning: Invalid day value: {}. Days should be 0-6.", d.trim());
                                            None
                                        }
                                    }
                                })
                                .collect();
                            
                            if !days.is_empty() {
                                recurrence = recurrence.with_days_of_week(&days);
                            }
                        }
                    }
                    
                    // Fix: Log the recurrence.interval before moving recurrence into config.recurrence
                    debug!("Set recurrence pattern: {:?} with interval: {}", frequency, recurrence.interval);
                    
                    // Set recurrence pattern in config
                    config.recurrence = Some(recurrence);
                }
                Err(e) => {
                    println!("Invalid recurrence frequency: {}", e);
                    return Err(e);
                }
            }
        }
    }

    // Handle contact group if provided (takes precedence over individual contacts)
    if let Some(group) = args.flags.get("--group") {
        if let Some(group_id) = group {
            let sanitized_group_id = sanitize_input(group_id, false);
            debug!("Using contact group: {}", sanitized_group_id);
            return crate::commands::contacts::create_calendar_event_with_group(config, &sanitized_group_id).await;
        }
    }
    
    // Handle contact names if provided
    if let Some(contacts) = args.flags.get("--contacts") {
        if let Some(contact_str) = contacts {
            let contact_str = sanitize_input(contact_str, true);
            let contact_names: Vec<&str> = contact_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            if !contact_names.is_empty() {
                debug!("Looking up contacts: {:?}", contact_names);
                return calendar::create_event_with_contacts(config, &contact_names).await;
            }
        }
    }
    
    // Create the event with async/await
    calendar::create_event(config).await
}

/// Safely sanitize user input to prevent injection attacks
fn sanitize_input(input: &str, allow_quotes: bool) -> String {
    // Trim quotes first if they're at the start/end of the input
    let trimmed = input.trim_matches('"').trim_matches('\'');
    
    // Escape special characters
    let mut result = String::new();
    
    for c in trimmed.chars() {
        match c {
            // If allow_quotes is true, we preserve quote characters for some fields
            '"' if allow_quotes => result.push('"'),
            '"' => result.push('\''), // Replace double quotes with single quotes if not allowed
            '\'' if allow_quotes => result.push('\''),
            '\'' => result.push('`'), // Replace single quotes if not allowed
            '\\' => result.push_str("\\\\"), // Escape backslashes
            ';' => result.push(','), // Replace semicolons with commas
            '&' => result.push_str("and"), // Replace & with "and"
            '|' => result.push('/'), // Replace pipes with slashes
            '<' => result.push('('), // Replace angle brackets with parentheses
            '>' => result.push(')'),
            '$' => result.push('$'), // Drop dollar signs which could be used for variable references
            '`' => result.push('\''), // Replace backticks
            '\n' => result.push(' '), // Replace newlines with spaces
            '\r' => result.push(' '), // Replace carriage returns with spaces
            _ if c.is_control() => {}, // Remove control characters
            _ => result.push(c),
        }
    }
    
    result
}

/// Validate email format with simple regex check
fn validate_email_format(email: &str) -> bool {
    let re = regex::Regex::new(r"^[A-Za-z0-9._%+-]{1,64}@[A-Za-z0-9.-]{1,255}\.[A-Za-z]{2,63}$").unwrap();
    re.is_match(email) && !contains_dangerous_chars(email)
}

/// Check for potentially dangerous characters in string
fn contains_dangerous_chars(input: &str) -> bool {
    input.contains('\'') || input.contains('\"') || input.contains('`') || 
    input.contains(';') || input.contains('&') || input.contains('|') ||
    input.contains('<') || input.contains('>')
}

async fn delete_calendar_event(args: CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Usage: calendar delete <title>");
        return Ok(());
    }
    
    let title = &args.args[1];
    // Add await to properly handle the async function
    calendar::delete_event(title, args.args.get(2).map(|s| s.as_str()).unwrap_or("")).await?;
    
    let mut events = state::load_events()?;
    events.retain(|e| e.title != args.args[1]);
    state::StateManager::new()?.save(&events)?;
    
    Ok(())
}

fn set_default_calendar(args: CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Usage: ducktape calendar set-default \"<name>\"");
        return Ok(());
    }
    
    let default_calendar = args.args[1].trim_matches('"').to_string();
    let mut config = crate::config::Config::load()?;
    config.calendar.default_calendar = Some(default_calendar);
    config.save()?;
    println!("Default calendar updated.");
    
    Ok(())
}

async fn import_calendar_events(args: CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Usage: ducktape calendar import \"<file_path>\" [--format csv|ics] [--calendar \"<calendar_name>\"]");
        println!("Example: ducktape calendar import \"events.csv\" --format csv --calendar \"Work\"");
        return Ok(());
    }

    let file_path = Path::new(&args.args[1]);
    if !file_path.exists() {
        println!("Error: File not found at path: {}", args.args[1]);
        return Ok(());
    }

    // Get format from --format flag, default to csv
    let format = args.flags.get("--format")
        .and_then(|f| f.as_ref())
        .map(|f| f.to_lowercase())
        .unwrap_or_else(|| "csv".to_string());

    if !["csv", "ics"].contains(&format.as_str()) {
        println!("Error: Unsupported format. Use --format csv or --format ics");
        return Ok(());
    }

    // Get target calendar if specified
    let calendar = args.flags.get("--calendar").and_then(|c| c.as_ref()).map(|c| c.to_string());

    match format.as_str() {
        "csv" => calendar::import_csv_events(file_path, calendar).await,
        "ics" => calendar::import_ics_events(file_path, calendar).await,
        _ => unreachable!()
    }
}