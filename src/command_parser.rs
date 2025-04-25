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
    // Use a Result to Option conversion pattern to handle the Result correctly
    match crate::parser::command::parse_command_natural(message) {
        Ok(parse_result) => {
            match parse_result {
                crate::parser::ParseResult::CommandString(cmd) => {
                    // Convert to old format
                    Some(ParsedCommand {
                        command_type: "command".to_string(),
                        details: serde_json::json!({ "command": cmd }),
                    })
                }
                crate::parser::ParseResult::StructuredCommand(args) => {
                    // Convert structured command to old format
                    Some(ParsedCommand {
                        command_type: args.command,
                        details: serde_json::json!({ "args": args.args }),
                    })
                }
            }
        }
        Err(_) => None,
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
