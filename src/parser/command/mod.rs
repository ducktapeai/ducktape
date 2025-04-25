//! Command parser module for DuckTape
//!
//! This module handles parsing of structured commands.

use crate::command_processor::CommandArgs;
use crate::parser::traits::ParseResult;
use anyhow::{Result, anyhow};
use async_trait::async_trait;

/// Parser for structured CLI commands
pub struct CommandParser;

impl CommandParser {
    /// Parse a command string into structured command arguments
    pub fn parse(&self, input: &str) -> Result<CommandArgs> {
        parse_command_with_clap(input)
    }
}

#[async_trait]
impl crate::parser::Parser for CommandParser {
    /// Create a new CommandParser
    fn new() -> Result<Self> {
        Ok(CommandParser)
    }

    /// Parse a command input string
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Split the input into tokens while preserving quoted strings
        let tokens =
            shell_words::split(input).map_err(|e| anyhow!("Failed to parse command: {}", e))?;

        // Parse with clap
        parse_with_clap(tokens)
    }
}

/// Parse a command string with clap
pub fn parse_command_with_clap(input: &str) -> Result<CommandArgs> {
    // Split the input into tokens while preserving quoted strings
    let tokens =
        shell_words::split(input).map_err(|e| anyhow!("Failed to parse command: {}", e))?;

    // Parse with clap
    match parse_with_clap(tokens)? {
        ParseResult::StructuredCommand(cmd) => Ok(cmd),
        ParseResult::CommandString(_) => Err(anyhow!("Unexpected command string result")),
    }
}

/// Parse tokens with clap
pub fn parse_with_clap(tokens: Vec<String>) -> Result<ParseResult> {
    use crate::cli;
    use clap::Parser;

    // Check if we have any arguments
    if tokens.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    // Add "ducktape" as the first token if not present
    let tokens_with_binary = if tokens[0] != "ducktape" {
        let mut new_tokens = vec!["ducktape".to_string()];
        new_tokens.extend(tokens);
        new_tokens
    } else {
        tokens
    };

    // Parse using Clap
    match cli::Cli::try_parse_from(&tokens_with_binary) {
        Ok(cli) => {
            // Convert from Clap command to CommandArgs
            let cmd_args = cli::convert_to_command_args(&cli)
                .ok_or_else(|| anyhow!("Failed to convert parsed command to CommandArgs"))?;

            Ok(ParseResult::StructuredCommand(cmd_args))
        }
        Err(e) => {
            // This is likely not a structured command but a natural language input
            Err(anyhow!("Not a structured command: {}", e))
        }
    }
}

/// Parse a natural language command string
pub fn parse_command_natural(input: &str) -> Result<ParseResult> {
    // This is a placeholder until natural language processing is implemented
    // For now, just return the original string
    Ok(ParseResult::CommandString(input.to_string()))
}
