use anyhow::{anyhow, Result};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use std::process::Command;
use crate::state::{NoteItem, StateManager};

const NOTES_DIR: &str = "notes";

pub struct Note {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NoteConfig<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub folder: Option<&'a str>,
}

impl<'a> NoteConfig<'a> {
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self {
            title,
            content,
            folder: None,
        }
    }
}

pub fn create_note_local(title: &str, content: &str, tags: &[String]) -> Result<()> {
    let mut notes_dir = dirs::home_dir().expect("Could not find home directory");
    notes_dir.push(".ducktape");
    notes_dir.push(NOTES_DIR);
    create_dir_all(&notes_dir)?;

    let filename = sanitize_filename(title);
    let mut file_path = notes_dir;
    file_path.push(format!("{}.md", filename));

    let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tags_str = if !tags.is_empty() {
        format!("\nTags: {}", tags.join(", "))
    } else {
        String::new()
    };

    let content = format!(
        "# {}\nCreated: {}{}\n\n{}\n",
        title, created_at, tags_str, content
    );

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;

    file.write_all(content.as_bytes())?;
    println!("Note '{}' created successfully", title);
    Ok(())
}

pub fn create_note_apple(config: NoteConfig) -> Result<()> {
    let folder_code = if let Some(folder) = config.folder {
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
            folder, folder
        )
    } else {
        "tell default account".to_string()
    };

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
        folder_code,
        config.title,
        config.content
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;
    
    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        println!("Note created: {}", config.title);
        Ok(())
    } else {
        Err(anyhow!("Failed to create note: {}", result))
    }
}

pub fn list_notes_local() -> Result<()> {
    let mut notes_dir = dirs::home_dir().expect("Could not find home directory");
    notes_dir.push(".ducktape");
    notes_dir.push(NOTES_DIR);

    if !notes_dir.exists() {
        println!("No notes found");
        return Ok(());
    }

    println!("Notes:");
    for entry in std::fs::read_dir(notes_dir)? {
        let entry = entry?;
        if entry.path().extension().map_or(false, |ext| ext == "md") {
            if let Some(title) = entry.path().file_stem() {
                println!("  - {}", title.to_string_lossy().replace('_', " "));
            }
        }
    }
    Ok(())
}

pub fn list_notes_apple() -> Result<()> {
    let script = r#"tell application "Notes"
        try
            set output to {}
            repeat with n in notes
                copy {name:name of n, folder:(name of folder of container of n)} to end of output
            end repeat
            return output
        on error errMsg
            return "Error: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Error") {
        Err(anyhow!("Failed to list notes: {}", result))
    } else {
        println!("Notes:");
        // Parse and display the notes list
        for note in result.lines() {
            if note.contains("name:") {
                println!("  - {}", note.trim());
            }
        }
        Ok(())
    }
}

pub fn read_note_local(title: &str) -> Result<()> {
    let mut file_path = get_notes_dir();
    file_path.push(format!("{}.md", sanitize_filename(title)));

    if !file_path.exists() {
        return Err(anyhow!("Note '{}' not found", title));
    }

    let content = std::fs::read_to_string(file_path)?;
    println!("{}", content);
    Ok(())
}

fn get_notes_dir() -> PathBuf {
    let mut notes_dir = dirs::home_dir().expect("Could not find home directory");
    notes_dir.push(".ducktape");
    notes_dir.push(NOTES_DIR);
    notes_dir
}

fn sanitize_filename(filename: &str) -> String {
    filename.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")
}

pub fn create_note(config: NoteConfig) -> Result<()> {
    let folder_script = if let Some(folder) = config.folder {
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
            folder, folder
        )
    } else {
        "tell default account".to_string()
    };

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
        folder_script,
        config.title,
        config.content
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.contains("Success") {
        let note_item = NoteItem {
            title: config.title.to_string(),
            content: config.content.to_string(),
            folder: config.folder.map(String::from),
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        StateManager::new()?.add(note_item)?;
        
        println!("Note created: {}", config.title);
        Ok(())
    } else {
        Err(anyhow!("Failed to create note: {}", result))
    }
}

pub fn list_notes() -> Result<()> {
    let notes = crate::state::load_notes()?;
    
    if notes.is_empty() {
        println!("No notes found");
        return Ok(());
    }

    println!("Notes:");
    for note in notes {
        println!("  - {} [{}]", note.title, 
            note.folder.as_deref().unwrap_or("Default"));
    }
    Ok(())
}
