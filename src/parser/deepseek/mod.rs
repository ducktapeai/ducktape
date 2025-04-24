//! DeepSeek parser module for DuckTape
//!
//! This module provides natural language processing capabilities
//! using the DeepSeek API for parsing user input into structured commands.

use crate::parser::traits::{ParseResult, Parser};
use anyhow::Result;
use async_trait::async_trait;
use log::debug;

/// Parser that uses DeepSeek models for natural language understanding
pub struct DeepSeekParser;

impl DeepSeekParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Parser for DeepSeekParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Note: Previously this used OpenAI parser as fallback, but we've removed that dependency
        debug!("DeepSeek parser: Processing input: {}", input);

        // Basic implementation that prefixes the input with "ducktape"
        // This should be replaced with an actual implementation using the DeepSeek API
        let command = if input.trim().starts_with("ducktape") {
            input.trim().to_string()
        } else {
            format!("ducktape {}", input.trim())
        };

        // Use our new utility function to sanitize the command
        let sanitized = crate::parser::utils::sanitize_nlp_command(&command);
        debug!("DeepSeek parser: Generated command: {}", sanitized);

        Ok(ParseResult::CommandString(sanitized))
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}
