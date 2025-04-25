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
pub mod time_extractor;
pub mod utils;

/// Parser that uses Grok/X.AI models for natural language understanding
pub struct GrokParser;

impl GrokParser {
    pub fn new() -> Result<Self> {
        // Check for XAI_API_KEY upfront to avoid misleading errors
        check_xai_api_key()?;
        Ok(Self)
    }

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

        // Special pattern detection for direct event creation commands
        let input_lower = input.to_lowercase();
        let is_event_creation = input_lower.contains("create an event")
            || input_lower.contains("schedule a meeting")
            || input_lower.contains("create a meeting")
            || input_lower.contains("add an event")
            || input_lower.contains("create event")
            || (input_lower.contains("schedule")
                && (input_lower.contains("meeting") || input_lower.contains("event")));

        if is_event_creation {
            debug!("Detected calendar event creation intent: {}", input);
            // For these commands, we want to ensure they're treated as calendar events
            match self.parse_natural_language(input).await {
                Ok(command) => {
                    debug!("Grok parser generated calendar command: {}", command);
                    let sanitized = self.sanitize_command(&command);
                    Ok(ParseResult::CommandString(sanitized))
                }
                Err(e) => {
                    warn!("Failed to parse event creation command with API, using fallback method");
                    // Use fallback mechanism with simple format for basic functionality
                    let sanitized = utils::sanitize_nlp_command(input);
                    // Always enhance with contacts and zoom, even in fallback
                    let enhanced = utils::enhance_command_with_contacts(&sanitized, input);
                    let enhanced = utils::enhance_command_with_zoom(&enhanced, input);
                    Ok(ParseResult::CommandString(enhanced))
                }
            }
        } else {
            // Normal flow for non-event creation commands
            match self.parse_natural_language(input).await {
                Ok(command) => {
                    debug!("Grok parser generated command: {}", command);
                    let sanitized = self.sanitize_command(&command);
                    Ok(ParseResult::CommandString(sanitized))
                }
                Err(e) => {
                    error!("Grok parser error: {}", e);
                    Err(e)
                }
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
