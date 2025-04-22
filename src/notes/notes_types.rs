//! Type definitions for the notes module.

use serde::{Deserialize, Serialize};

/// Configuration for creating a note
#[derive(Debug, Clone)]
pub struct NoteConfig<'a> {
    /// Title of the note
    pub title: &'a str,
    /// Content of the note
    pub content: &'a str,
    /// Optional folder to store the note in
    pub folder: Option<&'a str>,
}

impl<'a> NoteConfig<'a> {
    /// Create a new note configuration with default settings
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self { title, content, folder: None }
    }

    /// Create a new note configuration with a specified folder
    pub fn with_folder(title: &'a str, content: &'a str, folder: &'a str) -> Self {
        Self { title, content, folder: Some(folder) }
    }
}

/// Represents a note from Apple Notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteItem {
    /// Title of the note
    pub title: String,
    /// Folder containing the note
    pub folder: String,
    /// Creation date of the note (if available)
    pub created: Option<String>,
    /// Modification date of the note (if available)
    pub modified: Option<String>,
}

/// Custom error type for notes operations
#[derive(Debug, thiserror::Error)]
pub enum NotesError {
    #[error("Notes application is not running")]
    NotRunning,

    #[error("Note '{0}' not found")]
    NoteNotFound(String),

    #[error("Folder '{0}' not found")]
    FolderNotFound(String),

    #[error("AppleScript execution failed: {0}")]
    ScriptError(String),

    #[error("Failed to parse AppleScript output: {0}")]
    ParseError(String),
}
