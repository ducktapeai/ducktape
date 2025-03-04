use anyhow::Result;
use crate::commands::{CommandArgs, CommandExecutor};
use std::future::Future;
use std::pin::Pin;
use crate::calendar;
use crate::state;
use log::debug;

pub struct CalendarCommand;

impl CommandExecutor for CalendarCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            handle_calendar_command(args)
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "calendar" | "calendars" | "calendar-props" | "delete-event" | "list-events")
    }
}

fn handle_calendar_command(args: CommandArgs) -> Result<()> {
    match args.command.as_str() {
        "calendars" => calendar::list_calendars(),
        "calendar-props" => calendar::list_event_properties(),
        "list-events" => list_events(),
        "delete-event" => delete_event(args),
        "calendar" => {
            match args.args.get(0).map(|s| s.to_lowercase()).as_deref() {
                Some("create") => create_calendar_event(args),
                Some("delete") => delete_calendar_event(args),
                Some("set-default") => set_default_calendar(args),
                _ => {
                    println!("Unknown calendar command. Use 'calendar create', 'calendar delete', or 'calendar set-default'.");
                    Ok(())
                }
            }
        },
        _ => {
            println!("Unknown calendar command");
            Ok(())
        }
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

fn delete_event(args: CommandArgs) -> Result<()> {
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

fn create_calendar_event(args: CommandArgs) -> Result<()> {
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
    
    // Handle location flag
    if let Some(location) = args.flags.get("--location") {
        if let Some(loc) = location {
            config.location = Some(loc.trim_matches('"').to_string());
        }
    }
    
    // Handle description/notes flag
    if let Some(description) = args.flags.get("--notes") {
        if let Some(desc) = description {
            config.description = Some(desc.trim_matches('"').to_string());
        }
    }
    
    // Handle reminder flag
    if let Some(reminder) = args.flags.get("--reminder") {
        if let Some(mins) = reminder {
            if let Ok(minutes) = mins.parse::<i32>() {
                config.reminder = Some(minutes);
            }
        }
    }
    
    // Handle timezone flag
    if let Some(timezone) = args.flags.get("--timezone") {
        if let Some(tz) = timezone {
            config.timezone = Some(tz.trim_matches('"').to_string());
        }
    }
    
    // Handle all-day flag
    if args.flags.contains_key("--all-day") {
        config.all_day = true;
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
                                recurrence = recurrence.with_interval(interval_val);
                            }
                        }
                    }
                    
                    // Handle until date
                    if let Some(until) = args.flags.get("--until") {
                        if let Some(until_str) = until {
                            recurrence = recurrence.with_end_date(until_str);
                        }
                    }
                    
                    // Handle count
                    if let Some(count) = args.flags.get("--count") {
                        if let Some(count_str) = count {
                            if let Ok(count_val) = count_str.parse::<u32>() {
                                recurrence = recurrence.with_count(count_val);
                            }
                        }
                    }
                    
                    // Handle days (for weekly recurrence)
                    if let Some(days) = args.flags.get("--days") {
                        if let Some(days_str) = days {
                            let days: Vec<u8> = days_str
                                .split(',')
                                .filter_map(|d| d.trim().parse::<u8>().ok())
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

fn delete_calendar_event(args: CommandArgs) -> Result<()> {
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