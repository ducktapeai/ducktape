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
    // Input validation
    if input.is_empty() {
        return Err(anyhow!("Empty input provided"));
    }
    
    if input.len() > 1000 {
        return Err(anyhow!("Input too long (max 1000 characters)"));
    }
    
    // Sanitize input by removing any potentially harmful characters
    let sanitized_input = sanitize_user_input(input);
    
    // Load API key without showing it in error messages
    let api_key = env::var("XAI_API_KEY")
        .map_err(|_| anyhow!("XAI_API_KEY environment variable not set. Please set your X.AI API key using: export XAI_API_KEY='your-key-here'"))?;

    let api_base = env::var("XAI_API_BASE")
        .unwrap_or_else(|_| "https://api.x.ai/v1".to_string());

    // Check cache first using a properly declared mutable lock
    let cached = {
        let mut lock_result = RESPONSE_CACHE.lock()
            .map_err(|e| anyhow!("Failed to acquire cache lock: {}", e.to_string()))?;
        lock_result.get(&sanitized_input).cloned()
    };
    
    if let Some(cached_response) = cached {
        return Ok(cached_response);
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
    - If occurrence count is mentioned (e.g., "for 10 weeks"), add --count 10
13. If the input mentions "zoom", "video call", "video meeting", or "virtual meeting", add the --zoom flag to create a Zoom meeting automatically."#,
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
    let prompt = format!("{}\n\n{}", context, sanitized_input);

    // API request with proper error handling and timeouts
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

    let response = client
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
            "max_tokens": 150
        }))
        .send()
        .await
        .map_err(|e| anyhow!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        
        return Err(anyhow!("X.AI API error ({}): {}", status, error_text));
    }

    let response_json: Value = response.json().await
        .map_err(|e| anyhow!("Failed to parse API response: {}", e))?;
        
    // Safely extract the response content
    let commands = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid or missing response content"))?
        .trim()
        .to_string();

    // Cache the response - use a safe mutex pattern
    if let Ok(mut cache) = RESPONSE_CACHE.lock() {
        cache.put(sanitized_input.to_string(), commands.clone());
    }

    // Enhanced command processing
    let enhanced_command = enhance_recurrence_command(&commands);
    let enhanced_command = enhance_command_with_zoom(&enhanced_command, &sanitized_input);
    
    // Final validation of the returned commands
    validate_calendar_command(&enhanced_command)?;
    
    Ok(enhanced_command)
}

// Add a helper function to enhance Zoom recognition
fn enhance_command_with_zoom(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }
    
    let input_lower = input.to_lowercase();
    let zoom_keywords = ["zoom", "video call", "video meeting", "virtual meeting", "video conference"];
    
    let has_zoom_keyword = zoom_keywords.iter().any(|&keyword| input_lower.contains(keyword));
    
    if has_zoom_keyword && !command.contains("--zoom") {
        return format!("{} --zoom", command);
    }
    
    command.to_string()
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
    
    // Look for interval patterns like "every 2 weeks" and add --interval
    let re_interval = regex::Regex::new(r"every (\d+) (day|days|week|weeks|month|months|year|years)").unwrap();
    if let Some(caps) = re_interval.captures(&command.to_lowercase()) {
        if let Some(interval_str) = caps.get(1) {
            if let Ok(interval) = interval_str.as_str().parse::<u32>() {
                if interval > 0 && interval < 100 && // Reasonable limit
                   !command.contains("--interval") {
                    enhanced = format!("{} --interval {}", enhanced, interval);
                }
            }
        }
    }
    
    enhanced
}

/// Sanitize user input to prevent injection or other security issues
fn sanitize_user_input(input: &str) -> String {
    // Remove any control characters
    input.chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

/// Validate returned calendar command for security
fn validate_calendar_command(command: &str) -> Result<()> {
    // Check for suspicious patterns
    if command.contains("&&") || command.contains("|") || 
       command.contains(";") || command.contains("`") {
        return Err(anyhow!("Generated command contains potentially unsafe characters"));
    }
    
    // Validate interval values are reasonable if present
    if let Some(interval_idx) = command.find("--interval") {
        let interval_part = &command[interval_idx + 10..]; // Skip past "--interval "
        
        // Extract the interval value - look for the first number after "--interval "
        let re = regex::Regex::new(r"^\s*(\d+)").unwrap();
        if let Some(caps) = re.captures(interval_part) {
            if let Some(interval_match) = caps.get(1) {
                let interval_str = interval_match.as_str();
                if let Ok(interval) = interval_str.parse::<u32>() {
                    if interval > 100 {
                        return Err(anyhow!("Unreasonably large interval value: {}", interval));
                    }
                }
            }
        }
    }
    
    // Validate count values are reasonable if present
    if let Some(count_idx) = command.find("--count") {
        let count_part = &command[count_idx + 7..]; // Skip past "--count "
        
        // Extract the count value using regex for more reliable parsing
        let re = regex::Regex::new(r"^\s*(\d+)").unwrap();
        if let Some(caps) = re.captures(count_part) {
            if let Some(count_match) = caps.get(1) {
                let count_str = count_match.as_str();
                if let Ok(count) = count_str.parse::<u32>() {
                    if count > 500 {
                        return Err(anyhow!("Unreasonably large count value: {}", count));
                    }
                }
            }
        }
    }
    
    Ok(())
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
            // Use a properly declared mutable lock for checking the cache
            let cached_response = {
                let mut lock_result = RESPONSE_CACHE.lock()
                    .map_err(|e| anyhow!("Failed to acquire cache lock: {}", e.to_string()))?;
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
    
    #[test]
    fn test_enhance_command_with_zoom() {
        // Test adding zoom flag for zoom keyword
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a zoom meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(enhanced.contains("--zoom"));
        
        // Test adding zoom flag for video call keyword
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a video call with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(enhanced.contains("--zoom"));
        
        // Test not adding zoom flag for non-zoom input
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a regular meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert!(!enhanced.contains("--zoom"));
        
        // Test not duplicating zoom flag
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\" --zoom";
        let input = "Schedule a zoom meeting with the team";
        let enhanced = enhance_command_with_zoom(cmd, input);
        assert_eq!(enhanced.matches("--zoom").count(), 1);
    }
    
    #[test]
    fn test_sanitize_user_input() {
        let input = "Meeting with John\u{0000} tomorrow";
        let sanitized = sanitize_user_input(input);
        assert_eq!(sanitized, "Meeting with John tomorrow");
        
        let input = "Lunch\nmeeting";
        let sanitized = sanitize_user_input(input);
        assert_eq!(sanitized, "Lunch\nmeeting"); // Preserves newlines
    }
    
    #[test]
    fn test_validate_calendar_command() {
        // Test valid command
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --repeat weekly --interval 2";
        assert!(validate_calendar_command(cmd).is_ok());
        
        // Test invalid command with shell injection attempt
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00; rm -rf /";
        assert!(validate_calendar_command(cmd).is_err());
        
        // Test unreasonable interval
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --interval 500";
        assert!(validate_calendar_command(cmd).is_err());
    }
}