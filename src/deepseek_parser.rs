use anyhow::{Result, anyhow};
use log::debug;
use reqwest::Client;
use serde_json::json;
use std::env;

// Use the parser trait from the root crate
use crate::parser_trait::{ParseResult, Parser};
use async_trait::async_trait;

/// Parser that uses DeepSeek's models for natural language understanding
pub struct DeepSeekParser;

impl DeepSeekParser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Parser for DeepSeekParser {
    async fn parse_input(&self, input: &str) -> anyhow::Result<ParseResult> {
        match parse_natural_language(input).await {
            Ok(command) => {
                debug!("DeepSeek parser generated command: {}", command);

                // Before parsing, sanitize quotes in the command
                let sanitized_command = sanitize_nlp_command(&command);
                debug!("Sanitized command: {}", sanitized_command);

                // For now, return as a command string
                // In the future, we could parse this into structured CommandArgs here
                Ok(ParseResult::CommandString(sanitized_command))
            }
            Err(e) => {
                debug!("DeepSeek parser error: {}", e);
                Err(e)
            }
        }
    }

    fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
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
pub async fn parse_natural_language(input: &str) -> Result<String> {
    // Replace with your actual DeepSeek API endpoint and parameters.
    let deepseek_endpoint = "https://api.deepseek.example/v1/parse";
    let api_key = env::var("DEEPSEEK_API_KEY").map_err(|_| anyhow!("DEEPSEEK_API_KEY not set"))?;
    let client = Client::new();

    let body = json!({
        "input": input,
        "model": "default", // adjust if necessary
    });

    let response = client
        .post(deepseek_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(command) = response["command"].as_str() {
        Ok(command.to_string())
    } else {
        Err(anyhow!("DeepSeek failed to parse command"))
    }
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
