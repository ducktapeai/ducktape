use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug, error};
use chrono::{Datelike, Local};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSearchResult {
    pub title: String,
    pub date: String,  // YYYY-MM-DD format
    pub start_time: Option<String>, // HH:MM format
    pub end_time: Option<String>,   // HH:MM format
    pub location: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

/// Search for events on the internet using a search query
/// Returns a list of potential events found
pub async fn search_events(query: &str) -> Result<Vec<EventSearchResult>> {
    info!("Searching for events with query: {}", query);
    
    // Try to use Grok for online search first
    match search_events_with_grok(query).await {
        Ok(events) if !events.is_empty() => {
            info!("Found {} events via Grok search", events.len());
            return Ok(events);
        }
        Ok(_) => info!("No events found via Grok search, falling back to mock data"),
        Err(e) => info!("Grok search failed: {}, falling back to mock data", e),
    }
    
    // Fallback to the existing mock implementation
    info!("Using fallback mock search implementation");
    
    // Convert query to lowercase for easier matching
    let query_lower = query.to_lowercase();
    
    // Check for sports-specific searches first
    if let Some(events) = search_rugby_events(&query_lower).await? {
        return Ok(events);
    }
    
    // For now, return mock results based on the query
    let mut events = Vec::new();
    
    if query_lower.contains("concert") {
        events.push(EventSearchResult {
            title: "Live Music Concert".to_string(),
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            start_time: Some("19:00".to_string()),
            end_time: Some("22:00".to_string()),
            location: Some("Music Hall".to_string()),
            description: Some("Live concert featuring local bands".to_string()),
            url: Some("https://example.com/concert".to_string()),
        });
    } else if query_lower.contains("conference") {
        events.push(EventSearchResult {
            title: "Tech Conference".to_string(),
            date: (chrono::Local::now() + chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
            start_time: Some("09:00".to_string()),
            end_time: Some("17:00".to_string()),
            location: Some("Convention Center".to_string()),
            description: Some("Annual technology conference with speakers and workshops".to_string()),
            url: Some("https://example.com/conference".to_string()),
        });
    } else if query_lower.contains("sports") || query_lower.contains("game") {
        events.push(EventSearchResult {
            title: "Local Sports Game".to_string(),
            date: (chrono::Local::now() + chrono::Duration::days(3)).format("%Y-%m-%d").to_string(),
            start_time: Some("15:00".to_string()),
            end_time: Some("17:30".to_string()),
            location: Some("City Stadium".to_string()),
            description: Some("Local teams competing in seasonal match".to_string()),
            url: Some("https://example.com/sports".to_string()),
        });
    }
    
    // Always add a generic event as fallback
    events.push(EventSearchResult {
        title: format!("Found Event: {}", query),
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        start_time: Some("12:00".to_string()),
        end_time: Some("13:00".to_string()),
        location: Some("Local Venue".to_string()),
        description: Some(format!("Event related to search: {}", query)),
        url: None,
    });
    
    info!("Found {} events matching search query", events.len());
    Ok(events)
}

/// Use Grok's capabilities to search the internet for events
async fn search_events_with_grok(query: &str) -> Result<Vec<EventSearchResult>> {
    let api_key = std::env::var("XAI_API_KEY")
        .map_err(|_| anyhow!("XAI_API_KEY environment variable not set"))?;
    
    let api_base = std::env::var("XAI_API_BASE")
        .unwrap_or_else(|_| "https://api.x.ai/v1".to_string());
    
    info!("Searching for events using Grok API: {}", query);
    
    let client = Client::new();
    let current_date = Local::now().format("%Y-%m-%d").to_string();
    
    // Build a prompt that explicitly tells Grok to search the web
    let system_prompt = format!(
        r#"You are a web search assistant that finds upcoming events based on user queries.
Current date: {}

Your task:
1. SEARCH THE WEB for real, upcoming events matching the user's query - this is crucial
2. Find the MOST RELEVANT events from official sources
3. For sports events, look for official league or team websites
4. Format each event with these details:
   - Title (include teams/performers and venue)
   - Date (YYYY-MM-DD format)
   - Start time (HH:MM 24-hour format)
   - End time (estimate if not available)
   - Location (full venue name and city)
   - Description (include teams/participants and context)
   - URL (official event page if available)

You MUST use your web search capability to find REAL events with ACTUAL dates.
DO NOT provide fictional or placeholder data.
If you can't find information with high confidence, return an empty array.

Format your response ONLY as a JSON array:
[
  {{
    "title": "Real Event Name",
    "date": "2025-03-15",
    "start_time": "19:30",
    "end_time": "22:00",
    "location": "Real Venue Name, City, Country",
    "description": "Accurate description of this specific event",
    "url": "https://real-website.com/event"
  }}
]

Respond ONLY with the JSON array. Do not include any explanatory text."#,
        current_date
    );
    
    // Create an explicit search query that forces web search
    let search_prompt = format!(
        "Search the web for the next time {} plays or performs. I need REAL upcoming events with accurate dates, times, and locations. Find the official schedule.", 
        query
    );
    
    debug!("Sending Grok API request with system prompt: {}", system_prompt);
    debug!("User prompt: {}", search_prompt);
    
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
                    "content": search_prompt
                }
            ],
            "temperature": 0.1,  // Lower temperature for more factual responses
            "max_tokens": 1500,  // Increased to allow for more complete responses
            "web_search": true    // Explicitly enable web search
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
    
    // Parse the response
    debug!("Received Grok API response: {}", response_text);
    
    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| anyhow!("Failed to parse Grok response: {}", e))?;
    
    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid response format"))?;
    
    debug!("Extracted content from Grok response: {}", content);
    
    // Extract the JSON part from the response
    let json_content = extract_json_from_text(content)?;
    
    if json_content.is_empty() {
        info!("No events found in Grok response");
        return Ok(Vec::new());
    }
    
    debug!("Extracted JSON content: {}", json_content);
    
    // Parse the JSON into our event structure
    let events: Vec<EventSearchResult> = serde_json::from_str(&json_content)
        .map_err(|e| anyhow!("Failed to parse events from response: {}, content: {}", e, json_content))?;
    
    info!("Successfully parsed {} events from Grok response", events.len());
    Ok(events)
}

/// Helper function to extract JSON from text that might contain markdown and other content
fn extract_json_from_text(text: &str) -> Result<String> {
    // Look for JSON array between ```json and ``` markers
    if let Some(start_idx) = text.find("```json") {
        if let Some(end_idx) = text[start_idx + 7..].find("```") {
            return Ok(text[start_idx + 7..start_idx + 7 + end_idx].trim().to_string());
        }
    }
    
    // Look for plain JSON array
    if let Some(start_idx) = text.find('[') {
        if let Some(end_idx) = text[start_idx..].rfind(']') {
            return Ok(text[start_idx..start_idx + end_idx + 1].to_string());
        }
    }
    
    // If no JSON found, return an empty array
    Ok("[]".to_string())
}

// Keep the existing rugby_events search as fallback
async fn search_rugby_events(query: &str) -> Result<Option<Vec<EventSearchResult>>> {
    // Define common rugby teams and tournaments for matching
    let rugby_keywords = [
        "rugby", "springboks", "all blacks", "wallabies", "six nations", 
        "world cup", "rugby championship"
    ];
    
    // Check if this is a rugby-related query
    let is_rugby_query = rugby_keywords.iter().any(|keyword| query.contains(keyword));
    
    if !is_rugby_query {
        return Ok(None);
    }
    
    info!("Detected rugby-related search query");
    let mut events = Vec::new();
    
    // Check for specific team matchups
    if query.contains("springboks") && query.contains("all blacks") {
        // Hardcoded test fixtures for Springboks vs All Blacks (2025)
        // In a real implementation, these would come from a sports API
        
        // These dates are made up for demo purposes
        let current_year = Local::now().year();
        
        events.push(EventSearchResult {
            title: format!("Rugby Championship: Springboks vs All Blacks"),
            date: format!("{}-09-06", current_year),
            start_time: Some("19:35".to_string()),
            end_time: Some("21:30".to_string()),
            location: Some("Ellis Park, Johannesburg, South Africa".to_string()),
            description: Some(format!(
                "Rugby Championship {}: South Africa vs New Zealand in the first of two matches.", 
                current_year
            )),
            url: Some("https://www.sarugby.co.za/fixtures".to_string()),
        });
        
        events.push(EventSearchResult {
            title: format!("Rugby Championship: All Blacks vs Springboks"),
            date: format!("{}-09-13", current_year),
            start_time: Some("19:05".to_string()),
            end_time: Some("21:00".to_string()),
            location: Some("Eden Park, Auckland, New Zealand".to_string()),
            description: Some(format!(
                "Rugby Championship {}: New Zealand vs South Africa in the second Test.", 
                current_year
            )),
            url: Some("https://www.allblacks.com/fixtures".to_string()),
        });
    } else if query.contains("springboks") {
        // General Springboks fixtures
        let current_year = Local::now().year();
        
        events.push(EventSearchResult {
            title: format!("Rugby Championship: Springboks vs Wallabies"),
            date: format!("{}-08-16", current_year),
            start_time: Some("17:00".to_string()),
            end_time: Some("19:00".to_string()),
            location: Some("Loftus Versfeld, Pretoria, South Africa".to_string()),
            description: Some(format!(
                "Rugby Championship {}: South Africa vs Australia", 
                current_year
            )),
            url: Some("https://www.sarugby.co.za/fixtures".to_string()),
        });
        
        events.push(EventSearchResult {
            title: format!("Rugby Championship: Argentina vs Springboks"),
            date: format!("{}-08-23", current_year),
            start_time: Some("20:10".to_string()),
            end_time: Some("22:00".to_string()),
            location: Some("Estadio Único Madre de Ciudades, Santiago del Estero, Argentina".to_string()),
            description: Some(format!(
                "Rugby Championship {}: Argentina vs South Africa", 
                current_year
            )),
            url: Some("https://www.sarugby.co.za/fixtures".to_string()),
        });
        
        // Add All Blacks matches too
        events.push(EventSearchResult {
            title: format!("Rugby Championship: Springboks vs All Blacks"),
            date: format!("{}-09-06", current_year),
            start_time: Some("19:35".to_string()),
            end_time: Some("21:30".to_string()),
            location: Some("Ellis Park, Johannesburg, South Africa".to_string()),
            description: Some(format!(
                "Rugby Championship {}: South Africa vs New Zealand in the first of two matches.", 
                current_year
            )),
            url: Some("https://www.sarugby.co.za/fixtures".to_string()),
        });
    } else if query.contains("all blacks") {
        // General All Blacks fixtures
        let current_year = Local::now().year();
        
        events.push(EventSearchResult {
            title: format!("Rugby Championship: All Blacks vs Argentina"),
            date: format!("{}-08-16", current_year),
            start_time: Some("19:05".to_string()),
            end_time: Some("21:00".to_string()),
            location: Some("Eden Park, Auckland, New Zealand".to_string()),
            description: Some(format!(
                "Rugby Championship {}: New Zealand vs Argentina", 
                current_year
            )),
            url: Some("https://www.allblacks.com/fixtures".to_string()),
        });
        
        events.push(EventSearchResult {
            title: format!("Rugby Championship: All Blacks vs Wallabies"),
            date: format!("{}-08-23", current_year),
            start_time: Some("19:05".to_string()),
            end_time: Some("21:00".to_string()),
            location: Some("Wellington Regional Stadium, Wellington, New Zealand".to_string()),
            description: Some(format!(
                "Rugby Championship {}: New Zealand vs Australia", 
                current_year
            )),
            url: Some("https://www.allblacks.com/fixtures".to_string()),
        });
        
        // Add Springboks matches too
        events.push(EventSearchResult {
            title: format!("Rugby Championship: All Blacks vs Springboks"),
            date: format!("{}-09-13", current_year),
            start_time: Some("19:05".to_string()),
            end_time: Some("21:00".to_string()),
            location: Some("Eden Park, Auckland, New Zealand".to_string()),
            description: Some(format!(
                "Rugby Championship {}: New Zealand vs South Africa in the second Test.", 
                current_year
            )),
            url: Some("https://www.allblacks.com/fixtures".to_string()),
        });
    }
    
    if events.is_empty() {
        // Generic rugby event if no specific matches found
        events.push(EventSearchResult {
            title: format!("Rugby Match: {}", query),
            date: (chrono::Local::now() + chrono::Duration::days(30)).format("%Y-%m-%d").to_string(),
            start_time: Some("19:00".to_string()),
            end_time: Some("21:00".to_string()),
            location: Some("National Stadium".to_string()),
            description: Some(format!("Rugby match related to your search: {}", query)),
            url: Some("https://www.worldrugby.org/fixtures".to_string()),
        });
    }
    
    info!("Found {} rugby events matching search query", events.len());
    Ok(Some(events))
}

/// Convert a search result into a calendar event command
pub fn event_to_calendar_command(event: &EventSearchResult, calendar: Option<&str>) -> String {
    // Get config and use default calendar if no calendar is specified
    let config = match crate::config::Config::load() {
        Ok(config) => config,
        Err(_) => return format_command(event, calendar.unwrap_or("Calendar")) // Fallback if config can't be loaded
    };

    // Only use the provided calendar if it was explicitly specified, otherwise use default
    let calendar_name = if calendar.is_some() && calendar != Some("National Rugby League") {
        calendar.unwrap_or("Calendar")
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