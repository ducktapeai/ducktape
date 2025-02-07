use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::env;

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
