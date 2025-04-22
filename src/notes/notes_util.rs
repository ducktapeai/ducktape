//! Utility functions for the notes module.

/// Helper function to escape strings for AppleScript to prevent command injection
pub fn escape_applescript_string(input: &str) -> String {
    // First replace double quotes with escaped quotes for AppleScript
    let escaped = input.replace("\"", "\"\"");

    // Remove any control characters that could interfere with AppleScript execution
    escaped
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

/// Parse a notes list from AppleScript output
pub fn parse_notes_list(output: &str) -> Vec<(String, String)> {
    let mut notes = Vec::new();
    
    // Parse the output format which is expected to be a list of records
    // in the format {name:"Note Title", folder:"Folder Name"}, ...
    let output = output.trim_matches('{').trim_matches('}');
    
    if output.is_empty() {
        return notes;
    }
    
    // Split by record boundaries
    let records: Vec<&str> = output.split("}, {").collect();
    
    for record in records {
        let clean_record = record.replace('{', "").replace('}', "");
        let mut title = String::new();
        let mut folder = String::new();
        
        // Extract properties
        for prop in clean_record.split(", ") {
            if prop.starts_with("name:") {
                title = prop.trim_start_matches("name:").trim_matches('"').to_string();
            } else if prop.starts_with("folder:") {
                folder = prop.trim_start_matches("folder:").trim_matches('"').to_string();
            }
        }
        
        if !title.is_empty() {
            notes.push((title, folder));
        }
    }
    
    notes
}

/// Format text for display in the terminal
pub fn format_note_for_display(title: &str, content: &str) -> String {
    format!("Title: {}\n\n{}", title, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_applescript_string() {
        let input = "Note with \"quotes\"";
        let escaped = escape_applescript_string(input);
        assert_eq!(escaped, "Note with \"\"quotes\"\"");
        
        let input_with_control = "Note with \x07 bell";
        let escaped = escape_applescript_string(input_with_control);
        assert_eq!(escaped, "Note with  bell");
    }

    #[test]
    fn test_parse_notes_list() {
        let input = "{name:\"Note 1\", folder:\"Folder 1\"}, {name:\"Note 2\", folder:\"Folder 2\"}";
        let notes = parse_notes_list(input);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0], ("Note 1".to_string(), "Folder 1".to_string()));
        assert_eq!(notes[1], ("Note 2".to_string(), "Folder 2".to_string()));
    }
}