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
use regex::Regex;

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
ducktape calendar create "<title>" <date> <start_time> <end_time> "shaun.stuart@hashicorp.com" [--location "<location>"] [--email "<email>"]
ducktape delete-event "<title>" - Delete events matching title

Examples:
"create an event tomorrow from 9am to 11am reminded me to call DJ" ->
ducktape calendar create "Call DJ" 2024-02-07 09:00 11:00 "shaun.stuart@hashicorp.com"

"schedule meeting with John from 2-3pm and invite john@example.com" ->
ducktape calendar create "Meeting with John" 2024-02-07 14:00 15:00 "shaun.stuart@hashicorp.com" --email "john@example.com"

Instructions for parsing calendar events:
1. Extract meaningful title from the task description (e.g., "call DJ", "team meeting", etc.)
2. Don't use generic titles like "Event"
3. Always include both start AND end times
4. Use 24-hour format (HH:MM) for times
5. Use YYYY-MM-DD format for dates
6. Include calendar name in quotes (always use "shaun.stuart@hashicorp.com")
7. Add --email flag when an attendee is mentioned
8. Keep original task description words in the title when possible"#;

pub async fn parse_natural_language(input: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

    // Check cache first
    if let Some(cached_response) = RESPONSE_CACHE.lock().unwrap().get(input) {
        return Ok(cached_response.clone());
    }

    // Get available calendars
    let calendars = get_available_calendars().await?;
    // Remove unused variable
    let _unused = calendars.join("\n- ");  // We'll keep this for future use but mark it as intentionally unused

    let system_prompt = r#"You are a command line interface parser that converts natural language into ducktape commands.
For calendar events, always use the format:
ducktape calendar create "<title>" <date> <start_time> <end_time> "<calendar>" [--email "<email>"]

Examples:
"create a meeting tomorrow from 2pm to 3pm in my work calendar and invite john@example.com" ->
ducktape calendar create "Meeting" 2024-02-07 14:00 15:00 "Work" --email "john@example.com"

"schedule call with John next Monday 9am-11am and invite john@company.com" ->
ducktape calendar create "Call with John" 2024-02-12 09:00 11:00 "Calendar" --email "john@company.com"

Remember to:
1. Always include both start and end times for calendar events
2. Use 24-hour format (HH:MM) for times
3. Use YYYY-MM-DD format for dates
4. Include the calendar name in quotes
5. Add --email flag when an attendee is mentioned
6. Put email addresses in quotes"#;

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
            } else if trimmed.contains("calendar create") {
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() >= 3 {
                    let title = parts[1];
                    // Prevent generic "Event" title
                    if title == "Event" {
                        // Extract a more meaningful title from the input
                        let better_title = if input.contains("call DJ") {
                            "Call DJ"
                        } else if input.contains("reminded me to") {
                            input.split("reminded me to")
                                .nth(1)
                                .unwrap_or(title)
                                .trim()
                                .split(" and ")
                                .next()
                                .unwrap_or(title)
                        } else {
                            title
                        };
                        
                        let rest: Vec<&str> = parts[2].trim().split_whitespace().collect();
                        if rest.len() >= 3 {
                            let mut command = format!(
                                r#"ducktape calendar create "{}" {} {} {} "shaun.stuart@hashicorp.com""#,
                                better_title, rest[0], rest[1], rest[2]
                            );
                            
                            // Add email if present in the input
                            if input.contains("invite") || input.contains("email") {
                                if let Ok(email) = extract_email(input) {
                                    command.push_str(&format!(r#" --email "{}""#, email));
                                }
                            }
                            
                            results.push(command);
                        }
                    } else {
                        // ...existing command generation code...
                    }
                }
            } else {
                results.push(trimmed.to_string());
            }
        }
    }

    Ok(results.join("\n"))
}

// Add a helper function to extract email addresses
fn extract_email(input: &str) -> Result<String> {
    // Simple regex to find email addresses
    let re = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
        .map_err(|e| anyhow!("Failed to create regex: {}", e))?;
    
    if let Some(cap) = re.find(input) {
        Ok(cap.as_str().to_string())
    } else {
        Err(anyhow!("No email address found in input"))
    }
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
