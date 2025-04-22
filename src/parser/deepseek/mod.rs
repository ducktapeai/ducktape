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
        // Temporarily use OpenAI parser as a fallback until DeepSeek implementation is completed
        debug!("DeepSeek parser: Using OpenAI parser as fallback for input: {}", input);
        
        // Forward to OpenAI parser
        let openai_parser = crate::parser::openai::OpenAIParser::new()?;
        openai_parser.parse_input(input).await
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}