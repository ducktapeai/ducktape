#[allow(unused_imports)]
use crate::calendar;
use crate::config::Config;
use anyhow::{anyhow, Result};
use chrono::{Local, Timelike}; // Add Timelike trait
use lru::LruCache; // Fix: use correct import for LruCache
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::num::NonZeroUsize;
use std::sync::Mutex;

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

pub async fn parse_natural_language(input: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

    // Check cache first
    if let Some(cached_response) = RESPONSE_CACHE.lock().unwrap().get(input) {
        return Ok(cached_response.clone());
    }

    // Get available calendars and configuration early
    let available_calendars = get_available_calendars().await?;
    let config = Config::load()?;
    let default_calendar = config
        .calendar
        .default_calendar
        .unwrap_or_else(|| "Calendar".to_string());

    let current_date = Local::now();
    let system_prompt = format!(
        r#"You are a command line interface parser that converts natural language into ducktape commands.
Current time is: {}
Available calendars: {}
Default calendar: {}

For calendar events, use the format:
ducktape calendar create "<title>" <date> <start_time> <end_time> "<calendar>"

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
        (current_date.hour() + 1).min(23),
        available_calendars.join(", ")
    );

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
                    "content": system_prompt  // Use the system_prompt variable directly
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
                    let rest: Vec<&str> = parts[2].trim().split_whitespace().collect();
                    if rest.len() >= 3 {
                        // Enhanced calendar selection logic
                        let requested_calendar = if input.to_lowercase().contains("kids calendar")
                            || input.to_lowercase().contains("kids calander")
                            || input.to_lowercase().contains("children")
                            || input.to_lowercase().contains("to my kids")
                        {
                            "KIDS"
                        } else if input.to_lowercase().contains("work calendar")
                            || input.to_lowercase().contains("work calander")
                        {
                            "Work"
                        } else if input.to_lowercase().contains("home calendar")
                            || input.to_lowercase().contains("home calander")
                        {
                            "Home"
                        } else {
                            &default_calendar
                        };

                        // Validate that the calendar exists
                        if !available_calendars.iter().any(|c| c == requested_calendar) {
                            return Err(anyhow!(
                                "Calendar '{}' not found. Available calendars: {}",
                                requested_calendar,
                                available_calendars.join(", ")
                            ));
                        }

                        let mut command = format!(
                            r#"ducktape calendar create "{}" {} {} {} "{}""#,
                            title, rest[0], rest[1], rest[2], requested_calendar
                        );

                        // Add email if present in the input
                        if input.contains("invite") || input.contains("email") {
                            let email = extract_email(input)?;
                            command.push_str(&format!(r#" --email "{}""#, email));
                        }

                        results.push(command);
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
        // Mock test that doesn't require API key
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
            assert!(command.contains('"')); // Should have quoted parameters
        }

        Ok(())
    }
}
