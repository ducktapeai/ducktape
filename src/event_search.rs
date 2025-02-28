use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug, error};
use chrono::Local;
use std::io::{self, Write};
use std::env;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSearchResult {
    pub title: String,
    pub date: String,  // YYYY-MM-DD format
    pub start_time: Option<String>, // HH:MM format
    pub end_time: Option<String>,   // HH:MM format
    pub location: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

impl EventSearchResult {
    pub fn display(&self) -> String {
        let mut output = format!("{} - {} ", self.title, self.date);
        
        if let Some(start) = &self.start_time {
            output.push_str(&format!("({})", start));
        }
        
        if let Some(location) = &self.location {
            output.push_str(&format!("\n   Location: {}", location));
        }
        
        if let Some(description) = &self.description {
            let desc = if description.len() > 100 {
                format!("{}...", &description[..100])
            } else {
                description.clone()
            };
            output.push_str(&format!("\n   Description: {}", desc));
        }
        
        if let Some(url) = &self.url {
            output.push_str(&format!("\n   URL: {}", url));
        }
        
        output
    }
}

/// Search for events on the internet using a search query
/// Returns a list of potential events found
pub async fn search_events(query: &str, calendar: Option<&str>) -> Result<()> {
    println!("🔎 Searching for events matching: \"{}\"", query);
    
    // Attempt to search for events with Grok's web search first
    info!("Searching for events with query: {}", query);
    
    let api_key_exists = std::env::var("XAI_API_KEY").is_ok();
    if !api_key_exists {
        println!("⚠️  XAI_API_KEY environment variable is not set. Using fallback data only.");
        println!("To enable internet search, set your Grok API key: export XAI_API_KEY='your-key'");
    }
    
    // Try real-time search with Grok first if API key exists
    if api_key_exists {
        match search_events_with_grok(query).await {
            Ok(events) if !events.is_empty() => {
                info!("Found {} events via Grok search", events.len());
                
                // Display the found events
                println!("\n✅ Found {} potential events:", events.len());
                for (i, event) in events.iter().enumerate() {
                    println!("{}. {}", i + 1, event.display());
                }
                
                // Let the user select an event
                print!("\nEnter the number of the event to add to calendar (or 0 to cancel): ");
                io::stdout().flush()?;
                
                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                
                let choice: usize = choice.trim().parse().unwrap_or(0);
                if choice == 0 || choice > events.len() {
                    println!("No event selected.");
                    return Ok(());
                }
                
                let selected_event = &events[choice - 1];
                let calendar_name = calendar.unwrap_or("Calendar");
                
                println!("📅 Adding event to calendar: {}", calendar_name);
                let command = event_to_calendar_command(selected_event, Some(calendar_name));
                println!("Converting to command: {}", command);
                
                // Execute the calendar command
                match create_calendar_event_from_command(&command).await {
                    Ok(_) => {
                        println!("✅ Successfully added event to calendar!");
                        Ok(())
                    },
                    Err(e) => {
                        println!("❌ Failed to add event: {}", e);
                        Err(e)
                    }
                }
            },
            
            Ok(_) => {
                info!("No events found via Grok web search");
                println!("No events found via internet search. Falling back to local data...");
                use_mock_event_data(query, calendar).await
            },
            
            Err(e) => {
                error!("Error searching for events with Grok: {}", e);
                
                // Check if it's an API key error
                if e.to_string().contains("XAI_API_KEY") {
                    println!("⚠️  Web search requires the XAI_API_KEY environment variable to be set.");
                    println!("Please set this variable with your Grok API key:");
                    println!("export XAI_API_KEY='your-api-key-here'");
                } else if e.to_string().contains("web_search is not enabled") {
                    println!("⚠️  Web search is not enabled for your Grok API key.");
                    println!("Please check your Grok account settings to enable web search capabilities.");
                } else {
                    println!("❌ Error while searching for events: {}", e);
                    println!("Falling back to local data...");
                }
                
                use_mock_event_data(query, calendar).await
            }
        }
    } else {
        // If API key doesn't exist, go straight to mock data
        use_mock_event_data(query, calendar).await
    }
}

/// Create a calendar event from a command string
pub async fn create_calendar_event_from_command(command: &str) -> Result<()> {
    // Parse the command manually instead of calling main::handle_command
    if !command.starts_with("ducktape calendar create") {
        return Err(anyhow!("Invalid calendar create command"));
    }
    
    // Extract parts of the command
    let parts: Vec<&str> = command.split('"')
        .enumerate()
        .map(|(i, part)| if i % 2 == 0 { part.trim() } else { part })
        .collect();
    
    if parts.len() < 3 {
        return Err(anyhow!("Invalid calendar create command format"));
    }
    
    // Extract title (it's in the first quoted part)
    let title = parts[1];
    
    // Extract the rest of the parameters
    let remaining: Vec<&str> = parts[2].trim().split_whitespace().collect();
    if remaining.len() < 3 {
        return Err(anyhow!("Missing required parameters in calendar command"));
    }
    
    let date = remaining[0];
    let start_time = remaining[1];
    let end_time = remaining[2];
    
    // Find the calendar name (it's in the next quoted part)
    // Fix: Create a calendar_name String variable to store the result 
    // of either branch, ensuring consistent types
    let calendar_name = if parts.len() >= 5 {
        parts[3].to_string() // Convert &str to String to match the else branch
    } else {
        // Use default calendar from config
        let config = crate::config::Config::load()?;
        config.calendar.default_calendar.clone().unwrap_or_else(|| "Calendar".to_string())
    };
    
    // Create the event config with calendar_name as borrowed str slice
    let mut config = crate::calendar::EventConfig::new(title, date, start_time);
    config.end_time = Some(end_time);
    config.calendars = vec![&calendar_name]; // Borrow String as &str
    
    // Extract location if present in command
    if command.contains("--location") {
        if let Some(start) = command.find("--location") {
            if let Some(loc_start) = command[start..].find('"') {
                if let Some(loc_end) = command[start + loc_start + 1..].find('"') {
                    let location = &command[start + loc_start + 1..start + loc_start + 1 + loc_end];
                    config.location = Some(location.to_string());
                }
            }
        }
    }
    
    // Extract notes/description if present in command
    if command.contains("--notes") {
        if let Some(start) = command.find("--notes") {
            if let Some(notes_start) = command[start..].find('"') {
                if let Some(notes_end) = command[start + notes_start + 1..].find('"') {
                    let notes = &command[start + notes_start + 1..start + notes_start + 1 + notes_end];
                    config.description = Some(notes.to_string());
                }
            }
        }
    }
    
    // Extract contacts if present in command
    if command.contains("--contacts") {
        if let Some(start) = command.find("--contacts") {
            if let Some(contacts_start) = command[start..].find('"') {
                if let Some(contacts_end) = command[start + contacts_start + 1..].find('"') {
                    let contacts = &command[start + contacts_start + 1..start + contacts_start + 1 + contacts_end];
                    let contact_names: Vec<&str> = contacts.split(',').map(|s| s.trim()).collect();
                    return crate::calendar::create_event_with_contacts(config, &contact_names);
                }
            }
        }
    }
    
    // Create the event
    crate::calendar::create_event(config)
}

/// Use Grok's capabilities to search the internet for events
async fn search_events_with_grok(query: &str) -> Result<Vec<EventSearchResult>> {
    let api_key = std::env::var("XAI_API_KEY")
        .map_err(|_| anyhow!("XAI_API_KEY environment variable not set. Please export XAI_API_KEY='your-key-here'"))?;
    
    let api_base = std::env::var("XAI_API_BASE")
        .unwrap_or_else(|_| "https://api.x.ai/v1".to_string());
    
    info!("Searching for events using Grok API: {}", query);
    println!("🔍 Searching the internet for events...");
    
    // Set to true to get more debugging information
    let debug_mode = std::env::var("DUCKTAPE_DEBUG").unwrap_or_default() == "true";
    
    let client = Client::new();
    let current_date = Local::now().format("%Y-%m-%d").to_string();
    
    // Enhanced prompt that specifically targets sports schedules and events
    let system_prompt = format!(
        r#"You are a sports and events research assistant that searches the web for upcoming events.
Current date: {}

Your task:
1. SEARCH THE WEB for real, upcoming events matching the user's query
2. Focus on finding OFFICIAL sources like team websites, league websites, and official ticket vendors
3. For sports events, find the NEXT GAME in the schedule for the requested team
4. Format each event with these details:
   - Title: Include both teams (for sports) or full event name
   - Date: Use YYYY-MM-DD format
   - Start time: Use 24-hour HH:MM format
   - End time: Estimate if not available (sports games usually last 2-3 hours)
   - Location: Include the venue name, city, and state/country
   - Description: Include details about the matchup, event significance, etc.
   - URL: Link to official event page, team schedule, or ticket vendor

YOU MUST SEARCH THE WEB for this information. DO NOT invent events or use placeholder data.
If you're not 100% confident the data is accurate, return an empty array.

Format your response ONLY as a JSON array:
[
  {{
    "title": "Team A vs Team B",
    "date": "2025-03-15",
    "start_time": "19:30",
    "end_time": "22:00",
    "location": "Stadium Name, City, State/Country",
    "description": "Accurate description of this specific matchup",
    "url": "https://official-team-website.com/schedule"
  }}
]"#,
        current_date
    );
    
    // Specifically craft the search query for real-time sports data
    let search_prompt = format!(
        "Search for {}'s next game or match. I need the EXACT date, time, opponent, and venue from the OFFICIAL website or reliable sports data provider. This is for a calendar entry so accuracy is crucial.", 
        query
    );
    
    debug!("Sending Grok API request with system prompt: {}", system_prompt);
    debug!("User prompt: {}", search_prompt);
    
    // Set debug environment variable to see API requests and responses
    if debug_mode {
        println!("System prompt: {}", system_prompt);
        println!("User prompt: {}", search_prompt);
    }
    
    // Prepare request payload with explicit web search enabled
    let request_payload = json!({
        "model": "grok-2-latest",
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": search_prompt
            }
        ],
        "temperature": 0.1,     // Low temperature for factual responses
        "max_tokens": 2000,     // Increased token limit
        "web_search": true,     // Explicitly enable web search
        "search_priority": 0.95 // Prioritize search results highly
    });
    
    if debug_mode {
        println!("Request payload: {}", serde_json::to_string_pretty(&request_payload).unwrap_or_default());
    }
    
    let response = client
        .post(format!("{}/chat/completions", api_base))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_payload)
        .send()
        .await?;
    
    let status = response.status();
    let response_text = response.text().await?;
    
    // Log the full response in debug mode
    if debug_mode {
        println!("API Response Status: {}", status);
        println!("API Response Body: {}", response_text);
    }
    
    if !status.is_success() {
        if response_text.contains("not enabled") && response_text.contains("web_search") {
            return Err(anyhow!(
                "Grok API web_search is not enabled for your API key. Please check your Grok account settings."
            ));
        }
        
        return Err(anyhow!(
            "Grok API error: Status {}, Response: {}",
            status,
            response_text
        ));
    }
    
    // Parse the response
    debug!("Received Grok API response: {}", response_text);
    
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| anyhow!("Failed to parse Grok response: {}", e))?;
    
    // Check if response contains a warning about web search
    if let Some(warnings) = response_json.get("warnings") {
        if warnings.is_array() && !warnings.as_array().unwrap().is_empty() {
            for warning in warnings.as_array().unwrap() {
                if let Some(warning_str) = warning.as_str() {
                    if warning_str.contains("web_search") {
                        info!("Grok warning: {}", warning_str);
                        println!("⚠️ Grok API warning: {}", warning_str);
                    }
                }
            }
        }
    }
    
    // Extract the content from the response
    let content = match response_json["choices"][0]["message"]["content"].as_str() {
        Some(content) => content,
        None => {
            // If content is missing, log the structure of the response
            if debug_mode {
                println!("Response JSON structure: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());
            }
            return Err(anyhow!("Invalid response format: content field not found in response"));
        }
    };
    
    debug!("Extracted content from Grok response: {}", content);
    
    // Try different extraction methods to find the JSON data
    let json_content = extract_json_from_text(content)?;
    
    if json_content.is_empty() {
        // If we couldn't extract JSON, log the full content in debug mode
        if debug_mode {
            println!("Could not extract JSON from response content: {}", content);
        }
        info!("No JSON events found in Grok response");
        return Ok(Vec::new());
    }
    
    debug!("Extracted JSON content: {}", json_content);
    
    // Parse the JSON into our event structure
    match serde_json::from_str::<Vec<EventSearchResult>>(&json_content) {
        Ok(events) => {
            info!("Successfully parsed {} events from Grok response", events.len());
            Ok(events)
        },
        Err(e) => {
            // If parsing fails, try to fix common JSON issues and retry
            error!("Failed to parse events: {}", e);
            if debug_mode {
                println!("JSON parse error: {}. Raw JSON: {}", e, json_content);
            }
            
            // Try to clean up the JSON and parse again
            let cleaned_json = clean_json(&json_content);
            match serde_json::from_str::<Vec<EventSearchResult>>(&cleaned_json) {
                Ok(events) => {
                    info!("Successfully parsed {} events from cleaned JSON", events.len());
                    Ok(events)
                },
                Err(e2) => {
                    error!("Failed to parse events after cleaning: {}", e2);
                    if debug_mode {
                        println!("JSON parse error after cleaning: {}. Cleaned JSON: {}", e2, cleaned_json);
                    }
                    
                    // As a final fallback, try to manually extract key information
                    if let Some(event) = manually_extract_event_info(content) {
                        info!("Manually extracted event info");
                        Ok(vec![event])
                    } else {
                        Err(anyhow!("Failed to parse events: {}", e))
                    }
                }
            }
        }
    }
}

/// Helper function to use mock data when internet search fails
async fn use_mock_event_data(query: &str, calendar: Option<&str>) -> Result<()> {
    // Fallback to mock data
    let mock_events = create_mock_events(query);
    if !mock_events.is_empty() {
        println!("\n🔍 Found {} events (mock data):", mock_events.len());
        for (i, event) in mock_events.iter().enumerate() {
            println!("{}. {}", i + 1, event.display());
        }
        
        print!("\nEnter the number of the event to add to calendar (or 0 to cancel): ");
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        let choice: usize = choice.trim().parse().unwrap_or(0);
        if choice == 0 || choice > mock_events.len() {
            println!("No event selected.");
            return Ok(());
        }
        
        let selected_event = &mock_events[choice - 1];
        let calendar_name = calendar.unwrap_or("Calendar");
        
        println!("📅 Adding event to calendar: {}", calendar_name);
        let command = event_to_calendar_command(selected_event, Some(calendar_name));
        println!("Converting to command: {}", command);
        
        return create_calendar_event_from_command(&command).await;
    }
    
    println!("No events found matching your query. Please try a more specific search term.");
    Ok(())
}

/// Create mock events for demonstration and fallback purposes
fn create_mock_events(query: &str) -> Vec<EventSearchResult> {
    let mut events = Vec::new();
    let query_lower = query.to_lowercase();
    let current_date = Local::now();
    
    // Specific mock data for rugby teams
    if query_lower.contains("springbok") || query_lower.contains("all black") || query_lower.contains("rugby") {
        events.push(EventSearchResult {
            title: "South Africa Springboks vs New Zealand All Blacks".to_string(),
            date: (current_date + chrono::Duration::days(60)).format("%Y-%m-%d").to_string(),
            start_time: Some("17:45".to_string()),
            end_time: Some("19:45".to_string()),
            location: Some("Ellis Park Stadium, Johannesburg, South Africa".to_string()),
            description: Some("Rugby Championship match between South Africa and New Zealand".to_string()),
            url: Some("https://www.sarugby.co.za/fixtures".to_string()),
        });
        
        events.push(EventSearchResult {
            title: "New Zealand All Blacks vs South Africa Springboks".to_string(),
            date: (current_date + chrono::Duration::days(67)).format("%Y-%m-%d").to_string(),
            start_time: Some("19:05".to_string()),
            end_time: Some("21:05".to_string()),
            location: Some("Eden Park, Auckland, New Zealand".to_string()),
            description: Some("Rugby Championship return match between New Zealand and South Africa".to_string()),
            url: Some("https://www.allblacks.com/fixtures".to_string()),
        });
        
        if events.len() > 0 {
            return events;
        }
    }
    
    // If "lakers" is in the query, add Lakers games
    if query_lower.contains("lakers") || query_lower.contains("basketball") || query_lower.contains("nba") {
        // Add a few upcoming Lakers games
        events.push(EventSearchResult {
            title: "Los Angeles Lakers vs Golden State Warriors".to_string(),
            date: (current_date + chrono::Duration::days(3)).format("%Y-%m-%d").to_string(),
            start_time: Some("19:30".to_string()),
            end_time: Some("22:00".to_string()),
            location: Some("Crypto.com Arena, Los Angeles, CA".to_string()),
            description: Some("Regular season NBA game between the Los Angeles Lakers and Golden State Warriors".to_string()),
            url: Some("https://www.nba.com/lakers/schedule".to_string()),
        });
        
        events.push(EventSearchResult {
            title: "Los Angeles Lakers vs Boston Celtics".to_string(),
            date: (current_date + chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
            start_time: Some("20:00".to_string()),
            end_time: Some("22:30".to_string()),
            location: Some("Crypto.com Arena, Los Angeles, CA".to_string()),
            description: Some("Regular season NBA matchup between historic rivals".to_string()),
            url: Some("https://www.nba.com/lakers/schedule".to_string()),
        });
        
        return events;
    }
    
    // If NFL or football related terms are in the query
    if query_lower.contains("nfl") || query_lower.contains("football") || query_lower.contains("49ers") || 
       query_lower.contains("chiefs") || query_lower.contains("ravens") {
        
        events.push(EventSearchResult {
            title: "Kansas City Chiefs vs San Francisco 49ers".to_string(),
            date: (current_date + chrono::Duration::days(21)).format("%Y-%m-%d").to_string(),
            start_time: Some("16:30".to_string()),
            end_time: Some("20:00".to_string()),
            location: Some("Arrowhead Stadium, Kansas City, MO".to_string()),
            description: Some("NFL regular season game between the Chiefs and 49ers".to_string()),
            url: Some("https://www.nfl.com/schedules/".to_string()),
        });
        
        events.push(EventSearchResult {
            title: "Baltimore Ravens vs Cincinnati Bengals".to_string(),
            date: (current_date + chrono::Duration::days(14)).format("%Y-%m-%d").to_string(),
            start_time: Some("13:00".to_string()),
            end_time: Some("16:30".to_string()),
            location: Some("M&T Bank Stadium, Baltimore, MD".to_string()),
            description: Some("AFC North division rivalry game between the Ravens and Bengals".to_string()),
            url: Some("https://www.nfl.com/schedules/".to_string()),
        });
        
        return events;
    }
    
    // If "concert" is in the query, add concert events
    if query_lower.contains("concert") || query_lower.contains("music") || 
       query_lower.contains("tour") || query_lower.contains("festival") {
        
        let artists = ["Taylor Swift", "Ed Sheeran", "Beyoncé", "The Weeknd", "Bad Bunny"];
        let venues = ["Madison Square Garden", "Staples Center", "Wembley Stadium", "O2 Arena", "Coachella Valley"];
        
        // Find matching artist if any
        let mut artist = "";
        for a in artists {
            if query_lower.contains(&a.to_lowercase()) {
                artist = a;
                break;
            }
        }
        
        if artist.is_empty() {
            // If no specific artist mentioned, use the query or default
            if query_lower.contains("concert") {
                artist = match query_lower.split_whitespace().next() {
                    Some(first_word) if first_word != "concert" => first_word,
                    _ => "Live Music",
                };
            } else {
                artist = "Concert";
            }
        }
        
        events.push(EventSearchResult {
            title: format!("{} World Tour", artist),
            date: (current_date + chrono::Duration::days(14)).format("%Y-%m-%d").to_string(),
            start_time: Some("20:00".to_string()),
            end_time: Some("23:00".to_string()),
            location: Some(format!("{}, New York, NY", venues[0])),
            description: Some(format!("Live music event featuring {} performing their latest hits", artist)),
            url: Some("https://www.ticketmaster.com".to_string()),
        });
        
        events.push(EventSearchResult {
            title: format!("{} Concert Tour", artist),
            date: (current_date + chrono::Duration::days(21)).format("%Y-%m-%d").to_string(),
            start_time: Some("19:30".to_string()),
            end_time: Some("22:30".to_string()),
            location: Some(format!("{}, Los Angeles, CA", venues[1])),
            description: Some(format!("{} performing live with special guests", artist)),
            url: Some("https://www.ticketmaster.com".to_string()),
        });
        
        return events;
    }
    
    // Generic fallback for any other query
    if events.is_empty() {
        events.push(EventSearchResult {
            title: format!("Event: {}", query),
            date: (current_date + chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
            start_time: Some("18:00".to_string()),
            end_time: Some("20:00".to_string()),
            location: Some("Event Venue".to_string()),
            description: Some(format!("Event related to '{}'", query)),
            url: None,
        });
    }
    
    events
}

/// Clean up JSON to fix common issues
fn clean_json(json_str: &str) -> String {
    let mut result = json_str.to_string();
    
    // Replace escaped quotes with actual quotes
    result = result.replace("\\\"", "\"");
    
    // Ensure the string starts with [ and ends with ]
    if !result.trim().starts_with('[') {
        result = format!("[{}", result);
    }
    
    if !result.trim().ends_with(']') {
        result = format!("{}]", result);
    }
    
    result
}

/// Attempt to manually extract event information from text
fn manually_extract_event_info(text: &str) -> Option<EventSearchResult> {
    // Look for patterns like dates (YYYY-MM-DD)
    let date_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}").ok()?;
    let time_pattern = Regex::new(r"\d{1,2}:\d{2}").ok()?;
    
    let date = date_pattern.find(text)?.as_str().to_string();
    let start_time = time_pattern.find(text)?.as_str().to_string();
    
    // Try to find a title
    let title = if text.contains("vs") {
        let parts: Vec<&str> = text.split("vs").collect();
        if parts.len() > 1 {
            let before = parts[0].trim().split_whitespace().take(4).collect::<Vec<_>>().join(" ");
            let after = parts[1].trim().split_whitespace().take(4).collect::<Vec<_>>().join(" ");
            format!("{} vs {}", before, after)
        } else {
            "Event".to_string()
        }
    } else {
        "Event".to_string()
    };
    
    // Extract location if possible
    let location = if let Some(loc_start) = text.find("at ") {
        let loc_text = &text[loc_start + 3..];
        let loc_end = loc_text.find('.').unwrap_or_else(|| loc_text.find(',').unwrap_or(40));
        Some(loc_text[..loc_end.min(40)].trim().to_string())
    } else {
        None
    };
    
    // Fix the borrow of moved value by cloning start_time
    let start_time_clone = start_time.clone();
    let end_hour = start_time.split(':').next().unwrap_or("19").parse::<i32>().unwrap_or(19) + 2;
    
    Some(EventSearchResult {
        title,
        date,
        start_time: Some(start_time_clone),
        end_time: Some(format!("{}:00", end_hour)),
        location,
        description: Some(text.trim().chars().take(200).collect::<String>()),
        url: None
    })
}

/// Helper function to extract JSON from text that might contain markdown and other content
fn extract_json_from_text(text: &str) -> Result<String> {
    // Look for JSON array between ```json and ``` markers
    if let Some(start_idx) = text.find("```json") {
        if let Some(end_idx) = text[start_idx + 7..].find("```") {
            return Ok(text[start_idx + 7..start_idx + 7 + end_idx].trim().to_string());
        }
    }
    
    // Look for JSON array between ``` and ``` markers
    if let Some(start_idx) = text.find("```") {
        if let Some(end_idx) = text[start_idx + 3..].find("```") {
            let code_block = text[start_idx + 3..start_idx + 3 + end_idx].trim();
            if (code_block.starts_with('[') && code_block.ends_with(']')) || (code_block.starts_with('{') && code_block.ends_with('}')) {
                return Ok(code_block.to_string());
            }
        }
    }
    
    // Look for plain JSON array
    if let Some(start_idx) = text.find('[') {
        if let Some(end_idx) = text[start_idx..].rfind(']') {
            return Ok(text[start_idx..start_idx + end_idx + 1].to_string());
        }
    }
    
    // Check for single JSON object and wrap in array
    if let Some(start_idx) = text.find('{') {
        if let Some(end_idx) = text[start_idx..].rfind('}') {
            let obj = text[start_idx..start_idx + end_idx + 1].to_string();
            return Ok(format!("[{}]", obj));
        }
    }
    
    // If no JSON found, return an empty array
    Ok("[]".to_string())
}

/// Convert a search result into a calendar event command
pub fn event_to_calendar_command(event: &EventSearchResult, calendar: Option<&str>) -> String {
    // Get config and use default calendar if no calendar is specified
    let config = match crate::config::Config::load() {
        Ok(config) => config,
        Err(_) => return format_command(event, calendar.unwrap_or("Calendar")) // Fallback if config can't be loaded
    };

    // Only use the provided calendar if it was explicitly specified, otherwise use default
    let calendar_name = if let Some(cal) = calendar {
        cal
    } else {
        // Use the default calendar from config if available
        config.calendar.default_calendar.as_deref().unwrap_or("Calendar")
    };
    
    format_command(event, calendar_name)
}

/// Helper function to format the calendar command
fn format_command(event: &EventSearchResult, calendar_name: &str) -> String {
    let mut command = format!(
        "ducktape calendar create \"{}\" {} {} {}",
        event.title,
        event.date,
        event.start_time.as_deref().unwrap_or("12:00"),
        event.end_time.as_deref().unwrap_or("13:00")
    );
    
    // Add calendar
    command.push_str(&format!(" \"{}\"", calendar_name));
    
    // Add location if available
    if let Some(location) = &event.location {
        command.push_str(&format!(" --location \"{}\"", location));
    }
    
    // Add notes with description and URL if available
    let mut notes = String::new();
    
    if let Some(desc) = &event.description {
        notes.push_str(desc);
    }
    
    if let Some(url) = &event.url {
        if !notes.is_empty() {
            notes.push_str("\n\n");
        }
        notes.push_str(&format!("Event URL: {}", url));
    }
    
    if !notes.is_empty() {
        command.push_str(&format!(" --notes \"{}\"", notes));
    }
    
    command
}