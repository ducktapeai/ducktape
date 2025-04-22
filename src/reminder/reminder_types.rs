//! Type definitions for reminder functionality

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration for creating a new reminder
#[derive(Debug, Clone)]
pub struct ReminderConfig<'a> {
    /// Title of the reminder item
    pub title: &'a str,
    /// Lists to add the reminder item to (empty uses default list)
    pub lists: Vec<&'a str>,
    /// Optional reminder time in format "YYYY-MM-DD HH:MM"
    pub reminder_time: Option<&'a str>,
    /// Optional notes/details for the reminder
    pub notes: Option<String>,
}

impl<'a> ReminderConfig<'a> {
    /// Create a new ReminderConfig with just a title
    pub fn new(title: &'a str) -> Self {
        Self { title, lists: Vec::new(), reminder_time: None, notes: None }
    }

    /// Set the lists for this reminder
    pub fn with_lists(mut self, lists: Vec<&'a str>) -> Self {
        self.lists = lists;
        self
    }

    /// Set notes for this reminder
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Set reminder time for this reminder
    pub fn with_reminder(mut self, time: &'a str) -> Self {
        self.reminder_time = Some(time);
        self
    }
}

/// Represents a reminder item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderItem {
    /// The title/name of the reminder item
    pub title: String,
    /// The lists this reminder belongs to
    pub lists: Vec<String>,
    /// Optional reminder time in ISO format
    pub reminder_time: Option<String>,
    /// Optional notes for the reminder item
    pub notes: Option<String>,
    /// Whether the reminder is completed
    pub completed: bool,
}

/// Error types specific to reminder operations
#[derive(Error, Debug)]
pub enum ReminderError {
    /// Error when Reminders app is not available
    #[error("Reminders application is not running")]
    NotRunning,

    /// Error when a specific list cannot be found
    #[error("List not found: {0}")]
    ListNotFound(String),

    /// Error when a reminder item cannot be found
    #[error("Reminder not found: {0}")]
    ReminderNotFound(String),

    /// Error when invalid input is provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Error when executing an AppleScript
    #[error("AppleScript execution error: {0}")]
    ScriptError(String),

    /// General error
    #[error("General reminder error: {0}")]
    General(String),
}
