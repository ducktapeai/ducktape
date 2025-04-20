use crate::config::Config;
use crate::state::{CalendarItem, StateManager};
use crate::zoom::{ZoomClient, ZoomMeetingOptions, calculate_meeting_duration, format_zoom_time};
use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use csv::ReaderBuilder;
use ical::parser::ical::IcalParser;
use ical::parser::ical::component::IcalEvent;
use log::{debug, error, info};
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

mod calendar_applescript;
mod calendar_contacts;
mod calendar_import;
#[cfg(test)]
mod calendar_tests;
mod calendar_types;
mod calendar_validation;

pub use calendar_applescript::*;
pub use calendar_contacts::*;
pub use calendar_import::*;
pub use calendar_types::*;
pub use calendar_validation::*;

/// Custom error type for calendar operations
#[derive(Debug, thiserror::Error)]
pub enum CalendarError {
    #[error("Calendar application is not running")]
    NotRunning,

    #[error("Calendar '{0}' not found")]
    #[allow(dead_code)] // Kept for future use
    CalendarNotFound(String),

    #[error("Invalid date/time format: {0}")]
    InvalidDateTime(String),

    #[error("AppleScript execution failed: {0}")]
    ScriptError(String),
}

pub async fn list_calendars() -> Result<()> {
    // First ensure Calendar.app is running
    ensure_calendar_running().await?;

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

    let output = tokio::process::Command::new("osascript").arg("-e").arg(script).output().await?;

    if output.status.success() {
        println!("Available calendars:");
        let calendars = String::from_utf8_lossy(&output.stdout);
        if calendars.trim().is_empty() {
            println!("  No calendars found. Please ensure Calendar.app is properly configured.");
        } else {
            // Create a HashSet for deduplication
            let mut unique_calendars: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for calendar in calendars.trim_matches('{').trim_matches('}').split(", ") {
                unique_calendars.insert(calendar.trim_matches('"').to_string());
            }
            // Sort the calendars for consistent display
            let mut sorted_calendars: Vec<_> = unique_calendars.into_iter().collect();
            sorted_calendars.sort();
            for calendar in sorted_calendars {
                println!("  - {}", calendar);
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

pub async fn create_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);

    // Fix: Bring validate into scope for EventConfig
    use crate::calendar::calendar_validation::validate_event_config;

    // Validate the event configuration first
    validate_event_config(&config)?;

    // First verify Calendar.app is running and get available calendars
    ensure_calendar_running().await?;

    // Get list of available calendars first
    let available_calendars = get_available_calendars().await?;
    debug!("Available calendars: {:?}", available_calendars);

    // Load configuration and get default calendar if none specified
    let app_config = Config::load()?;
    let requested_calendars = if config.calendars.is_empty() {
        vec![app_config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string())]
    } else {
        // Validate that specified calendars exist
        let requested: Vec<String> = config.calendars.iter().map(|s| s.to_string()).collect();
        let valid_calendars: Vec<String> = requested
            .into_iter()
            .filter(|cal| {
                let exists =
                    available_calendars.iter().any(|available| available.eq_ignore_ascii_case(cal));
                if !exists {
                    error!("Calendar '{}' not found in available calendars", cal);
                }
                exists
            })
            .collect();

        if valid_calendars.is_empty() {
            return Err(anyhow!(
                "None of the specified calendars were found. Available calendars: {}",
                available_calendars.join(", ")
            ));
        }
        valid_calendars
    };

    let mut last_error = None;
    let mut success_count = 0;
    let total_calendars = requested_calendars.len();

    // Clone the calendars Vec for state management
    let calendars_for_state = requested_calendars.clone();

    for calendar in requested_calendars {
        info!("Attempting to create event in calendar: {}", calendar);
        let this_config = EventConfig { calendars: vec![calendar.clone()], ..config.clone() };

        match create_single_event(this_config).await {
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
        // Save the event to state
        let calendar_item = CalendarItem {
            title: config.title.clone(),
            date: config.start_date.clone(),
            time: config.start_time.clone(),
            calendars: calendars_for_state,
            all_day: config.all_day,
            location: config.location,
            description: config.description,
            email: if !config.emails.is_empty() { Some(config.emails.join(", ")) } else { None },
            reminder: config.reminder,
        };
        StateManager::new()?.add(calendar_item)?;
        info!("Calendar event created in {}/{} calendars", success_count, total_calendars);
        Ok(())
    } else {
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to create event in any calendar")))
    }
}

pub async fn get_available_calendars() -> Result<Vec<String>> {
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

    let output = tokio::process::Command::new("osascript").arg("-e").arg(script).output().await?;

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

async fn create_single_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);

    // Parse start datetime with improved date handling
    let start_datetime = format!(
        "{} {}",
        config.start_date,
        if config.all_day { "00:00" } else { &config.start_time }
    );

    debug!("Parsing start datetime: {}", start_datetime);

    // Parse the date directly from user input, not using the current date components
    let start_dt = NaiveDateTime::parse_from_str(&start_datetime, "%Y-%m-%d %H:%M")
        .map_err(|e| anyhow!("Invalid start datetime: {}", e))?;

    // Log parsed components for debugging
    info!(
        "Using explicitly parsed date components: year={}, month={} ({}), day={}, raw_date={}",
        start_dt.year(),
        start_dt.month(),
        match start_dt.month() {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        },
        start_dt.day(),
        config.start_date
    );

    // Convert to local timezone with consistent type
    let local_start = if let Some(tz_str) = config.timezone.as_deref() {
        match Tz::from_str(tz_str) {
            Ok(tz) => {
                let tz_dt = tz.from_local_datetime(&start_dt).single().ok_or_else(|| {
                    anyhow!("Invalid or ambiguous start time in timezone {}", tz_str)
                })?;
                tz_dt.with_timezone(&Local)
            }
            Err(_) => {
                error!("Invalid timezone specified: {}. Using local timezone.", tz_str);
                Local::now()
                    .timezone()
                    .from_local_datetime(&start_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Invalid or ambiguous start time"))?
            }
        }
    } else {
        Local::now()
            .timezone()
            .from_local_datetime(&start_dt)
            .single()
            .ok_or_else(|| anyhow!("Invalid or ambiguous start time"))?
    };

    // Parse end datetime with improved handling and validation
    let end_dt = if let Some(ref end_time) = config.end_time {
        let end_datetime = format!("{} {}", config.start_date, end_time);
        debug!("End datetime string: {}", end_datetime);

        let naive_end = NaiveDateTime::parse_from_str(&end_datetime, "%Y-%m-%d %H:%M")
            .map_err(|e| anyhow!("Invalid end datetime: {}", e))?;

        // Check that end time is after start time
        if naive_end <= start_dt {
            debug!(
                "End time {} is not after start time {}, adding 1 day",
                naive_end.format("%H:%M"),
                start_dt.format("%H:%M")
            );

            // If end time is earlier than start time, assume it's the next day
            let next_day = start_dt
                .date()
                .succ_opt()
                .ok_or_else(|| anyhow!("Failed to calculate next day for end time"))?;

            let adjusted_end = NaiveDateTime::new(next_day, naive_end.time());

            Local::now()
                .timezone()
                .from_local_datetime(&adjusted_end)
                .single()
                .ok_or_else(|| anyhow!("Invalid or ambiguous end time"))?
        } else {
            if let Some(tz_str) = config.timezone.as_deref() {
                match Tz::from_str(tz_str) {
                    Ok(tz) => {
                        let tz_dt =
                            tz.from_local_datetime(&naive_end).single().ok_or_else(|| {
                                anyhow!("Invalid or ambiguous end time in timezone {}", tz_str)
                            })?;
                        tz_dt.with_timezone(&Local)
                    }
                    Err(_) => Local::now()
                        .timezone()
                        .from_local_datetime(&naive_end)
                        .single()
                        .ok_or_else(|| anyhow!("Invalid or ambiguous end time"))?,
                }
            } else {
                Local::now()
                    .timezone()
                    .from_local_datetime(&naive_end)
                    .single()
                    .ok_or_else(|| anyhow!("Invalid or ambiguous end time"))?
            }
        }
    } else {
        // If no end time is specified, default to one hour after start time
        local_start + chrono::Duration::hours(1)
    };

    if end_dt <= local_start {
        return Err(anyhow!("End time must be after start time"));
    }

    // Log the final start and end times for debugging
    debug!("Final start time: {}", local_start.format("%Y-%m-%d %H:%M"));
    debug!("Final end time: {}", end_dt.format("%Y-%m-%d %H:%M"));

    // Create Zoom meeting if requested
    let mut zoom_meeting_info = String::new();
    if config.create_zoom_meeting {
        info!("Creating Zoom meeting for event: {}", config.title);
        let mut client = ZoomClient::new()?;
        let zoom_start_time = format_zoom_time(&config.start_date, &config.start_time)?;
        let duration = if let Some(end_time) = &config.end_time {
            calculate_meeting_duration(&config.start_time, end_time)?
        } else {
            60 // Default 1 hour
        };
        let meeting_options = ZoomMeetingOptions {
            topic: config.title.to_string(),
            start_time: zoom_start_time,
            duration,
            password: None,
            agenda: config.description.clone(),
        };
        match client.create_meeting(meeting_options).await {
            Ok(meeting) => {
                info!("Created Zoom meeting: ID={}, URL={}", meeting.id, meeting.join_url);
                let password_info =
                    meeting.password.map_or(String::new(), |p| format!("\nPassword: {}", p));
                zoom_meeting_info = format!(
                    "\n\n--------------------\nZoom Meeting\n--------------------\nJoin URL: {}{}",
                    meeting.join_url, password_info
                );
            }
            Err(e) => {
                error!("Failed to create Zoom meeting: {}", e);
                zoom_meeting_info = "\n\nNote: Zoom meeting creation failed.".to_string();
            }
        }
    } else if let Some(url) = &config.zoom_join_url {
        let password_info = config
            .zoom_password
            .as_ref()
            .map_or(String::new(), |p| format!("\nPassword: {}", p));
        zoom_meeting_info = format!(
            "\n\n--------------------\nZoom Meeting\n--------------------\nJoin URL: {}{}",
            url, password_info
        );
    }

    // Build description with Zoom info
    let full_description = if !zoom_meeting_info.is_empty() {
        match &config.description {
            Some(desc) if !desc.is_empty() => format!("{}{}", desc, zoom_meeting_info),
            _ => format!("Created by Ducktape 🦆{}", zoom_meeting_info),
        }
    } else {
        config.description.as_deref().unwrap_or("Created by Ducktape 🦆").to_string()
    };

    // Build extra properties (location)
    let mut extra = String::new();
    if let Some(loc) = &config.location {
        if !loc.is_empty() {
            extra.push_str(&format!(", location:\"{}\"", loc));
        }
    }

    // Build attendees block
    let mut attendees_block = String::new();
    if !config.emails.is_empty() {
        info!("Adding {} attendee(s): {}", config.emails.len(), config.emails.join(", "));
        for email in &config.emails {
            // Skip adding the calendar owner as attendee if it's the same as the calendar name
            // This avoids the issue where calendar owners don't appear as attendees
            if config.calendars.len() == 1 && config.calendars[0] == *email {
                debug!("Skipping calendar owner {} as explicit attendee", email);
                continue;
            }
            attendees_block.push_str(&format!(
                r#"
                    try
                        tell newEvent
                            make new attendee at end of attendees with properties {{email:"{}"}}
                        end tell
                    on error errMsg
                        log "Failed to add attendee {}: " & errMsg
                    end try"#,
                email, email
            ));
        }
    }

    // Build recurrence rule (RFC 5545 format)
    let recurrence_code = if let Some(recurrence) = &config.recurrence {
        let mut parts = vec![
            format!("FREQ={}", recurrence.frequency.to_rfc5545()),
            format!("INTERVAL={}", recurrence.interval),
        ];
        if let Some(count) = recurrence.count {
            parts.push(format!("COUNT={}", count));
        }
        if let Some(end_date) = &recurrence.end_date {
            let end_naive =
                NaiveDateTime::parse_from_str(&format!("{} 23:59", end_date), "%Y-%m-%d %H:%M")
                    .map_err(|e| anyhow!("Invalid recurrence end date: {}", e))?;
            parts.push(format!("UNTIL={}", end_naive.format("%Y%m%dT%H%M%SZ")));
        }
        if recurrence.frequency == RecurrenceFrequency::Weekly
            && !recurrence.days_of_week.is_empty()
        {
            let days: Vec<&str> = recurrence
                .days_of_week
                .iter()
                .map(|&d| match d {
                    0 => "SU",
                    1 => "MO",
                    2 => "TU",
                    3 => "WE",
                    4 => "TH",
                    5 => "FR",
                    6 => "SA",
                    _ => "MO",
                })
                .collect();
            parts.push(format!("BYDAY={}", days.join(",")));
        }
        let rule_string = parts.join(";");
        format!(
            r#"
                    tell newEvent
                        set its recurrence to "{}"
                    end tell"#,
            rule_string
        )
    } else {
        String::new()
    };

    // Generate AppleScript
    let script = format!(
        r#"tell application "Calendar"
            try
                if not (exists calendar "{calendar_name}") then
                    error "Calendar '{calendar_name}' not found"
                end if
                tell calendar "{calendar_name}"
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
                    set newEvent to make new event with properties {{summary:"{title}", start date:startDate, end date:endDate, description:"{description}"{extra}}}
                    {all_day_code}
                    {reminder_code}
                    {recurrence_code}
                    {attendees_block}
                    save
                end tell
                reload calendars
                return "Success: Event created"
            on error errMsg
                log errMsg
                error "Failed to create event: " & errMsg
            end try
        end tell"#,
        calendar_name = config.calendars[0],
        title = config.title,
        description = full_description,
        start_year = local_start.format("%Y"),
        start_month = local_start.format("%-m"),
        start_day = local_start.format("%-d"),
        start_hours = local_start.format("%-H"),
        start_minutes = local_start.format("%-M"),
        end_year = end_dt.format("%Y"),
        end_month = end_dt.format("%-m"),
        end_day = end_dt.format("%-d"),
        end_hours = end_dt.format("%-H"),
        end_minutes = end_dt.format("%-M"),
        extra = extra,
        all_day_code = if config.all_day { "set allday event of newEvent to true" } else { "" },
        reminder_code = if let Some(minutes) = config.reminder {
            format!(
                r#"set theAlarm to make new display alarm at end of newEvent
                    set trigger interval of theAlarm to -{}"#,
                minutes * 60
            )
        } else {
            String::new()
        },
        recurrence_code = recurrence_code,
        attendees_block = attendees_block,
    );

    debug!("Generated AppleScript:\n{}", script);

    // Execute AppleScript
    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);

    if result.contains("Success") {
        info!(
            "Calendar event created: {} at {}",
            config.title,
            local_start.format("%Y-%m-%d %H:%M")
        );
        Ok(())
    } else {
        error!("AppleScript error: STDOUT: {} | STDERR: {}", result, error_output);
        Err(anyhow!("Failed to create event: {}", error_output))
    }
}

async fn ensure_calendar_running() -> Result<()> {
    let check_script = r#"tell application "Calendar"
        if it is not running then
            launch
            delay 1
        end if
        return true
    end tell"#;

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(check_script)
        .output()
        .await
        .map_err(|e| CalendarError::ScriptError(e.to_string()))?;

    if output.status.success() { Ok(()) } else { Err(CalendarError::NotRunning.into()) }
}

/// Lookup a contact by name and return their email addresses
pub async fn lookup_contact(name: &str) -> Result<Vec<String>> {
    debug!("Looking up contact: {}", name);
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

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await
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
