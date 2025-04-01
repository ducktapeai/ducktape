# Ducktape Calendar Module Instructions

## Overview

The Calendar module in Ducktape provides functionality for interacting with Apple Calendar to manage events. This document outlines the key components, usage patterns, and integration points for working with calendar functionality.

## Key Components

## Be careful when changing the calendar.rs file as it can break key functionality

- be careful changing the the functions that create and event and add email addresses using apple script

### EventConfig

The primary configuration struct for calendar events:

```rust
pub struct EventConfig {
    pub title: String,                          // Event title (required)
    pub start_date: String,                     // Start date YYYY-MM-DD (required)
    pub start_time: String,                     // Start time HH:MM (required)
    pub end_date: Option<String>,               // End date YYYY-MM-DD (optional)
    pub end_time: Option<String>,               // End time HH:MM (optional)
    pub calendars: Vec<String>,                 // Target calendars (defaults to user's default)
    pub all_day: bool,                          // Whether event is all-day
    pub location: Option<String>,               // Event location
    pub description: Option<String>,            // Event description
    pub emails: Vec<String>,                    // Attendee emails
    pub reminder: Option<i32>,                  // Reminder in minutes before event
    pub timezone: Option<String>,               // Event timezone
    pub recurrence: Option<RecurrencePattern>,  // Recurrence settings
    pub create_zoom_meeting: bool,              // Whether to create Zoom meeting
    pub zoom_meeting_id: Option<u64>,           // Zoom meeting ID if exists
    pub zoom_join_url: Option<String>,          // Zoom join URL if exists
    pub zoom_password: Option<String>,          // Zoom password if exists
}
```

### RecurrencePattern

Defines how events repeat:

```rust
pub struct RecurrencePattern {
    pub frequency: RecurrenceFrequency,  // Daily, Weekly, Monthly, Yearly
    pub interval: u32,                   // Repeat every X units (e.g., every 2 weeks)
    pub end_date: Option<String>,        // When recurrence ends
    pub count: Option<u32>,              // Number of occurrences
    pub days_of_week: Vec<u8>,           // For weekly: days to repeat (0=Sunday, 1=Monday, etc.)
}
```

### RecurrenceFrequency

```rust
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}
```

## Core Functionality

### Creating Events

The main function for creating calendar events:

```rust
// Basic event creation
pub async fn create_event(config: EventConfig) -> Result<()>

// Example usage:
let config = EventConfig::new("Meeting with Team", "2025-04-01", "10:00")
    .with_calendar("Work")
    .with_location("Conference Room A")
    .with_description("Quarterly planning meeting")
    .with_end_time("11:30")
    .with_reminder(15);  // 15 minutes before

create_event(config).await?;
```

### Creating Events with Contact Lookup

```rust
// Create event with contact names (will look up emails)
pub async fn create_event_with_contacts(config: EventConfig, contact_names: &[&str]) -> Result<()>

// Example usage:
let config = EventConfig::new("Project Discussion", "2025-04-02", "14:00");
create_event_with_contacts(config, &["John Smith", "Sarah Johnson"]).await?;
```

### Managing Calendars

```rust
// List available calendars
pub async fn list_calendars() -> Result<()>

// Get available calendars as Vec<String>
pub async fn get_available_calendars() -> Result<Vec<String>>
```

### Deleting Events

```rust
// Delete events by title (and optionally date)
pub async fn delete_event(title: &str, date: &str) -> Result<()>
```

### Importing Events

```rust
// Import from CSV file
pub async fn import_csv_events(file_path: &Path, target_calendar: Option<String>) -> Result<()>

// Import from ICS (iCalendar) file
pub async fn import_ics_events(file_path: &Path, target_calendar: Option<String>) -> Result<()>
```

### Contact Management

```rust
// Look up contact emails by name
pub async fn lookup_contact(name: &str) -> Result<Vec<String>>
```

## Data Validation

The module performs validation on input data:

1. **Date Format**: Ensures dates are in YYYY-MM-DD format
2. **Time Format**: Ensures times are in HH:MM format
3. **Email Format**: Validates email addresses
4. **Character Safety**: Prevents AppleScript injection via dangerous characters

```rust
pub fn validate_date_format(date: &str) -> bool
pub fn validate_time_format(time: &str) -> bool
pub fn validate_email(email: &str) -> bool
```

## Integration with Zoom

The calendar module integrates with Zoom to automatically create meetings:

```rust
// Example with Zoom meeting creation
let config = EventConfig::new("Team Meeting", "2025-04-03", "15:00")
    .with_end_time("16:00")
    .with_zoom_meeting(true);  // This will create a Zoom meeting

create_event(config).await?;
```

## CSV Import Format

When importing events from CSV, the file should have these headers:

- **Required**: title, date, start_time
- **Optional**: end_time, description, location, calendar, attendees

Example CSV format:
```csv
title,date,start_time,end_time,description,location,attendees
Team Meeting,2025-04-01,10:00,11:00,Weekly sync,Conference Room,john@example.com;jane@example.com
Client Call,2025-04-02,14:30,15:00,Project update,Zoom,client@company.com
```

## Error Handling

The calendar module uses custom error types for detailed error reporting:

```rust
pub enum CalendarError {
    NotRunning,
    CalendarNotFound(String),
    InvalidDateTime(String),
    ScriptError(String),
}
```

## Asynchronous Operations

All calendar operations are asynchronous to prevent blocking the main thread:

```rust
// All public functions are async
pub async fn function_name() -> Result<()>
```

## Configuration Integration

The calendar module integrates with the application's configuration system:

1. Uses default calendar from Config if none specified
2. Uses default reminder times from Config
3. Uses default duration from Config for events without end time

## Thread Safety

The calendar implementation is designed to be thread-safe:

1. No shared mutable state
2. All operations are isolated
3. Properly handles concurrent access to Apple Calendar via AppleScript

## Examples

### Basic Event Creation

```rust
let config = EventConfig::new("Dentist Appointment", "2025-04-15", "14:30")
    .with_end_time("15:30")
    .with_location("Dental Office")
    .with_reminder(60);  // 1 hour reminder

create_event(config).await?;
```

### Recurring Event

```rust
let recurrence = RecurrencePattern::new(RecurrenceFrequency::Weekly)
    .with_interval(2)  // Every 2 weeks
    .with_days_of_week(&[1, 3])  // Monday and Wednesday
    .with_count(10);  // For 10 occurrences

let config = EventConfig::new("Team Standup", "2025-04-01", "09:00")
    .with_end_time("09:30")
    .with_recurrence(recurrence);

create_event(config).await?;
```

### Event with Attendees

```rust
let config = EventConfig::new("Project Kickoff", "2025-04-10", "10:00")
    .with_end_time("11:30")
    .with_location("Main Conference Room")
    .with_description("Initial project planning session")
    .with_emails(vec!["team@company.com", "client@example.com"]);

create_event(config).await?;
```

### All-Day Event

```rust
let config = EventConfig::new("Company Holiday", "2025-05-01", "00:00");
config.all_day = true;

create_event(config).await?;
```

## Best Practices

1. **Always validate user input** before creating events
2. **Handle timezones explicitly** when dealing with international events
3. **Use contact lookup** when possible instead of hardcoded email addresses
4. **Provide meaningful descriptions** for better calendar organization
5. **Set appropriate reminders** for important events
6. **Use recurrence patterns** for repeating events instead of creating multiple events
7. **Check for existing events** before creating new ones to avoid duplicates
8. **Add proper error handling** for failed calendar operations
9. **Respect user's default calendar** settings when appropriate
10. **Include Zoom information** in the description when relevant

## Apple Calendar Integration

The calendar module uses AppleScript to interact with Apple Calendar:

1. Ensures Calendar.app is running before operations
2. Creates events in the user's specified calendar(s)
3. Adds attendees as participants in the event
4. Sets appropriate reminders based on configuration
5. Handles recurrence patterns for repeating events