use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn get_superbowl_info() -> Result<String> {
    // Adjust endpoint and parameters as needed.
    let endpoint = "https://api.deepseek.example/v1/reason";
    let api_key = env::var("DEEPSEEK_API_KEY").map_err(|_| anyhow!("DEEPSEEK_API_KEY not set"))?;
    let client = Client::new();
    let body = json!({
        "query": "When is the next Super Bowl?",
    });

    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(date) = response["date"].as_str() {
        Ok(date.to_string())
    } else {
        Err(anyhow!("Could not retrieve Super Bowl date"))
    }
}
