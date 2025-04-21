//! Todo and reminder management functionality.
//
// This module provides integration with macOS Reminders.app.

use anyhow::Result;

mod todo_types;
mod todo_applescript;
mod todo_validation;
mod todo_util;

pub use todo_types::*;
pub use todo_applescript::*;
pub use todo_validation::*;

/// Create a new todo/reminder
pub async fn create_todo(config: TodoConfig<'_>) -> Result<()> {
    // Implementation relies on the todo_applescript module
    todo_applescript::create_single_todo(config).await
}

/// List available reminder lists
pub async fn list_reminder_lists() -> Result<Vec<String>> {
    todo_applescript::get_reminder_lists().await
}

/// Get todos from a specific list or all lists
pub async fn get_todos(list_name: Option<&str>) -> Result<Vec<TodoItem>> {
    todo_applescript::fetch_todos(list_name).await
}

/// Delete a todo by title and list
pub async fn delete_todo(title: &str, list_name: Option<&str>) -> Result<()> {
    todo_applescript::delete_todo(title, list_name).await
}