/// OpenAI Parser - DEPRECATED
///
/// This module is deprecated. OpenAI parser functionality has been removed.
/// Use crate::parser::grok module instead.
///
/// This module is kept for backward compatibility to prevent breaking changes, but will be removed in a future version.

use crate::command_processor::CommandArgs;
use crate::parser_trait::ParseResult;
use crate::parser::traits::Parser; // Add Parser trait to scope
use anyhow::{Result, anyhow};
use async_trait::async_trait;

// Define a stub OpenAIParser that redirects to Grok
#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed. Use GrokParser instead.")]
pub struct OpenAIParser;

#[async_trait]
impl crate::parser_trait::Parser for OpenAIParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Create and use a GrokParser instead
        let parser = crate::parser::grok::GrokParser::new()?;
        parser.parse_input(input).await
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}

// Redirect parse_natural_language to use Grok
#[deprecated(
    since = "0.12.0",
    note = "OpenAI parser has been removed. Use parser::grok module instead."
)]
pub async fn parse_natural_language(input: &str) -> Result<String> {
    let parser = crate::parser::grok::GrokParser::new()?;
    match parser.parse_input(input).await? {
        ParseResult::CommandString(cmd) => Ok(cmd),
        ParseResult::StructuredCommand(_) => Err(anyhow!("Expected command string but got structured command")),
    }
}

// Stub utility functions that were in OpenAI parser
#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed. Use parser::utils module instead.")]
pub fn sanitize_nlp_command(command: &str) -> String {
    crate::parser::utils::sanitize_nlp_command(command)
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn sanitize_user_input(input: &str) -> String {
    input.trim().to_string()
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn validate_calendar_command(_cmd: &str) -> bool {
    true
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn extract_contact_names(_input: &str) -> Vec<String> {
    Vec::new()
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn extract_emails(_input: &str) -> Vec<String> {
    Vec::new()
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn enhance_command_with_contacts(command: &str, _contacts: &[String]) -> String {
    command.to_string()
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn enhance_command_with_zoom(command: &str) -> String {
    command.to_string()
}

#[deprecated(since = "0.12.0", note = "OpenAI parser has been removed.")]
pub fn enhance_command_with_recurrence(command: &str, _recurrence: &str) -> String {
    command.to_string()
}
