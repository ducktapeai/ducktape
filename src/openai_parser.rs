use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use chrono::Local;

const SYSTEM_PROMPT: &str = r#"You are a command parser for the DuckTape CLI tool. Convert natural language to DuckTape commands.
Available commands and their formats:

Calendar:
ducktape calendar "<title>" <date> <time> [calendar-name] [--location] [--description] [--email] [--all-day]

Todo:
ducktape todo "<title>" [--notes] [--lists] [--reminder-time]

Notes:
ducktape note "<title>" --content "<content>" [--folder]

Examples:
"Schedule a meeting tomorrow at 2pm" -> ducktape calendar "Meeting" 2024-02-06 14:00 "Work"
"Add grocery shopping to my todo list" -> ducktape todo "Grocery Shopping" --lists "Personal"
"Write down project ideas" -> ducktape note "Project Ideas" --content "Project brainstorming" --folder "Work"
"Remind me to call John next Monday" -> ducktape todo "Call John" --reminder-time "2024-02-12 09:00"

Rules:
1. Always return only the command, no explanations
2. Use proper date and time formatting (YYYY-MM-DD HH:MM)
3. When dates like "tomorrow" or "next Monday" are mentioned, calculate the actual date from the current date context
4. Ensure all text parameters are properly quoted
5. For calendar events, default to "Work" calendar if none specified"#;

pub async fn parse_natural_language(input: &str) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

    // Add current date context to help with relative dates
    let context = format!("Current date and time: {}", 
        Local::now().format("%Y-%m-%d %H:%M"));
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
                    "content": SYSTEM_PROMPT
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
    let command = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid response format"))?
        .trim()
        .to_string();

    Ok(command)
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
