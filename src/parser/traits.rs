//! Parser traits module for DuckTape
//!
//! This module defines the core traits and types for the parser system,
//! providing a unified interface for different parser implementations.

use anyhow::{Result, anyhow};
use async_trait::async_trait;

/// Result type from parsing input
#[derive(Debug)]
pub enum ParseResult {
    /// Command string in Ducktape CLI format (e.g. "ducktape calendar create ...")
    CommandString(String),
    /// Structured command arguments
    StructuredCommand(crate::command_processor::CommandArgs),
}

/// Parser trait for all parser implementations
#[async_trait]
pub trait Parser: Send + Sync {
    /// Parse input to either a command string or structured command
    async fn parse_input(&self, input: &str) -> Result<ParseResult>;

    /// Create a new instance of this parser
    fn new() -> Result<Self>
    where
        Self: Sized;
}

/// Parser factory for creating parsers by name
pub fn create_parser(name: &str) -> Result<Box<dyn Parser + Send + Sync>> {
    match name.to_lowercase().as_str() {
        "grok" => {
            let parser = crate::parser::grok::GrokParser::new()?;
            Ok(Box::new(parser))
        }
        "deepseek" => {
            let parser = crate::parser::deepseek::DeepSeekParser::new()?;
            Ok(Box::new(parser))
        }
        "terminal" => crate::parser::terminal::create_terminal_parser(),
        "command" => {
            let parser = crate::parser::command::CommandParser::new()?;
            Ok(Box::new(parser))
        }
        _ => Err(anyhow!("Unknown parser type: {}", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_factory() {
        // This test just ensures that the parser factory can create various parser types
        // It doesn't actually test parsing functionality
        let parser_types = ["terminal", "command"];

        for parser_type in parser_types {
            let result = create_parser(parser_type);
            assert!(result.is_ok(), "Failed to create parser: {}", parser_type);
        }
    }
}
