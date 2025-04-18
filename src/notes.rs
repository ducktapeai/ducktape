use anyhow::Result;
use anyhow::anyhow;
use chrono::Local;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct NoteConfig<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub folder: Option<&'a str>,
}

impl<'a> NoteConfig<'a> {
    #[allow(dead_code)]
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self { title, content, folder: None }
    }
}

// Helper function to escape strings for AppleScript to prevent command injection
fn escape_applescript_string(input: &str) -> String {
    // First replace double quotes with escaped quotes for AppleScript
    let escaped = input.replace("\"", "\"\"");

    // Remove any control characters that could interfere with AppleScript execution
    escaped
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

#[allow(dead_code)]
pub fn create_note(config: NoteConfig) -> Result<()> {
    let folder_script = if let Some(folder) = config.folder {
        let escaped_folder = escape_applescript_string(folder);
        format!("tell folder \"{}\" of default account", escaped_folder)
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

    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Success") {
        println!("Note created: {}", config.title);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to create note: {}", result))
    }
}

#[allow(dead_code)]
pub fn list_notes() -> Result<()> {
    let script = r#"tell application "Notes"
        try
            set noteList to {}
            repeat with n in notes
                copy (name of n) to end of noteList
            end repeat
            return noteList
        on error errMsg
            return "Error: " & errMsg
        end try
    end tell"#;

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

    let result = String::from_utf8_lossy(&output.stdout);
    if result.starts_with("Error:") {
        Err(anyhow::anyhow!("Failed to list notes: {}", result))
    } else {
        println!("Notes:");
        let notes = result.trim_matches('{').trim_matches('}');
        if !notes.is_empty() {
            for note in notes.split(", ") {
                println!("  - {}", note.trim_matches('"'));
            }
        } else {
            println!("  No notes found");
        }
        Ok(())
    }
}

// Optional: Keep these for future use with #[allow(dead_code)]
#[allow(dead_code)]
const NOTES_DIR: &str = "notes";

#[allow(dead_code)]
struct Note {
    title: String,
    content: String,
    tags: Vec<String>,
}

// Rest of unused functions can be removed if not needed for future development
#[allow(dead_code)]
pub fn create_note_local(title: &str, content: &str, tags: &[String]) -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let mut notes_dir = home_dir;
    notes_dir.push(".ducktape");
    notes_dir.push(NOTES_DIR);
    create_dir_all(&notes_dir)?;

    let filename = sanitize_filename(title);
    let mut file_path = notes_dir;
    file_path.push(format!("{}.md", filename));

    let created_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tags_str =
        if !tags.is_empty() { format!("\nTags: {}", tags.join(", ")) } else { String::new() };

    let content = format!("# {}\nCreated: {}{}\n\n{}\n", title, created_at, tags_str, content);

    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)?;

    file.write_all(content.as_bytes())?;
    println!("Note '{}' created successfully", title);
    Ok(())
}

#[allow(dead_code)]
pub fn create_note_apple(config: NoteConfig) -> Result<()> {
    let folder_code = if let Some(folder) = config.folder {
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
        folder_code, escaped_title, escaped_content
    );

    let output = Command::new("osascript").arg("-e").arg(&script).output()?;
    let result = String::from_utf8_lossy(&output.stdout);

    if result.contains("Success") {
        println!("Note created: {}", config.title);
        Ok(())
    } else {
        Err(anyhow!("Failed to create note: {}", result))
    }
}

#[allow(dead_code)]
pub fn list_notes_local() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let mut notes_dir = home_dir;
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

#[allow(dead_code)]
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

    let output = Command::new("osascript").arg("-e").arg(script).output()?;

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

#[allow(dead_code)]
pub fn read_note_local(title: &str) -> Result<()> {
    let mut file_path = get_notes_dir()?;
    file_path.push(format!("{}.md", sanitize_filename(title)));

    if !file_path.exists() {
        return Err(anyhow!("Note '{}' not found", title));
    }

    let content = std::fs::read_to_string(file_path)?;
    println!("{}", content);
    Ok(())
}

#[allow(dead_code)]
fn get_notes_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let mut notes_dir = home_dir;
    notes_dir.push(".ducktape");
    notes_dir.push(NOTES_DIR);
    Ok(notes_dir)
}

#[allow(dead_code)]
fn sanitize_filename(filename: &str) -> String {
    filename.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")
}
