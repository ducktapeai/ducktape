//! Command parser module for DuckTape
//!
//! This module provides command parsing functionality for structured DuckTape commands.
//! It serves as a bridge between the raw command string and the command processor,
//! handling both legacy and new parsing approaches.

use crate::command_processor::CommandArgs;
use crate::parser::traits::{ParseResult, Parser};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use clap::Parser as ClapParser;
use log::debug;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

// Import the Cli struct for parsing
use crate::cli::{self, Cli, Commands};

/// Modern command parser implementation
pub struct CommandParser;

/// Structured command representation for natural language parsing
#[derive(Debug, Serialize)]
pub struct ParsedCommand {
    pub command_type: String,
    pub details: serde_json::Value,
}

/// Structured representation of a schedule command
#[derive(Debug, Serialize)]
struct ScheduleCommand {
    event_type: String,
    event_name: String,
    person: String,
    day: String,
    time: String,
}

impl CommandParser {
    /// Create a new command parser instance
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Parse command line arguments using the Clap parser
/// 
/// This function takes a slice of strings and parses them using the Clap parser.
/// It returns a ParseResult with the parsed command and arguments.
pub fn parse_with_clap<I, T>(args: I) -> Result<ParseResult>
where
    I: IntoIterator<Item = T>,
    T: Into<String> + Clone,
{
    let os_args: Vec<String> = std::iter::once("ducktape".to_string())
        .chain(args.into_iter().map(|s| s.into()))
        .collect();

    // Use try_parse_from instead of parse_from to get Result<Cli, Error>
    let cli_result = Cli::try_parse_from(os_args);
    
    match cli_result {
        Ok(cli) => {
            let cmd_args = cli::convert_to_command_args(&cli);
            match cmd_args {
                Some(args) => Ok(ParseResult::StructuredCommand(args)),
                None => Ok(ParseResult::CommandString("help".to_string()))
            }
        }
        Err(e) => {
            Err(anyhow!("Failed to parse command: {}", e))
        }
    }
}

#[async_trait]
impl Parser for CommandParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Split input into words while respecting quotes
        let args = match shell_words::split(input) {
            Ok(words) => words,
            Err(e) => return Err(anyhow!("Failed to parse input: {}", e)),
        };
        
        parse_with_clap(args)
    }
    
    fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Legacy command parsing - deprecated in favor of Clap-based parsing
#[deprecated(since = "0.13.0", note = "Use the Clap-based command line parser instead")]
pub fn parse_command(cmd: &str) -> Result<CommandArgs> {
    match shell_words::split(cmd) {
        Ok(args) => {
            match parse_with_clap(args) {
                Ok(ParseResult::StructuredCommand(cmd_args)) => Ok(cmd_args),
                Ok(ParseResult::CommandString(_)) => Err(anyhow!("Unexpected parse result type")),
                Err(e) => Err(e),
            }
        },
        Err(e) => Err(anyhow!("Failed to parse command: {}", e)),
    }
}

/// Parse a command message into a structured command object using natural language patterns
pub fn parse_command_natural(message: &str) -> Option<ParsedCommand> {
    // Match schedule command: "schedule a <type> <what> with <who> <when> at <time>"
    if let Some(schedule) = parse_schedule(message) {
        return Some(ParsedCommand {
            command_type: "schedule".to_string(),
            details: serde_json::to_value(schedule).unwrap(),
        });
    }

    None
}

/// Parse a schedule command using regex pattern matching
fn parse_schedule(message: &str) -> Option<ScheduleCommand> {
    // Use regex to parse the schedule command
    let re =
        Regex::new(r"schedule a (\w+) (\w+) with (\w+) (\w+) at (\d+(?::\d+)?(?:am|pm)?)").ok()?;

    if let Some(caps) = re.captures(message) {
        return Some(ScheduleCommand {
            event_type: caps.get(1)?.as_str().to_string(),
            event_name: caps.get(2)?.as_str().to_string(),
            person: caps.get(3)?.as_str().to_string(),
            day: caps.get(4)?.as_str().to_string(),
            time: caps.get(5)?.as_str().to_string(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_calendar_command() {
        let input = vec![
            "calendar".to_string(),
            "create".to_string(),
            "Meeting".to_string(),
            "2023-10-15".to_string(),
            "14:00".to_string(),
            "15:00".to_string(),
            "Work".to_string(),
        ];
        
        let result = parse_with_clap(input);
        assert!(result.is_ok());
        
        if let Ok(ParseResult::StructuredCommand(cmd_args)) = result {
            assert_eq!(cmd_args.command, "calendar");
            assert!(cmd_args.args.contains(&"create".to_string()));
        } else {
            panic!("Expected StructuredCommand parse result");
        }
    }
    
    #[test]
    fn test_parse_invalid_command() {
        let input = vec![
            "invalid_command".to_string(),
        ];
        
        let result = parse_with_clap(input);
        assert!(result.is_err());
    }
}