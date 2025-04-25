//! API module for Grok parser
//!
//! This module handles the communication with the Grok/X.AI API
//! for natural language processing.

use super::cache;
use super::time_extractor::extract_time_from_title;
use super::utils::{
    enhance_command_with_contacts, enhance_command_with_zoom, enhance_recurrence_command,
    fix_calendar_end_time_format, sanitize_nlp_command,
};
use crate::config::Config;
use crate::parser::natural_language::utils::validate_calendar_command;
use anyhow::{Result, anyhow};
use chrono::{Local, Timelike};
use log::{debug, error, warn};
use reqwest::Client;
use serde_json::{Value, json};
use std::env;

/// Helper function to get available calendars
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

/// Parse natural language input into a Ducktape command
///
/// This function does the following:
/// 1. Validates and sanitizes the input
/// 2. Checks the cache for a previously processed identical input
/// 3. Prepares the query to be sent to the Grok/X.AI API
/// 4. Handles the API response
/// 5. Enhances the command with appropriate flags and formatting
///
/// # Arguments
///
/// * `input` - The natural language input to parse
///
/// # Returns
///
/// A Result containing the Ducktape command string
pub async fn parse_natural_language(input: &str) -> Result<String> {
    // Input validation
    if input.is_empty() {
        return Err(anyhow!("Empty input provided"));
    }

    if input.len() > 1000 {
        return Err(anyhow!("Input too long (max 1000 characters)"));
    }

    // Sanitize input by removing any potentially harmful characters
    let sanitized_input = crate::parser::natural_language::utils::sanitize_user_input(input);
    debug!("Sanitized input: {}", sanitized_input);

    // Check if this is a todo/reminder request vs a calendar event
    let input_lower = sanitized_input.to_lowercase();
    let is_todo_request = input_lower.contains("todo")
        || input_lower.contains("reminder")
        || input_lower.contains("task")
        || (input_lower.contains("remind") && !input_lower.contains("meeting"))
        || input_lower.contains("checklist");

    // Check cache first
    if let Some(cached_response) = cache::get_cached_response(&sanitized_input) {
        debug!("Using cached response for input");
        return Ok(cached_response);
    }

    // Load API key without showing it in error messages
    let api_key = env::var("XAI_API_KEY")
        .map_err(|_| anyhow!("XAI_API_KEY environment variable not set. Please set your X.AI API key using: export XAI_API_KEY='your-key-here'"))?;

    let api_base = env::var("XAI_API_BASE").unwrap_or_else(|_| "https://api.x.ai/v1".to_string());

    // Get available calendars and configuration
    let available_calendars = match get_available_calendars().await {
        Ok(cals) => cals,
        Err(e) => {
            warn!("Failed to get available calendars: {}", e);
            vec!["Calendar".to_string(), "Work".to_string(), "Home".to_string(), "KIDS".to_string()]
        }
    };

    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Failed to load config: {}, using defaults", e);
            Config::default()
        }
    };

    let default_calendar =
        config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string());

    let current_date = Local::now();
    let current_hour = current_date.hour();

    // Build system prompt based on the type of request
    let system_prompt = if is_todo_request {
        // For todo/reminder requests
        format!(
            r#"You are a command line interface parser that converts natural language into ducktape commands.
Current time is: {current_time}
Available reminder lists: Reminders, Work, Personal, Urgent

For todo/reminder items, use the format:
ducktape todo create "<title>" [list1] [list2] [--remind "<YYYY-MM-DD HH:MM>"] [--notes "<additional details>"]

Rules:
1. If no specific time is mentioned, do not add the --remind flag.
2. If a time is specified, use --remind with format "YYYY-MM-DD HH:MM".
3. If today or tomorrow is mentioned, use the actual date ({today} or {tomorrow}).
4. If no list is specified, use just one argument: the title.
5. If notes or details are provided, add them with --notes flag.
6. If input mentions "work", add the "Work" list.
7. If input mentions "personal", add the "Personal" list.
8. If input mentions "urgent" or "important", add the "Urgent" list."#,
            current_time = current_date.format("%Y-%m-%d %H:%M"),
            today = current_date.format("%Y-%m-%d"),
            tomorrow = (current_date + chrono::Duration::days(1)).format("%Y-%m-%d")
        )
    } else {
        // For calendar events
        format!(
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
11. If contact names are mentioned in the input and no --contacts flag is provided, automatically include a --contacts flag with the detected names.
12. If input mentions scheduling "with" someone, add their name to --contacts.
13. If input mentions inviting, sending to, or emailing someone@domain.com, add it with --email.
14. Multiple email addresses should be comma-separated.
15. Multiple contact names should be comma-separated.
16. If the input mentions recurring events or repetition:
    - For "daily" recurrence: use --repeat daily
    - For "weekly" recurrence: use --repeat weekly
    - For "monthly" recurrence: use --repeat monthly
    - For "yearly" or "annual" recurrence: use --repeat yearly
    - If specific interval is mentioned (e.g., "every 2 weeks"), add --interval 2
    - If specific end date is mentioned (e.g., "until March 15"), add --until YYYY-MM-DD
    - If occurrence count is mentioned (e.g., "for 10 weeks"), add --count 10
17. If the input mentions "zoom", "video call", "video meeting", or "virtual meeting", add the --zoom flag to create a Zoom meeting automatically."#,
            current_time = current_date.format("%Y-%m-%d %H:%M"),
            calendars = available_calendars.join(", "),
            default_cal = default_calendar,
            today = current_date.format("%Y-%m-%d"),
            next_hour = (current_hour + 1).min(23)
        )
    };

    let context = format!("Current date and time: {}", Local::now().format("%Y-%m-%d %H:%M"));
    let prompt = format!("{}\n\n{}", context, sanitized_input);

    debug!("Sending request to Grok API with prompt: {}", prompt);

    // API request with proper error handling and timeouts
    let client = match Client::builder().timeout(std::time::Duration::from_secs(30)).build() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create HTTP client: {}", e);
            return Err(anyhow!("Failed to create HTTP client: {}", e));
        }
    };

    let response = match client
        .post(format!("{}/chat/completions", api_base))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
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
            "max_tokens": 200
        }))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("API request to Grok failed: {}", e);
            return Err(anyhow!("API request failed: {}", e));
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());

        error!("X.AI API error ({}): {}", status, error_text);
        return Err(anyhow!("X.AI API error ({}): {}", status, error_text));
    }

    let response_json: Value = match response.json().await {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to parse Grok API response: {}", e);
            return Err(anyhow!("Failed to parse API response: {}", e));
        }
    };

    // Safely extract the response content
    let commands = match response_json["choices"][0]["message"]["content"].as_str() {
        Some(content) => content.trim().to_string(),
        None => {
            error!("Invalid or missing response content from Grok API");
            return Err(anyhow!("Invalid or missing response content"));
        }
    };

    debug!("Received command from Grok API: {}", commands);

    // Cache the response
    cache::store_response(&sanitized_input, &commands);

    // Enhanced command processing with proper pipeline
    let mut enhanced_command = commands.clone();

    // Apply all enhancements in sequence
    enhanced_command = enhance_recurrence_command(&enhanced_command);
    enhanced_command = enhance_command_with_contacts(&enhanced_command, &sanitized_input);
    enhanced_command = enhance_command_with_zoom(&enhanced_command, &sanitized_input);
    enhanced_command = extract_time_from_title(&enhanced_command, &sanitized_input);
    enhanced_command = fix_calendar_end_time_format(&enhanced_command);

    // Final validation of the returned commands
    match validate_calendar_command(&enhanced_command) {
        Ok(_) => {
            debug!("Successfully parsed natural language input to command: {}", enhanced_command);
            Ok(enhanced_command)
        }
        Err(e) => {
            error!("Command validation failed: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::natural_language::grok::GrokParser;
    use crate::parser::traits::ParseResult;

    #[tokio::test]
    async fn test_parse_natural_language() -> Result<()> {
        // Mock test that doesn't require actual API key
        let inputs = [
            "Schedule a team meeting tomorrow at 2pm",
            "Remind me to buy groceries",
            "Take notes about the project meeting",
        ];

        for input in inputs {
            // Mock a response for testing
            let mock_response = format!(
                "ducktape calendar create \"Test Event\" 2024-02-07 14:00 15:00 \"Calendar\""
            );

            // Store mock in cache so we don't make actual API calls
            cache::store_response(input, &mock_response);

            let result = parse_natural_language(input).await?;
            assert!(result.starts_with("ducktape"));
            assert!(result.contains('"'));
        }

        Ok(())
    }
}
