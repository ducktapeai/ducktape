//! Natural language parsing module for DuckTape
//!
//! This module provides common functionality for natural language parsers.

use crate::parser::traits::Parser;
use anyhow::Result;
use async_trait::async_trait;

// Expose our time parser modules
pub mod time_parser_fix;
pub mod time_parser_integration;

/// Common functionality for all natural language parsers
#[async_trait]
pub trait NaturalLanguageParser: Parser {
    /// Parse natural language into a command string
    async fn parse_natural_language(&self, input: &str) -> Result<String>;

    /// Sanitize NLP-generated commands
    fn sanitize_command(&self, command: &str) -> String;
}

/// Helper functions shared across NL parsers
pub mod utils {
    use anyhow::Result;
    use log::debug;

    /// Sanitize user input to prevent injection
    pub fn sanitize_user_input(input: &str) -> String {
        // Filter out control characters except for newlines and tabs
        input
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
            .collect::<String>()
    }

    /// Extract contact names from natural language input
    pub fn extract_contact_names(input: &str) -> Vec<String> {
        let mut contact_names = Vec::new();
        let input_lower = input.to_lowercase();

        // Check for different contact-related keywords
        let text_to_parse = if input_lower.contains(" with ") {
            debug!("Found 'with' keyword for contact extraction");
            input.split(" with ").nth(1)
        } else if input_lower.contains(" to ") {
            debug!("Found 'to' keyword for contact extraction");
            input.split(" to ").nth(1)
        } else if input_lower.contains("invite ") {
            debug!("Found 'invite' keyword for contact extraction");
            // Special handling for invite keyword which might not have a space before it
            let parts: Vec<&str> = input.splitn(2, "invite ").collect();
            if parts.len() > 1 { Some(parts[1]) } else { None }
        } else if input_lower.contains(" and invite ") {
            debug!("Found 'and invite' keyword for contact extraction");
            input.split(" and invite ").nth(1)
        } else {
            None
        };

        if let Some(after_word) = text_to_parse {
            debug!("Text to parse for contacts: '{}'", after_word);

            // Pattern to detect email addresses (simple version)
            let email_pattern =
                regex::Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();

            // Split by known separators that indicate multiple people
            for name_part in after_word.split([',', ';', '.']) {
                let name_part = name_part.trim();

                // Skip empty parts
                if name_part.is_empty() {
                    continue;
                }

                // Skip if the whole part looks like an email address
                if email_pattern.is_match(name_part) {
                    debug!("Skipping email-like string: {}", name_part);
                    continue;
                }

                // Further process parts with "and" to extract multiple names
                if name_part.contains(" and ") {
                    let and_parts: Vec<&str> = name_part.split(" and ").collect();
                    for and_part in and_parts {
                        let final_name = refine_name(and_part);
                        if !final_name.is_empty() && !email_pattern.is_match(&final_name) {
                            contact_names.push(final_name);
                        }
                    }
                } else {
                    // Process single name
                    let final_name = refine_name(name_part);
                    if !final_name.is_empty() && !email_pattern.is_match(&final_name) {
                        contact_names.push(final_name);
                    }
                }
            }
        }

        debug!("Extracted contact names: {:?}", contact_names);
        contact_names
    }

    /// Refine a name by removing trailing stop words
    pub fn refine_name(name_part: &str) -> String {
        let stop_words = ["at", "on", "tomorrow", "today", "for", "about", "regarding"];

        let mut final_name = name_part.trim().to_string();
        for word in &stop_words {
            if let Some(pos) = final_name.to_lowercase().find(&format!(" {}", word)) {
                final_name = final_name[0..pos].trim().to_string();
            }
        }

        final_name
    }

    /// Validate calendar command for security
    pub fn validate_calendar_command(command: &str) -> Result<()> {
        use anyhow::anyhow;

        // Security checks
        if command.contains("&&")
            || command.contains("|")
            || command.contains(";")
            || command.contains("`")
        {
            return Err(anyhow!("Generated command contains potentially unsafe characters"));
        }

        // Only check calendar commands
        if command.contains("calendar create") {
            // Check for reasonably sized intervals for recurring events
            if command.contains("--interval") {
                let re = regex::Regex::new(r"--interval (\d+)").unwrap();
                if let Some(caps) = re.captures(command) {
                    if let Some(interval_match) = caps.get(1) {
                        if let Ok(interval) = interval_match.as_str().parse::<i32>() {
                            if interval > 100 {
                                return Err(anyhow!("Unreasonable interval value: {}", interval));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// Re-export submodules
pub mod command_mapping;
pub mod grok;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_contact_names() {
        // Test with single contact using "with" pattern
        let input = "Schedule a meeting with John Smith tomorrow at 2pm";
        let contacts = utils::extract_contact_names(input);
        assert_eq!(contacts, vec!["John Smith"]);

        // Test with single contact using "invite" pattern
        let input = "Schedule a meeting tomorrow at 2pm and invite Jane Doe";
        let contacts = utils::extract_contact_names(input);
        assert_eq!(contacts, vec!["Jane Doe"]);

        // Test with multiple contacts using commas
        let input = "Meeting with Alice Johnson, Bob Brown tomorrow";
        let contacts = utils::extract_contact_names(input);
        assert_eq!(contacts, vec!["Alice Johnson", "Bob Brown"]);

        // Test with multiple contacts using "and"
        let input = "Meeting with Alice Johnson and Bob Brown tomorrow";
        let contacts = utils::extract_contact_names(input);
        assert_eq!(contacts, vec!["Alice Johnson", "Bob Brown"]);

        // Test with "and invite" pattern for multiple contacts
        let input =
            "create an event called Team Meeting tonight and invite Shaun Stuart and Joe Buck";
        let contacts = utils::extract_contact_names(input);
        assert_eq!(contacts, vec!["Shaun Stuart", "Joe Buck"]);

        // Test with mixed separators (both comma and "and")
        let input = "Schedule a meeting with Alice Johnson, Bob Brown and Jane Doe tomorrow";
        let contacts = utils::extract_contact_names(input);
        assert_eq!(contacts, vec!["Alice Johnson", "Bob Brown", "Jane Doe"]);
    }
}
