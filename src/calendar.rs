use crate::config::Config;
use crate::state::{CalendarItem, StateManager};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::{debug, error, info}; // Import the info macro
use std::process::Command;

// Remove unused constants
// const ALL_DAY_DURATION: i64 = 86400;
// const DEFAULT_DURATION: i64 = 3600;

/// Custom error type for calendar operations
#[derive(Debug, thiserror::Error)]
pub enum CalendarError {
    #[error("Calendar application is not running")]
    NotRunning,
    #[error("Calendar '{0}' not found")]
    CalendarNotFound(String),
    #[error("Invalid date/time format: {0}")]
    InvalidDateTime(String),
    #[error("AppleScript execution failed: {0}")]
    ScriptError(String),
}

/// Configuration for a calendar event
#[derive(Debug)]
pub struct EventConfig<'a> {
    pub title: &'a str,
    pub start_date: &'a str,
    pub start_time: &'a str,
    pub end_date: Option<&'a str>, // New field
    pub end_time: Option<&'a str>, // New field
    pub calendars: Vec<&'a str>,   // Changed from Option<&'a str> to Vec<&'a str>
    pub all_day: bool,
    pub location: Option<String>,
    pub description: Option<String>,
    pub emails: Vec<String>,  // Changed from Option<String> to Vec<String>
    pub reminder: Option<i32>, // Minutes before event to show reminder
}

impl<'a> EventConfig<'a> {
    /// Creates a new EventConfig with required fields
    pub fn new(title: &'a str, start_date: &'a str, start_time: &'a str) -> Self {
        Self {
            title,
            start_date,
            start_time,
            end_date: None,
            end_time: None,
            calendars: Vec::new(),
            all_day: false,
            location: None,
            description: None,
            emails: Vec::new(),  // Initialize empty vector
            reminder: None,
        }
    }
}

pub fn list_calendars() -> Result<()> {
    // First ensure Calendar.app is running
    let launch_script = r#"
        tell application "Calendar"
            launch
            delay 1
        end tell
    "#;

    Command::new("osascript")
        .arg("-e")
        .arg(launch_script)
        .output()?;

    let script = r#"tell application "Calendar"
        try
            set output to {}
            repeat with aCal in calendars
                set calInfo to name of aCal
                -- Removed block referencing 'account'
                copy calInfo to end of output
            end repeat
            return output
        on error errMsg
            error "Failed to list calendars: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    if output.status.success() {
        println!("Available calendars:");
        let calendars = String::from_utf8_lossy(&output.stdout);
        if calendars.trim().is_empty() {
            println!("  No calendars found. Please ensure Calendar.app is properly configured.");
        } else {
            for calendar in calendars.trim_matches('{').trim_matches('}').split(", ") {
                println!("  - {}", calendar.trim_matches('"'));
            }
        }
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to list calendars: {}\nPlease ensure Calendar.app is running and properly configured.",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn create_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);
    
    // First verify Calendar.app is running and get available calendars
    ensure_calendar_running()?;
    
    // Get list of available calendars first
    let available_calendars = get_available_calendars()?;
    debug!("Available calendars: {:?}", available_calendars);

    // Load configuration and get default calendar if none specified
    let app_config = Config::load()?;
    let requested_calendars = if config.calendars.is_empty() {
        vec![app_config
            .calendar
            .default_calendar
            .unwrap_or_else(|| "Calendar".to_string())]
    } else {
        // Validate that specified calendars exist
        let requested: Vec<String> = config.calendars.iter().map(|&s| s.to_string()).collect();
        let valid_calendars: Vec<String> = requested
            .into_iter()
            .filter(|cal| {
                let exists = available_calendars.iter().any(|available| available.eq_ignore_ascii_case(cal));
                if !exists {
                    error!("Calendar '{}' not found in available calendars", cal);
                }
                exists
            })
            .collect();

        if valid_calendars.is_empty() {
            return Err(anyhow!("None of the specified calendars were found. Available calendars: {}", 
                available_calendars.join(", ")));
        }
        valid_calendars
    };

    let mut last_error = None;
    let mut success_count = 0;
    let total_calendars = requested_calendars.len();

    // Clone the calendars Vec before the loop
    let calendars_for_state = requested_calendars.clone();

    for calendar in requested_calendars {
        info!("Attempting to create event in calendar: {}", calendar);
        let this_config = EventConfig {
            title: config.title,
            start_date: config.start_date,
            start_time: config.start_time,
            end_date: config.end_date,
            end_time: config.end_time,
            calendars: vec![&calendar],
            all_day: config.all_day,
            location: config.location.clone(),
            description: config.description.clone(),
            emails: config.emails.clone(),
            reminder: config.reminder,
        };
        
        match create_single_event(this_config) {
            Ok(_) => {
                success_count += 1;
                info!("Successfully created event in calendar '{}'", calendar);
            }
            Err(e) => {
                error!("Failed to create event in calendar '{}': {}", calendar, e);
                last_error = Some(e);
            }
        }
    }

    if success_count > 0 {
        // Save the event to state using the cloned calendars
        let calendar_item = CalendarItem {
            title: config.title.to_string(),
            date: config.start_date.to_string(),
            time: config.start_time.to_string(),
            calendars: calendars_for_state,  // Use the cloned Vec here
            all_day: config.all_day,
            location: config.location,
            description: config.description,
            email: if !config.emails.is_empty() {
                Some(config.emails.join(", "))
            } else {
                None
            },
            reminder: config.reminder,
        };
        StateManager::new()?.add(calendar_item)?;
        info!(
            "Calendar event created in {}/{} calendars",
            success_count, total_calendars
        );
        Ok(())
    } else {
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to create event in any calendar")))
    }
}

fn get_available_calendars() -> Result<Vec<String>> {
    let script = r#"tell application "Calendar"
        try
            set output to {}
            repeat with aCal in calendars
                set calInfo to name of aCal
                copy calInfo to end of output
            end repeat
            return output
        on error errMsg
            error "Failed to list calendars: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;
    if output.status.success() {
        let calendars = String::from_utf8_lossy(&output.stdout);
        Ok(calendars
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .map(|s| s.trim_matches('"').to_string())
            .collect())
    } else {
        Err(anyhow!(
            "Failed to get available calendars: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn create_single_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);

    // Parse start datetime
    let start_datetime = format!(
        "{} {}",
        config.start_date,
        if config.all_day {
            "00:00"
        } else {
            config.start_time
        }
    );
    let start_dt = NaiveDateTime::parse_from_str(&start_datetime, "%Y-%m-%d %H:%M")
        .map_err(|e| CalendarError::InvalidDateTime(e.to_string()))?;

    // Parse end datetime
    let end_dt = if let Some(end_time) = config.end_time {
        let end_datetime = format!("{} {}", config.start_date, end_time);
        NaiveDateTime::parse_from_str(&end_datetime, "%Y-%m-%d %H:%M")
            .map_err(|e| CalendarError::InvalidDateTime(e.to_string()))?
    } else {
        start_dt + chrono::Duration::hours(1)
    };

    // Convert to local DateTime
    let local_start: DateTime<Local> = Local::now()
        .timezone()
        .from_local_datetime(&start_dt)
        .single()
        .ok_or_else(|| anyhow!("Invalid or ambiguous start time"))?;

    let local_end: DateTime<Local> = Local::now()
        .timezone()
        .from_local_datetime(&end_dt)
        .single()
        .ok_or_else(|| anyhow!("Invalid or ambiguous end time"))?;

    // Validate that end time is after start time
    if local_end <= local_start {
        return Err(anyhow!("End time must be after start time"));
    }

    // First verify Calendar.app is running
    ensure_calendar_running()?;

    // Build extras for properties: include location if non-empty.
    let mut extra = String::new();
    if let Some(loc) = &config.location {
        if !loc.is_empty() {  // Remove unnecessary parentheses
            extra.push_str(&format!(", location:\"{}\"", loc));
        }
    }

    // Build attendees code with improved logging and error handling
    let mut attendees_code = String::new();
    if !config.emails.is_empty() {
        attendees_code.push_str("\n                    -- Add attendees");
        let mut added_emails = Vec::new();
        for email in &config.emails {
            let email = email.trim();
            if !added_emails.contains(&email) {
                info!("Adding attendee: {}", email);
                attendees_code.push_str(&format!(r#"
                    try
                        make new attendee at end of attendees of newEvent with properties {{email:"{0}"}}
                        log "Successfully added attendee: {0}"
                    on error errMsg
                        log "Failed to add attendee {0}: " & errMsg
                        error "Failed to add attendee {0}: " & errMsg
                    end try"#,
                    email
                ));
                added_emails.push(email);
            }
        }
    }

    // Build the AppleScript with improved error handling and logging
    let script = format!(
        r#"tell application "Calendar"
            try
                -- Find calendar and ensure it exists
                if not (exists calendar "{calendar_name}") then
                    error "Calendar '{calendar_name}' not found"
                end if

                tell calendar "{calendar_name}"
                    -- Create event dates
                    set startDate to current date
                    set year of startDate to {start_year}
                    set month of startDate to {start_month}
                    set day of startDate to {start_day}
                    set hours of startDate to {start_hours}
                    set minutes of startDate to {start_minutes}
                    set seconds of startDate to 0

                    set endDate to current date
                    set year of endDate to {end_year}
                    set month of endDate to {end_month}
                    set day of endDate to {end_day}
                    set hours of endDate to {end_hours}
                    set minutes of endDate to {end_minutes}
                    set seconds of endDate to 0

                    -- Create new event with logging
                    log "Creating event: {title}"
                    set newEvent to make new event with properties {{summary:"{title}", start date:startDate, end date:endDate, description:"{description}"{extra}}}
                    log "Event created successfully"

                    {all_day_code}
                    {reminder_code}
                    
                    -- Add attendees with error handling
                    {attendees_code}

                    -- Save changes
                    save
                    log "Event saved with attendees"
                end tell

                -- Force calendar refresh
                reload calendars
                
                return "Success: Event created"
            on error errMsg
                log errMsg
                error "Failed to create event: " & errMsg
            end try
        end tell"#,
        calendar_name = config.calendars[0],
        start_year = local_start.format("%Y"),
        start_month = local_start.format("%-m"),
        start_day = local_start.format("%-d"),
        start_hours = local_start.format("%-H"),
        start_minutes = local_start.format("%-M"),
        end_year = local_end.format("%Y"),
        end_month = local_end.format("%-m"),
        end_day = local_end.format("%-d"),
        end_hours = local_end.format("%-H"),
        end_minutes = local_end.format("%-M"),
        title = config.title,
        description = config.description.as_deref().unwrap_or("Created by DuckTape"),
        extra = extra,
        all_day_code = if config.all_day { "\n                    set allday event of newEvent to true" } else { "" },
        reminder_code = if let Some(minutes) = config.reminder {
            format!(
                r#"
                    -- Add reminder alarm
                    set theAlarm to make new display alarm at end of newEvent
                    set trigger interval of theAlarm to -{}"#,
                minutes * 60
            )
        } else { String::new() },
        attendees_code = attendees_code
    );

    debug!("Generated AppleScript:\n{}", script);
    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);

    if result.contains("Success") {
        info!(
            "Calendar event created: {} at {} ({} timezone)",
            config.title,
            format!("{} {}", config.start_date, config.start_time),
            local_start.offset()
        );
        if !config.emails.is_empty() {
            let formatted_emails = config.emails.join(", ");
            info!("Added {} attendee(s): {}", config.emails.len(), formatted_emails);
        }
        Ok(())
    } else {
        error!("AppleScript error: STDOUT: {} | STDERR: {}", result, error_output);
        if result.contains("Calendar '") && result.contains("' not found") {
            if let Some(cal_id) = config.calendars.get(0) {
                return Err(CalendarError::CalendarNotFound(cal_id.to_string()).into());
            }
        }
        Err(if error_output.is_empty() && result.is_empty() {
            CalendarError::ScriptError("Unknown error occurred".to_string()).into()
        } else if !error_output.is_empty() {  // Changed isEmpty() to is_empty()
            CalendarError::ScriptError(error_output.to_string()).into()
        } else {
            CalendarError::ScriptError(result.to_string()).into()
        })
    }
}

fn ensure_calendar_running() -> Result<()> {
    let check_script = r#"tell application "Calendar" to if it is running then return true"#;
    let check = Command::new("osascript")
        .arg("-e")
        .arg(check_script)
        .output()
        .map_err(|e| CalendarError::ScriptError(e.to_string()))?;

    if !check.status.success() {
        Err(CalendarError::NotRunning.into())
    } else {
        Ok(())
    }
}

pub fn list_event_properties() -> Result<()> {
    // First verify Calendar.app is running
    ensure_calendar_running()?;

    let script = r#"tell application "Calendar"
        try
            set propList to {}
            
            -- Basic properties that we can set
            copy "summary (title)" to end of propList
            copy "start date" to end of propList
            copy "end date" to end of propList
            copy "allday" to end of propList
            copy "description" to end of propList
            copy "location" to end of propList
            copy "url" to end of propList
            copy "calendar" to end of propList
            copy "recurrence" to end of propList
            copy "status" to end of propList
            copy "availability" to end of propList
            
            return propList
        on error errMsg
            error "Failed to get event properties: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    if output.status.success() {
        println!("Available Calendar Event Properties:");
        let properties = String::from_utf8_lossy(&output.stdout);
        if !properties.trim().is_empty() {
            for prop in properties.trim_matches('{').trim_matches('}').split(", ") {
                println!("  - {}", prop.trim_matches('"'));
            }
        } else {
            println!("  No properties found. Calendar might not be accessible.");
        }
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to get event properties: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn delete_event(title: &str, _date: &str) -> Result<()> {
    let script = format!(
        r#"tell application "Calendar"
            try
                set foundEvents to {{}}
                repeat with c in calendars
                    tell c
                        set matchingEvents to (every event whose summary contains "{0}")
                        repeat with evt in matchingEvents
                            copy evt to end of foundEvents
                        end repeat
                    end tell
                end repeat
                
                if (count of foundEvents) is 0 then
                    error "No matching events found for title: {0}"
                end if
                
                repeat with evt in foundEvents
                    set evtTitle to summary of evt
                    log "Deleting event: " & evtTitle
                    delete evt
                end repeat
                
                return "Success: Events deleted"
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
        title
    );

    let output = Command::new("osascript").arg("-e").arg(&script).output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        println!("Calendar event(s) deleted containing title: {}", title);
        Ok(())
    } else {
        Err(anyhow!("Failed to delete events: {}", result))
    }
}

/// Lookup a contact by name and return their email addresses
pub fn lookup_contact(name: &str) -> Result<Vec<String>> {
    let script = format!(
        r#"tell application "Contacts"
            set the_emails to {{}}
            
            try
                set the_people to (every person whose name contains "{}")
                repeat with the_person in the_people
                    if exists email of the_person then
                        repeat with the_email in (get every email of the_person)
                            if value of the_email is not missing value then
                                set the end of the_emails to (value of the_email as text)
                            end if
                        end repeat
                    end if
                end repeat
                
                return the_emails
            on error errMsg
                log "Error looking up contact: " & errMsg
                return {{}}
            end try
        end tell"#,
        name.replace("\"", "\\\"")
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;

    if output.status.success() {
        let emails = String::from_utf8_lossy(&output.stdout);
        debug!("Raw contact lookup output: {}", emails);
        
        let email_list: Vec<String> = emails
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .filter(|s| !s.is_empty() && !s.contains("missing value"))
            .map(|s| s.trim_matches('"').trim().to_string())
            .collect();
        
        if email_list.is_empty() {
            debug!("No emails found for contact '{}'", name);
        } else {
            debug!("Found {} email(s) for '{}': {:?}", email_list.len(), name, email_list);
        }
        
        Ok(email_list)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Contact lookup error: {}", error);
        Ok(Vec::new())
    }
}

/// Enhanced event creation with contact lookup
pub fn create_event_with_contacts(mut config: EventConfig, contact_names: &[&str]) -> Result<()> {
    // Look up emails for each contact name
    let mut found_emails = Vec::new();
    for name in contact_names {
        match lookup_contact(name) {
            Ok(emails) => {
                if emails.is_empty() {
                    debug!("No email found for contact: {}", name);
                } else {
                    found_emails.extend(emails.into_iter().map(|e| e.trim().to_string()));
                }
            }
            Err(e) => {
                error!("Failed to lookup contact {}: {}", name, e);
            }
        }
    }

    // Add found emails to config
    if !found_emails.is_empty() {
        debug!("Adding {} found email(s) to event", found_emails.len());
        config.emails.extend(found_emails);
        
        // Deduplicate and clean emails
        config.emails = config.emails
            .into_iter()
            .map(|e| e.trim().to_string())
            .collect::<Vec<_>>();
        config.emails.sort_unstable();
        config.emails.dedup();
    }

    // Create the event with the updated config
    create_event(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_config() -> EventConfig<'static> {
        EventConfig {
            title: "Test Event",
            start_date: "2024-02-21",
            start_time: "14:30",
            end_date: None,
            end_time: None,
            calendars: vec!["Test Calendar"],
            all_day: false,
            location: Some("Test Location".to_string()),
            description: Some("Test Description".to_string()),
            emails: vec!["test@example.com".to_string()],  // Use vector for emails
            reminder: Some(30),
        }
    }

    #[test]
    fn test_event_config_new() {
        let config = EventConfig::new("Test", "2024-02-21", "14:30");
        assert_eq!(config.title, "Test");
        assert_eq!(config.start_date, "2024-02-21");
        assert_eq!(config.start_time, "14:30");
        assert!(!config.all_day);
        assert!(config.calendars.is_empty());
        assert!(config.reminder.is_none());
    }

    #[test]
    fn test_parse_datetime() {
        let config = create_test_config();
        let datetime = format!("{} {}", config.start_date, config.start_time);
        let result = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_datetime() {
        let config = EventConfig::new("Test", "invalid-date", "25:00");
        let datetime = format!("{} {}", config.start_date, config.start_time);
        let result = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M");
        assert!(result.is_err());
    }

    #[test]
    fn test_calendar_not_found_error() {
        let config = EventConfig {
            title: "Test Event",
            start_date: "2024-02-21",
            start_time: "14:30",
            end_date: None,
            end_time: Some("15:30"), // Add end time
            calendars: vec!["NonexistentCalendar"],
            all_day: false,
            location: None,
            description: None,
            emails: Vec::new(),  // Initialize empty vector
            reminder: None,
        };

        let result = create_single_event(config); // Use create_single_event instead of create_event
        assert!(result.is_err());

        let err = result.unwrap_err();
        let calendar_err = err.downcast_ref::<CalendarError>();
        assert!(matches!(
            calendar_err,
            Some(CalendarError::CalendarNotFound(_))
        ));
    }
}
