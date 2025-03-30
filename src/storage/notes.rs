use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Represents a note in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// Storage for managing notes
pub struct NotesStorage {
    storage_path: PathBuf,
}

impl NotesStorage {
    /// Creates a new NotesStorage instance
    pub fn new() -> Result<Self> {
        let storage_dir = get_storage_dir()?;
        let notes_dir = storage_dir.join("notes");

        if !notes_dir.exists() {
            log::debug!("Creating notes directory: {:?}", notes_dir);
            fs::create_dir_all(&notes_dir)
                .context(format!("Failed to create notes directory at {:?}", notes_dir))?;
        } else {
            log::debug!("Notes directory exists: {:?}", notes_dir);
        }

        Ok(Self { storage_path: notes_dir })
    }

    fn get_notes_file(&self) -> PathBuf {
        self.storage_path.join("notes.json")
    }

    /// Lists all notes in storage
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let file_path = self.get_notes_file();
        log::debug!("Attempting to read notes from: {:?}", file_path);

        if !file_path.exists() {
            log::debug!("Notes file does not exist yet, returning empty list");
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&file_path)
            .context(format!("Failed to read notes file at {:?}", file_path))?;

        if content.trim().is_empty() {
            log::debug!("Notes file is empty, returning empty list");
            return Ok(Vec::new());
        }

        let notes: Vec<Note> = serde_json::from_str(&content)
            .context(format!("Failed to parse notes JSON from {:?}", file_path))?;

        log::debug!("Successfully read {} notes", notes.len());
        Ok(notes)
    }

    /// Adds a new note to storage
    pub fn add_note(&self, note: &Note) -> Result<()> {
        let mut notes = self.list_notes()?;
        notes.push(note.clone());

        let file_path = self.get_notes_file();
        log::debug!("Writing {} notes to: {:?}", notes.len(), file_path);

        let content =
            serde_json::to_string_pretty(&notes).context("Failed to serialize notes to JSON")?;

        fs::write(&file_path, content)
            .context(format!("Failed to write notes file at {:?}", file_path))?;

        log::debug!("Note added successfully with ID: {}", note.id);
        Ok(())
    }

    /// Deletes a note by ID
    pub fn delete_note(&self, id: &str) -> Result<bool> {
        let mut notes = self.list_notes()?;
        let initial_len = notes.len();

        notes.retain(|note| note.id != id);

        if notes.len() == initial_len {
            log::debug!("Note with ID {} not found for deletion", id);
            return Ok(false);
        }

        let file_path = self.get_notes_file();
        log::debug!("Writing {} notes after deletion to: {:?}", notes.len(), file_path);

        let content =
            serde_json::to_string_pretty(&notes).context("Failed to serialize notes to JSON")?;

        fs::write(&file_path, content)
            .context(format!("Failed to write notes file after deletion at {:?}", file_path))?;

        log::debug!("Note with ID {} deleted successfully", id);
        Ok(true)
    }

    /// Gets a single note by ID
    pub fn get_note(&self, id: &str) -> Result<Option<Note>> {
        let notes = self.list_notes()?;
        let note = notes.iter().find(|n| n.id == id).cloned();

        if note.is_some() {
            log::debug!("Found note with ID: {}", id);
        } else {
            log::debug!("Note with ID {} not found", id);
        }

        Ok(note)
    }

    /// Updates a note content by ID
    pub fn update_note(&self, id: &str, content: &str) -> Result<bool> {
        let mut notes = self.list_notes()?;
        let mut found = false;

        for note in &mut notes {
            if note.id == id {
                note.content = content.to_string();
                note.updated_at = Local::now();
                found = true;
                break;
            }
        }

        if !found {
            log::debug!("Note with ID {} not found for update", id);
            return Ok(false);
        }

        let file_path = self.get_notes_file();
        log::debug!("Writing updated notes to: {:?}", file_path);

        let content = serde_json::to_string_pretty(&notes)
            .context("Failed to serialize updated notes to JSON")?;

        fs::write(&file_path, content)
            .context(format!("Failed to write updated notes file at {:?}", file_path))?;

        log::debug!("Note with ID {} updated successfully", id);
        Ok(true)
    }
}

/// Helper function to get the storage directory
fn get_storage_dir() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;
    let storage_dir = home_dir.join(".ducktape");

    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir)?;
    }

    Ok(storage_dir)
}
