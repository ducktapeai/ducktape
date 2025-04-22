//! Notes management functionality.
//! 
//! This module provides integration with macOS Notes.app.

use anyhow::Result;

mod notes_types;
mod notes_applescript;
mod notes_validation;
mod notes_util;

pub use notes_types::*;
pub use notes_applescript::*;
pub use notes_validation::*;

/// Create a new note in Apple Notes
pub async fn create_note(config: NoteConfig<'_>) -> Result<()> {
    // Implementation relies on the notes_applescript module
    notes_applescript::create_note(config).await
}

/// List all notes from Apple Notes
pub async fn list_notes() -> Result<Vec<NoteItem>> {
    notes_applescript::list_notes().await
}

/// Get notes folders from Apple Notes
pub async fn get_note_folders() -> Result<Vec<String>> {
    notes_applescript::get_note_folders().await
}

/// Delete a note by title
pub async fn delete_note(title: &str, folder: Option<&str>) -> Result<()> {
    notes_applescript::delete_note(title, folder).await
}

/// Search notes by keyword
pub async fn search_notes(keyword: &str) -> Result<Vec<NoteItem>> {
    notes_applescript::search_notes(keyword).await
}