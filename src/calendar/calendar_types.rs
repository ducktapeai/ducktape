//! Types, enums, and error definitions for the calendar module.
//
// This module contains all core types, enums, and error types used by the calendar system.

use thiserror::Error;

/// Custom error type for calendar operations
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CalendarError {
    #[error("Calendar application is not running")]
    NotRunning,

    #[error("Calendar '{{0}}' not found")]
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
    pub fn from_str_custom(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "daily" | "day" | "days" => Ok(RecurrenceFrequency::Daily),
            "weekly" | "week" | "weeks" => Ok(RecurrenceFrequency::Weekly),
            "monthly" | "month" | "months" => Ok(RecurrenceFrequency::Monthly),
            "yearly" | "year" | "years" | "annual" | "annually" => Ok(RecurrenceFrequency::Yearly),
            _ => Err(anyhow::anyhow!("Invalid recurrence frequency: {}", s)),
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
            create_zoom_meeting: false,
            zoom_meeting_id: None,
            zoom_join_url: None,
            zoom_password: None,
        }
    }
    pub fn with_recurrence(mut self, recurrence: RecurrencePattern) -> Self {
        self.recurrence = Some(recurrence);
        self
    }
    pub fn with_zoom_meeting(mut self, enable: bool) -> Self {
        self.create_zoom_meeting = enable;
        self
    }
}
