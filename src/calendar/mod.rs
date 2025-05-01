use crate::config::Config;
use crate::state::{CalendarItem, StateManager};
use crate::zoom::{ZoomClient, ZoomMeetingOptions, calculate_meeting_duration, format_zoom_time};
use anyhow::{Result, anyhow};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use log::{debug, error, info};
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
    #[error("Calendar '{{0}}' not found")]
    #[allow(dead_code)]
    CalendarNotFound(String),
    #[error("Invalid date/time format: {0}")]
    InvalidDateTime(String),
    #[error("AppleScript execution failed: {0}")]
    ScriptError(String),
}

pub async fn create_event(config: EventConfig) -> Result<()> {
    debug!("Creating event with config: {:?}", config);
    use crate::calendar::calendar_validation::validate_event_config;
    validate_event_config(&config)?;
    ensure_calendar_running().await?;
    let available_calendars = get_available_calendars().await?;
    debug!("Available calendars: {:?}", available_calendars);
    let app_config = Config::load()?;
    let requested_calendars = if config.calendars.is_empty() {
        vec![app_config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string())]
    } else {
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
