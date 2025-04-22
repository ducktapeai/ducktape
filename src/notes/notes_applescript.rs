//! AppleScript integration for Apple Notes.

use anyhow::{Result, anyhow};
use log::{debug, error, info};
use tokio::process::Command;

use crate::notes::notes_types::{NoteConfig, NoteItem, NotesError};
use crate::notes::notes_util::{escape_applescript_string, parse_notes_list};
use crate::notes::notes_validation::{
    validate_folder_name, validate_note_config, validate_note_title, validate_search_keyword,
};

/// Creates a new note in Apple Notes
pub async fn create_note(config: NoteConfig<'_>) -> Result<()> {
    // Validate the note configuration
    validate_note_config(&config)?;

    // First ensure Notes.app is running
    ensure_notes_running().await?;

    let folder_script = if let Some(folder) = config.folder {
        let escaped_folder = escape_applescript_string(folder);
        format!(
            r#"
            set targetFolder to missing value
            repeat with f in folders
                if name of f is "{}" then
                    set targetFolder to f
                    exit repeat
                end if
            end repeat
            if targetFolder is missing value then
                set targetFolder to make new folder with properties {{name:"{}"}}
            end if
            tell targetFolder"#,
            escaped_folder, escaped_folder
        )
    } else {
        "tell default account".to_string()
    };

    // Escape title and content to prevent command injection
    let escaped_title = escape_applescript_string(config.title);
    let escaped_content = escape_applescript_string(config.content);

    let script = format!(
        r#"tell application "Notes"
            try
                {}
                    make new note with properties {{name:"{}", body:"{}"}}
                end tell
                return "Success: Note created"
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
        folder_script, escaped_title, escaped_content
    );

    debug!("Executing AppleScript for note creation: {}", escaped_title);
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await
        .map_err(|e| NotesError::ScriptError(e.to_string()))?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Success") {
        info!("Note created: {}", config.title);
        Ok(())
    } else {
        let error_message = result.to_string();
        error!("Failed to create note: {}", error_message);
        Err(anyhow!("Failed to create note: {}", error_message))
    }
}

/// Lists all notes from Apple Notes
pub async fn list_notes() -> Result<Vec<NoteItem>> {
    // First ensure Notes.app is running
    ensure_notes_running().await?;

    let script = r#"tell application "Notes"
        try
            set notesList to {}
            repeat with n in notes
                set noteFolder to "Notes"
                try
                    set noteFolder to name of container of n
                end try
                set noteInfo to {name:name of n, folder:noteFolder}
                copy noteInfo to end of notesList
            end repeat
            return notesList
        on error errMsg
            return "Error: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await
        .map_err(|e| NotesError::ScriptError(e.to_string()))?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Error") {
        error!("Failed to list notes: {}", result);
        return Err(anyhow!("Failed to list notes: {}", result));
    }

    // Parse the notes list from the AppleScript output
    let notes_list = parse_notes_list(&result);

    // Convert to our NoteItem struct
    let note_items: Vec<NoteItem> = notes_list
        .into_iter()
        .map(|(title, folder)| NoteItem {
            title,
            folder,
            created: None, // Apple Notes doesn't easily provide creation date via AppleScript
            modified: None, // Apple Notes doesn't easily provide modification date via AppleScript
        })
        .collect();

    Ok(note_items)
}

/// Gets a list of all note folders from Apple Notes
pub async fn get_note_folders() -> Result<Vec<String>> {
    // First ensure Notes.app is running
    ensure_notes_running().await?;

    let script = r#"tell application "Notes"
        try
            set folderList to {}
            repeat with f in folders
                copy (name of f) to end of folderList
            end repeat
            return folderList
        on error errMsg
            return "Error: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .await
        .map_err(|e| NotesError::ScriptError(e.to_string()))?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Error") {
        error!("Failed to get note folders: {}", result);
        return Err(anyhow!("Failed to get note folders: {}", result));
    }

    let folders = result
        .trim_matches('{')
        .trim_matches('}')
        .split(", ")
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_matches('"').to_string())
        .collect();

    Ok(folders)
}

/// Deletes a note by title (and optionally folder)
pub async fn delete_note(title: &str, folder: Option<&str>) -> Result<()> {
    // Validate inputs
    validate_note_title(title)?;
    if let Some(folder_name) = folder {
        validate_folder_name(folder_name)?;
    }

    // First ensure Notes.app is running
    ensure_notes_running().await?;

    let escaped_title = escape_applescript_string(title);

    let folder_condition = if let Some(folder_name) = folder {
        let escaped_folder = escape_applescript_string(folder_name);
        format!(
            "name of n is \"{}\" and name of container of n is \"{}\"",
            escaped_title, escaped_folder
        )
    } else {
        format!("name of n is \"{}\"", escaped_title)
    };

    let script = format!(
        r#"tell application "Notes"
            try
                set noteFound to false
                repeat with n in notes
                    if {} then
                        delete n
                        set noteFound to true
                        exit repeat
                    end if
                end repeat
                
                if noteFound then
                    return "Success: Note deleted"
                else
                    return "Error: Note not found"
                end if
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
        folder_condition
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await
        .map_err(|e| NotesError::ScriptError(e.to_string()))?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Success") {
        info!("Note deleted: {}", title);
        Ok(())
    } else if result.contains("Note not found") {
        let folder_info = folder.map_or("".to_string(), |f| format!(" in folder '{}'", f));
        error!("Note '{}'{} not found", title, folder_info);
        Err(NotesError::NoteNotFound(title.to_string()).into())
    } else {
        let error_message = result.to_string();
        error!("Failed to delete note: {}", error_message);
        Err(anyhow!("Failed to delete note: {}", error_message))
    }
}

/// Searches notes by keyword
pub async fn search_notes(keyword: &str) -> Result<Vec<NoteItem>> {
    // Validate the search keyword
    validate_search_keyword(keyword)?;

    // First ensure Notes.app is running
    ensure_notes_running().await?;

    let escaped_keyword = escape_applescript_string(keyword);

    let script = format!(
        r#"tell application "Notes"
            try
                set matchingNotes to {{}}
                repeat with n in notes
                    if name of n contains "{0}" or body of n contains "{0}" then
                        set noteFolder to "Notes"
                        try
                            set noteFolder to name of container of n
                        end try
                        set noteInfo to {{name:name of n, folder:noteFolder}}
                        copy noteInfo to end of matchingNotes
                    end if
                end repeat
                return matchingNotes
            on error errMsg
                return "Error: " & errMsg
            end try
        end tell"#,
        escaped_keyword
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await
        .map_err(|e| NotesError::ScriptError(e.to_string()))?;

    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Error") {
        error!("Failed to search notes: {}", result);
        return Err(anyhow!("Failed to search notes: {}", result));
    }

    // Parse the notes list from the AppleScript output
    let notes_list = parse_notes_list(&result);

    // Convert to our NoteItem struct
    let note_items: Vec<NoteItem> = notes_list
        .into_iter()
        .map(|(title, folder)| NoteItem { title, folder, created: None, modified: None })
        .collect();

    Ok(note_items)
}

/// Ensures the Notes application is running
async fn ensure_notes_running() -> Result<()> {
    let check_script = r#"tell application "Notes"
        if it is not running then
            launch
            delay 1
        end if
        return true
    end tell"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(check_script)
        .output()
        .await
        .map_err(|e| NotesError::ScriptError(e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Failed to ensure Notes app is running: {}", error);
        Err(NotesError::NotRunning.into())
    }
}
