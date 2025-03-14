use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone, DateTime};
use chrono_tz::Tz;
use log::{debug, error, info};
use regex::Regex;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::str::FromStr;
use ical::parser::ical::IcalParser;
use ical::parser::ical::component::IcalEvent;
use csv::ReaderBuilder;
use crate::config::Config;
use crate::state::{CalendarItem, StateManager};
use crate::zoom::{ZoomClient, ZoomMeetingOptions, format_zoom_time, calculate_meeting_duration};

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
        Self {
            frequency,
            interval: 1,
            end_date: None,
            count: None,
            days_of_week: Vec::new(),
        }
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
            return Err(CalendarError::InvalidDateTime(format!("Invalid date format: {}", self.start_date)).into());
        }
        
        // Validate time format (HH:MM)
        if !validate_time_format(&self.start_time) {
            return Err(CalendarError::InvalidDateTime(format!("Invalid time format: {}", self.start_time)).into());
        }
        
        // Validate end time if specified
        if let Some(end_time) = &self.end_time {
            if !validate_time_format(end_time) {
                return Err(CalendarError::InvalidDateTime(format!("Invalid end time format: {}", end_time)).into());
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
    let re = Regex::new(r"^[A-Za-z0-9._%+-]{1,64}@(?:[A-Za-z0-9-]{1,63}\.){1,125}[A-Za-z]{2,63}$").unwrap();
    
    if !re.is_match(email) {
        return false;
    }
    
    // Check for dangerous characters that could cause script injection
    !contains_dangerous_characters(email)
}

/// Check for potentially dangerous characters that could cause AppleScript injection
fn contains_dangerous_characters(input: &str) -> bool {
    input.contains('\'') || input.contains('\"') || input.contains('`') || 
    input.contains(';') || input.contains('&') || input.contains('|') ||
    input.contains('<') || input.contains('>') || input.contains('$')
}

/// Check for characters that could break AppleScript specifically
fn contains_dangerous_chars_for_script(input: &str) -> bool {
    input.contains('\"') || input.contains('\\') || input.contains('¬')
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

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await?;

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
        vec![app_config
            .calendar
            .default_calendar
            .unwrap_or_else(|| "Calendar".to_string())]
    } else {
        // Validate that specified calendars exist
        let requested: Vec<String> = config.calendars.iter().map(|s| s.to_string()).collect();
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

    // Clone the calendars Vec for state management
    let calendars_for_state = requested_calendars.clone();

    for calendar in requested_calendars {
        info!("Attempting to create event in calendar: {}", calendar);
        let this_config = EventConfig {
            calendars: vec![calendar.clone()],
            ..config.clone()
        };
        
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

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await?;

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
                let tz_dt = tz.from_local_datetime(&start_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Invalid or ambiguous start time in timezone {}", tz_str))?;
                tz_dt.with_timezone(&Local)
            },
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
                    let tz_dt = tz.from_local_datetime(&naive_end)
                        .single()
                        .ok_or_else(|| anyhow!("Invalid or ambiguous end time in timezone {}", tz_str))?;
                    tz_dt.with_timezone(&Local)
                },
                Err(_) => {
                    Local::now()
                        .timezone()
                        .from_local_datetime(&naive_end)
                        .single()
                        .ok_or_else(|| anyhow!("Invalid or ambiguous end time"))?
                }
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
                let password_info = meeting.password.map_or(String::new(), |p| format!("\nPassword: {}", p));
                zoom_meeting_info = format!(
                    "\n\n--------------------\nZoom Meeting\n--------------------\nJoin URL: {}{}",
                    meeting.join_url, password_info
                );
            },
            Err(e) => {
                error!("Failed to create Zoom meeting: {}", e);
                zoom_meeting_info = "\n\nNote: Zoom meeting creation failed.".to_string();
            }
        }
    } else if let Some(url) = &config.zoom_join_url {
        let password_info = config.zoom_password.as_ref().map_or(String::new(), |p| format!("\nPassword: {}", p));
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
    let mut recurrence_rule = String::new();
    let recurrence_code = if let Some(recurrence) = &config.recurrence {
        let mut parts = vec![
            format!("FREQ={}", recurrence.frequency.to_rfc5545()),
            format!("INTERVAL={}", recurrence.interval),
        ];
        
        if let Some(count) = recurrence.count {
            parts.push(format!("COUNT={}", count));
        }
        
        if let Some(end_date) = &recurrence.end_date {
            let end_naive = NaiveDateTime::parse_from_str(&format!("{} 23:59", end_date), "%Y-%m-%d %H:%M")
                .map_err(|e| anyhow!("Invalid recurrence end date: {}", e))?;
            parts.push(format!("UNTIL={}", end_naive.format("%Y%m%dT%H%M%SZ")));
        }
        
        if recurrence.frequency == RecurrenceFrequency::Weekly && !recurrence.days_of_week.is_empty() {
            let days: Vec<&str> = recurrence.days_of_week.iter().map(|&d| match d {
                0 => "SU", 1 => "MO", 2 => "TU", 3 => "WE", 4 => "TH", 5 => "FR", 6 => "SA",
                _ => "MO",
            }).collect();
            parts.push(format!("BYDAY={}", days.join(",")));
        }
        
        recurrence_rule = parts.join(";");
        format!(
            r#"
                    tell newEvent
                        set its recurrence to "{}"
                    end tell"#,
            recurrence_rule
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

    if output.status.success() {
        Ok(())
    } else {
        Err(CalendarError::NotRunning.into())
    }
}

pub async fn list_event_properties() -> Result<()> {
    ensure_calendar_running().await?;

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

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await?;

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

pub async fn delete_event(title: &str, _date: &str) -> Result<()> {
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

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        println!("Calendar event(s) deleted containing title: {}", title);
        Ok(())
    } else {
        Err(anyhow!("Failed to delete events: {}", result))
    }
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

/// Enhanced event creation with contact lookup
pub async fn create_event_with_contacts(config: EventConfig, contact_names: &[&str]) -> Result<()> {
    // Look up emails for each contact name
    let mut found_emails = Vec::new();
    for name in contact_names {
        match lookup_contact(name).await {
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

    // Create a new config with the found emails
    let mut config = EventConfig {
        emails: Vec::with_capacity(config.emails.len() + found_emails.len()),
        ..config
    };
    config.emails.extend(found_emails);
    
    // Deduplicate and clean emails
    config.emails.sort_unstable();
    config.emails.dedup();
    
    // Create the event with the updated config
    create_event(config).await
}

pub async fn import_csv_events(file_path: &Path, target_calendar: Option<String>) -> Result<()> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')  // Use comma as delimiter for CSV
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .comment(Some(b'#'))
        .from_reader(file);
    
    // Read headers
    let headers: Vec<String> = rdr.headers()?
        .iter()
        .map(|h| h.trim().to_lowercase())
        .collect();

    debug!("Found headers: {:?}", headers);
    
    // Validate required headers are present
    let required_headers = ["title", "date", "start_time"];
    for header in &required_headers {
        if !headers.contains(&header.to_string()) {
            return Err(anyhow!("Required header '{}' not found in CSV. Required headers: title, date, start_time", header));
        }
    }
    
    let header_positions: std::collections::HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.to_string(), i))
        .collect();

    let mut success_count = 0;
    let mut error_count = 0;

    // Get default calendar for fallback
    let app_config = Config::load()?;
    let default_calendar = app_config
        .calendar
        .default_calendar
        .unwrap_or_else(|| "Calendar".to_string());

    // Process each record
    for (row_num, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to read row {}: {}", row_num + 1, e);
                error_count += 1;
                continue;
            }
        };
        debug!("Processing record values: {:?}", record);

        // Extract required fields with safe access
        let title = match record.get(header_positions["title"]) {
            Some(t) if !t.trim().is_empty() => t.trim(),
            _ => {
                error!("Row {}: Missing or empty title", row_num + 1);
                error_count += 1;
                continue;
            }
        };

        let date = match record.get(header_positions["date"]) {
            Some(d) if validate_date_format(d.trim()) => d.trim(),
            _ => {
                error!("Row {}: Invalid or missing date format (required: YYYY-MM-DD)", row_num + 1);
                error_count += 1;
                continue;
            }
        };

        let start_time = match record.get(header_positions["start_time"]) {
            Some(t) if validate_time_format(t.trim()) => t.trim(),
            _ => {
                error!("Row {}: Invalid or missing time format (required: HH:MM)", row_num + 1);
                error_count += 1;
                continue;
            }
        };
        
        let mut config = EventConfig::new(title, date, start_time);

        // Set optional end time if present
        if let Some(&pos) = header_positions.get("end_time") {
            if let Some(end_time) = record.get(pos) {
                let end_time = end_time.trim();
                if !end_time.is_empty() {
                    if validate_time_format(end_time) {
                        config.end_time = Some(end_time.to_string());
                    } else {
                        error!("Row {}: Invalid end time format (required: HH:MM)", row_num + 1);
                        error_count += 1;
                        continue;
                    }
                }
            }
        }

        // Collect attendee emails
        let mut attendee_emails = Vec::new();

        // Handle calendar field - if it's an email, add as attendee
        if let Some(&pos) = header_positions.get("calendar") {
            if let Some(value) = record.get(pos) {
                let value = value.trim();
                if !value.is_empty() && validate_email(value) {
                    debug!("Found email in calendar field: {}", value);
                    attendee_emails.push(value.to_string());
                }
            }
        }

        // Handle attendees field
        if let Some(&pos) = header_positions.get("attendees") {
            if let Some(attendees) = record.get(pos) {
                if !attendees.is_empty() {
                    debug!("Processing attendees field: {}", attendees);
                    for email in attendees.split(|c| c == ';' || c == ',') {
                        let email = email.trim();
                        if !email.is_empty() && validate_email(email) {
                            debug!("Found valid email in attendees: {}", email);
                            attendee_emails.push(email.to_string());
                        }
                    }
                }
            }
        }

        // Set calendar - prefer command line target, then default
        if let Some(cal) = &target_calendar {
            debug!("Using command line calendar: {}", cal);
            config.calendars = vec![cal.to_string()];
        } else {
            debug!("Using default calendar: {}", default_calendar);
            config.calendars = vec![default_calendar.clone()];
        }

        // Set description
        if let Some(&pos) = header_positions.get("description") {
            if let Some(desc) = record.get(pos) {
                let desc = desc.trim();
                if !desc.is_empty() {
                    config.description = Some(desc.to_string());
                }
            }
        }

        // Set location
        if let Some(&pos) = header_positions.get("location") {
            if let Some(loc) = record.get(pos) {
                let loc = loc.trim();
                if !loc.is_empty() {
                    config.location = Some(loc.to_string());
                }
            }
        }

        // Set deduplicated attendees
        if !attendee_emails.is_empty() {
            attendee_emails.sort();
            attendee_emails.dedup();
            debug!("Setting {} attendees for event: {}", 
                attendee_emails.len(), 
                attendee_emails.join(", "));
            config.emails = attendee_emails;
        }

        // Create the event
        debug!("Creating event with config: {:?}", config);
        match create_event(config).await {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                error_count += 1;
                error!("Failed to import event: {}", e);
            }
        }
    }

    if success_count > 0 || error_count > 0 {
        println!("Import completed: {} events imported successfully, {} failed", success_count, error_count);
    } else {
        println!("No events found in the CSV file");
    }
    Ok(())
}

pub async fn import_ics_events(file_path: &Path, target_calendar: Option<String>) -> Result<()> {
    let buf = BufReader::new(File::open(file_path)?);
    let parser = IcalParser::new(buf);
    
    let mut success_count = 0;
    let mut error_count = 0;

    for calendar in parser {
        match calendar {
            Ok(cal) => {
                for event in cal.events {
                    if let Err(e) = import_ical_event(event, &target_calendar).await {
                        error_count += 1;
                        error!("Failed to import ICS event: {}", e);
                    } else {
                        success_count += 1;
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse ICS calendar: {}", e);
                return Err(anyhow!("Failed to parse ICS file: {}", e));
            }
        }
    }

    println!("Import completed: {} events imported successfully, {} failed", success_count, error_count);
    Ok(())
}

async fn import_ical_event(event: IcalEvent, target_calendar: &Option<String>) -> Result<()> {
    // Get required properties
    let summary = event.properties
        .iter()
        .find(|p| p.name == "SUMMARY")
        .and_then(|p| p.value.as_ref())
        .ok_or_else(|| anyhow!("Event missing SUMMARY"))?;

    let dt_start = event.properties
        .iter()
        .find(|p| p.name == "DTSTART")
        .and_then(|p| p.value.as_ref())
        .ok_or_else(|| anyhow!("Event missing DTSTART"))?;

    // Parse start datetime
    let start_dt = NaiveDateTime::parse_from_str(dt_start, "%Y%m%dT%H%M%S")?;
    
    let mut config = EventConfig::new(
        summary,
        &start_dt.format("%Y-%m-%d").to_string(),
        &start_dt.format("%H:%M").to_string(),
    );

    // Set target calendar if specified
    if let Some(cal) = target_calendar {
        config.calendars = vec![cal.to_string()];
    }

    // Get optional end time
    if let Some(dt_end) = event.properties
        .iter()
        .find(|p| p.name == "DTEND")
        .and_then(|p| p.value.as_ref()) {
        if let Ok(end_dt) = NaiveDateTime::parse_from_str(dt_end, "%Y%m%dT%H%M%S") {
            config.end_time = Some(end_dt.format("%H:%M").to_string());
        }
    }

    // Get description
    if let Some(desc) = event.properties
        .iter()
        .find(|p| p.name == "DESCRIPTION")
        .and_then(|p| p.value.as_ref()) {
        config.description = Some(desc.to_string());
    }

    // Get location
    if let Some(loc) = event.properties
        .iter()
        .find(|p| p.name == "LOCATION")
        .and_then(|p| p.value.as_ref()) {
        config.location = Some(loc.to_string());
    }

    // Get attendees
    let attendees: Vec<String> = event.properties
        .iter()
        .filter(|p| p.name == "ATTENDEE")
        .filter_map(|p| p.value.as_ref())
        .map(|v| v.trim_start_matches("mailto:").to_string())
        .collect();

    if !attendees.is_empty() {
        config.emails = attendees;
    }

    // Handle recurrence rule if present
    if let Some(rrule) = event.properties
        .iter()
        .find(|p| p.name == "RRULE")
        .and_then(|p| p.value.as_ref()) {
        if let Some(recurrence) = parse_ical_recurrence(rrule) {
            config.recurrence = Some(recurrence);
        }
    }

    create_event(config).await
}

fn parse_ical_recurrence(rrule: &str) -> Option<RecurrencePattern> {
    let parts = rrule.split(';')  // Removed mut as it's not needed
        .map(|s| s.split('='))
        .filter_map(|mut kv| Some((kv.next()?, kv.next()?)));

    let mut frequency = None;
    let mut interval = 1;
    let mut end_date = None;
    let mut count = None;
    let mut days = Vec::new();

    for (key, value) in parts {
        match key {
            "FREQ" => {
                frequency = match value {
                    "DAILY" => Some(RecurrenceFrequency::Daily),
                    "WEEKLY" => Some(RecurrenceFrequency::Weekly),
                    "MONTHLY" => Some(RecurrenceFrequency::Monthly),
                    "YEARLY" => Some(RecurrenceFrequency::Yearly),
                    _ => None
                };
            },
            "INTERVAL" => {
                if let Ok(val) = value.parse() {
                    interval = val;
                }
            },
            "UNTIL" => {
                if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%SZ") {
                    end_date = Some(dt.format("%Y-%m-%d").to_string());
                }
            },
            "COUNT" => {
                if let Ok(val) = value.parse() {
                    count = Some(val);
                }
            },
            "BYDAY" => {
                days = value.split(',')
                    .filter_map(|day| match day {
                        "SU" => Some(0),
                        "MO" => Some(1),
                        "TU" => Some(2),
                        "WE" => Some(3),
                        "TH" => Some(4),
                        "FR" => Some(5),
                        "SA" => Some(6),
                        _ => None
                    })
                    .collect();
            },
            _ => {}
        }
    }

    frequency.map(|f| {
        let mut pattern = RecurrencePattern::new(f).with_interval(interval);
        if let Some(end) = end_date {
            pattern = pattern.with_end_date(&end);
        }
        if let Some(c) = count {
            pattern = pattern.with_count(c);
        }
        if !days.is_empty() {
            pattern = pattern.with_days_of_week(&days);
        }
        pattern
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Timelike};
    
    // ...existing code...
    
    #[test]
    fn test_event_import_csv_header_validation() {
        // Test basic CSV header validation requirements
        let required = ["title", "date", "time"];
        
        let valid_headers = vec!["title", "date", "time", "location", "description", "attendees", "calendar"];
        assert!(required.iter().all(|&req| valid_headers.contains(&req)));
        
        let missing_title = vec!["date", "time", "location", "description"];
        assert!(!required.iter().all(|&req| missing_title.contains(&req)));
    }
    
    #[test]
    fn test_event_import_csv_date_time_parsing() {
        // Test parsing of date and time from CSV format
        let date_str = "2025-03-20";
        let time_str = "10:30";
        
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let time = chrono::NaiveTime::parse_from_str(time_str, "%H:%M").unwrap();
        let dt = chrono::NaiveDateTime::new(date, time);
        
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 20);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }
    
    #[test]
    fn test_event_import_email_validation() {
        // Test email validation for attendees and calendar fields
        assert!(validate_email("user@example.com"));
        assert!(validate_email("user.name@example.co.uk"));
        assert!(!validate_email("invalid-email"));
    }
    
    #[test]
    fn test_csv_event_import_email_validation() {
        // Test email validation used in CSV import
        assert!(validate_email("user@example.com"));
        assert!(validate_email("user.name@example.co.uk"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("user@"));
        assert!(!validate_email("@example.com"));
    }
    
    #[test]
    fn test_csv_date_time_parsing() {
        // Test CSV date and time format parsing
        let date = NaiveDate::parse_from_str("2025-03-20", "%Y-%m-%d").unwrap();
        let time = chrono::NaiveTime::parse_from_str("10:30", "%H:%M").unwrap();
        let dt = chrono::NaiveDateTime::new(date, time);
        
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 20);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }
    
    #[test]
    fn test_csv_header_requirements() {
        // Test CSV header validation requirements
        let required = ["title", "date", "time"];
        
        let valid_headers = vec!["title", "date", "time", "location", "description"];
        assert!(required.iter().all(|&req| valid_headers.contains(&req)));
        
        let missing_title = vec!["date", "time", "location", "description"];
        assert!(!required.iter().all(|&req| missing_title.contains(&req)));
    }
}