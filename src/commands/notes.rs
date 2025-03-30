use crate::commands::{CommandArgs, CommandExecutor};
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use colored::*;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::process::Command;

/// Represents a note in Apple Notes
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppleNote {
    /// Unique identifier for the note in Apple Notes
    pub id: String,
    /// Title of the note
    pub title: String,
    /// Content/body of the note
    pub content: String,
    /// Creation timestamp
    pub created_at: DateTime<Local>,
    /// Last modification timestamp
    pub updated_at: DateTime<Local>,
}

/// Storage implementation for Apple Notes application
///
/// Provides methods to interact with the Apple Notes application via AppleScript,
/// allowing DuckTape to create, read, update, and delete notes from the native
/// macOS Notes application.
struct AppleNotesStorage;

impl AppleNotesStorage {
    /// Creates a new AppleNotesStorage instance
    ///
    /// # Returns
    ///
    /// A `Result` containing the initialized storage or an error if initialization failed
    pub fn new() -> Result<Self> {
        debug!("Initializing Apple Notes storage");
        Ok(Self {})
    }

    /// Lists all notes from Apple Notes
    ///
    /// Uses the most basic possible AppleScript approach to get notes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `AppleNote` objects or an error if the operation failed
    pub fn list_notes(&self) -> Result<Vec<AppleNote>> {
        debug!("Listing notes from Apple Notes using minimal approach");

        // First, check if Notes app is running
        let check = Command::new("pgrep")
            .arg("-x")
            .arg("Notes")
            .output()
            .context("Failed to check if Notes app is running")?;

        let is_running = check.status.success();
        debug!("Apple Notes app running check: {}", is_running);

        if !is_running {
            info!("Notes app is not running, starting it");

            // Launch Notes app
            let _launch = Command::new("open")
                .arg("-a")
                .arg("Notes")
                .output()
                .context("Failed to launch Notes app")?;

            // Wait a moment for Notes to start
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        debug!("Fetching list of notes through AppleScript");

        // This is the most basic AppleScript possible - just return the first 5 notes
        let script = r#"
            tell application "Notes"
                set theNotes to {}
                
                try
                    repeat with i from 1 to 5
                        set thisNote to note i
                        set noteId to id of thisNote
                        set noteTitle to name of thisNote
                        
                        -- Use a delimiter that's unlikely to appear in titles
                        set end of theNotes to noteId & "|||" & noteTitle
                    end repeat
                    
                    return theNotes
                on error errMsg
                    return {"ERROR: " & errMsg}
                end try
            end tell
        "#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .context("Failed to execute AppleScript to list notes")?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Error listing notes: {}", error_message);
            return Err(anyhow::anyhow!("Failed to list notes: {}", error_message));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        debug!("Raw AppleScript output: {:?}", output_str);

        // Check for error
        if output_str.contains("ERROR:") {
            let error_msg =
                output_str.lines().find(|l| l.contains("ERROR:")).unwrap_or("Unknown error");

            error!("AppleScript returned an error: {}", error_msg);
            return Err(anyhow::anyhow!("AppleScript error: {}", error_msg));
        }

        // Parse output
        let mut notes = Vec::new();

        for line in output_str.lines() {
            let line = line.trim();

            // Skip empty lines, brackets, etc.
            if line.is_empty() || line == "{" || line == "}" {
                continue;
            }

            // Remove quotes and other formatting
            let clean_line = line.trim_start_matches('"').trim_end_matches('"').trim();

            if clean_line.is_empty() {
                continue;
            }

            debug!("Processing line: {:?}", clean_line);

            // Split by our custom delimiter
            if clean_line.contains("|||") {
                let parts: Vec<&str> = clean_line.split("|||").collect();

                if parts.len() >= 2 {
                    let id = parts[0].trim().to_string();
                    let title = parts[1].trim().to_string();

                    // Fetch the full note details
                    if let Ok(Some(full_note)) = self.get_note(&id) {
                        notes.push(full_note);
                    } else {
                        // Create a simplified note if we can't get full details
                        let note = AppleNote {
                            id,
                            title,
                            content: "[Content not available]".to_string(),
                            created_at: Local::now(),
                            updated_at: Local::now(),
                        };
                        notes.push(note);
                    }
                }
            }
        }

        info!("Successfully retrieved {} notes from Apple Notes", notes.len());
        Ok(notes)
    }

    /// Adds a new note to Apple Notes
    ///
    /// Creates a new note in the Apple Notes application with the specified title and content.
    ///
    /// # Arguments
    ///
    /// * `title` - The title for the new note
    /// * `content` - The content/body for the new note
    ///
    /// # Returns
    ///
    /// A `Result` containing the newly created `AppleNote` or an error if creation failed
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
    ///
    /// Retrieves a note from Apple Notes using its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the note to retrieve
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<AppleNote>` (None if the note wasn't found) or an error
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
    ///
    /// Updates an existing note in Apple Notes with new title and content.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the note to update
    /// * `title` - The new title for the note
    /// * `content` - The new content for the note
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean (true if updated, false if not found) or an error
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
    ///
    /// Removes a note from Apple Notes using its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the note to delete
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean (true if deleted, false if not found) or an error
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

    /// Creates a note from parsed AppleScript output parts
    ///
    /// # Arguments
    ///
    /// * `parts` - Vector of strings containing note details (id, title, content, etc.)
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `AppleNote` or an error
    fn create_note_from_parts(&self, parts: &[&str]) -> Result<AppleNote> {
        if parts.len() < 5 {
            return Err(anyhow::anyhow!("Insufficient data to create note"));
        }

        let id = parts[0].trim().to_string();
        let title = parts[1].trim().to_string();
        let content = parts[2].trim().to_string();
        let created_at = parse_apple_date(parts[3])?;
        let updated_at = parse_apple_date(parts[4])?;

        Ok(AppleNote { id, title, content, created_at, updated_at })
    }
}

/// Parses the output from AppleScript into structured note data
///
/// # Arguments
///
/// * `output` - The raw output string from AppleScript
///
/// # Returns
///
/// A `Result` containing a vector of parsed `AppleNote` objects or an error
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

/// Parses date strings from Apple Notes into DateTime objects
///
/// # Arguments
///
/// * `date_str` - The date string to parse
///
/// # Returns
///
/// A `Result` containing the parsed `DateTime<Local>` or an error
fn parse_apple_date(date_str: &str) -> Result<DateTime<Local>> {
    debug!("Parsing Apple date: {}", date_str);

    // Parse date string returned by AppleScript
    // Format could be one of several variations including:
    // - "Saturday, March 31, 2025 at 10:15:32 AM"
    // - "Monday, 31 March 2025 at 10:27:47 am"
    // We'll try multiple format patterns to be flexible

    let result = chrono::NaiveDateTime::parse_from_str(date_str, "%A, %B %d, %Y at %I:%M:%S %p")
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(date_str, "%a, %b %d, %Y at %I:%M:%S %p")
        })
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(date_str, "%A, %d %B %Y at %I:%M:%S %p"))
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(date_str, "%a, %d %b %Y at %I:%M:%S %p"))
        // Case-insensitive AM/PM
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(
                &date_str.replace(" am", " AM").replace(" pm", " PM"),
                "%A, %d %B %Y at %I:%M:%S %p",
            )
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(
                &date_str.replace(" am", " AM").replace(" pm", " PM"),
                "%A, %B %d, %Y at %I:%M:%S %p",
            )
        });

    match result {
        Ok(dt) => {
            let local_dt = Local
                .from_local_datetime(&dt)
                .earliest()
                .ok_or_else(|| anyhow::anyhow!("Failed to convert to local datetime"))?;

            debug!("Successfully parsed date: {} -> {}", date_str, local_dt);
            Ok(local_dt)
        }
        Err(e) => {
            error!("Failed to parse date '{}': {}", date_str, e);
            Err(anyhow::anyhow!("Failed to parse date: {} (Error: {})", date_str, e))
        }
    }
}

/// Command executor for the notes command.
///
/// This command provides integration with Apple Notes app, allowing users to:
/// - List all notes
/// - Add new notes
/// - View note details
/// - Update existing notes
/// - Delete notes
pub struct NotesCommand;

impl CommandExecutor for NotesCommand {
    fn can_handle(&self, command: &str) -> bool {
        command == "notes" || command == "note"
    }

    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            debug!("Notes command called with args: {:?}, flags: {:?}", args.args, args.flags);

            // Initialize Apple Notes storage
            let storage = match AppleNotesStorage::new() {
                Ok(storage) => storage,
                Err(err) => {
                    error!("Failed to initialize Apple Notes storage: {}", err);
                    println!(
                        "Error: Could not connect to Apple Notes. Make sure the Notes app is installed."
                    );
                    return Ok(());
                }
            };

            // Handle command based on subcommand or if no subcommand, handle as add note with flags
            match args.args.get(0).map(|s| s.as_str()) {
                Some("add") => handle_add_note_positional(&storage, &args),
                Some("list") => handle_list_notes(&storage),
                Some("view") => handle_view_note(&storage, &args),
                Some("delete") => handle_delete_note(&storage, &args),
                Some("update") => handle_update_note(&storage, &args),
                _ => {
                    // If no subcommand but we have a title argument, treat as add with flags
                    if !args.args.is_empty() {
                        handle_add_note_with_flags(&storage, &args)
                    } else {
                        // Default to list if no arguments provided
                        handle_list_notes(&storage)
                    }
                }
            }
        })
    }
}

/// Handle adding a note when using positional arguments
///
/// Processes a command like: ducktape note add "title" "content"
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
/// * `args` - Command arguments
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn handle_add_note_positional(storage: &AppleNotesStorage, args: &CommandArgs) -> Result<()> {
    if args.args.len() < 3 {
        println!("Usage: ducktape note add <title> <content>");
        println!("Example: ducktape note add \"Shopping List\" \"Milk, Eggs, Bread\"");
        return Ok(());
    }

    let title = &args.args[1];
    let content = args.args[2..].join(" ");

    add_note(storage, title, &content)
}

/// Handle adding a note when using flags
///
/// Processes a command like: ducktape note "title" --content "content"
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
/// * `args` - Command arguments
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn handle_add_note_with_flags(storage: &AppleNotesStorage, args: &CommandArgs) -> Result<()> {
    let title = &args.args[0];

    // Check if --content flag is provided
    let content = match args.flags.get("--content") {
        Some(Some(content)) => content,
        _ => {
            println!("Usage: ducktape note \"<title>\" --content \"<content>\"");
            println!("Example: ducktape note \"Shopping List\" --content \"Milk, Eggs, Bread\"");
            return Ok(());
        }
    };

    add_note(storage, title, content)
}

/// Common function to add a note to Apple Notes
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
/// * `title` - The title for the new note
/// * `content` - The content for the new note
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn add_note(storage: &AppleNotesStorage, title: &str, content: &str) -> Result<()> {
    match storage.add_note(title, content) {
        Ok(note) => {
            println!("Note added successfully to Apple Notes");
            println!("Title: {}", note.title);
            println!("ID: {}", note.id);
            Ok(())
        }
        Err(err) => {
            error!("Failed to add note to Apple Notes: {}", err);
            println!("Error: Failed to add note to Apple Notes. {}", err);
            Ok(())
        }
    }
}

/// Handle listing all notes from Apple Notes
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn handle_list_notes(storage: &AppleNotesStorage) -> Result<()> {
    debug!("Executing notes list command with Apple Notes");
    match storage.list_notes() {
        Ok(notes) => {
            if notes.is_empty() {
                println!("No notes found in Apple Notes");
                return Ok(());
            }

            println!("{} notes found in Apple Notes:", notes.len());
            for note in &notes {
                let truncated_content = if note.content.len() > 50 {
                    format!("{}...", &note.content[..47])
                } else {
                    note.content.clone()
                };
                println!(
                    "{}: {} - {} ({})",
                    note.id.blue(),
                    note.title.green(),
                    truncated_content,
                    note.created_at.format("%Y-%m-%d %H:%M:%S").to_string().yellow()
                );
            }
            Ok(())
        }
        Err(err) => {
            error!("Failed to list notes from Apple Notes: {}", err);
            println!("Error: Failed to list notes from Apple Notes. {}", err);
            Ok(())
        }
    }
}

/// Handle viewing a specific note by ID
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
/// * `args` - Command arguments
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn handle_view_note(storage: &AppleNotesStorage, args: &CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Please provide a note ID");
        println!("Usage: ducktape note view <note-id>");
        return Ok(());
    }

    let id = &args.args[1];
    match storage.get_note(id) {
        Ok(Some(note)) => {
            println!("{}: {}", "ID".blue(), note.id);
            println!("{}: {}", "Title".green(), note.title);
            println!("{}: {}", "Created".yellow(), note.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("{}: {}", "Updated".yellow(), note.updated_at.format("%Y-%m-%d %H:%M:%S"));
            println!("\n{}", note.content);
        }
        Ok(None) => println!("Note with ID {} not found in Apple Notes", id),
        Err(err) => {
            error!("Failed to get note from Apple Notes: {}", err);
            println!("Error: Failed to get note from Apple Notes. {}", err);
        }
    }
    Ok(())
}

/// Handle deleting a note by ID
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
/// * `args` - Command arguments
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn handle_delete_note(storage: &AppleNotesStorage, args: &CommandArgs) -> Result<()> {
    if args.args.len() < 2 {
        println!("Please provide a note ID");
        println!("Usage: ducktape note delete <note-id>");
        return Ok(());
    }

    let id = &args.args[1];
    match storage.delete_note(id) {
        Ok(true) => println!("Note deleted successfully from Apple Notes"),
        Ok(false) => println!("Note with ID {} not found in Apple Notes", id),
        Err(err) => {
            error!("Failed to delete note from Apple Notes: {}", err);
            println!("Error: Failed to delete note from Apple Notes. {}", err);
        }
    }
    Ok(())
}

/// Handle updating a note
///
/// # Arguments
///
/// * `storage` - The AppleNotesStorage instance
/// * `args` - Command arguments
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
fn handle_update_note(storage: &AppleNotesStorage, args: &CommandArgs) -> Result<()> {
    if args.args.len() < 4 {
        println!("Please provide a note ID, title, and content");
        println!("Usage: ducktape note update <note-id> <new-title> <new-content>");
        return Ok(());
    }

    let id = &args.args[1];
    let title = &args.args[2];
    let content = args.args[3..].join(" ");

    match storage.update_note(id, title, &content) {
        Ok(true) => println!("Note updated successfully in Apple Notes"),
        Ok(false) => println!("Note with ID {} not found in Apple Notes", id),
        Err(err) => {
            error!("Failed to update note in Apple Notes: {}", err);
            println!("Error: Failed to update note in Apple Notes. {}", err);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock for AppleNotesStorage to avoid actual AppleScript calls during tests
    struct MockAppleNotesStorage {
        notes: Vec<AppleNote>,
    }

    impl MockAppleNotesStorage {
        fn new() -> Self {
            let now = Local::now();

            // Create some test notes
            let notes = vec![
                AppleNote {
                    id: "note-id-1".to_string(),
                    title: "Test Note 1".to_string(),
                    content: "This is test note 1 content".to_string(),
                    created_at: now,
                    updated_at: now,
                },
                AppleNote {
                    id: "note-id-2".to_string(),
                    title: "Test Note 2".to_string(),
                    content: "This is test note 2 content".to_string(),
                    created_at: now,
                    updated_at: now,
                },
            ];

            Self { notes }
        }

        fn list_notes(&self) -> Result<Vec<AppleNote>> {
            Ok(self.notes.clone())
        }

        fn add_note(&mut self, title: &str, content: &str) -> Result<AppleNote> {
            let now = Local::now();
            let note = AppleNote {
                id: format!("new-note-id-{}", self.notes.len() + 1),
                title: title.to_string(),
                content: content.to_string(),
                created_at: now,
                updated_at: now,
            };

            self.notes.push(note.clone());
            Ok(note)
        }

        fn get_note(&self, id: &str) -> Result<Option<AppleNote>> {
            let note = self.notes.iter().find(|n| n.id == id).cloned();
            Ok(note)
        }

        fn update_note(&mut self, id: &str, title: &str, content: &str) -> Result<bool> {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.title = title.to_string();
                note.content = content.to_string();
                note.updated_at = Local::now();
                Ok(true)
            } else {
                Ok(false)
            }
        }

        fn delete_note(&mut self, id: &str) -> Result<bool> {
            let initial_len = self.notes.len();
            self.notes.retain(|n| n.id != id);
            Ok(self.notes.len() < initial_len)
        }
    }

    #[test]
    fn test_handle_list_notes_empty() {
        let storage = MockAppleNotesStorage { notes: vec![] };
        let result = handle_list_notes_mock(&storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_notes_with_notes() {
        let storage = MockAppleNotesStorage::new();
        let result = handle_list_notes_mock(&storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_add_note_with_flags() {
        let mut storage = MockAppleNotesStorage::new();
        let initial_count = storage.notes.len();

        // Create command args with flags
        let mut flags = HashMap::new();
        flags.insert("--content".to_string(), Some("Test content".to_string()));

        let args = CommandArgs {
            command: "note".to_string(),
            args: vec!["Test Title".to_string()],
            flags,
        };

        let result = handle_add_note_with_flags_mock(&mut storage, &args);
        assert!(result.is_ok());
        assert_eq!(storage.notes.len(), initial_count + 1);

        // Verify the new note was added with correct title and content
        let new_note = storage.notes.last().unwrap();
        assert_eq!(new_note.title, "Test Title");
        assert_eq!(new_note.content, "Test content");
    }

    #[test]
    fn test_handle_add_note_with_positional_args() {
        let mut storage = MockAppleNotesStorage::new();
        let initial_count = storage.notes.len();

        // Create command args with positional arguments
        let args = CommandArgs {
            command: "note".to_string(),
            args: vec!["add".to_string(), "Test Title".to_string(), "Test content".to_string()],
            flags: HashMap::new(),
        };

        let result = handle_add_note_positional_mock(&mut storage, &args);
        assert!(result.is_ok());
        assert_eq!(storage.notes.len(), initial_count + 1);

        // Verify the new note was added with correct title and content
        let new_note = storage.notes.last().unwrap();
        assert_eq!(new_note.title, "Test Title");
        assert_eq!(new_note.content, "Test content");
    }

    #[test]
    fn test_handle_view_note_existing() {
        let storage = MockAppleNotesStorage::new();
        let existing_id = "note-id-1";

        // Create command args to view an existing note
        let args = CommandArgs {
            command: "note".to_string(),
            args: vec!["view".to_string(), existing_id.to_string()],
            flags: HashMap::new(),
        };

        let result = handle_view_note_mock(&storage, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_view_note_nonexistent() {
        let storage = MockAppleNotesStorage::new();
        let nonexistent_id = "nonexistent-id";

        // Create command args to view a nonexistent note
        let args = CommandArgs {
            command: "note".to_string(),
            args: vec!["view".to_string(), nonexistent_id.to_string()],
            flags: HashMap::new(),
        };

        let result = handle_view_note_mock(&storage, &args);
        assert!(result.is_ok()); // Should still be Ok, just print a message
    }

    #[test]
    fn test_handle_update_note_existing() {
        let mut storage = MockAppleNotesStorage::new();
        let existing_id = "note-id-1";

        // Create command args to update an existing note
        let args = CommandArgs {
            command: "note".to_string(),
            args: vec![
                "update".to_string(),
                existing_id.to_string(),
                "Updated Title".to_string(),
                "Updated content".to_string(),
            ],
            flags: HashMap::new(),
        };

        let result = handle_update_note_mock(&mut storage, &args);
        assert!(result.is_ok());

        // Verify the note was updated
        let updated_note = storage.notes.iter().find(|n| n.id == existing_id).unwrap();
        assert_eq!(updated_note.title, "Updated Title");
        assert_eq!(updated_note.content, "Updated content");
    }

    #[test]
    fn test_handle_delete_note_existing() {
        let mut storage = MockAppleNotesStorage::new();
        let initial_count = storage.notes.len();
        let existing_id = "note-id-1";

        // Create command args to delete an existing note
        let args = CommandArgs {
            command: "note".to_string(),
            args: vec!["delete".to_string(), existing_id.to_string()],
            flags: HashMap::new(),
        };

        let result = handle_delete_note_mock(&mut storage, &args);
        assert!(result.is_ok());
        assert_eq!(storage.notes.len(), initial_count - 1);

        // Verify the note no longer exists
        assert!(storage.notes.iter().find(|n| n.id == existing_id).is_none());
    }

    #[test]
    fn test_handle_delete_note_nonexistent() {
        let mut storage = MockAppleNotesStorage::new();
        let initial_count = storage.notes.len();
        let nonexistent_id = "nonexistent-id";

        // Create command args to delete a nonexistent note
        let args = CommandArgs {
            command: "note".to_string(),
            args: vec!["delete".to_string(), nonexistent_id.to_string()],
            flags: HashMap::new(),
        };

        let result = handle_delete_note_mock(&mut storage, &args);
        assert!(result.is_ok()); // Should still be Ok, just print a message
        assert_eq!(storage.notes.len(), initial_count); // No notes should be deleted
    }

    #[test]
    fn test_parse_apple_date() {
        let date_formats = [
            "Monday, 31 March 2025 at 10:27:47 am",
            "Monday, March 31, 2025 at 10:27:47 AM",
            "Mon, 31 Mar 2025 at 10:27:47 am",
        ];

        for format in &date_formats {
            let result = parse_apple_date(format);
            assert!(result.is_ok(), "Failed to parse date: {}", format);

            if let Ok(date) = result {
                assert_eq!(date.format("%Y-%m-%d").to_string(), "2025-03-31");
            }
        }
    }

    // Mock handler functions that use our mock implementation

    fn handle_list_notes_mock(storage: &MockAppleNotesStorage) -> Result<()> {
        match storage.list_notes() {
            Ok(notes) => {
                if notes.is_empty() {
                    println!("No notes found in Apple Notes");
                    return Ok(());
                }

                println!("{} notes found in Apple Notes:", notes.len());
                for note in &notes {
                    let truncated_content = if note.content.len() > 50 {
                        format!("{}...", &note.content[..47])
                    } else {
                        note.content.clone()
                    };
                    println!(
                        "{}: {} - {} ({})",
                        note.id,
                        note.title,
                        truncated_content,
                        note.created_at.format("%Y-%m-%d %H:%M:%S")
                    );
                }
                Ok(())
            }
            Err(err) => {
                println!("Error: Failed to list notes from Apple Notes. {}", err);
                Ok(())
            }
        }
    }

    fn handle_add_note_with_flags_mock(
        storage: &mut MockAppleNotesStorage,
        args: &CommandArgs,
    ) -> Result<()> {
        let title = &args.args[0];

        // Check if --content flag is provided
        let content = match args.flags.get("--content") {
            Some(Some(content)) => content,
            _ => {
                println!("Usage: ducktape note \"<title>\" --content \"<content>\"");
                println!(
                    "Example: ducktape note \"Shopping List\" --content \"Milk, Eggs, Bread\""
                );
                return Ok(());
            }
        };

        add_note_mock(storage, title, content)
    }

    fn handle_add_note_positional_mock(
        storage: &mut MockAppleNotesStorage,
        args: &CommandArgs,
    ) -> Result<()> {
        if args.args.len() < 3 {
            println!("Usage: ducktape note add <title> <content>");
            println!("Example: ducktape note add \"Shopping List\" \"Milk, Eggs, Bread\"");
            return Ok(());
        }

        let title = &args.args[1];
        let content = &args.args[2];

        add_note_mock(storage, title, content)
    }

    fn add_note_mock(
        storage: &mut MockAppleNotesStorage,
        title: &str,
        content: &str,
    ) -> Result<()> {
        match storage.add_note(title, content) {
            Ok(note) => {
                println!("Note added successfully to Apple Notes");
                println!("Title: {}", note.title);
                println!("ID: {}", note.id);
                Ok(())
            }
            Err(err) => {
                println!("Error: Failed to add note to Apple Notes. {}", err);
                Ok(())
            }
        }
    }

    fn handle_view_note_mock(storage: &MockAppleNotesStorage, args: &CommandArgs) -> Result<()> {
        if args.args.len() < 2 {
            println!("Please provide a note ID");
            println!("Usage: ducktape note view <note-id>");
            return Ok(());
        }

        let id = &args.args[1];
        match storage.get_note(id) {
            Ok(Some(note)) => {
                println!("ID: {}", note.id);
                println!("Title: {}", note.title);
                println!("Created: {}", note.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!("Updated: {}", note.updated_at.format("%Y-%m-%d %H:%M:%S"));
                println!("\n{}", note.content);
            }
            Ok(None) => println!("Note with ID {} not found in Apple Notes", id),
            Err(err) => {
                println!("Error: Failed to get note from Apple Notes. {}", err);
            }
        }
        Ok(())
    }

    fn handle_update_note_mock(
        storage: &mut MockAppleNotesStorage,
        args: &CommandArgs,
    ) -> Result<()> {
        if args.args.len() < 4 {
            println!("Please provide a note ID, title, and content");
            println!("Usage: ducktape note update <note-id> <new-title> <new-content>");
            return Ok(());
        }

        let id = &args.args[1];
        let title = &args.args[2];
        let content = &args.args[3];

        match storage.update_note(id, title, content) {
            Ok(true) => println!("Note updated successfully in Apple Notes"),
            Ok(false) => println!("Note with ID {} not found in Apple Notes", id),
            Err(err) => {
                println!("Error: Failed to update note in Apple Notes. {}", err);
            }
        }
        Ok(())
    }

    fn handle_delete_note_mock(
        storage: &mut MockAppleNotesStorage,
        args: &CommandArgs,
    ) -> Result<()> {
        if args.args.len() < 2 {
            println!("Please provide a note ID");
            println!("Usage: ducktape note delete <note-id>");
            return Ok(());
        }

        let id = &args.args[1];
        match storage.delete_note(id) {
            Ok(true) => println!("Note deleted successfully from Apple Notes"),
            Ok(false) => println!("Note with ID {} not found in Apple Notes", id),
            Err(err) => {
                println!("Error: Failed to delete note from Apple Notes. {}", err);
            }
        }
        Ok(())
    }
}
