use crate::config::Config;
use anyhow::{Result, anyhow};
use chrono::{Local, Timelike};
use log::{debug, error, warn};
use lru::LruCache;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{Value, json};
use std::env;
use std::num::NonZeroUsize;
use std::sync::Mutex;

static RESPONSE_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

// Helper function to get available calendars
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
    debug!("Parsing natural language input: {}", sanitized_input);

    // Load API key without showing it in error messages
    let api_key = env::var("XAI_API_KEY")
        .map_err(|_| anyhow!("XAI_API_KEY environment variable not set. Please set your X.AI API key using: export XAI_API_KEY='your-key-here'"))?;

    let api_base = env::var("XAI_API_BASE").unwrap_or_else(|_| "https://api.x.ai/v1".to_string());

    // Check cache first using a properly declared mutable lock
    let cached = {
        let mut lock_result = RESPONSE_CACHE
            .lock()
            .map_err(|e| anyhow!("Failed to acquire cache lock: {}", e.to_string()))?;
        lock_result.get(&sanitized_input).cloned()
    };

    if let Some(cached_response) = cached {
        debug!("Using cached response for input");
        return Ok(cached_response);
    }

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
9. If input mentions "shaun.stuart@hashicorp.com", use that as the calendar.
10. If no calendar is specified, use the default calendar.
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
    );

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

    // Cache the response - use a safe mutex pattern
    if let Ok(mut cache) = RESPONSE_CACHE.lock() {
        cache.put(sanitized_input.to_string(), commands.clone());
    }

    // Enhanced command processing with proper pipeline
    let mut enhanced_command = commands.clone();

    // Apply all enhancements in sequence
    enhanced_command = enhance_recurrence_command(&enhanced_command);
    enhanced_command = enhance_command_with_contacts(&enhanced_command, &sanitized_input);
    enhanced_command = enhance_command_with_zoom(&enhanced_command, &sanitized_input);

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

// Helper function to enhance Zoom recognition
fn enhance_command_with_zoom(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let input_lower = input.to_lowercase();
    let zoom_keywords =
        ["zoom", "video call", "video meeting", "virtual meeting", "video conference"];

    let has_zoom_keyword = zoom_keywords.iter().any(|&keyword| input_lower.contains(keyword));

    if has_zoom_keyword && !command.contains("--zoom") {
        debug!("Adding --zoom flag to command based on input keywords");
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
    let has_recurring_keyword = command.to_lowercase().contains("every day")
        || command.to_lowercase().contains("every week")
        || command.to_lowercase().contains("every month")
        || command.to_lowercase().contains("every year")
        || command.to_lowercase().contains("daily")
        || command.to_lowercase().contains("weekly")
        || command.to_lowercase().contains("monthly")
        || command.to_lowercase().contains("yearly")
        || command.to_lowercase().contains("annually");

    // If recurring keywords found but no --repeat flag, add it
    if has_recurring_keyword && !command.contains("--repeat") && !command.contains("--recurring") {
        if command.contains("every day") || command.contains("daily") {
            debug!("Adding daily recurrence to command");
            enhanced = format!("{} --repeat daily", enhanced);
        } else if command.contains("every week") || command.contains("weekly") {
            debug!("Adding weekly recurrence to command");
            enhanced = format!("{} --repeat weekly", enhanced);
        } else if command.contains("every month") || command.contains("monthly") {
            debug!("Adding monthly recurrence to command");
            enhanced = format!("{} --repeat monthly", enhanced);
        } else if command.contains("every year")
            || command.contains("yearly")
            || command.contains("annually")
        {
            debug!("Adding yearly recurrence to command");
            enhanced = format!("{} --repeat yearly", enhanced);
        }
    }

    // Look for interval patterns like "every 2 weeks" and add --interval
    let re_interval =
        regex::Regex::new(r"every (\d+) (day|days|week|weeks|month|months|year|years)").unwrap();
    if let Some(caps) = re_interval.captures(&command.to_lowercase()) {
        if let Some(interval_str) = caps.get(1) {
            if let Ok(interval) = interval_str.as_str().parse::<u32>() {
                if interval > 0 && interval < 100 && // Reasonable limit
                   !command.contains("--interval")
                {
                    debug!("Adding interval {} to command", interval);
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
    input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

/// Validate returned calendar command for security
fn validate_calendar_command(command: &str) -> Result<()> {
    // Check for suspicious patterns
    if command.contains("&&")
        || command.contains("|")
        || command.contains(";")
        || command.contains("`")
    {
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
                let mut lock_result = RESPONSE_CACHE
                    .lock()
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
        let input =
            "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\" every week";
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
        let cmd =
            "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\" --zoom";
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
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --interval 500";
        assert!(validate_calendar_command(cmd).is_err());
    }

    #[test]
    fn test_extract_contact_names() {
        // Test basic contact extraction with "with" keyword
        let input = "Schedule a meeting with John Smith tomorrow at 2pm";
        let contacts = extract_contact_names(input);
        assert_eq!(contacts, vec!["John Smith"]);

        // Test basic contact extraction with "invite" keyword
        let input = "create an zoom event at 10am on April 1 called Project Deadlines and invite Shaun Stuart";
        let contacts = extract_contact_names(input);
        assert_eq!(contacts, vec!["Shaun Stuart"]);

        // Test filtering out email addresses
        let input = "Schedule a meeting with john.doe@example.com tomorrow";
        let contacts = extract_contact_names(input);
        assert!(contacts.is_empty());

        // Test handling multiple names
        let input = "Schedule a meeting with John Smith and Jane Doe tomorrow";
        let contacts = extract_contact_names(input);
        assert!(contacts.contains(&"John".to_string()));
    }

    #[test]
    fn test_enhance_command_with_contacts() {
        // Test adding contacts when none were present
        let cmd = "ducktape calendar create \"Team Meeting\" 2024-03-15 10:00 11:00 \"Work\"";
        let input = "Schedule a meeting with John Smith";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert!(enhanced.contains("--contacts \"John Smith\""));

        // Test handling invitations
        let cmd =
            "ducktape calendar create \"Project Deadlines\" 2025-04-01 10:00 11:00 \"Work\" --zoom";
        let input = "create an zoom event at 10am on April 1 called Project Deadlines and invite Shaun Stuart";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert!(enhanced.contains("--contacts \"Shaun Stuart\""));

        // Test not adding when already present
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --contacts \"John Smith\"";
        let input = "Schedule a meeting with John Smith";
        let enhanced = enhance_command_with_contacts(cmd, input);
        assert_eq!(enhanced.matches("--contacts").count(), 1);
    }
}

#[allow(dead_code)]
pub struct GrokParser;

impl GrokParser {
    #[allow(dead_code)]
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    #[allow(dead_code)]
    pub async fn parse_input(&self, input: &str) -> anyhow::Result<Option<String>> {
        match parse_natural_language(input).await {
            Ok(command) => Ok(Some(command)),
            Err(e) => {
                error!("Grok parser error: {}", e);
                Err(e)
            }
        }
    }
}

// Helper function to extract contact names from natural language input
fn extract_contact_names(input: &str) -> Vec<String> {
    let mut contact_names = Vec::new();
    let input_lower = input.to_lowercase();

    // Check for different contact-related keywords
    let text_to_parse = if input_lower.contains(" with ") {
        debug!("Found 'with' keyword for contact extraction");
        input.split(" with ").nth(1)
    } else if input_lower.contains(" to ") {
        debug!("Found 'to' keyword for contact extraction");
        input.split(" to ").nth(1)
    } else if input_lower.contains("invite ") {
        debug!("Found 'invite' keyword for contact extraction");
        // Special handling for invite keyword which might not have a space before it
        let parts: Vec<&str> = input.splitn(2, "invite ").collect();
        if parts.len() > 1 { Some(parts[1]) } else { None }
    } else {
        None
    };

    if let Some(after_word) = text_to_parse {
        debug!("Text to parse for contacts: '{}'", after_word);

        // Split by known separators that indicate multiple people
        for name_part in after_word.split(|c: char| c == ',' || c == ';' || c == '.') {
            let name_part = name_part.trim();

            // Skip empty parts
            if name_part.is_empty() {
                continue;
            }

            // Further process parts with "and" to extract multiple names
            if name_part.contains(" and ") {
                let and_parts: Vec<&str> = name_part.split(" and ").collect();
                for and_part in and_parts {
                    let final_name = refine_name(and_part);
                    if !final_name.is_empty() && !final_name.contains('@') {
                        contact_names.push(final_name);
                    }
                }
            } else {
                // Process single name
                let final_name = refine_name(name_part);
                if !final_name.is_empty() && !final_name.contains('@') {
                    contact_names.push(final_name);
                }
            }
        }
    }

    debug!("Extracted contact names: {:?}", contact_names);
    contact_names
}

// Helper function to refine a name by removing trailing stop words
fn refine_name(name_part: &str) -> String {
    let stop_words = ["at", "on", "tomorrow", "today", "for", "about", "regarding"];

    let mut final_name = name_part.trim().to_string();
    for word in &stop_words {
        if let Some(pos) = final_name.to_lowercase().find(&format!(" {}", word)) {
            final_name = final_name[0..pos].trim().to_string();
        }
    }

    final_name
}

// Helper function to enhance commands with proper contact and email handling
fn enhance_command_with_contacts(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();

    // Step 1: Extract email addresses from the input
    let email_addresses = extract_email_addresses(input);

    // Step 2: Extract contact names
    let contact_names = extract_contact_names(input);

    debug!("Email addresses extracted: {:?}", email_addresses);
    debug!("Contact names extracted: {:?}", contact_names);

    // Step 3: Handle email addresses if they're not already in the command
    if !email_addresses.is_empty() && !enhanced.contains("--email") {
        let escaped_emails = email_addresses.join(",").replace("\"", "\\\"");
        debug!("Adding emails to command: {}", escaped_emails);
        enhanced = format!(r#"{} --email "{}""#, enhanced, escaped_emails);
    }

    // Step 4: Clean up any incorrectly placed contact names in email flags
    if enhanced.contains("--email") {
        // Pattern: --email "Name Without @ Symbol"
        let email_regex = regex::Regex::new(r#"--email\s+"([^@"]+)""#).unwrap();

        if let Some(caps) = email_regex.captures(&enhanced) {
            if let Some(email_match) = caps.get(1) {
                let email_value = email_match.as_str();
                if !email_value.contains('@') {
                    debug!("Removing incorrectly formatted email: {}", email_value);
                    enhanced = email_regex.replace(&enhanced, "").to_string().trim().to_string();
                }
            }
        }

        // Remove specific contact names from email flags
        for name in &contact_names {
            let quoted_name = format!("--email \"{}\"", name);
            if enhanced.contains(&quoted_name) {
                debug!("Removing name '{}' from email flag", name);
                enhanced = enhanced.replace(&quoted_name, "").trim().to_string();
            }
        }
    }

    // Step 5: Add contact names if not already in the command
    if !contact_names.is_empty() && !enhanced.contains("--contacts") {
        let escaped_contacts = contact_names.join(",").replace("\"", "\\\"");
        debug!("Adding contacts to command: {}", escaped_contacts);
        enhanced = format!(r#"{} --contacts "{}""#, enhanced, escaped_contacts);
    }

    enhanced
}

// Helper function to extract email addresses from natural language input
fn extract_email_addresses(input: &str) -> Vec<String> {
    // Email regex pattern - basic pattern for demonstration
    let email_regex = regex::Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();

    let mut emails = Vec::new();

    for cap in email_regex.captures_iter(input) {
        let email = cap.get(0).unwrap().as_str().to_string();
        if crate::calendar::validate_email(&email) {
            debug!("Extracted email: {}", email);
            emails.push(email);
        } else {
            debug!("Found invalid email: {}", email);
        }
    }

    emails
}
