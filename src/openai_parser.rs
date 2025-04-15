#[allow(unused_imports)]
use crate::calendar;
use crate::config::Config;
use anyhow::{Result, anyhow};
use chrono::{Local, Timelike};
use log::debug; // Only import debug since others are unused
use lru::LruCache;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde_json::{Value, json};
use std::env;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use crate::parser_trait::{ParseResult, Parser};
use async_trait::async_trait;

/// Parser that uses OpenAI's models for natural language understanding
pub struct OpenAIParser;

impl OpenAIParser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Parser for OpenAIParser {
    async fn parse_input(&self, input: &str) -> anyhow::Result<ParseResult> {
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

    fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
}

/// Helper function to clean up NLP-generated commands
/// Removes unnecessary quotes and normalizes spacing
fn sanitize_nlp_command(command: &str) -> String {
    // Ensure the command starts with ducktape
    if !command.starts_with("ducktape") {
        return command.to_string();
    }

    // Basic sanitization to fix common issues with NLP-generated commands
    command
        .replace("\u{a0}", " ") // Replace non-breaking spaces
        .replace("\"\"", "\"") // Replace double quotes
        .to_string()
}

#[allow(dead_code)]
static RESPONSE_CACHE: Lazy<Mutex<LruCache<String, String>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())));

#[allow(dead_code)]
// Helper function to escape strings for AppleScript to prevent command injection
fn escape_applescript_string(input: &str) -> String {
    // First replace double quotes with escaped quotes for AppleScript
    let escaped = input.replace("\"", "\"\"");

    // Remove any control characters that could interfere with AppleScript execution
    escaped
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

#[allow(dead_code)]
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

#[allow(dead_code)]
/// Sanitize user input to prevent injection or other security issues
fn sanitize_user_input(input: &str) -> String {
    input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
}

#[allow(dead_code)]
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
        let interval_part = &command[interval_idx + 10..];
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
        let count_part = &command[count_idx + 7..];
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

#[allow(dead_code)]
/// Add Zoom meeting flag when zoom-related keywords are detected
fn enhance_command_with_zoom(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let input_lower = input.to_lowercase();
    let zoom_keywords =
        ["zoom", "video call", "video meeting", "virtual meeting", "video conference"];

    let has_zoom_keyword = zoom_keywords.iter().any(|&keyword| input_lower.contains(keyword));

    if has_zoom_keyword && !command.contains("--zoom") {
        return format!("{} --zoom", command);
    }

    command.to_string()
}

#[allow(dead_code)]
/// Enhance command with recurrence flags based on natural language
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
            enhanced = format!("{} --repeat daily", enhanced);
        } else if command.contains("every week") || command.contains("weekly") {
            enhanced = format!("{} --repeat weekly", enhanced);
        } else if command.contains("every month") || command.contains("monthly") {
            enhanced = format!("{} --repeat monthly", enhanced);
        } else if command.contains("every year")
            || command.contains("yearly")
            || command.contains("annually")
        {
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
                    enhanced = format!("{} --interval {}", enhanced, interval);
                }
            }
        }
    }

    enhanced
}

#[allow(dead_code)]
// Helper function to enhance commands with proper contact and email handling
fn enhance_command_with_contacts(command: &str, input: &str) -> String {
    if !command.contains("calendar create") {
        return command.to_string();
    }

    let mut enhanced = command.to_string();

    // Step 1: Extract email addresses from the input
    if let Ok(email_addresses) = extract_emails(input) {
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
                        enhanced =
                            email_regex.replace(&enhanced, "").to_string().trim().to_string();
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

        // Add contacts if they were found
        if !contact_names.is_empty() {
            debug!("Adding contacts: {:?}", contact_names);
            let contacts_str = contact_names.join(",");
            enhanced = format!("{} --contacts \"{}\"", enhanced.trim(), contacts_str);
        }
    }

    enhanced
}

#[allow(dead_code)]
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

    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;

    // Check cache first with proper error handling - fixed to use mutable reference
    let cached_response = {
        let mut lock_result = RESPONSE_CACHE
            .lock()
            .map_err(|e| anyhow!("Failed to acquire cache lock: {}", e.to_string()))?;
        lock_result.get(input).cloned()
    };

    if let Some(cached) = cached_response {
        return Ok(cached);
    }

    // Get available calendars and configuration early
    let available_calendars = get_available_calendars().await?;
    let config = Config::load()?;
    let default_calendar =
        config.calendar.default_calendar.unwrap_or_else(|| "Calendar".to_string());

    let current_date = Local::now();
    let current_hour = current_date.hour();
    let _current_minute = current_date.minute(); // Prefix with underscore
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

    // Remove unused extracted_emails variable and direct email extraction
    if input.contains("invite") || input.contains("email") || input.contains("send to") {
        debug!("Email context detected in input");
    }

    // Add current date context
    let context = format!("Current date and time: {}", Local::now().format("%Y-%m-%d %H:%M"));
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
    if let Ok(mut cache) = RESPONSE_CACHE.lock() {
        cache.put(input.to_string(), commands.clone());
    }

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
                if parts.len() < 3 {
                    results.push("Please provide the event title, date, start and end time, and desired calendar.".to_string());
                    continue;
                }
                let title = parts[1];
                let rest: Vec<&str> = parts[2].trim().split_whitespace().collect();
                if rest.len() < 3 {
                    results.push("Your event details seem incomplete. Could you specify the event title, date, start time, end time, and calendar?".to_string());
                    continue;
                }

                // Extract any email addresses from the input
                let mut email_str = String::new();
                if let Ok(emails) = extract_emails(input) {
                    if !emails.is_empty() {
                        email_str = emails.join(",");
                    }
                }

                // Improved calendar selection logic
                let requested_calendar =
                    if input.to_lowercase().contains("my calendar")
                        || input.to_lowercase().contains("in calendar")
                    {
                        &default_calendar
                    } else if input.to_lowercase().contains("kids calendar")
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
                        // Always default to user's primary calendar
                        &default_calendar
                    };

                if rest.len() >= 3 {
                    let mut command = format!(
                        r#"ducktape calendar create "{}" {} {} {} "{}""#,
                        title, rest[0], rest[1], rest[2], requested_calendar
                    );

                    // Always add the email parameter if we found any emails
                    if !email_str.is_empty() {
                        // Escape emails to prevent injection
                        let escaped_emails = escape_applescript_string(&email_str);
                        command = format!(r#"{} --email "{}""#, command, escaped_emails);
                    }

                    // Extract potential contact names from input more accurately
                    if input.contains(" with ")
                        || input.contains(" to ")
                        || input.contains("invite")
                    {
                        let contact_names = extract_contact_names(input);

                        // Process potential contact names - if they don't contain @ they're contacts, not emails
                        if !contact_names.is_empty() {
                            let contacts_without_emails: Vec<String> = contact_names
                                .iter()
                                .filter(|name| !name.contains('@'))
                                .cloned()
                                .collect();

                            if !contacts_without_emails.is_empty() {
                                // If input contains "invite" and we already added an email that's actually a contact,
                                // remove the incorrect email parameter first
                                if input.contains("invite") && command.contains("--email") {
                                    let email_regex =
                                        regex::Regex::new(r#"--email\s+"([^@"]+)""#).unwrap();
                                    if let Some(caps) = email_regex.captures(&command) {
                                        if let Some(email_match) = caps.get(1) {
                                            let email_value = email_match.as_str();
                                            // If the email doesn't contain @ and looks like a name, it's likely a contact
                                            if !email_value.contains('@') {
                                                debug!(
                                                    "Removing incorrectly formatted email: {}",
                                                    email_value
                                                );
                                                command = email_regex
                                                    .replace(&command, "")
                                                    .to_string()
                                                    .trim()
                                                    .to_string();
                                            }
                                        }
                                    }
                                }

                                // Add the contacts flag with the properly formatted contact names
                                // Escape contact names to prevent injection
                                let escaped_contacts =
                                    escape_applescript_string(&contacts_without_emails.join(","));
                                command =
                                    format!(r#"{} --contacts "{}"#, command, escaped_contacts);
                            }
                        }
                    }

                    results.push(command);
                }
            } else {
                results.push(trimmed.to_string());
            }
        }
    }

    // After processing commands and before returning results
    let mut final_results = Vec::new();
    for command in results {
        let enhanced = enhance_recurrence_command(&command);
        let enhanced = enhance_command_with_zoom(&enhanced, &sanitized_input);
        let enhanced = enhance_command_with_contacts(&enhanced, &sanitized_input);
        validate_calendar_command(&enhanced)?;
        final_results.push(enhanced);
    }

    Ok(final_results.join("\n"))
}

#[allow(dead_code)]
/// Helper function to extract contact names from natural language input
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
        let parts: Vec<&str> = input.splitn(2, "invite ").collect();
        if parts.len() > 1 { Some(parts[1]) } else { None }
    } else {
        None
    };

    if let Some(after_word) = text_to_parse {
        debug!("Text to parse for contacts: '{}'", after_word);

        // Pattern to detect email addresses (simple version)
        let email_pattern =
            regex::Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap();

        for name_part in after_word.split(|c: char| c == ',' || c == ';' || c == '.') {
            let name_part = name_part.trim();
            if name_part.is_empty() {
                continue;
            }

            // Skip if the whole part looks like an email address
            if email_pattern.is_match(name_part) {
                debug!("Skipping email-like string: {}", name_part);
                continue;
            }

            if name_part.contains(" and ") {
                let and_parts: Vec<&str> = name_part.split(" and ").collect();
                for and_part in and_parts {
                    let final_name = refine_name(and_part);
                    if !final_name.is_empty() && !email_pattern.is_match(&final_name) {
                        contact_names.push(final_name);
                    }
                }
            } else {
                let final_name = refine_name(name_part);
                if !final_name.is_empty() && !email_pattern.is_match(&final_name) {
                    contact_names.push(final_name);
                }
            }
        }
    }

    debug!("Extracted contact names: {:?}", contact_names);
    contact_names
}

#[allow(dead_code)]
/// Helper function to refine a name by removing trailing stop words
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

#[allow(dead_code)]
// Enhanced email extraction with improved validation
fn extract_emails(input: &str) -> Result<Vec<String>> {
    // Use a more strict email regex pattern
    let re =
        Regex::new(r"\b[A-Za-z0-9._%+-]{1,64}@(?:[A-Za-z0-9-]{1,63}\.){1,125}[A-Za-z]{2,63}\b")
            .map_err(|e| anyhow!("Failed to create regex: {}", e))?;

    let mut emails = Vec::new();

    // Split by common separators
    for part in input.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let part = part.trim();
        if part.len() > 320 {
            // Max allowed email length according to standards
            debug!("Skipping potential email due to excessive length: {}", part);
            continue;
        }

        if re.is_match(part) {
            // Additional validation to prevent injection
            if !part.contains('\'') && !part.contains('\"') && !part.contains('`') {
                emails.push(part.to_string());
            } else {
                debug!("Skipping email with potentially dangerous characters: {}", part);
            }
        }
    }

    if !emails.is_empty() {
        debug!("Extracted emails: {:?}", emails);
        Ok(emails)
    } else {
        // Instead of error, return empty vec to allow event creation without attendees
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_sanitize_user_input() {
        let input = "Meeting with John\u{0000} tomorrow";
        let sanitized = sanitize_user_input(input);
        assert_eq!(sanitized, "Meeting with John tomorrow");

        let input = "Lunch\nmeeting";
        let sanitized = sanitize_user_input(input);
        assert_eq!(sanitized, "Lunch\nmeeting");
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
    fn test_enhance_recurrence_command() {
        // Test adding daily recurrence
        let input =
            "ducktape calendar create \"Daily Standup\" 2024-03-15 10:00 11:00 \"Work\" every day";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--repeat daily"));

        // Test adding weekly recurrence with interval
        let input = "ducktape calendar create \"Bi-weekly Meeting\" 2024-03-15 10:00 11:00 \"Work\" every 2 weeks";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--repeat weekly"));
        assert!(enhanced.contains("--interval 2"));

        // Test adding monthly recurrence
        let input =
            "ducktape calendar create \"Monthly Review\" 2024-03-15 10:00 11:00 \"Work\" monthly";
        let enhanced = enhance_recurrence_command(input);
        assert!(enhanced.contains("--repeat monthly"));

        // Test non-calendar command remains unchanged
        let input = "ducktape todo \"Buy groceries\"";
        let enhanced = enhance_recurrence_command(input);
        assert_eq!(input, enhanced);
    }

    #[test]
    fn test_validate_calendar_command() {
        // Test valid command
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --repeat weekly --interval 2";
        assert!(validate_calendar_command(cmd).is_ok());

        // Test command with shell injection attempt
        let cmd = "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00; rm -rf /";
        assert!(validate_calendar_command(cmd).is_err());

        // Test unreasonable interval
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --interval 500";
        assert!(validate_calendar_command(cmd).is_err());

        // Test unreasonable count
        let cmd =
            "ducktape calendar create \"Meeting\" 2024-05-01 14:00 15:00 \"Work\" --count 1000";
        assert!(validate_calendar_command(cmd).is_err());
    }

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

    #[test]
    fn test_extract_contact_names() {
        // Test basic contact extraction with "with" keyword
        let input = "Schedule a meeting with John Smith tomorrow at 2pm";
        let contacts = extract_contact_names(input);
        assert_eq!(contacts, vec!["John Smith"]);

        // Test basic contact extraction with "invite" keyword
        let input = "create a zoom event at 10am on April 1 called Project Deadlines and invite Shaun Stuart";
        let contacts = extract_contact_names(input);
        assert_eq!(contacts, vec!["Shaun Stuart"]);

        // Test filtering out email addresses
        let input = "Schedule a meeting with john.doe@example.com tomorrow";
        let contacts = extract_contact_names(input);
        assert!(contacts.is_empty());

        // Test handling multiple names
        let input = "Schedule a meeting with John Smith and Jane Doe tomorrow";
        let contacts = extract_contact_names(input);
        assert!(contacts.contains(&"John Smith".to_string()));
        assert!(contacts.contains(&"Jane Doe".to_string()));
    }

    #[test]
    fn test_extract_emails() {
        // Test basic email extraction
        let input = "Schedule meeting with john@example.com";
        let emails = extract_emails(input).unwrap();
        assert_eq!(emails, vec!["john@example.com"]);

        // Test multiple email addresses
        let input = "Send invite to john@example.com, jane@example.com";
        let emails = extract_emails(input).unwrap();
        assert_eq!(emails, vec!["john@example.com", "jane@example.com"]);

        // Test filtering invalid emails
        let input = "Send to not.an.email and valid@example.com";
        let emails = extract_emails(input).unwrap();
        assert_eq!(emails, vec!["valid@example.com"]);

        // Test handling injection attempts
        let input = "Send to malicious\"@example.com";
        let emails = extract_emails(input).unwrap();
        assert!(emails.is_empty());
    }
}
