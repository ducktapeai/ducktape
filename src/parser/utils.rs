//! Common utility functions shared across all parser implementations
//!
//! This module provides general utility functions that are used by multiple
//! parser implementations.


/// Preprocess input string before further parsing
///
/// Standardizes whitespace, removes excess spaces, etc.
pub fn preprocess_input(input: &str) -> String {
    input.trim().to_string()
}

/// Validate if a string looks like a valid email address
///
/// This is a simple validation check to determine if a string is likely an email
pub fn is_email(text: &str) -> bool {
    text.contains('@') && text.contains('.')
}

/// Normalize whitespace in command strings
///
/// Ensures consistent spacing in command strings
pub fn normalize_spacing(command: &str) -> String {
    // Replace multiple spaces with a single space
    let mut result = String::new();
    let mut last_was_space = false;

    for c in command.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Helper function to clean up NLP-generated commands
/// Removes unnecessary quotes and normalizes spacing
pub fn sanitize_nlp_command(command: &str) -> String {
    // Clean up the command
    let cleaned = command
        .replace("\u{a0}", " ") // Replace non-breaking spaces
        .replace("\"\"", "\""); // Replace double quotes

    // Ensure the command starts with ducktape
    if !cleaned.starts_with("ducktape") {
        return format!("ducktape {}", cleaned);
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_input() {
        assert_eq!(preprocess_input("  hello  "), "hello");
        assert_eq!(preprocess_input("\n test \t"), "test");
    }

    #[test]
    fn test_is_email() {
        assert!(is_email("user@example.com"));
        assert!(!is_email("not an email"));
        assert!(!is_email("missing@domain"));
    }

    #[test]
    fn test_normalize_spacing() {
        assert_eq!(normalize_spacing("too    many   spaces"), "too many spaces");
        assert_eq!(normalize_spacing(" leading trailing "), "leading trailing");
    }

    #[test]
    fn test_sanitize_nlp_command() {
        // Test handling of non-breaking spaces
        let input = "ducktape\u{a0}calendar create \"Meeting\"";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape calendar create \"Meeting\"");

        // Test handling of double quotes
        let input = "ducktape calendar create \"\"Meeting\"\"";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape calendar create \"Meeting\"");

        // Test non-ducktape command with prefix added
        let input = "create a meeting tomorrow at 3pm";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "ducktape create a meeting tomorrow at 3pm");
    }
}
