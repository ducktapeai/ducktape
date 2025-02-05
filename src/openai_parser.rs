#[allow(unused_imports)]
use crate::calendar;
use anyhow::{anyhow, Result};
use chrono::Local;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::sync::Mutex;
use lru::LruCache;  // Fix: use correct import for LruCache
use std::num::NonZeroUsize;

static RESPONSE_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

// Function to get available calendars
async fn get_available_calendars() -> Result<Vec<String>> {
    // Execute AppleScript to get calendars
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

const SYSTEM_PROMPT: &str = r#"You are a command parser for the DuckTape CLI tool. Convert natural language to DuckTape commands.
Available commands and their formats:

Calendar:
ducktape calendar "<title>" <date> <time> "shaun.stuart@hashicorp.com"
ducktape delete-event "<title>" - Delete events matching title

Todo:
ducktape todo "<title>" --lists "<list-name>"

Notes:
ducktape note "<title>" --content "<content>" [--folder "<folder>"]

Examples:
"Schedule a meeting tomorrow at 2pm" ->
ducktape calendar "Meeting" 2024-02-06 14:00 "shaun.stuart@hashicorp.com"

"Add todo item about domain" ->
ducktape todo "Check domain settings" --lists "surfergolfer"

"delete the meeting about ASB" ->
ducktape delete-event "Meeting about ASB"

For multiple commands, each command must be on a separate line and properly formatted.
Example of multiple commands:
"schedule meeting tomorrow and add todo" ->
ducktape calendar "Team Meeting" 2024-02-06 09:00 "shaun.stuart@hashicorp.com"
ducktape todo "Follow up on meeting" --lists "Work"

Rules:
1. Always use "shaun.stuart@hashicorp.com" as the calendar name for all calendar events
2. Use proper date/time format: YYYY-MM-DD HH:MM
3. Calculate actual dates for relative terms like "tomorrow", "next week"
4. Quote all text parameters properly
5. For todos, always use --lists flag with the appropriate list name
6. Return multiple commands on separate lines
7. Never include calendar name inside the event title"#;

pub async fn parse_natural_language(input: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

    // Check cache first
    if let Some(cached_response) = RESPONSE_CACHE.lock().unwrap().get(input) {
        return Ok(cached_response.clone());
    }

    // Get available calendars
    let calendars = get_available_calendars().await?;
    let calendar_list = calendars.join("\n- ");
    let system_prompt =
        SYSTEM_PROMPT.replace("{available_calendars}", &format!("- {}", calendar_list));

    // Add current date context
    let context = format!(
        "Current date and time: {}",
        Local::now().format("%Y-%m-%d %H:%M")
    );
    let prompt = format!("{}\n\n{}", context, input);

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-4",
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

    if !response.status().is_success() {
        return Err(anyhow!("OpenAI API error: {}", response.status()));
    }

    let response_json: Value = response.json().await?;
    let commands = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid response format"))?
        .trim()
        .to_string();

    // Cache the response before returning
    RESPONSE_CACHE
        .lock()
        .unwrap()
        .put(input.to_string(), commands.clone());

    // Process each command separately
    let mut results = Vec::new();
    for cmd in commands.lines() {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() {
            if trimmed.contains("todo") {
                // Format todo command correctly
                if trimmed.contains("--lists") {
                    results.push(trimmed.to_string());
                } else {
                    results.push(format!("{} --lists \"surfergolfer\"", trimmed));
                }
            } else if trimmed.contains("calendar") {
                // Ensure proper calendar command format
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() >= 3 {
                    // Extract title and rest of command
                    let title = parts[1];
                    let rest: Vec<&str> = parts[2].trim().split_whitespace().collect();
                    if rest.len() >= 2 {
                        results.push(format!(
                            "ducktape calendar \"{}\" {} {} \"shaun.stuart@hashicorp.com\"",
                            title, rest[0], rest[1]
                        ));
                    }
                }
            } else {
                results.push(trimmed.to_string());
            }
        }
    }

    Ok(results.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_parse_natural_language() -> Result<()> {
        // Note: These tests require a valid OPENAI_API_KEY environment variable
        let inputs = [
            "Schedule a team meeting tomorrow at 2pm",
            "Remind me to buy groceries",
            "Take notes about the project meeting",
        ];

        for input in inputs {
            let command = parse_natural_language(input).await?;
            assert!(command.starts_with("ducktape "));
            assert!(command.contains('"')); // Should have quoted parameters
        }

        Ok(())
    }
}
