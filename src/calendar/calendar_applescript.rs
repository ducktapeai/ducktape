//! AppleScript and Calendar.app integration for DuckTape calendar module.
//
// This module provides async functions for interacting with macOS Calendar.app via AppleScript.

use crate::calendar::{EventConfig, RecurrenceFrequency};
use crate::zoom::{ZoomClient, ZoomMeetingOptions, calculate_meeting_duration, format_zoom_time};
use anyhow::{Result, anyhow};
use chrono::Datelike;
use chrono::TimeZone; // Keep for Local.from_local_datetime
use chrono::{Local, NaiveDateTime, NaiveTime, Timelike}; // Added Timelike
use log::{debug, error, info};
use std::process::Command;
use std::str::FromStr;

/// Ensure Calendar.app is running
pub async fn ensure_calendar_running() -> Result<()> {
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
        .map_err(|e| anyhow!(e.to_string()))?;

    if output.status.success() { Ok(()) } else { Err(anyhow!("Calendar is not running")) }
}

/// List all calendars
pub async fn list_calendars() -> Result<()> {
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
            let mut unique_calendars: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for calendar in calendars.trim_matches('{').trim_matches('}').split(", ") {
                unique_calendars.insert(calendar.trim_matches('"').to_string());
            }
            let mut sorted_calendars: Vec<_> = unique_calendars.into_iter().collect();
            sorted_calendars.sort();
            for calendar in sorted_calendars {
                println!("  - {}", calendar);
            }
        }
        Ok(())
    } else {
        Err(anyhow!("Failed to list calendars: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

/// Get available calendars
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

/// Create a single event in Calendar.app
pub async fn create_single_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config (times expected to be local): {:?}", config);

    let start_datetime_str = format!(
        "{} {}",
        config.start_date,
        if config.all_day { "00:00" } else { &config.start_time }
    );
    debug!("Parsing start datetime from config (expected to be local): {}", start_datetime_str);
    let naive_start_dt = NaiveDateTime::parse_from_str(&start_datetime_str, "%Y-%m-%d %H:%M")
        .map_err(|e| anyhow!("Invalid start datetime from config: {}", e))?;

    // Assume NaiveDateTime from config is in the user's local timezone.
    let local_start = Local.from_local_datetime(&naive_start_dt).single()
        .ok_or_else(|| anyhow!("Failed to interpret config start time {} as local system time", naive_start_dt))?;

    if let Some(original_tz_str) = config.timezone.as_deref() {
        info!(
            "Event time was originally specified in timezone: {}. Parsed as local time: {} (Raw config date: {}, time: {})",
            original_tz_str, local_start, config.start_date, config.start_time
        );
    } else {
        info!("Event time parsed as local time: {} (Raw config date: {}, time: {})", local_start, config.start_date, config.start_time);
    }
    // Log the components that were parsed to form naive_start_dt for clarity
    info!(
        "Using explicitly parsed date components from config for local_start: year={}, month={} ({}), day={}, time={}:{}, raw_date={}, raw_time={}",
        naive_start_dt.year(),
        naive_start_dt.month(),
        match naive_start_dt.month() {
            1 => "January", 2 => "February", 3 => "March", 4 => "April", 5 => "May", 6 => "June",
            7 => "July", 8 => "August", 9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "Unknown",
        },
        naive_start_dt.day(),
        naive_start_dt.hour(), naive_start_dt.minute(),
        config.start_date, config.start_time
    );


    let end_dt = if let Some(ref end_time_str) = config.end_time {
        let naive_start_time_obj = NaiveTime::parse_from_str(&config.start_time, "%H:%M")
            .map_err(|e| anyhow!("Invalid start_time format for comparison: {} - {}", config.start_time, e))?;
        let naive_end_time_obj = NaiveTime::parse_from_str(end_time_str, "%H:%M")
            .map_err(|e| anyhow!("Invalid end_time format for comparison: {} - {}", end_time_str, e))?;

        // Check if start and end times from config are identical.
        // Note: config.start_date is used for both, so we only compare times.
        if naive_start_time_obj == naive_end_time_obj {
            debug!(
                "End time {} is identical to start time {}. Defaulting to 1-hour duration from local_start.",
                end_time_str, config.start_time
            );
            local_start + chrono::Duration::hours(1)
        } else {
            // Construct NaiveDateTime for the end time using config.start_date and end_time_str.
            // These are assumed to be components of a local time.
            let naive_end_dt_candidate = NaiveDateTime::parse_from_str(
                &format!("{} {}", config.start_date, end_time_str), "%Y-%m-%d %H:%M"
            ).map_err(|e| anyhow!("Invalid NaiveDateTime from end_time {} on start_date {}: {}", end_time_str, config.start_date, e))?;

            let final_naive_end_dt = if naive_end_dt_candidate.time() < naive_start_dt.time() {
                // End time is earlier than start time (e.g., 23:00 to 02:00), implies crossing midnight.
                // This uses naive_start_dt which is derived from config.start_date and config.start_time.
                debug!(
                    "End time {} on (local) start date {} is earlier than (local) start time {}. Assuming event crosses midnight to the next day.",
                    end_time_str, config.start_date, config.start_time
                );
                let next_day_date = naive_start_dt.date().succ_opt()
                    .ok_or_else(|| anyhow!("Failed to calculate next day for event crossing midnight"))?;
                NaiveDateTime::new(next_day_date, naive_end_dt_candidate.time())
            } else {
                // End time is after start time, on the same day.
                naive_end_dt_candidate
            };

            // Convert final_naive_end_dt (which is a local naive datetime) to DateTime<Local>
            Local.from_local_datetime(&final_naive_end_dt).single()
                .ok_or_else(|| anyhow!("Failed to interpret final naive end time {} as local system time", final_naive_end_dt))?
        }
    } else {
        // No config.end_time, default to 1 hour from local_start.
        debug!("No end time specified, using local_start + 1 hour.");
        local_start + chrono::Duration::hours(1)
    };

    // Re-assign end_dt if the safeguard was triggered or if it's not after local_start.
    let end_dt = if end_dt <= local_start {
        error!(
            "Calculated end_dt {} was not after local_start {}. Forcing 1-hour duration. Review config: start_date={}, start_time={}, end_time={:?}",
            end_dt, local_start, config.start_date, config.start_time, config.end_time
        );
        local_start + chrono::Duration::hours(1)
    } else {
        end_dt
    };

    debug!("Final local start time: {}", local_start.format("%Y-%m-%d %H:%M"));
    debug!("Final end time: {}", end_dt.format("%Y-%m-%d %H:%M"));
    let mut zoom_meeting_info = String::new();
    if config.create_zoom_meeting {
        info!("Creating Zoom meeting for event: {}", config.title);
        let mut client = ZoomClient::new()?;
        let zoom_start_time = format_zoom_time(&config.start_date, &config.start_time)?;
        
        // Calculate duration for Zoom based on the final local_start and end_dt
        let meeting_duration_minutes = (end_dt - local_start).num_minutes();
        let zoom_api_duration = if meeting_duration_minutes <= 0 {
            debug!("Calculated meeting duration for Zoom is zero or negative ({} minutes). Defaulting Zoom duration to 60 minutes.", meeting_duration_minutes);
            60 // Default to 60 minutes if duration is zero/negative
        } else {
            meeting_duration_minutes as u32
        };

        let meeting_options = ZoomMeetingOptions {
            topic: config.title.to_string(),
            start_time: zoom_start_time,
            duration: zoom_api_duration, // Use calculated duration
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
    let full_description = if !zoom_meeting_info.is_empty() {
        match &config.description {
            Some(desc) if !desc.is_empty() => format!("{}{}", desc, zoom_meeting_info),
            _ => format!("Created by Ducktape 🦆{}", zoom_meeting_info),
        }
    } else {
        config.description.as_deref().unwrap_or("Created by Ducktape 🦆").to_string()
    };
    let mut extra = String::new();
    if let Some(loc) = &config.location {
        if !loc.is_empty() {
            extra.push_str(&format!(", location:\"{}\"", loc));
        }
    }
    let mut attendees_block = String::new();
    if !config.emails.is_empty() {
        info!("Adding {} attendee(s): {}", config.emails.len(), config.emails.join(", "));
        for email in &config.emails {
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
    let script = format!(
        r#"tell application "Calendar"
            try
                set calFound to false
                repeat with cal in calendars
                    if name of cal is "{calendar_name}" then
                        set calFound to true
                        tell cal
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
                        end tell
                        exit repeat
                    end if
                end repeat
                
                if not calFound then
                    error "Calendar '{calendar_name}' not found in available calendars"
                end if
                
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

/// List event properties
pub async fn list_event_properties() -> Result<()> {
    // ...implementation moved from calendar.rs...
    Ok(())
}

/// Delete an event by title and date (placeholder implementation)
pub async fn delete_event(_title: &str, _date: &str) -> Result<()> {
    // TODO: Implement event deletion
    println!("Event deletion not yet implemented");
    Ok(())
}
