use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::{debug, error};
use std::process::Command;
use crate::state::{CalendarItem, StateManager};

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
    pub end_date: Option<&'a str>,    // New field
    pub end_time: Option<&'a str>,    // New field
    pub calendars: Vec<&'a str>,  // Changed from Option<&'a str> to Vec<&'a str>
    pub all_day: bool,
    pub location: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub reminder: Option<i32>,  // Minutes before event to show reminder
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
            email: None,
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
                try
                    tell aCal
                        if account is not missing value then
                            set calInfo to calInfo & " (Account: " & (title of account) & ")"
                        end if
                    end tell
                end try
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
    
    let calendars = if config.calendars.is_empty() {
        vec!["Calendar"]
    } else {
        config.calendars.clone()  // Clone here to avoid the move
    };

    let mut success_count = 0;
    let total_calendars = calendars.len();
    let mut errors = Vec::new(); // Collect errors from each calendar

    for calendar in calendars {
        let this_config = EventConfig {
            title: config.title,
            start_date: config.start_date,
            start_time: config.start_time,
            end_date: config.end_date,
            end_time: config.end_time,
            calendars: vec![calendar],
            all_day: config.all_day,
            location: config.location.clone(),
            description: config.description.clone(),
            email: config.email.clone(),
            reminder: config.reminder,
        };

        match create_single_event(this_config) {
            Ok(_) => success_count += 1,
            Err(e) => errors.push(e),
        }
    }

    if success_count > 0 {
        // Save the event to state
        let calendar_item = CalendarItem {
            title: config.title.to_string(),
            date: config.start_date.to_string(),
            time: config.start_time.to_string(),
            calendars: config.calendars.iter().map(|&s| s.to_string()).collect(),
            all_day: config.all_day,
            location: config.location,
            description: config.description,
            email: config.email,
            reminder: config.reminder,
        };
        StateManager::new()?.add(calendar_item)?;
        
        println!(
            "Calendar event created in {}/{} calendars",
            success_count,
            total_calendars
        );
        Ok(())
    } else {
        // Replace errors.iter().find(...) with into_iter()
        if let Some(err) = errors.into_iter().find(|err| err.to_string().contains("not found")) {
            return Err(err);
        }
        Err(anyhow!("Failed to create event in any calendar"))
    }
}

fn create_single_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);
    
    // Parse start datetime
    let start_datetime = format!("{} {}", config.start_date, 
        if config.all_day { "00:00" } else { config.start_time });
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

    // Convert to local DateTime using the current offset once for both start and end
    let local_offset = Local::now().offset().clone();
    let local_start = local_offset.from_local_datetime(&start_dt)
        .single()
        .ok_or_else(|| anyhow!("Invalid or ambiguous start time"))?;
    let local_end = local_offset.from_local_datetime(&end_dt)
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
        if !loc.is_empty() {
            extra.push_str(&format!(", location:\"{}\"", loc));
        }
    }

    // Set up a separate code block for marking the event as an all-day event.
    let all_day_code = if config.all_day {
        "\n                set allday event of newEvent to true"
    } else {
        ""
    };

    // Update the email code block by removing the line setting host name.
    let email_code = if let Some(email_addr) = &config.email {
        format!(
            r#"
                tell newEvent
                    make new attendee at end with properties {{email:"{0}", display name:"{0}"}}
                end tell
        "#,
            email_addr
        )
    } else {
        String::new()
    };

    // Build reminder code block if provided
    let reminder_code = if let Some(minutes) = config.reminder {
        format!(
            r#"
                -- Add reminder alarm
                set theAlarm to make new display alarm at end of newEvent
                set trigger interval of theAlarm to -{}"#,
            minutes * 60  // Convert minutes to seconds for Calendar.app
        )
    } else {
        String::new()
    };

    let script = format!(
        r#"tell application "Calendar"
            try
                -- Find calendar
                set calendarName to "{calendar_name}"
                set targetCal to missing value
                repeat with c in calendars
                    if name of c is calendarName then
                        set targetCal to c
                        exit repeat
                    end if
                end repeat
                if targetCal is missing value then
                    error "Calendar '" & calendarName & "' not found"
                end if
                -- Set up start date and end date
                set startDate to current date
                set endDate to current date
                
                -- Configure start date
                set year of startDate to {start_year}
                set month of startDate to {start_month}
                set day of startDate to {start_day}
                set hours of startDate to {start_hours}
                set minutes of startDate to {start_minutes}
                set seconds of startDate to 0
                
                -- Configure end date
                set year of endDate to {end_year}
                set month of endDate to {end_month}
                set day of endDate to {end_day}
                set hours of endDate to {end_hours}
                set minutes of endDate to {end_minutes}
                set seconds of endDate to 0
                
                -- Build properties and create the event
                tell targetCal
                    set newEvent to make new event at end with properties {{summary:"{title}", start date:startDate, end date:endDate, description:"{description}"{extra}}}
                    {email_code}
                    {all_day_code}
                    {reminder_code}
                end tell
                return "Success: Event created"
            on error errMsg
                return "Error: " & errMsg
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
        all_day_code = all_day_code,
        email_code = email_code,
        reminder_code = reminder_code
    );

    println!("Debug: Generated AppleScript:\n{}", script);
    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);
    
    if result.contains("Success") {
        println!(
            "Calendar event created: {} at {} ({} timezone)",
            config.title,
            format!("{} {}", config.start_date, config.start_time),
            local_start.offset()
        );
        Ok(())
    } else {
        // First check for calendar not found error
        if result.contains("Calendar '") && result.contains("' not found") {
            if let Some(cal_id) = config.calendars.get(0) {
                return Err(CalendarError::CalendarNotFound(cal_id.to_string()).into());
            }
        }

        // Log debug information
        if let Some(cal_id) = config.calendars.get(0) {
            debug!("Attempted to find calendar matching '{}'", cal_id);
        }
        if !error_output.is_empty() {
            debug!("AppleScript error: {}", error_output);
        }

        // Return appropriate error
        Err(if result.is_empty() {
            CalendarError::ScriptError("Unknown error occurred".to_string()).into()
        } else {
            CalendarError::ScriptError(result.to_string()).into()
        })
    }
}

/// Ensures Calendar.app is running and ready
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

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;


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

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        println!("Calendar event(s) deleted containing title: {}", title);
        Ok(())
    } else {
        Err(anyhow!("Failed to delete events: {}", result))
    }
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
            email: Some("test@example.com".to_string()),
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
    fn test_parse_invalid_datetime() {
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
            end_time: None,
            calendars: vec!["NonexistentCalendar"],
            all_day: false,
            location: None,
            description: None,
            email: None,
            reminder: None,
        };

        let result = create_event(config);
        assert!(result.is_err());
        
        match result.unwrap_err().downcast::<CalendarError>() {
            Ok(CalendarError::CalendarNotFound(name)) => {
                assert_eq!(name, "NonexistentCalendar");
            }
            other => panic!("Expected CalendarNotFound error, got {:?}", other),
        }
    }

    #[test]
    fn test_create_single_event_invalid_time() {
        // Provide an invalid start_time to trigger a parsing error.
        let config = EventConfig {
            title: "Invalid Time Event",
            start_date: "2024-02-21",
            start_time: "25:00", // invalid time
            end_date: None,
            end_time: Some("26:00"), // invalid end time as well
            calendars: vec!["Test Calendar"],
            all_day: false,
            location: Some("Test Location".to_string()),
            description: Some("Test Description".to_string()),
            email: Some("invite@test.com".to_string()),
            reminder: Some(15),
        };
        let result = create_single_event(config);
        match result {
            Err(e) => {
                let err_str = e.to_string();
                assert!(err_str.contains("Invalid date/time"), "Error did not mention invalid date/time: {}", err_str);
            }
            Ok(_) => panic!("Expected error for invalid time, but got success"),
        }
    }

    #[test]
    fn test_create_single_event_with_invite() {
        // Using a non-existent calendar so that AppleScript fails and returns error.
        let config = EventConfig {
            title: "Invite Test Event",
            start_date: "2024-02-21",
            start_time: "14:30",
            end_date: None,
            end_time: Some("15:30"),
            calendars: vec!["NonexistentCalendar"],
            all_day: false,
            location: Some("Test Location".to_string()),
            description: Some("Test Invitation".to_string()),
            email: Some("invite@test.com".to_string()),
            reminder: None,
        };
        let result = create_single_event(config);
        // We expect an error because the calendar does not exist.
        match result {
            Err(e) => {
                let err_str = e.to_string();
                // Check the error message contains indication of AppleScript failure.
                assert!(err_str.contains("not found") || err_str.contains("Error:"), "Unexpected error: {}", err_str);
            }
            Ok(_) => panic!("Expected failure due to calendar not found, but got success"),
        }
    }
}
