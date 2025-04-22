//! Reminder management functionality.
//
// This module provides integration with macOS Reminders.app.

use anyhow::Result;

mod reminder_applescript;
mod reminder_types;
mod reminder_util;
mod reminder_validation;

pub use reminder_applescript::*;
pub use reminder_types::*;
pub use reminder_validation::*;

/// Create a new reminder
pub async fn create_reminder(config: ReminderConfig<'_>) -> Result<()> {
    // Implementation relies on the reminder_applescript module
    reminder_applescript::create_single_reminder(config).await
}

/// List available reminder lists
pub async fn list_reminder_lists() -> Result<Vec<String>> {
    reminder_applescript::get_reminder_lists().await
}

/// Get reminders from a specific list or all lists
pub async fn get_reminders(list_name: Option<&str>) -> Result<Vec<ReminderItem>> {
    reminder_applescript::fetch_reminders(list_name).await
}

/// Delete a reminder by title and list
pub async fn delete_reminder(title: &str, list_name: Option<&str>) -> Result<()> {
    reminder_applescript::delete_reminder(title, list_name).await
}
