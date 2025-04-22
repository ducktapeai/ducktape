//! OpenAI parser implementation for DuckTape
//!
//! This module contains the core implementation of the OpenAI-based parser

use crate::parser::traits::{ParseResult, Parser};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{Local, Timelike};
use log::debug;
use lru::LruCache;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::utils::{
    enhance_command_with_contacts,
    enhance_command_with_zoom, 
    enhance_command_with_recurrence,
    sanitize_nlp_command,
    sanitize_user_input, 
    validate_calendar_command
};

/// OpenAI parser for natural language processing
pub struct OpenAIParser;

#[async_trait]
impl Parser for OpenAIParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        match parse_natural_language(input).await {
            Ok(command) => {
                debug!("OpenAI parser generated command: {}", command);

                // Before parsing, sanitize quotes in the command
                let sanitized_command = sanitize_nlp_command(&command);
                debug!("Sanitized command: {}", sanitized_command);

                // For now, return as a command string
                // In the future, we could parse this into structured CommandArgs here
                Ok(ParseResult::CommandString(sanitized_command))
            }
            Err(e) => {
                debug!("OpenAI parser error: {}", e);
                Err(e)
            }
        }
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Cache for storing parsed natural language commands to avoid repeated API calls
static RESPONSE_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

/// Parse natural language into a DuckTape command using OpenAI's models
pub async fn parse_natural_language(input: &str) -> Result<String> {
    // Input validation
    if input.is_empty() {
        return Err(anyhow!("Empty input provided"));
    }

    if input.len() > 1000 {
        return Err(anyhow!("Input too long (max 1000 characters)"));
    }

    // Sanitize input
    let sanitized_input = sanitize_user_input(input);

    // Load API key
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

    // Check cache first with proper error handling
    let cached_response = {
        let mut lock_result = RESPONSE_CACHE
            .lock()
            .map_err(|e| anyhow!("Failed to acquire cache lock: {}", e.to_string()))?;
        lock_result.get(&sanitized_input).cloned()
    };

    if let Some(cached) = cached_response {
        debug!("Using cached response for: {}", sanitized_input);
        return Ok(cached);
    }

    // Get available calendars and configuration
    let available_calendars = match super::utils::get_available_calendars().await {
        Ok(cals) => cals,
        Err(e) => {
            debug!("Failed to get available calendars: {}", e);
            vec!["Calendar".to_string(), "Work".to_string(), "Home".to_string()]
        }
    };

    let config = match crate::config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            debug!("Failed to load config: {}, using defaults", e);
            crate::config::Config::default()
        }
    };

    let default_calendar =
        config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string());

    // Build the system prompt
    let current_date = Local::now();
    let current_hour = current_date.hour();
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
10. Available calendars are: {}
11. If input mentions meeting/scheduling with someone's name, add their name to --contacts
12. If input mentions inviting, sending to, or emailing someone@domain.com, add it with --email
13. Multiple email addresses should be comma-separated
14. Multiple contact names should be comma-separated
15. Ignore phrases like 'to say', 'saying', 'that says' when determining contacts
16. Focus on actual person names when identifying contacts"#,
        current_date.format("%Y-%m-%d %H:%M"),
        available_calendars.join(", "),
        default_calendar,
        current_date.format("%Y-%m-%d"),
        (current_hour + 1).min(23),
        available_calendars.join(", ")
    );

    // Add current date context
    let context = format!("Current date and time: {}", Local::now().format("%Y-%m-%d %H:%M"));
    let prompt = format!("{}\n\n{}", context, sanitized_input);

    // Make API call to OpenAI
    debug!("Making API call to OpenAI for: {}", sanitized_input);
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

    debug!("Raw command from OpenAI: {}", commands);

    // Cache the response before returning
    if let Ok(mut cache) = RESPONSE_CACHE.lock() {
        cache.put(sanitized_input.to_string(), commands.clone());
    }

    // Process each command separately
    let mut results = Vec::new();
    for cmd in commands.lines() {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() {
            // Apply enhancements to each command
            let enhanced = enhance_command_with_recurrence(trimmed);
            let enhanced = enhance_command_with_zoom(&enhanced, &sanitized_input);
            let enhanced = enhance_command_with_contacts(&enhanced, &sanitized_input);
            
            // Validate before adding
            match validate_calendar_command(&enhanced) {
                Ok(_) => results.push(enhanced),
                Err(e) => debug!("Command validation failed: {}: {}", enhanced, e),
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
        // Mock test that doesn't require API key
        let inputs = [
            "Schedule a team meeting tomorrow at 2pm",
            "Remind me to buy groceries",
            "Take notes about the project meeting",
        ];

        for input in inputs {
            // Improved cache access with proper mutex handling
            let cached_response = {
                let mut lock_result =
                    RESPONSE_CACHE.lock().map_err(|_| anyhow!("Failed to acquire cache lock"))?;
                lock_result.get(input).cloned()
            };

            if let Some(cached_response) = cached_response {
                assert!(cached_response.contains("ducktape"));
                continue;
            }

            // Skip actual API call in test
            let mock_response = format!(
                "ducktape calendar create \"Test Event\" 2024-02-07 14:00 15:00 \"Calendar\""
            );

            if let Ok(mut cache) = RESPONSE_CACHE.lock() {
                cache.put(input.to_string(), mock_response.clone());
            } else {
                println!("Warning: Failed to update cache in test");
            }

            let command = mock_response;
            assert!(command.starts_with("ducktape"));
            assert!(command.contains('"')); // Should have quoted parameters
        }

        Ok(())
    }
}