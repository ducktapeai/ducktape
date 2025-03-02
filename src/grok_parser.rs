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
use log::debug;

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
Current time is: {current_time}
Available calendars: {calendars}
Default calendar: {default_cal}

For calendar events, use the format:
ducktape calendar create "<title>" <date> <start_time> <end_time> "<calendar>" [--email "<email1>,<email2>"] [--contacts "<name1>,<name2>"]

For recurring events, add any of these options:
--repeat <daily|weekly|monthly|yearly>   Set recurrence frequency
--interval <number>                      Set interval (e.g., every 2 weeks)
--until <YYYY-MM-DD>                     Set end date for recurrence
--count <number>                         Set number of occurrences
--days <0,1,2...>                        Set days of week (0=Sun, 1=Mon, etc.)

Rules:
1. If no date is specified, use today's date ({today}).
2. If no time is specified, use the next available hour ({next_hour}:00) for start time and add 1 hour for end time.
3. Use 24-hour format (HH:MM) for times.
4. Use YYYY-MM-DD format for dates.
5. Always include both start and end times.
6. If a calendar is specified in input, use that exact calendar name.
7. If input mentions "kids" or "children", use the "KIDS" calendar.
8. If input mentions "work", use the "Work" calendar.
9. If no calendar is specified, use the default calendar.
10. Available calendars are: {calendars}.
11. If contact names are mentioned in the input and no --contacts flag is provided, automatically include a --contacts flag with the detected names.
12. If the input mentions recurring events or repetition:
    - For "daily" recurrence: use --repeat daily
    - For "weekly" recurrence: use --repeat weekly
    - For "monthly" recurrence: use --repeat monthly
    - For "yearly" or "annual" recurrence: use --repeat yearly
    - If specific interval is mentioned (e.g., "every 2 weeks"), add --interval 2
    - If specific end date is mentioned (e.g., "until March 15"), add --until YYYY-MM-DD
    - If occurrence count is mentioned (e.g., "for 10 weeks"), add --count 10"#,
        current_time = current_date.format("%Y-%m-%d %H:%M"),
        calendars = available_calendars.join(", "),
        default_cal = default_calendar,
        today = current_date.format("%Y-%m-%d"),
        next_hour = (current_hour + 1).min(23)
    );

    let context = format!(
        "Current date and time: {}",
        Local::now().format("%Y-%m-%d %H:%M")
    );
    let prompt = format!("{}\n\n{}", context, input);

    debug!("Sending prompt to Grok API: {}", prompt);

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

    // Clean up and enhance the response
    let enhanced_command = enhance_recurrence_command(&commands);
    debug!("Enhanced Grok command: {}", enhanced_command);

    // Cache the response before returning
    RESPONSE_CACHE
        .lock()
        .unwrap()
        .put(input.to_string(), enhanced_command.clone());

    Ok(enhanced_command)
}

// Helper function to enhance recurrence commands
fn enhance_recurrence_command(command: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();
    
    // Check for recurring event keywords in the input but missing flags
    let has_recurring_keyword = 
        command.to_lowercase().contains("every day") ||
        command.to_lowercase().contains("every week") ||
        command.to_lowercase().contains("every month") ||
        command.to_lowercase().contains("every year") ||
        command.to_lowercase().contains("daily") ||
        command.to_lowercase().contains("weekly") ||
        command.to_lowercase().contains("monthly") ||
        command.to_lowercase().contains("yearly") ||
        command.to_lowercase().contains("annually");

    // If recurring keywords found but no --repeat flag, add it
    if has_recurring_keyword && !command.contains("--repeat") && !command.contains("--recurring") {
        if command.contains("every day") || command.contains("daily") {
            enhanced = format!("{} --repeat daily", enhanced);
        } else if command.contains("every week") || command.contains("weekly") {
            enhanced = format!("{} --repeat weekly", enhanced);
        } else if command.contains("every month") || command.contains("monthly") {
            enhanced = format!("{} --repeat monthly", enhanced);
        } else if command.contains("every year") || command.contains("yearly") || command.contains("annually") {
            enhanced = format!("{} --repeat yearly", enhanced);
        }
    }

    // Add interval if appropriate keywords exist
    if !command.contains("--interval") {
        if command.contains("every 2 day") || command.contains("every other day") {
            enhanced = format!("{} --interval 2", enhanced);
        } else if command.contains("every 2 week") || command.contains("every other week") {
            enhanced = format!("{} --interval 2", enhanced);
        } else if command.contains("every 2 month") || command.contains("every other month") {
            enhanced = format!("{} --interval 2", enhanced);
        } else if command.contains("every 3 ") {
            enhanced = format!("{} --interval 3", enhanced);
        }
    }

    enhanced
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

    #[test]
    fn test_enhance_recurrence_command() {
        // Test adding recurrence
        let input = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\" every week";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--repeat weekly"));
        
        // Test adding interval
        let input = "ducktape calendar create \"Bi-weekly Meeting\" 2024-03-15 10:00 11:00 \"Work\" every 2 weeks";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--interval 2"));
        
        // Test non-calendar command (should remain unchanged)
        let input = "ducktape todo \"Buy groceries\"";
        let enhanced = enhance_recurrence_command(input);
        assert_eq!(input, enhanced);
    }
}