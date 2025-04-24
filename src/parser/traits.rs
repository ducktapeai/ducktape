//! Parser traits module for DuckTape
//!
//! This module defines the core traits and types for the parser system,
//! providing a unified interface for different parser implementations.

use crate::command_processor::CommandArgs;
use crate::config::{Config, LLMProvider};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::info;

/// Represents the result of parsing an input
#[derive(Debug)]
pub enum ParseResult {
    /// A simple command string that needs further processing
    CommandString(String),
    /// A fully structured command ready for execution
    StructuredCommand(CommandArgs),
}

/// Core parser trait that all parser implementations must implement
#[async_trait]
pub trait Parser: Send + Sync {
    /// Parse a string input into a ParseResult
    ///
    /// # Arguments
    ///
    /// * `input` - The string to parse
    ///
    /// # Returns
    ///
    /// A Result containing either a ParseResult or an error
    async fn parse_input(&self, input: &str) -> Result<ParseResult>;

    /// Create a new instance of the parser
    fn new() -> Result<Self>
    where
        Self: Sized;
}

/// Factory for creating the appropriate parser based on configuration
pub struct ParserFactory;

impl ParserFactory {
    /// Create a parser based on the current configuration
    ///
    /// This will return an appropriate parser implementation based on the
    /// LLMProvider specified in the config.
    pub fn create_parser() -> Result<Box<dyn Parser + Send + Sync>> {
        let config = Config::load()?;

        match config.language_model.provider {
            Some(LLMProvider::Grok) => {
                info!("Creating Grok parser");
                let parser = crate::parser::grok::GrokParser::new()?;
                Ok(Box::new(parser))
            }
            Some(LLMProvider::DeepSeek) => {
                info!("Creating DeepSeek parser");
                let parser = crate::parser::deepseek::DeepSeekParser::new()?;
                Ok(Box::new(parser))
            }
            None => {
                info!("Creating Terminal parser (no language model selected)");
                crate::parser::terminal::create_terminal_parser()
            }
        }
    }

    /// Create a specific parser by name
    ///
    /// This is useful for testing or when you need to specify
    /// a parser that's different from what's in the config.
    pub fn create_parser_by_name(name: &str) -> Result<Box<dyn Parser + Send + Sync>> {
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
            let result = ParserFactory::create_parser_by_name(parser_type);
            assert!(result.is_ok(), "Failed to create parser: {}", parser_type);
        }
    }
}
