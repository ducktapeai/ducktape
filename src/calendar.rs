use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::{debug, error};
use std::process::Command;

/// Duration in seconds for different event types
const ALL_DAY_DURATION: i64 = 86400;
const DEFAULT_DURATION: i64 = 3600;

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
    pub date: &'a str,
    pub time: &'a str,
    pub calendar: Option<&'a str>,
    pub all_day: bool,
    pub location: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
}

impl<'a> EventConfig<'a> {
    /// Creates a new EventConfig with required fields
    pub fn new(title: &'a str, date: &'a str, time: &'a str) -> Self {
        Self {
            title,
            date,
            time,
            calendar: None,
            all_day: false,
            location: None,
            description: None,
            email: None,
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
    
    let datetime = format!("{} {}", config.date, if config.all_day { "00:00" } else { config.time });
    let dt = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M")
        .map_err(|e| CalendarError::InvalidDateTime(e.to_string()))?;

    let local_dt: DateTime<Local> = Local::now()
        .timezone()
        .from_local_datetime(&dt)
        .single()
        .ok_or_else(|| anyhow!("Invalid or ambiguous local time"))?;

    // First verify Calendar.app is running
    ensure_calendar_running()?;

    // Use simple duration in seconds: 86400 for all-day, 3600 otherwise.
    let duration = if config.all_day { ALL_DAY_DURATION } else { DEFAULT_DURATION };

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

    // Build email code block if provided, using documented Apple syntax
    let email_code = if let Some(email_addr) = &config.email {
        format!(
            r#"
                -- Add attendee
                tell application "Calendar"
                    tell newEvent
                        make new attendee at end with properties {{email:"{}", display name:"{}"}}
                    end tell
                end tell"#,
            email_addr,
            email_addr  // Using email as display name, could be parameterized further
        )
    } else {
        String::new()
    };

    let script = format!(
        r#"tell application "Calendar"
            try
                -- Find calendar
                set calendarName to "{}"
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
                -- Set up start date
                set startDate to current date
                set year of startDate to {}
                set month of startDate to {}
                set day of startDate to {}
                set hours of startDate to {}
                set minutes of startDate to {}
                set seconds of startDate to 0
                -- Build properties and create the event
                set props to {{summary:"{}", start date:startDate, end date:(startDate + {}), description:"{}"{}}}"#,
        config.calendar.unwrap_or("Calendar"),
        local_dt.format("%Y"),
        local_dt.format("%-m"),
        local_dt.format("%-d"),
        local_dt.format("%-H"),
        local_dt.format("%-M"),
        config.title,
        duration,
        config.description.as_deref().unwrap_or("Created by DuckTape"),
        extra
    );

    // Add conditional blocks after the main script
    let script = if !all_day_code.is_empty() || !email_code.is_empty() {
        format!(
            r#"{}
                tell targetCal
                    set newEvent to make new event at end with properties props
                    {}{}
                end tell
                return "Success: Event created"
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
            script,
            all_day_code,
            email_code
        )
    } else {
        format!(
            r#"{}
                tell targetCal
                    make new event at end with properties props
                end tell
                return "Success: Event created"
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
            script
        )
    };

    println!("Debug: Generated AppleScript:\n{}", script);
    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);
    if result.contains("Success") {
        println!(
            "Calendar event created: {} at {} ({} timezone)",
            config.title,
            format!("{} {}", config.date, config.time),
            local_dt.offset()
        );
        Ok(())
    } else {
        if let Some(cal_id) = config.calendar {
            println!("Debug: Attempted to find calendar matching '{}'", cal_id);
            if !error_output.is_empty() {
                println!("Debug log:\n{}", error_output);
            }
        }
        Err(anyhow!(
            "Failed to create calendar event: {}",
            if result.is_empty() {
                "Unknown error occurred"
            } else {
                &result
            }
        ))
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
