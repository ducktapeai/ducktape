//! Command parser compatibility module (Deprecated)
//!
//! This module is kept for backward compatibility and redirects to the new modular structure.
//! Use the `crate::parser::command` module instead.

// Re-export the necessary types and functions for backward compatibility
#[deprecated(since = "0.13.0", note = "Use crate::parser::command module instead")]
pub use crate::parser::command::parse_with_clap;

// Re-export legacy types for backward compatibility
pub use regex::Regex;
pub use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ParsedCommand {
    pub command_type: String,
    pub details: serde_json::Value,
}

#[derive(Debug)]
pub struct UserMessage {
    #[allow(dead_code)]
    pub content: String,
    #[allow(dead_code)]
    pub timestamp: String,
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub sender: String,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub content: String,
    pub success: bool,
    pub command_id: String,
}

/// Use the new parser::command module internally
///
/// Converts between the new and old ParsedCommand types
#[allow(deprecated)]
pub fn parse_command(message: &str) -> Option<ParsedCommand> {
    // Delegate to the new parse_command_natural
    if let Some(new_cmd) = crate::parser::command::parse_command_natural(message) {
        // Convert between the two types
        Some(ParsedCommand { command_type: new_cmd.command_type, details: new_cmd.details })
    } else {
        None
    }
}

// Process command using new module internally
#[allow(deprecated)]
pub fn process_command(message: UserMessage) -> CommandResponse {
    let parsed = parse_command(&message.content);

    match parsed {
        Some(cmd) => {
            let response = format!(
                "Processing command: {}. Details: {}",
                cmd.command_type,
                cmd.details.to_string()
            );

            CommandResponse { content: response, success: true, command_id: message.id }
        }
        None => CommandResponse {
            content: "Sorry, I didn't understand that command.".to_string(),
            success: false,
            command_id: message.id,
        },
    }
}
