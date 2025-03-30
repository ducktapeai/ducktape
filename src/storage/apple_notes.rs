use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Represents a note in Apple Notes
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppleNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// Storage implementation for Apple Notes
pub struct AppleNotesStorage;

impl AppleNotesStorage {
    /// Creates a new AppleNotesStorage instance
    pub fn new() -> Result<Self> {
        debug!("Initializing Apple Notes storage");
        Ok(Self {})
    }

    /// Lists all notes from Apple Notes
    pub fn list_notes(&self) -> Result<Vec<AppleNote>> {
        debug!("Listing notes from Apple Notes");

        // Use AppleScript to get notes from Apple Notes
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"
                tell application "Notes"
                    set allNotes to {}
                    repeat with theNote in notes
                        set noteId to id of theNote as string
                        set noteTitle to name of theNote as string
                        set noteContent to body of theNote as string
                        set noteCreateDate to creation date of theNote as string
                        set noteModDate to modification date of theNote as string
                        set end of allNotes to noteId & "|" & noteTitle & "|" & noteContent & "|" & noteCreateDate & "|" & noteModDate
                    end repeat
                    return allNotes
                end tell
            "#)
            .output()
            .context("Failed to execute AppleScript to list notes")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Error listing Apple Notes: {}", error_message);
            return Err(anyhow::anyhow!(
                "Failed to list notes from Apple Notes: {}",
                error_message
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let notes = parse_apple_notes_output(&output_str)?;

        info!("Successfully retrieved {} notes from Apple Notes", notes.len());
        Ok(notes)
    }

    /// Adds a new note to Apple Notes
    pub fn add_note(&self, title: &str, content: &str) -> Result<AppleNote> {
        debug!("Adding note to Apple Notes: {}", title);

        // Use AppleScript to create note in Apple Notes
        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!(r#"
                tell application "Notes"
                    set newNote to make new note with properties {{body:"{content}", name:"{title}"}}
                    set noteId to id of newNote as string
                    set noteCreateDate to creation date of newNote as string
                    set noteModDate to modification date of newNote as string
                    return noteId & "|" & noteCreateDate & "|" & noteModDate
                end tell
            "#))
            .output()
            .context("Failed to execute AppleScript to add note")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Error adding note to Apple Notes: {}", error_message);
            return Err(anyhow::anyhow!("Failed to add note to Apple Notes: {}", error_message));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.trim().split('|').collect();

        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Unexpected output format from AppleScript"));
        }

        let id = parts[0].to_string();
        let created_at = parse_apple_date(parts[1])?;
        let updated_at = parse_apple_date(parts[2])?;

        let note = AppleNote {
            id,
            title: title.to_string(),
            content: content.to_string(),
            created_at,
            updated_at,
        };

        info!("Successfully added note to Apple Notes with ID: {}", note.id);
        Ok(note)
    }

    /// Gets a specific note from Apple Notes by ID
    pub fn get_note(&self, id: &str) -> Result<Option<AppleNote>> {
        debug!("Getting note from Apple Notes with ID: {}", id);

        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!(r#"
                tell application "Notes"
                    try
                        set theNote to note id "{id}"
                        set noteId to id of theNote as string
                        set noteTitle to name of theNote as string
                        set noteContent to body of theNote as string
                        set noteCreateDate to creation date of theNote as string
                        set noteModDate to modification date of theNote as string
                        return noteId & "|" & noteTitle & "|" & noteContent & "|" & noteCreateDate & "|" & noteModDate
                    on error
                        return ""
                    end try
                end tell
            "#))
            .output()
            .context("Failed to execute AppleScript to get note")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Error getting note from Apple Notes: {}", error_message);
            return Err(anyhow::anyhow!("Failed to get note from Apple Notes: {}", error_message));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.trim().is_empty() {
            return Ok(None);
        }

        let parts: Vec<&str> = output_str.trim().split('|').collect();
        if parts.len() < 5 {
            return Err(anyhow::anyhow!("Unexpected output format from AppleScript"));
        }

        let note = AppleNote {
            id: parts[0].to_string(),
            title: parts[1].to_string(),
            content: parts[2].to_string(),
            created_at: parse_apple_date(parts[3])?,
            updated_at: parse_apple_date(parts[4])?,
        };

        debug!("Found note with ID: {}", note.id);
        Ok(Some(note))
    }

    /// Updates a note in Apple Notes
    pub fn update_note(&self, id: &str, title: &str, content: &str) -> Result<bool> {
        debug!("Updating note in Apple Notes with ID: {}", id);

        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                r#"
                tell application "Notes"
                    try
                        set theNote to note id "{id}"
                        set name of theNote to "{title}"
                        set body of theNote to "{content}"
                        return "success"
                    on error
                        return "not_found"
                    end try
                end tell
            "#
            ))
            .output()
            .context("Failed to execute AppleScript to update note")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Error updating note in Apple Notes: {}", error_message);
            return Err(anyhow::anyhow!("Failed to update note in Apple Notes: {}", error_message));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let success = output_str.trim() == "success";

        if success {
            info!("Successfully updated note with ID: {}", id);
        } else {
            info!("Note with ID {} not found for update", id);
        }

        Ok(success)
    }

    /// Deletes a note from Apple Notes
    pub fn delete_note(&self, id: &str) -> Result<bool> {
        debug!("Deleting note from Apple Notes with ID: {}", id);

        let output = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                r#"
                tell application "Notes"
                    try
                        delete note id "{id}"
                        return "success"
                    on error
                        return "not_found"
                    end try
                end tell
            "#
            ))
            .output()
            .context("Failed to execute AppleScript to delete note")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Error deleting note from Apple Notes: {}", error_message);
            return Err(anyhow::anyhow!(
                "Failed to delete note from Apple Notes: {}",
                error_message
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let success = output_str.trim() == "success";

        if success {
            info!("Successfully deleted note with ID: {}", id);
        } else {
            info!("Note with ID {} not found for deletion", id);
        }

        Ok(success)
    }
}

/// Helper function to parse Apple Notes output
fn parse_apple_notes_output(output: &str) -> Result<Vec<AppleNote>> {
    let mut notes = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with('{') && line.ends_with('}') {
            // Remove the enclosing braces and quotes
            let line = &line[1..line.len() - 1];
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let note = AppleNote {
                    id: parts[0].to_string(),
                    title: parts[1].to_string(),
                    content: parts[2].to_string(),
                    created_at: parse_apple_date(parts[3])?,
                    updated_at: parse_apple_date(parts[4])?,
                };
                notes.push(note);
            }
        }
    }

    Ok(notes)
}

/// Helper function to parse Apple date format
fn parse_apple_date(date_str: &str) -> Result<DateTime<Local>> {
    // Parse date string returned by AppleScript
    // Format is typically: "Saturday, March 31, 2025 at 10:15:32 AM"
    // We'll use a more tolerant parsing approach

    let datetime = chrono::NaiveDateTime::parse_from_str(date_str, "%A, %B %d, %Y at %I:%M:%S %p")
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(date_str, "%a, %b %d, %Y at %I:%M:%S %p")
        })
        .with_context(|| format!("Failed to parse date: {}", date_str))?;

    let local_dt = Local
        .from_local_datetime(&datetime)
        .earliest()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert to local datetime"))?;

    Ok(local_dt)
}
