//! Grok parser module for DuckTape
//!
//! This module provides natural language processing capabilities
//! using the Grok/X.AI API for parsing user input into structured commands.

use crate::parser::traits::{ParseResult, Parser};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::debug;

/// Parser that uses Grok/X.AI models for natural language understanding
pub struct GrokParser;

impl GrokParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Parser for GrokParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Temporarily use OpenAI parser as a fallback until Grok implementation is completed
        debug!("Grok parser: Using OpenAI parser as fallback for input: {}", input);
        
        // Forward to OpenAI parser
        let openai_parser = crate::parser::openai::OpenAIParser::new()?;
        openai_parser.parse_input(input).await
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}