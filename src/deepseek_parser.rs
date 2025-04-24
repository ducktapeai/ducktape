/// DeepSeek parser compatibility module (Deprecated)
///
/// This module is kept for backward compatibility and redirects to the new modular structure.
/// Use the `crate::parser::deepseek` module instead.
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::debug;
use reqwest::Client;
use serde_json::json;
use std::env; // Added missing import

use crate::parser_trait::{ParseResult, Parser};

// Re-export the new implementation for backward compatibility
#[allow(deprecated)]
pub use crate::parser::deepseek::DeepSeekParser as NewDeepSeekParser;

/// DeepSeek model parser implementation (deprecated)
///
/// Deprecated: Use `crate::parser::deepseek::DeepSeekParser` instead
#[deprecated(since = "0.13.0", note = "Use crate::parser::deepseek::DeepSeekParser instead")]
pub struct DeepSeekParser;

#[async_trait]
#[allow(deprecated)]
impl Parser for DeepSeekParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Delegate to the new implementation
        let parser = crate::parser::deepseek::DeepSeekParser::new()?;
        parser.parse_input(input).await
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Re-export of the parse_natural_language function for backward compatibility
///
/// Deprecated: Use `crate::parser::deepseek::parse_natural_language` instead
#[deprecated(
    since = "0.13.0",
    note = "Use crate::parser::deepseek::parse_natural_language instead"
)]
pub async fn parse_natural_language(input: &str) -> Result<String> {
    // Delegate to OpenAI implementation as DeepSeek currently uses OpenAI as fallback
    crate::parser::openai::parse_natural_language(input).await
}

/// Helper function to clean up NLP-generated commands
/// Removes unnecessary quotes and normalizes spacing
fn sanitize_nlp_command(command: &str) -> String {
    // Ensure the command starts with ducktape
    if !command.starts_with("ducktape") {
        return command.to_string();
    }

    // Basic sanitization to fix common issues with NLP-generated commands
    command
        .replace("\u{a0}", " ") // Replace non-breaking spaces
        .replace("\"\"", "\"") // Replace double quotes
        .to_string()
}

#[allow(dead_code)]
pub async fn get_superbowl_info() -> Result<String> {
    // Replace with your actual DeepSeek API endpoint and parameters.
    let deepseek_endpoint = "https://api.deepseek.example/v1/superbowl";
    let api_key = env::var("DEEPSEEK_API_KEY").map_err(|_| anyhow!("DEEPSEEK_API_KEY not set"))?;
    let client = Client::new();

    let response = client
        .get(deepseek_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(info) = response["info"].as_str() {
        Ok(info.to_string())
    } else {
        Err(anyhow!("DeepSeek failed to get Superbowl info"))
    }
}
