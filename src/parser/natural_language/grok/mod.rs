//! Grok parser module for DuckTape
//!
//! This module provides natural language processing capabilities
//! using the Grok/X.AI API for parsing user input into structured commands.

use crate::parser::natural_language::NaturalLanguageParser;
use crate::parser::traits::{ParseResult, Parser};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::{debug, error, warn};
use std::env;

pub mod api;
pub mod cache;
pub mod utils;

/// Parser that uses Grok/X.AI models for natural language understanding
pub struct GrokParser;

impl GrokParser {
    /// Create a new GrokParser instance
    pub fn create() -> Result<Self> {
        // Check for XAI_API_KEY upfront to avoid misleading errors
        check_xai_api_key()?;
        Ok(Self)
    }

    /// Check for the required XAI_API_KEY environment variable
    fn check_env_vars() -> Result<()> {
        check_xai_api_key()
    }
}

/// Helper function to check for XAI_API_KEY environment variable
fn check_xai_api_key() -> Result<()> {
    match env::var("XAI_API_KEY") {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!(
            "XAI_API_KEY environment variable not set. Please set your X.AI API key using: export XAI_API_KEY='your-key-here'"
        )),
    }
}

#[async_trait]
impl Parser for GrokParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Check environment variables first to catch missing XAI_API_KEY early
        check_xai_api_key()?;

        match self.parse_natural_language(input).await {
            Ok(command) => {
                debug!("Grok parser generated command: {}", command);
                let sanitized = self.sanitize_command(&command);

                // If the command starts with "ducktape calendar create" or "ducktape todo create",
                // we can directly return it as a command string
                if sanitized.starts_with("ducktape calendar create")
                    || sanitized.starts_with("ducktape todo create")
                {
                    Ok(ParseResult::CommandString(sanitized))
                } else {
                    // Try to convert to a structured command if possible
                    match crate::command_processor::CommandArgs::parse(&sanitized) {
                        Ok(args) => {
                            debug!("Successfully converted to structured command: {:?}", args);
                            Ok(ParseResult::StructuredCommand(args))
                        }
                        Err(e) => {
                            debug!(
                                "Could not convert to structured command: {}, returning command string",
                                e
                            );
                            Ok(ParseResult::CommandString(sanitized))
                        }
                    }
                }
            }
            Err(e) => {
                error!("Grok parser error: {}", e);
                Err(e)
            }
        }
    }

    fn new() -> Result<Self> {
        // Check for XAI_API_KEY upfront to avoid misleading errors
        check_xai_api_key()?;
        Ok(Self)
    }
}

#[async_trait]
impl NaturalLanguageParser for GrokParser {
    async fn parse_natural_language(&self, input: &str) -> Result<String> {
        api::parse_natural_language(input).await
    }

    fn sanitize_command(&self, command: &str) -> String {
        utils::sanitize_nlp_command(command)
    }
}

/// Factory function to create a Grok parser
pub fn create_grok_parser() -> Result<Box<dyn Parser + Send + Sync>> {
    let parser = GrokParser::new()?;
    Ok(Box::new(parser))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_grok_parser() -> Result<()> {
        let parser = GrokParser::new()?;
        let result = parser.parse_input("Schedule a team meeting tomorrow at 2pm").await;

        // We expect the parse to succeed even with mocked responses in test mode
        assert!(result.is_ok());

        if let Ok(ParseResult::CommandString(cmd)) = result {
            assert!(cmd.starts_with("ducktape"));
        } else {
            panic!("Expected CommandString parse result");
        }

        Ok(())
    }
}
