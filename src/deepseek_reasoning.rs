use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::json;
use std::env;

#[allow(dead_code)]
pub async fn get_superbowl_info() -> Result<String> {
    // Adjust endpoint and parameters as needed.
    let endpoint = "https://api.deepseek.example/v1/reason";

    // More secure API key handling that doesn't expose the key in error messages
    let api_key = env::var("DEEPSEEK_API_KEY")
        .map_err(|_| anyhow!("DEEPSEEK_API_KEY environment variable not set. Please set this variable in your environment"))?;

    // Add additional checks for API key security
    if api_key.trim().is_empty() {
        return Err(anyhow!("DEEPSEEK_API_KEY is empty. Please set a valid API key"));
    }

    // Create HTTP client with timeout for security
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

    let body = json!({
        "query": "When is the next Super Bowl?",
    });

    // Use proper error handling for network requests
    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("API request failed: {}", e))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| anyhow!("Failed to parse API response: {}", e))?;

    // Safely extract date from response with proper validation
    if let Some(date) = response["date"].as_str() {
        // Validate the date format (basic validation example)
        if date.len() < 100 && !date.contains(';') && !date.contains('|') {
            Ok(date.to_string())
        } else {
            Err(anyhow!("Invalid date format received from API"))
        }
    } else {
        Err(anyhow!("Could not retrieve Super Bowl date from API response"))
    }
}
