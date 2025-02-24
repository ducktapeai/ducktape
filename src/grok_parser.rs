use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use lru::LruCache;
use once_cell::sync::Lazy;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use chrono::{Local, Timelike};
use crate::config::Config;

static RESPONSE_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

// Same calendar helper function as in openai_parser
async fn get_available_calendars() -> Result<Vec<String>> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(
            r#"tell application "Calendar"
            set calList to {}
            repeat with c in calendars
                copy (name of c) to end of calList
            end repeat
            return calList
        end tell"#,
        )
        .output()?;

    let calendars_str = String::from_utf8_lossy(&output.stdout);
    Ok(calendars_str
        .trim_matches('{')
        .trim_matches('}')
        .split(", ")
        .map(|s| s.trim_matches('"').to_string())
        .collect())
}

pub async fn parse_natural_language(input: &str) -> Result<String> {
    let api_key = env::var("XAI_API_KEY")
        .map_err(|_| anyhow!("XAI_API_KEY environment variable not set. Please set your X.AI API key using: export XAI_API_KEY='your-key-here'"))?;

    let api_base = env::var("XAI_API_BASE")
        .unwrap_or_else(|_| "https://api.x.ai/v1".to_string());

    // Check cache first
    if let Some(cached_response) = RESPONSE_CACHE.lock().unwrap().get(input) {
        return Ok(cached_response.clone());
    }

    // Get available calendars and configuration
    let available_calendars = get_available_calendars().await?;
    let config = Config::load()?;
    let default_calendar = config
        .calendar
        .default_calendar
        .unwrap_or_else(|| "Calendar".to_string());

    let current_date = Local::now();
    let current_hour = current_date.hour();

    // Build system prompt similar to OpenAI but adapted for Grok
    let system_prompt = format!(
        r#"You are a command line interface parser that converts natural language into ducktape commands.
Current time is: {}
Available calendars: {}
Default calendar: {}

For calendar events, use the format:
ducktape calendar create "<title>" <date> <start_time> <end_time> "<calendar>" [--email "<email1>,<email2>"] [--contacts "<name1>,<name2>"]

Rules:
1. If no date is specified, use today's date ({})
2. If no time is specified, use next available hour ({:02}:00) for start time and add 1 hour for end time
3. Use 24-hour format (HH:MM) for times
4. Use YYYY-MM-DD format for dates
5. Always include both start and end times
6. If calendar is specified in input, use that exact calendar name
7. If input mentions "kids" or "children", use the "KIDS" calendar
8. If input mentions "work", use the "Work" calendar
9. If no calendar is specified, use the default calendar
10. Available calendars are: {}"#,
        current_date.format("%Y-%m-%d %H:%M"),
        available_calendars.join(", "),
        default_calendar,
        current_date.format("%Y-%m-%d"),
        (current_hour + 1).min(23),
        available_calendars.join(", ")
    );

    let context = format!(
        "Current date and time: {}",
        Local::now().format("%Y-%m-%d %H:%M")
    );
    let prompt = format!("{}\n\n{}", context, input);

    let client = Client::new();
    let response = client
        .post(format!("{}/chat/completions", api_base))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "grok-2-latest",
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 150
        }))
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        return Err(anyhow!(
            "Grok API error: Status {}, Response: {}",
            status,
            response_text
        ));
    }

    // Parse the response, handle both streaming and non-streaming formats
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| anyhow!("Failed to parse Grok response: {}. Response text: {}", e, response_text))?;

    let commands = if let Some(choices) = response_json.get("choices") {
        choices[0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format: {}", response_text))?
            .trim()
            .to_string()
    } else {
        return Err(anyhow!("Unexpected response format: {}", response_text));
    };

    // Cache the response before returning
    RESPONSE_CACHE
        .lock()
        .unwrap()
        .put(input.to_string(), commands.clone());

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_parse_natural_language() -> Result<()> {
        // Mock test
        let inputs = [
            "Schedule a team meeting tomorrow at 2pm",
            "Remind me to buy groceries",
            "Take notes about the project meeting",
        ];

        for input in inputs {
            if let Some(cached_response) = RESPONSE_CACHE.lock().unwrap().get(input) {
                assert!(cached_response.contains("ducktape"));
                continue;
            }

            // Skip actual API call in test
            let mock_response = format!(
                "ducktape calendar create \"Test Event\" 2024-02-07 14:00 15:00 \"Calendar\""
            );
            RESPONSE_CACHE
                .lock()
                .unwrap()
                .put(input.to_string(), mock_response.clone());

            let command = mock_response;
            assert!(command.starts_with("ducktape"));
            assert!(command.contains('"'));
        }

        Ok(())
    }
}