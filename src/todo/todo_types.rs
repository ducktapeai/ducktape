//! Type definitions for the todo module
//
// Contains data structures used for todo/reminder management

use serde::{Deserialize, Serialize};

/// Configuration for creating a new todo/reminder
#[derive(Debug, Clone)]
pub struct TodoConfig<'a> {
    /// Title of the todo item
    pub title: &'a str,
    /// Optional notes/details for the todo
    pub notes: Option<String>,
    /// Lists to add the todo item to (empty uses default list)
    pub lists: Vec<&'a str>,
    /// Optional reminder time in format "YYYY-MM-DD HH:MM"
    pub reminder_time: Option<&'a str>,
}

impl<'a> TodoConfig<'a> {
    /// Create a new TodoConfig with just a title
    pub fn new(title: &'a str) -> Self {
        Self { title, notes: None, lists: Vec::new(), reminder_time: None }
    }

    /// Set the lists for this todo
    pub fn with_lists(mut self, lists: Vec<&'a str>) -> Self {
        self.lists = lists;
        self
    }

    /// Set notes for this todo
    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    /// Set reminder time for this todo
    pub fn with_reminder(mut self, time: &'a str) -> Self {
        self.reminder_time = Some(time);
        self
    }
}

/// Represents a todo/reminder item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// The title/name of the todo item
    pub title: String,
    /// Optional notes for the todo item
    pub notes: Option<String>,
    /// The lists this todo belongs to
    pub lists: Vec<String>,
    /// Optional reminder time in ISO format
    pub reminder_time: Option<String>,
    /// Whether the todo is completed
    pub completed: bool,
}

/// Error types specific to todo operations
#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    /// Error when Reminders app is not available
    #[error("Reminders application is not running")]
    NotRunning,

    /// Error when a specific list cannot be found
    #[error("Reminder list '{0}' not found")]
    ListNotFound(String),

    /// Error when a todo item cannot be found
    #[error("Todo item '{0}' not found")]
    TodoNotFound(String),

    /// Error when executing an AppleScript
    #[error("AppleScript execution error: {0}")]
    ScriptError(String),

    /// General error
    #[error("Todo error: {0}")]
    General(String),
}
