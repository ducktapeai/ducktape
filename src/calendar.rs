use crate::config::Config;
use crate::state::{CalendarItem, StateManager};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use log::{debug, error, info};
use std::process::Command;
use std::str::FromStr;
use regex::Regex;
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
    pub zoom_meeting_id: Option<u64>,
    pub zoom_join_url: Option<String>,
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

pub async fn create_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);
    
    // Validate the event configuration first
    config.validate()?;
    
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

    // Clone the calendars Vec before the loop
    let calendars_for_state = requested_calendars.clone();

    // Clone the config values before the loop
    let title = config.title.clone();
    let start_date = config.start_date.clone();
    let start_time = config.start_time.clone();
    let end_date = config.end_date.clone();
    let end_time = config.end_time.clone();
    let all_day = config.all_day;
    let location = config.location.clone();
    let description = config.description.clone();
    let emails = config.emails.clone();
    let reminder = config.reminder;
    let timezone = config.timezone.clone();
    let recurrence = config.recurrence.clone();
    let create_zoom_meeting = config.create_zoom_meeting;
    let zoom_meeting_id = config.zoom_meeting_id;
    let zoom_join_url = config.zoom_join_url.clone();
    let zoom_password = config.zoom_password.clone();

    for calendar in requested_calendars.clone() {
        info!("Attempting to create event in calendar: {}", calendar);
        // Create a new config object with cloned values for each calendar
        let this_config = EventConfig {
            title: title.clone(),
            start_date: start_date.clone(),
            start_time: start_time.clone(),
            end_date: end_date.clone(),
            end_time: end_time.clone(),
            calendars: vec![calendar.clone()],  // Clone before moving
            all_day,
            location: location.clone(),
            description: description.clone(),
            emails: emails.clone(),
            reminder,
            timezone: timezone.clone(),
            recurrence: recurrence.clone(),
            create_zoom_meeting,
            zoom_meeting_id,
            zoom_join_url: zoom_join_url.clone(),
            zoom_password: zoom_password.clone(),
        };
        
        match create_single_event(this_config).await {
            Ok(_) => {
                success_count += 1;
                info!("Successfully created event in calendar '{}'", calendar);  // Use the cloned name
            }
            Err(e) => {
                error!("Failed to create event in calendar '{}': {}", calendar, e);  // Use the cloned name
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

async fn create_single_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);
    // Parse start datetime with timezone handling
    let start_datetime = format!(
        "{} {}",
        config.start_date,
        if config.all_day {
            "00:00"
        } else {
            &config.start_time
        }
    );
    
    // Parse the base datetime
    let start_dt = NaiveDateTime::parse_from_str(&start_datetime, "%Y-%m-%d %H:%M")
        .map_err(|e| CalendarError::InvalidDateTime(e.to_string()))?;
    // Handle timezone conversion if specified
    let local_start: DateTime<Local> = if let Some(tz_str) = config.timezone.as_deref() {
        match Tz::from_str(tz_str) {
            Ok(tz) => {
                let tz_dt = tz.from_local_datetime(&start_dt)
                    .single()
                    .ok_or_else(|| anyhow!("Invalid or ambiguous start time in timezone {}", tz_str))?;
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
            .map_err(|e| CalendarError::InvalidDateTime(e.to_string()))?;
        
        if let Some(tz_str) = config.timezone.as_deref() {
            match Tz::from_str(tz_str) {
                Ok(tz) => {
                    let tz_dt = tz.from_local_datetime(&naive_end)
                        .single()
                        .ok_or_else(|| anyhow!("Invalid or ambiguous end time in timezone {}", tz_str))?;
                    tz_dt.with_timezone(&Local)
                }
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
    // Validate that end time is after start time
    if end_dt <= local_start {
        return Err(anyhow!("End time must be after start time"));
    }
    
    // First verify Calendar.app is running
    ensure_calendar_running()?;
    
    // Create Zoom meeting if requested
    let mut zoom_meeting_info = String::new();
    if config.create_zoom_meeting {
        info!("Creating Zoom meeting for event: {}", config.title);
        
        // Create Zoom client
        let mut client = ZoomClient::new()?;
        
        // Format start time in Zoom format
        let zoom_start_time = format_zoom_time(&config.start_date, &config.start_time)?;
        
        // Calculate duration
        let duration = if let Some(end_time) = &config.end_time {
            calculate_meeting_duration(&config.start_time, end_time)?
        } else {
            60 // Default 1 hour
        };
        
        // Create meeting options
        let meeting_options = ZoomMeetingOptions {
            topic: config.title.to_string(),
            start_time: zoom_start_time,
            duration,
            password: None, // Auto-generate password
            agenda: config.description.clone(),
        };
        
        // Create the meeting - note this will use the existing runtime
        match client.create_meeting(meeting_options).await {
            Ok(meeting) => {
                info!("Created Zoom meeting: ID={}, URL={}", meeting.id, meeting.join_url);
                
                // Format Zoom information for calendar description
                let password_info = if let Some(password) = meeting.password {
                    format!("\nPassword: {}", password)
                } else {
                    String::new()
                };
                
                zoom_meeting_info = format!(
                    "\n\n--------------------\nZoom Meeting\n--------------------\nJoin URL: {}{}", 
                    meeting.join_url,
                    password_info
                );
            },
            Err(e) => {
                error!("Failed to create Zoom meeting: {}", e);
                zoom_meeting_info = "\n\nNote: Zoom meeting creation failed. Please create a meeting manually.".to_string();
            }
        }
    } else if let Some(url) = &config.zoom_join_url {
        // Use existing Zoom meeting URL
        let password_info = if let Some(password) = &config.zoom_password {
            format!("\nPassword: {}", password)
        } else {
            String::new()
        };
        
        zoom_meeting_info = format!(
            "\n\n--------------------\nZoom Meeting\n--------------------\nJoin URL: {}{}", 
            url,
            password_info
        );
    }
    
    // Build the description with Zoom info if available
    let full_description = if !zoom_meeting_info.is_empty() {
        // Combine the original description with Zoom info
        match &config.description {
            Some(desc) if !desc.is_empty() => format!("{}{}", desc, zoom_meeting_info),
            _ => format!("Created by Ducktape 🦆{}", zoom_meeting_info)
        }
    } else {
        config.description.as_deref().unwrap_or("Created by Ducktape 🦆").to_string()
    };
    
    // Build extras for properties: include location if non-empty.
    let mut extra = String::new();
    if let Some(loc) = &config.location {
        if !loc.is_empty() {
            extra.push_str(&format!(", location:\"{}\"", loc));
        }
    }
    
    // Build attendees code block with proper error handling
    let mut attendees_block = String::new();
    if !config.emails.is_empty() {
        info!("Adding {} attendee(s): {}", config.emails.len(), config.emails.join(", "));
        for email in &config.emails {
            attendees_block.push_str(&format!(r#"
                    try
                        tell newEvent
                            make new attendee with properties {{email:"{}"}}
                        end tell
                        log "Successfully added attendee: {}"
                    on error errMsg
                        log "Failed to add attendee {}: " & errMsg
                    end try"#,
                email, email, email
            ));
        }
    }
    
    // Build the recurrence string in RFC 5545 format
    let mut recurrence_rule = String::new();
    // Build the recurrence parameters separately - don't include directly in event properties
    let mut has_recurrence = false;
    if let Some(recurrence) = &config.recurrence {
        has_recurrence = true;
        info!("Building recurrence rule with frequency: {:?}, interval: {}", recurrence.frequency, recurrence.interval);
        
        // Convert day numbers to BYDAY format if needed
        let byday_str = if recurrence.frequency == RecurrenceFrequency::Weekly && !recurrence.days_of_week.is_empty() {
            // If days aren't specified for weekly recurrence, use the day of the start date
            let day_abbrs: Vec<String> = if recurrence.days_of_week.is_empty() {
                // Get day of week from start date (0=Sunday, 1=Monday, etc.)
                let day_of_week = local_start.weekday().num_days_from_sunday() as u8;
                info!("No days specified for weekly recurrence. Using day of start date: {}", day_of_week);
                vec![match day_of_week {
                    0 => "SU",
                    1 => "MO",
                    2 => "TU",
                    3 => "WE",
                    4 => "TH",
                    5 => "FR",
                    6 => "SA",
                    _ => "SU" // Default
                }.to_string()]
            } else {
                info!("Using specified days for weekly recurrence: {:?}", recurrence.days_of_week);
                recurrence.days_of_week.iter()
                    .map(|&d| match d {
                        0 => "SU",
                        1 => "MO",
                        2 => "TU",
                        3 => "WE",
                        4 => "TH",
                        5 => "FR",
                        6 => "SA",
                        _ => "MO" // Default
                    })
                    .map(|s| s.to_string())
                    .collect()
            };
            
            let byday = format!("BYDAY={}", day_abbrs.join(","));
            info!("Generated BYDAY parameter: {}", byday);
            byday
        } else {
            String::new()
        };
        
        // Build the complete recurrence rule
        let mut parts = vec![
            format!("FREQ={}", recurrence.frequency.to_rfc5545()),
            format!("INTERVAL={}", recurrence.interval),
        ];
        
        // Add count if specified
        if let Some(count) = recurrence.count {
            info!("Adding COUNT={} to recurrence rule", count);
            parts.push(format!("COUNT={}", count));
        }
        
        // Add end date if specified
        if let Some(end_date) = &recurrence.end_date {
            let end_naive = NaiveDateTime::parse_from_str(&format!("{} 23:59", end_date), "%Y-%m-%d %H:%M")
                .map_err(|e| CalendarError::InvalidDateTime(e.to_string()))?;
            
            let end_str = end_naive.format("%Y%m%dT%H%M%SZ").to_string();
            info!("Adding UNTIL={} to recurrence rule", end_str);
            parts.push(format!("UNTIL={}", end_str));
        }
        
        // Add BYDAY if needed
        if !byday_str.is_empty() {
            parts.push(byday_str);
        }
        
        // Join all parts
        recurrence_rule = parts.join(";");
        info!("Generated complete recurrence rule: {}", recurrence_rule);
    } else {
        info!("No recurrence specified for event");
    }
    
    // Format the recurrence script block properly with explicit quoting
    // This is a critical fix for the AppleScript syntax error
    let recurrence_code = if has_recurrence {
        format!(r#"
                    -- Set recurrence rule separately after creating the event
                    tell newEvent
                        set its recurrence to "{}"
                        log "Recurrence rule set to: " & (its recurrence as string)
                    end tell"#, 
                recurrence_rule)
    } else { 
        String::new() 
    };
    
    // Build the complete AppleScript with fixed ordering of operations
    // Ensure all code blocks are properly separated and contained in the right tell blocks
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
                    log "Creating event with recurrence: {recurrence_logging}"
                    set newEvent to make new event with properties {{summary:"{title}", start date:startDate, end date:endDate, description:"{description}"{extra}}}
                    log "Event created successfully"
                    
                    -- Set all-day flag if needed
                    {all_day_code}
                    
                    -- Add reminder if specified
                    {reminder_code}
                    
                    {recurrence_code}
                    
                    -- Add attendees with error handling after setting recurrence
                    {attendees_block}
                    
                    -- Save changes
                    save
                    log "Event saved"
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
        title = config.title,
        description = full_description,  // Use the description with Zoom info
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
        recurrence_logging = if recurrence_rule.is_empty() { "none" } else { &recurrence_rule },
        recurrence_code = recurrence_code,
        all_day_code = if config.all_day { 
            "\n                    set allday event of newEvent to true" 
        } else { 
            "" 
        },
        reminder_code = if let Some(minutes) = config.reminder {
            format!(
                r#"
                    -- Add reminder alarm
                    set theAlarm to make new display alarm at end of newEvent
                    set trigger interval of theAlarm to -{}"#,
                minutes * 60
            )
        } else { 
            String::new() 
        },
        attendees_block = attendees_block,
    );
    
    debug!("Generated AppleScript:\n{}", script);
    
    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);
    let error_output = String::from_utf8_lossy(&output.stderr);
    
    // Log AppleScript output for debugging
    if !output.stderr.is_empty() {
        info!("AppleScript log output: {}", error_output);
    }
    
    if result.contains("Success") {
        info!(
            "Calendar event created: {} at {} ({} timezone)",
            config.title,
            format!("{} {}", config.start_date, config.start_time),
            local_start.offset()
        );
        if !config.emails.is_empty() {
            info!("Added {} attendees: {}", config.emails.len(), config.emails.join(", "));
        }
        
        // Log recurrence info if applicable with improved feedback
        if let Some(recurrence) = &config.recurrence {
            let frequency_str = match recurrence.frequency {
                RecurrenceFrequency::Daily => "day",
                RecurrenceFrequency::Weekly => "week",
                RecurrenceFrequency::Monthly => "month",
                RecurrenceFrequency::Yearly => "year",
            };
            
            let interval_str = if recurrence.interval == 1 {
                format!("every {}", frequency_str)
            } else {
                format!("every {} {}s", recurrence.interval, frequency_str)
            };
            
            // Provide clearer feedback about recurring events
            println!("✅ Created recurring event that repeats {}. (Calendar will show the first instance)", 
                     interval_str);
            
            info!("Event is recurring: {} every {} {}s", 
                  recurrence.frequency.to_rfc5545(),
                  recurrence.interval,
                  frequency_str);
            
            if let Some(end_date) = &recurrence.end_date {
                info!("Recurrence ends on: {}", end_date);
                println!("   Recurrence ends on: {}", end_date);
            }
            
            if let Some(count) = recurrence.count {
                info!("Recurrence has {} occurrences", count);
                println!("   Series will have {} occurrences", count);
            }
            
            if recurrence.frequency == RecurrenceFrequency::Weekly && !recurrence.days_of_week.is_empty() {
                let day_names: Vec<String> = recurrence.days_of_week.iter()
                    .map(|&d| match d {
                        0 => "Sunday",
                        1 => "Monday",
                        2 => "Tuesday",
                        3 => "Wednesday",
                        4 => "Thursday",
                        5 => "Friday",
                        6 => "Saturday",
                        _ => "Unknown day"
                    })
                    .map(|s| s.to_string())
                    .collect();
                
                println!("   Recurring on: {}", day_names.join(", "));
            }
            
            // Add tip about viewing recurring events
            println!("\nTip: To see all occurrences in Calendar app:");
            println!("1. Double-click on the event to open its details");
            println!("2. You'll see the recurrence pattern displayed");
            println!("3. Scroll forward in your calendar to see future instances");
        }
        
        // Add information about Zoom meeting if created
        if config.create_zoom_meeting && !zoom_meeting_info.is_empty() {
            println!("\n✅ Zoom meeting linked to calendar event");
            println!("Check the event description for meeting details");
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
        } else if !error_output.is_empty() {
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
pub async fn create_event_with_contacts(mut config: EventConfig, contact_names: &[&str]) -> Result<()> {
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
    create_event(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recurrence_frequency_to_applescript() {
        assert_eq!(RecurrenceFrequency::Daily.to_applescript(), "daily");
        assert_eq!(RecurrenceFrequency::Weekly.to_applescript(), "weekly");
        assert_eq!(RecurrenceFrequency::Monthly.to_applescript(), "monthly");
        assert_eq!(RecurrenceFrequency::Yearly.to_applescript(), "yearly");
    }

    #[test]
    fn test_recurrence_frequency_to_rfc5545() {
        assert_eq!(RecurrenceFrequency::Daily.to_rfc5545(), "DAILY");
        assert_eq!(RecurrenceFrequency::Weekly.to_rfc5545(), "WEEKLY");
        assert_eq!(RecurrenceFrequency::Monthly.to_rfc5545(), "MONTHLY");
        assert_eq!(RecurrenceFrequency::Yearly.to_rfc5545(), "YEARLY");
    }

    #[test]
    fn test_recurrence_frequency_from_str() {
        // Test valid values
        assert_eq!(RecurrenceFrequency::from_str("daily").unwrap(), RecurrenceFrequency::Daily);
        assert_eq!(RecurrenceFrequency::from_str("day").unwrap(), RecurrenceFrequency::Daily);
        assert_eq!(RecurrenceFrequency::from_str("days").unwrap(), RecurrenceFrequency::Daily);
        
        assert_eq!(RecurrenceFrequency::from_str("weekly").unwrap(), RecurrenceFrequency::Weekly);
        assert_eq!(RecurrenceFrequency::from_str("week").unwrap(), RecurrenceFrequency::Weekly);
        
        assert_eq!(RecurrenceFrequency::from_str("monthly").unwrap(), RecurrenceFrequency::Monthly);
        assert_eq!(RecurrenceFrequency::from_str("month").unwrap(), RecurrenceFrequency::Monthly);
        
        assert_eq!(RecurrenceFrequency::from_str("yearly").unwrap(), RecurrenceFrequency::Yearly);
        assert_eq!(RecurrenceFrequency::from_str("annually").unwrap(), RecurrenceFrequency::Yearly);
        
        // Test case insensitivity
        assert_eq!(RecurrenceFrequency::from_str("DAILY").unwrap(), RecurrenceFrequency::Daily);
        assert_eq!(RecurrenceFrequency::from_str("Weekly").unwrap(), RecurrenceFrequency::Weekly);
        
        // Test invalid value
        assert!(RecurrenceFrequency::from_str("invalid").is_err());
        let err = RecurrenceFrequency::from_str("invalid").unwrap_err();
        assert!(err.to_string().contains("Invalid recurrence frequency: invalid"));
    }

    #[test]
    fn test_recurrence_pattern_creation() {
        let pattern = RecurrencePattern::new(RecurrenceFrequency::Daily);
        
        assert_eq!(pattern.frequency, RecurrenceFrequency::Daily);
        assert_eq!(pattern.interval, 1);
        assert_eq!(pattern.end_date, None);
        assert_eq!(pattern.count, None);
        assert!(pattern.days_of_week.is_empty());
    }

    #[test]
    fn test_recurrence_pattern_builder() {
        let pattern = RecurrencePattern::new(RecurrenceFrequency::Weekly)
            .with_interval(2)
            .with_end_date("2025-12-31")
            .with_days_of_week(&[1, 3, 5]);  // Monday, Wednesday, Friday
        
        assert_eq!(pattern.frequency, RecurrenceFrequency::Weekly);
        assert_eq!(pattern.interval, 2);
        assert_eq!(pattern.end_date, Some("2025-12-31".to_string()));
        assert_eq!(pattern.count, None);
        assert_eq!(pattern.days_of_week, vec![1, 3, 5]);
        
        // Test with_count instead of with_end_date
        let pattern = RecurrencePattern::new(RecurrenceFrequency::Monthly)
            .with_interval(3)
            .with_count(10);
        
        assert_eq!(pattern.frequency, RecurrenceFrequency::Monthly);
        assert_eq!(pattern.interval, 3);
        assert_eq!(pattern.end_date, None);
        assert_eq!(pattern.count, Some(10));
        assert!(pattern.days_of_week.is_empty());
    }

    #[test]
    fn test_event_config_with_recurrence() {
        let recurrence = RecurrencePattern::new(RecurrenceFrequency::Weekly)
            .with_interval(2)
            .with_days_of_week(&[1, 5]);  // Monday and Friday
            
        let mut config = EventConfig::new("Test Event", "2024-05-01", "10:00");
        config = config.with_recurrence(recurrence);
        
        assert!(config.recurrence.is_some());
        let rec = config.recurrence.unwrap();
        assert_eq!(rec.frequency, RecurrenceFrequency::Weekly);
        assert_eq!(rec.interval, 2);
        assert_eq!(rec.days_of_week, vec![1, 5]);
    }
}