//! Grok parser module for DuckTape
//!
//! This module provides natural language processing capabilities
//! using the Grok/X.AI API for parsing user input into structured commands.

use crate::parser::traits::{ParseResult, Parser};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::{debug, error};
use std::env;

/// Parser that uses Grok/X.AI models for natural language understanding
pub struct GrokParser;

impl GrokParser {
    /// Create a new GrokParser instance
    pub fn new() -> Result<Self> {
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
        debug!("Grok parser: Processing input: {}", input);

        // Check that XAI_API_KEY is set
        let api_key = match env::var("XAI_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                error!("XAI_API_KEY environment variable not set");
                return Err(anyhow!("XAI_API_KEY environment variable not set"));
            }
        };

        // TODO: Implement full Grok/X.AI API integration
        // For now, provide a basic implementation that returns the input as a command string
        debug!("Using XAI_API_KEY with length: {}", api_key.len());

        // Basic sanitization of the input
        let command = sanitize_nlp_command(input);

        // Return the sanitized input as a command string
        Ok(ParseResult::CommandString(command))
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Basic sanitization of natural language input
fn sanitize_nlp_command(command: &str) -> String {
    // Remove potentially problematic characters and trim whitespace
    let sanitized = command
        .trim()
        .replace(';', " ")
        .replace('&', " and ")
        .replace('|', " ")
        .replace('>', " ")
        .replace('<', " ")
        .replace('`', " ");

    debug!("Sanitized command: {}", sanitized);
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_sanitize_command() {
        let input = "create meeting; rm -rf /";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "create meeting  rm -rf /");

        let input = "schedule meeting & delete files";
        let sanitized = sanitize_nlp_command(input);
        assert_eq!(sanitized, "schedule meeting  and  delete files");
    }

    #[test]
    fn test_check_api_key() {
        // Test with API key set
        env::set_var("XAI_API_KEY", "test_key");
        let result = check_xai_api_key();
        assert!(result.is_ok());

        // Test with API key unset
        env::remove_var("XAI_API_KEY");
        let result = check_xai_api_key();
        assert!(result.is_err());
    }
}
