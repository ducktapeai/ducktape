//! Terminal parser module for DuckTape
//!
//! This module provides terminal command parsing functionality
//! for handling direct terminal input without using an LLM.

use crate::parser::command;
use crate::parser::traits::{ParseResult, Parser};
use anyhow::Result;
use async_trait::async_trait;
use log::debug;

/// Terminal Parser struct for handling direct terminal commands
pub struct TerminalParser;

#[async_trait]
impl Parser for TerminalParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        debug!("Terminal parser processing direct input: {}", input);
        
        // Delegate to the command parser for structured command parsing
        let command_parser = command::CommandParser::new()?;
        command_parser.parse_input(input).await
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Factory function to create a terminal parser
pub fn create_terminal_parser() -> Result<Box<dyn Parser + Send + Sync>> {
    Ok(Box::new(TerminalParser))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_terminal_parser() -> Result<()> {
        let parser = TerminalParser;
        let result = parser.parse_input("calendar create \"Test Meeting\" 2025-04-25 14:00 15:00 \"Work\"").await?;
        
        match result {
            ParseResult::CommandString(_) | ParseResult::StructuredCommand(_) => {
                // Either result type is acceptable for terminal input
                Ok(())
            }
        }
    }
}