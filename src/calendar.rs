use crate::config::Config;
use crate::state::{CalendarItem, StateManager};
use crate::zoom::{ZoomClient, ZoomMeetingOptions, calculate_meeting_duration, format_zoom_time};
use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use log::{debug, error, info};
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

// We need these imports for CSV and ICS processing
use csv::ReaderBuilder;
use ical::parser::ical::IcalParser;
use ical::parser::ical::component::IcalEvent;

/// Remove unused constants
// const ALL_DAY_DURATION: i64 = 86400;
// const DEFAULT_DURATION: i64 = 3600;

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

/// Recurrence frequency for repeating events
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceFrequency {
    #[allow(dead_code)]
    pub fn to_applescript(&self) -> &'static str {
        match self {
            RecurrenceFrequency::Daily => "daily",
            RecurrenceFrequency::Weekly => "weekly",
            RecurrenceFrequency::Monthly => "monthly",
            RecurrenceFrequency::Yearly => "yearly",
        }
    }

    /// Convert to RFC 5545 format for iCalendar
    pub fn to_rfc5545(&self) -> &'static str {
        match self {
            RecurrenceFrequency::Daily => "DAILY",
            RecurrenceFrequency::Weekly => "WEEKLY",
            RecurrenceFrequency::Monthly => "MONTHLY",
            RecurrenceFrequency::Yearly => "YEARLY",
        }
    }

    /// Parse recurrence frequency from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "daily" | "day" | "days" => Ok(RecurrenceFrequency::Daily),
            "weekly" | "week" | "weeks" => Ok(RecurrenceFrequency::Weekly),
            "monthly" | "month" | "months" => Ok(RecurrenceFrequency::Monthly),
            "yearly" | "year" | "years" | "annual" | "annually" => Ok(RecurrenceFrequency::Yearly),
            _ => Err(anyhow!("Invalid recurrence frequency: {}", s)),
        }
    }
}

/// Recurrence pattern for calendar events
#[derive(Debug, Clone)]
pub struct RecurrencePattern {
    /// Frequency of recurrence
    pub frequency: RecurrenceFrequency,

    /// Interval between occurrences (e.g., every 2 weeks)
    pub interval: u32,

    /// End date of recurrence (None for indefinite)
    pub end_date: Option<String>,

    /// Number of occurrences (None for indefinite)
    pub count: Option<u32>,

    /// Days of the week for weekly recurrence (0=Sunday, 1=Monday, etc.)
    pub days_of_week: Vec<u8>,
}

impl RecurrencePattern {
    /// Create a new simple recurrence pattern with the given frequency
    pub fn new(frequency: RecurrenceFrequency) -> Self {
        Self { frequency, interval: 1, end_date: None, count: None, days_of_week: Vec::new() }
    }

    /// Set the interval for recurrence
    pub fn with_interval(mut self, interval: u32) -> Self {
        self.interval = interval;
        self
    }

    /// Set the end date for recurrence
    pub fn with_end_date(mut self, end_date: &str) -> Self {
        self.end_date = Some(end_date.to_string());
        self
    }

    /// Set the count of occurrences
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Set the days of week for weekly recurrence
    pub fn with_days_of_week(mut self, days: &[u8]) -> Self {
        self.days_of_week = days.to_vec();
        self
    }
}

/// Configuration for a calendar event
#[derive(Debug, Clone)]
pub struct EventConfig {
    pub title: String,
    pub start_date: String,
    pub start_time: String,
    #[allow(dead_code)]
    pub end_date: Option<String>,
    pub end_time: Option<String>,
    pub calendars: Vec<String>,
    pub all_day: bool,
    pub location: Option<String>,
    pub description: Option<String>,
    pub emails: Vec<String>,
    pub reminder: Option<i32>,
    pub timezone: Option<String>,
    pub recurrence: Option<RecurrencePattern>,
    // Enhanced Zoom integration fields
    pub create_zoom_meeting: bool,
    #[allow(dead_code)]
    pub zoom_meeting_id: Option<u64>,
    #[allow(dead_code)]
    pub zoom_join_url: Option<String>,
    #[allow(dead_code)]
    pub zoom_password: Option<String>,
}

impl EventConfig {
    pub fn new(title: &str, date: &str, time: &str) -> Self {
        Self {
            title: title.to_string(),
            start_date: date.to_string(),
            start_time: time.to_string(),
            end_date: None,
            end_time: None,
            calendars: Vec::new(),
            all_day: false,
            location: None,
            description: None,
            emails: Vec::new(),
            reminder: None,
            timezone: None,
            recurrence: None,
            // Initialize Zoom fields
            create_zoom_meeting: false,
            zoom_meeting_id: None,
            zoom_join_url: None,
            zoom_password: None,
        }
    }

    // Method to easily set recurrence pattern
    #[allow(dead_code)]
    pub fn with_recurrence(mut self, recurrence: RecurrencePattern) -> Self {
        self.recurrence = Some(recurrence);
        self
    }

    // Method to enable Zoom meeting
    #[allow(dead_code)]
    pub fn with_zoom_meeting(mut self, enable: bool) -> Self {
        self.create_zoom_meeting = enable;
        self
    }

    pub fn validate(&self) -> Result<()> {
        // Validate date format (YYYY-MM-DD)
        if !validate_date_format(&self.start_date) {
            return Err(CalendarError::InvalidDateTime(format!(
                "Invalid date format: {}",
                self.start_date
            ))
            .into());
        }

        // Validate time format (HH:MM)
        if !validate_time_format(&self.start_time) {
            return Err(CalendarError::InvalidDateTime(format!(
                "Invalid time format: {}",
                self.start_time
            ))
            .into());
        }

        // Validate end time if specified
        if let Some(end_time) = &self.end_time {
            if !validate_time_format(end_time) {
                return Err(CalendarError::InvalidDateTime(format!(
                    "Invalid end time format: {}",
                    end_time
                ))
                .into());
            }
        }

        // Validate title doesn't contain dangerous characters
        if contains_dangerous_characters(&self.title) {
            return Err(anyhow!("Title contains potentially dangerous characters"));
        }

        // Validate location if specified
        if let Some(location) = &self.location {
            if contains_dangerous_characters(location) {
                return Err(anyhow!("Location contains potentially dangerous characters"));
            }
        }

        // Validate description if specified
        if let Some(description) = &self.description {
            if contains_dangerous_chars_for_script(description) {
                return Err(anyhow!("Description contains potentially dangerous characters"));
            }
        }

        // Validate emails
        for email in &self.emails {
            if !validate_email(email) {
                return Err(anyhow!("Invalid email format: {}", email));
            }
        }

        // Validate timezone if specified
        if let Some(timezone) = &self.timezone {
            // Basic validation - more comprehensive would check against a list of valid timezones
            if timezone.len() > 50 || contains_dangerous_chars_for_script(timezone) {
                return Err(anyhow!("Invalid timezone format"));
            }
        }

        // Validate recurrence if specified
        if let Some(recurrence) = &self.recurrence {
            if let Some(end_date) = &recurrence.end_date {
                if !validate_date_format(end_date) {
                    return Err(anyhow!("Invalid recurrence end date format: {}", end_date));
                }
            }
        }

        // If creating a Zoom meeting, validate needed fields
        if self.create_zoom_meeting {
            // Zoom requires an end time for calculating meeting duration
            if self.end_time.is_none() {
                return Err(anyhow!("End time is required for Zoom meetings"));
            }
        }

        Ok(())
    }
}

/// Validate date string has format YYYY-MM-DD
pub fn validate_date_format(date: &str) -> bool {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

    if !re.is_match(date) {
        return false;
    }

    // Further validate the date is reasonable
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        // Check date is within reasonable range
        let year = naive_date.year();
        return year >= 2000 && year <= 2100;
    }

    false
}

/// Validate time string has format HH:MM
pub fn validate_time_format(time: &str) -> bool {
    let re = Regex::new(r"^\d{1,2}:\d{2}$").unwrap();

    if !re.is_match(time) {
        return false;
    }

    // Further validate the time values
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
        return hours < 24 && minutes < 60;
    }

    false
}

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[A-Za-z0-9._%+-]{1,64}@(?:[A-Za-z0-9-]{1,63}\.){1,125}[A-Za-z]{2,63}$")
        .unwrap();

    if !re.is_match(email) {
        return false;
    }

    // Check for dangerous characters that could cause script injection
    !contains_dangerous_characters(email)
}

/// Check for potentially dangerous characters that could cause AppleScript injection
fn contains_dangerous_characters(input: &str) -> bool {
    input.contains('\"')
        || input.contains(';')
        || input.contains('&')
        || input.contains('|')
        || input.contains('<')
        || input.contains('>')
        || input.contains('$')
}

/// Check for characters that could break AppleScript specifically
fn contains_dangerous_chars_for_script(input: &str) -> bool {
    input.contains('\"') || input.contains('\\') || input.contains('¬')
}

pub async fn list_calendars() -> anyhow::Result<()> {
    use crate::calendar_legacy::fetch_calendars;
    
    log::info!("Fetching available calendars");
    let calendars = fetch_calendars()?;
    
    if calendars.is_empty() {
        println!("No calendars found");
    } else {
        println!("Available calendars:");
        for calendar in calendars {
            println!("- {}", calendar);
        }
    }
    
    Ok(())
}

/// List available event properties
pub async fn list_event_properties() -> anyhow::Result<()> {
    println!("Available event properties:");
    println!("- title: Title of the event");
    println!("- start_date: Start date (YYYY-MM-DD)");
    println!("- start_time: Start time (HH:MM)");
    println!("- end_date: End date (YYYY-MM-DD)");
    println!("- end_time: End time (HH:MM)");
    println!("- calendar: Calendar name");
    println!("- description: Event description");
    println!("- location: Event location");
    println!("- url: Event URL");
    println!("- all_day: Whether the event is all-day (true/false)");
    
    Ok(())
}

pub async fn create_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);

    // Validate the event configuration first
    config.validate()?;

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

    // Parse start datetime with timezone handling
    let start_datetime = format!(
        "{} {}",
        config.start_date,
        if config.all_day { "00:00" } else { &config.start_time }
    );

    let start_dt = NaiveDateTime::parse_from_str(&start_datetime, "%Y-%m-%d %H:%M")
        .map_err(|e| anyhow!("Invalid start datetime: {}", e))?;

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

    // Parse end datetime with similar timezone handling
    let end_dt = if let Some(ref end_time) = config.end_time {
        let end_datetime = format!("{} {}", config.start_date, end_time);
        let naive_end = NaiveDateTime::parse_from_str(&end_datetime, "%Y-%m-%d %H:%M")
            .map_err(|e| anyhow!("Invalid end datetime: {}", e))?;

        if let Some(tz_str) = config.timezone.as_deref() {
            match Tz::from_str(tz_str) {
                Ok(tz) => {
                    let tz_dt = tz.from_local_datetime(&naive_end).single().ok_or_else(|| {
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
    } else {
        local_start + chrono::Duration::hours(1)
    };

    if end_dt <= local_start {
        return Err(anyhow!("End time must be after start time"));
    }

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
        info!("Calendar event created: {} at {}", config.title, start_datetime);
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

/// Delete a calendar event based on title and date
pub async fn delete_event(title: &str, date: &str) -> Result<()> {
    info!("Attempting to delete event: {} on {}", title, date);
    
    // Validate inputs
    if !validate_date_format(date) {
        return Err(anyhow!("Invalid date format: {}", date));
    }
    
    // Sanitize inputs to prevent AppleScript injection
    let sanitized_title = title.replace('\"', "\\\"");
    let sanitized_date = date.replace('\"', "\\\"");
    
    // Script to delete the event
    let script = format!(
        r#"tell application "Calendar"
            try
                set searchDate to date "{date}"
                set eventsToDelete to (events whose summary is "{title}" and start date is greater than or equal to searchDate and start date is less than or equal to (searchDate + 1 * days))
                if length of eventsToDelete = 0 then
                    error "No events found matching title '{title}' on date {date}"
                end if

                repeat with evt in eventsToDelete
                    try
                        delete evt
                    on error errMsg
                        error "Failed to delete event: " & errMsg
                    end try
                end repeat

                return "Successfully deleted events matching '{title}' on {date}"
            on error errMsg
                error "Error deleting event: " & errMsg
            end try
        end tell"#,
        title = sanitized_title,
        date = sanitized_date
    );
    
    // Execute the AppleScript
    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if output.status.success() {
        info!("Successfully deleted event: {} on {}", title, date);
        println!("{}", stdout);
        Ok(())
    } else {
        error!("Failed to delete event: {} on {}: {}", title, date, stderr);
        Err(anyhow!("Failed to delete event: {}", stderr))
    }
}

/// Import events from a CSV file
pub async fn import_csv_events(file_path: &Path, calendar_name: Option<String>) -> Result<()> {
    info!("Importing events from CSV file: {:?}", file_path);
    
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(file);
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    let app_config = Config::load()?;
    let target_calendar = calendar_name.unwrap_or_else(|| 
        app_config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string())
    );
    
    for result in reader.records() {
        let record = match result {
            Ok(rec) => rec,
            Err(e) => {
                error!("Error reading CSV record: {}", e);
                error_count += 1;
                continue;
            }
        };
        
        // Extract fields from CSV - handle potential missing fields gracefully
        let title = record.get(0).unwrap_or("Untitled Event").trim().to_string();
        let date = record.get(1).unwrap_or("").trim();
        let start_time = record.get(2).unwrap_or("").trim();
        let end_time = record.get(3).map(|s| s.trim());
        let description = record.get(4).and_then(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        });
        let location = record.get(5).and_then(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        });
        let all_day = record.get(6)
            .map(|s| s.trim().to_lowercase() == "true" || s.trim() == "1")
            .unwrap_or(false);
        
        // Validate required fields
        if !validate_date_format(date) || (!all_day && !validate_time_format(start_time)) {
            error!("Invalid date/time format in CSV record: date={}, time={}", date, start_time);
            error_count += 1;
            continue;
        }
        
        // Create event config
        let mut config = EventConfig::new(&title, date, if all_day { "00:00" } else { start_time });
        config.calendars = vec![target_calendar.clone()];
        config.all_day = all_day;
        config.description = description;
        config.location = location;
        
        // Set end time if provided
        if let Some(end_time) = end_time {
            if !all_day && validate_time_format(end_time) {
                config.end_time = Some(end_time.to_string());
            }
        }
        
        // Create the event
        match create_event(config).await {
            Ok(_) => {
                info!("Successfully imported event: {}", title);
                success_count += 1;
            }
            Err(e) => {
                error!("Failed to import event '{}': {}", title, e);
                error_count += 1;
            }
        }
    }
    
    info!("CSV import complete: {} events imported, {} errors", success_count, error_count);
    
    if success_count > 0 {
        println!("Successfully imported {} events from CSV", success_count);
        if error_count > 0 {
            println!("Encountered {} errors during import", error_count);
        }
        Ok(())
    } else {
        Err(anyhow!("Failed to import any events from CSV file"))
    }
}

/// Import events from an ICS (iCalendar) file
pub async fn import_ics_events(file_path: &Path, calendar_name: Option<String>) -> Result<()> {
    info!("Importing events from ICS file: {:?}", file_path);
    
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    let parser = IcalParser::new(buf_reader);
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    let app_config = Config::load()?;
    let target_calendar = calendar_name.unwrap_or_else(|| 
        app_config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string())
    );
    
    for calendar in parser {
        match calendar {
            Ok(cal) => {
                for component in cal.events {
                    match import_ical_event(&component, &target_calendar).await {
                        Ok(_) => {
                            success_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to import ICS event: {}", e);
                            error_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error parsing ICS file: {}", e);
                error_count += 1;
            }
        }
    }
    
    info!("ICS import complete: {} events imported, {} errors", success_count, error_count);
    
    if success_count > 0 {
        println!("Successfully imported {} events from ICS", success_count);
        if error_count > 0 {
            println!("Encountered {} errors during import", error_count);
        }
        Ok(())
    } else {
        Err(anyhow!("Failed to import any events from ICS file"))
    }
}

/// Import a single event from an iCal component
async fn import_ical_event(event: &IcalEvent, calendar_name: &str) -> Result<()> {
    // Extract summary (title)
    let title = event.properties.iter()
        .find(|p| p.name == "SUMMARY")
        .and_then(|p| p.value.clone())
        .unwrap_or_else(|| "Untitled Event".to_string());
    
    // Extract start date/time
    let dtstart = event.properties.iter()
        .find(|p| p.name == "DTSTART")
        .and_then(|p| p.value.clone())
        .ok_or_else(|| anyhow!("Event missing DTSTART property"))?;
    
    // Check if it's an all-day event (no time component)
    // Simplest approach: just check if the DTSTART is 8 characters (YYYYMMDD) or longer
    let all_day = dtstart.len() <= 8;
    
    // Parse date and time
    let (date, time) = if all_day {
        // Just use the date portion if available, or try to parse the basic format
        if dtstart.len() >= 8 {
            (format!("{}-{}-{}", &dtstart[0..4], &dtstart[4..6], &dtstart[6..8]), "00:00".to_string())
        } else {
            return Err(anyhow!("Invalid DTSTART format for all-day event"));
        }
    } else {
        // Extract and format date and time
        if dtstart.len() >= 15 { // Basic format: YYYYMMDDTHHMMSS
            (
                format!("{}-{}-{}", &dtstart[0..4], &dtstart[4..6], &dtstart[6..8]),
                format!("{}:{}", &dtstart[9..11], &dtstart[11..13])
            )
        } else {
            return Err(anyhow!("Invalid DTSTART format: {}", dtstart));
        }
    };
    
    // Extract end date/time
    let end_time = event.properties.iter()
        .find(|p| p.name == "DTEND")
        .and_then(|p| p.value.clone())
        .and_then(|dtend| {
            if all_day || dtend.len() < 15 {
                None
            } else {
                Some(format!("{}:{}", &dtend[9..11], &dtend[11..13]))
            }
        });
    
    // Extract description
    let description = event.properties.iter()
        .find(|p| p.name == "DESCRIPTION")
        .and_then(|p| p.value.clone());
    
    // Extract location
    let location = event.properties.iter()
        .find(|p| p.name == "LOCATION")
        .and_then(|p| p.value.clone())
        .filter(|s| !s.is_empty());
    
    // Create event config
    let mut config = EventConfig::new(&title, &date, &time);
    config.calendars = vec![calendar_name.to_string()];
    config.all_day = all_day;
    config.description = description;
    config.location = location;
    config.end_time = end_time;
    
    // Create the event
    create_event(config).await
}

/// Function to create an event with contacts
pub async fn create_event_with_contacts(config: EventConfig, contact_names: &[&str]) -> Result<()> {
    info!("Creating event with contacts: {:?}", contact_names);
    
    // Look up emails for each contact name
    let mut found_emails = Vec::new();
    for name in contact_names {
        match lookup_contact(name).await {
            Ok(emails) => {
                if emails.is_empty() {
                    debug!("No email found for contact: {}", name);
                } else {
                    debug!("Found {} email(s) for contact {}: {:?}", emails.len(), name, emails);
                    found_emails.extend(emails.into_iter().map(|e| e.trim().to_string()));
                }
            }
            Err(e) => {
                error!("Failed to lookup contact {}: {}", name, e);
            }
        }
    }

    // Create a new config with the found emails
    let mut updated_config = EventConfig {
        emails: Vec::with_capacity(config.emails.len() + found_emails.len()),
        ..config.clone()
    };
    
    // Add existing emails from original config
    updated_config.emails.extend(config.emails.iter().cloned());
    
    // Add newly found emails
    updated_config.emails.extend(found_emails);

    // Deduplicate and clean emails
    updated_config.emails.sort_unstable();
    updated_config.emails.dedup();
    
    info!("Final email list for invitation: {:?}", updated_config.emails);

    // Create the event with the updated config
    create_event(updated_config).await
}

/// Look up contact emails by name in Apple Contacts
async fn lookup_contact(contact_name: &str) -> Result<Vec<String>> {
    // Sanitize the contact name for AppleScript
    let sanitized_name = contact_name.replace('\"', "\\\"");
    
    // AppleScript to fetch email addresses for the given contact
    let script = format!(
        r#"tell application "Contacts"
            try
                set matchingPeople to (every person whose name contains "{}")
                set emailList to {{}}
                
                repeat with onePerson in matchingPeople
                    set personEmails to every email of onePerson
                    repeat with oneEmail in personEmails
                        set end of emailList to value of oneEmail
                    end repeat
                end repeat
                
                return emailList
            on error errMsg
                error "Failed to get contacts: " & errMsg
            end try
        end tell"#,
        sanitized_name
    );
    
    debug!("Looking up contact: {}", contact_name);
    
    // Execute the AppleScript
    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await?;
    
    if output.status.success() {
        let emails_output = String::from_utf8_lossy(&output.stdout);
        
        // Parse the comma-separated list of emails
        let emails: Vec<String> = emails_output
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .filter(|s| !s.is_empty())
            .map(|s| s.trim_matches('"').to_string())
            .collect();
        
        debug!("Found {} email(s) for contact: {}", emails.len(), contact_name);
        Ok(emails)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Failed to lookup contact '{}': {}", contact_name, error);
        Err(anyhow!("Failed to lookup contact '{}': {}", contact_name, error))
    }
}
