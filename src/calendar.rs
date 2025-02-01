use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::debug;
use std::process::Command;

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

pub fn create_event(
    title: &str,
    date: &str,
    time: &str,
    calendar_id: Option<&str>,
    all_day: bool,
    location: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let datetime = format!("{} {}", date, if all_day { "00:00" } else { time });
    let dt = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M")
        .map_err(|_| anyhow!("Invalid date/time format. Please use YYYY-MM-DD HH:MM format"))?;

    let local_dt: DateTime<Local> = Local::now()
        .timezone()
        .from_local_datetime(&dt)
        .single()
        .ok_or_else(|| anyhow!("Invalid or ambiguous local time"))?;

    let tz_name = local_dt.offset().to_string();

    // First verify Calendar.app is running
    let check_script = r#"tell application "Calendar" to if it is running then return true"#;
    let check = Command::new("osascript")
        .arg("-e")
        .arg(check_script)
        .output()?;

    if !check.status.success() {
        return Err(anyhow!("Please ensure Calendar.app is running"));
    }

    // Use simple duration in seconds: 86400 for all-day, 3600 otherwise.
    let duration = if all_day { 86400 } else { 3600 };

    // Build extras for properties: include location if non-empty.
    let mut extra = String::new();
    if let Some(loc) = &location {
        if !loc.is_empty() {
            extra.push_str(&format!(", location:\"{}\"", loc));
        }
    }

    // Set up a separate code block for marking the event as an all-day event.
    let all_day_code = if all_day {
        "\n                set allday event of newEvent to true"
    } else {
        ""
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
                set props to {{summary:"{}", start date:startDate, end date:(startDate + {}), description:"{}"{}}}
                tell targetCal
                    set newEvent to make new event at end with properties props
                    {}
                end tell
                return "Success: Event created"
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
        calendar_id.unwrap_or("Calendar"),
        local_dt.format("%Y"),
        local_dt.format("%-m"),
        local_dt.format("%-d"),
        local_dt.format("%-H"),
        local_dt.format("%-M"),
        title,
        duration,
        description.as_deref().unwrap_or("Created by DuckTape"),
        extra,
        all_day_code
    );

    println!("Debug: Generated AppleScript:\n{}", script);
    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);
    if result.contains("Success") {
        println!(
            "Calendar event created: {} at {} ({} timezone)",
            title,
            format!("{} {}", date, time),
            local_dt.offset()
        );
        Ok(())
    } else {
        if let Some(cal_id) = calendar_id {
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

pub fn list_event_properties() -> Result<()> {
    // First verify Calendar.app is running
    let check_script = r#"tell application "Calendar" to if it is running then return true"#;
    let check = Command::new("osascript")
        .arg("-e")
        .arg(check_script)
        .output()?;

    if !check.status.success() {
        return Err(anyhow!("Please ensure Calendar.app is running"));
    }

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
