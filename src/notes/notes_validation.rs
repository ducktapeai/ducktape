//! Validation functions for notes operations.

use anyhow::{Result, anyhow};
use crate::notes::notes_types::NoteConfig;
use log::debug;

/// Validates a note configuration before creating a note
pub fn validate_note_config(config: &NoteConfig) -> Result<()> {
    // Title validation
    if config.title.is_empty() {
        return Err(anyhow!("Note title cannot be empty"));
    }
    
    if config.title.len() > 255 {
        return Err(anyhow!("Note title is too long (max 255 characters)"));
    }
    
    // Content validation - allow empty content
    if config.content.len() > 1_000_000 {  // 1MB limit for content
        return Err(anyhow!("Note content is too large (max 1MB)"));
    }
    
    // Folder validation if provided
    if let Some(folder) = config.folder {
        if folder.is_empty() {
            return Err(anyhow!("Folder name cannot be empty"));
        }
        
        if folder.len() > 255 {
            return Err(anyhow!("Folder name is too long (max 255 characters)"));
        }
    }
    
    debug!("Note configuration validated successfully: {:?}", config);
    Ok(())
}

/// Validates a note title before performing operations
pub fn validate_note_title(title: &str) -> Result<()> {
    if title.is_empty() {
        return Err(anyhow!("Note title cannot be empty"));
    }
    
    if title.len() > 255 {
        return Err(anyhow!("Note title is too long (max 255 characters)"));
    }
    
    Ok(())
}

/// Validates a note folder name before performing operations
pub fn validate_folder_name(folder: &str) -> Result<()> {
    if folder.is_empty() {
        return Err(anyhow!("Folder name cannot be empty"));
    }
    
    if folder.len() > 255 {
        return Err(anyhow!("Folder name is too long (max 255 characters)"));
    }
    
    Ok(())
}

/// Validates a search keyword
pub fn validate_search_keyword(keyword: &str) -> Result<()> {
    if keyword.is_empty() {
        return Err(anyhow!("Search keyword cannot be empty"));
    }
    
    if keyword.len() < 2 {
        return Err(anyhow!("Search keyword must be at least 2 characters"));
    }
    
    if keyword.len() > 100 {
        return Err(anyhow!("Search keyword is too long (max 100 characters)"));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_note_config_valid() {
        let config = NoteConfig::new("Test Note", "This is a test note");
        assert!(validate_note_config(&config).is_ok());
    }

    #[test]
    fn test_validate_note_config_empty_title() {
        let config = NoteConfig::new("", "This is a test note");
        assert!(validate_note_config(&config).is_err());
    }

    #[test]
    fn test_validate_note_config_empty_content() {
        let config = NoteConfig::new("Test Note", "");
        assert!(validate_note_config(&config).is_ok());
    }

    #[test]
    fn test_validate_note_title() {
        assert!(validate_note_title("Valid Title").is_ok());
        assert!(validate_note_title("").is_err());
    }

    #[test]
    fn test_validate_folder_name() {
        assert!(validate_folder_name("Valid Folder").is_ok());
        assert!(validate_folder_name("").is_err());
    }
}