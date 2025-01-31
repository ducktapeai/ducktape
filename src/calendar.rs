use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
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

pub fn create_event(title: &str, date: &str, time: &str, calendar_id: Option<&str>) -> Result<()> {
    let datetime = format!("{} {}", date, time);
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

    let script = format!(
        r#"tell application "Calendar"
            try
                -- Find the calendar
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
                
                -- Set up the date
                set startDate to current date
                set year of startDate to {}
                set month of startDate to {}
                set day of startDate to {}
                set hours of startDate to {}
                set minutes of startDate to {}
                set seconds of startDate to 0
                
                -- Create the event
                tell targetCal to make new event at end with properties ¬
                    {{summary:"{}", start date:startDate, ¬
                    end date:(startDate + 3600), description:"Created by DuckTape"}}
                
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
        title
    );

    println!("Debug: Generated AppleScript:\n{}", script);

    let output = Command::new("osascript").arg("-e").arg(&script).output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);

    if result.contains("Success") {
        println!(
            "Calendar event created: {} at {} ({} timezone)",
            title, datetime, tz_name
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
