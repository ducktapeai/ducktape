use anyhow::{anyhow, Result};
use log::{debug, error, info};
use reqwest::Client;
use secrecy::{Secret, ExposeSecret};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

// Constants for Zoom API
const ZOOM_API_BASE: &str = "https://api.zoom.us/v2";

#[derive(Debug, Clone)]
pub struct ZoomCredentials {
    pub account_id: Secret<String>,
    pub client_id: Secret<String>,
    pub client_secret: Secret<String>,
    access_token: Option<Secret<String>>,
}

impl ZoomCredentials {
    pub fn new() -> Result<Self> {
        let account_id = env::var("ZOOM_ACCOUNT_ID")
            .map(Secret::new)
            .map_err(|_| anyhow!("ZOOM_ACCOUNT_ID not found in environment"))?;
        
        let client_id = env::var("ZOOM_CLIENT_ID")
            .map(Secret::new)
            .map_err(|_| anyhow!("ZOOM_CLIENT_ID not found in environment"))?;
        
        let client_secret = env::var("ZOOM_CLIENT_SECRET")
            .map(Secret::new)
            .map_err(|_| anyhow!("ZOOM_CLIENT_SECRET not found in environment"))?;

        Ok(Self {
            account_id,
            client_id,
            client_secret,
            access_token: None,
        })
    }

    #[allow(dead_code)]
    pub fn with_credentials(account_id: String, client_id: String, client_secret: String) -> Self {
        Self {
            account_id: Secret::new(account_id),
            client_id: Secret::new(client_id),
            client_secret: Secret::new(client_secret),
            access_token: None,
        }
    }

    async fn get_access_token(&mut self) -> Result<String> {
        // If we already have a token, return it
        if let Some(token) = &self.access_token {
            return Ok(token.expose_secret().clone());
        }

        let client = Client::new();
        let token_url = "https://zoom.us/oauth/token";
        
        // Log the request for debugging
        debug!("Requesting Zoom OAuth token with account_id: {}", self.account_id.expose_secret());
        
        let response = client
            .post(token_url)
            .basic_auth(
                self.client_id.expose_secret(),
                Some(self.client_secret.expose_secret())
            )
            .form(&[
                ("grant_type", "account_credentials"),
                ("account_id", self.account_id.expose_secret()),
            ])
            .send()
            .await?;

        // Check for errors and provide more detailed error messages
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unable to get error response".into());
            
            // Log the full error for debugging
            error!("Zoom OAuth error response: {}", error_text);
            
            // Check for common error conditions
            let error_message = if error_text.contains("invalid_client") {
                "Invalid Zoom credentials. Please verify your Account ID, Client ID and Client Secret are correct and the app is enabled in Zoom Marketplace."
            } else if error_text.contains("invalid_scope") {
                "The app does not have the required permissions. Please update the app in Zoom Marketplace to include meeting:write and meeting:write:admin scopes."
            } else {
                &error_text
            };
            
            return Err(anyhow!("Zoom OAuth error ({}): {}", status, error_message));
        }

        let response_text = response.text().await?;
        debug!("Zoom OAuth response: {}", response_text);

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            #[allow(dead_code)]
            token_type: String,
            #[allow(dead_code)]
            expires_in: u64,
        }

        let token_data: TokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse OAuth response: {} - Response was: {}", e, response_text))?;

        // Store and return the token
        self.access_token = Some(Secret::new(token_data.access_token.clone()));
        Ok(token_data.access_token)
    }
}

#[derive(Debug, Serialize)]
pub struct ZoomMeetingOptions {
    pub topic: String,
    pub start_time: String,
    pub duration: u32,
    pub password: Option<String>,
    pub agenda: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ZoomMeetingResponse {
    pub id: u64,
    #[allow(dead_code)]
    pub host_id: String,
    #[allow(dead_code)]
    pub host_email: Option<String>,
    #[allow(dead_code)]
    pub topic: String,
    #[allow(dead_code)]
    pub start_time: String,
    #[allow(dead_code)]
    pub duration: u32,
    pub join_url: String,
    pub password: Option<String>,
}

pub struct ZoomClient {
    credentials: ZoomCredentials,
    client: Client,
}

impl ZoomClient {
    // Create a new Zoom client
    pub fn new() -> Result<Self> {
        let credentials = ZoomCredentials::new()?;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { credentials, client })
    }

    // Create a Zoom meeting
    pub async fn create_meeting(&mut self, options: ZoomMeetingOptions) -> Result<ZoomMeetingResponse> {
        debug!("Creating Zoom meeting with topic: {}", options.topic);
        
        // Get access token
        let token = self.credentials.get_access_token().await?;
        
        // Sanitize input data
        let sanitized_topic = sanitize_zoom_field(&options.topic, 200);
        let sanitized_agenda = options.agenda.as_deref().map(|a| sanitize_zoom_field(a, 2000));
        
        // Construct request body
        let body = serde_json::json!({
            "topic": sanitized_topic,
            "type": 2, // Scheduled meeting
            "start_time": options.start_time,
            "duration": options.duration,
            "password": options.password,
            "agenda": sanitized_agenda,
            "settings": {
                "join_before_host": true,
                "waiting_room": false,
                "host_video": true,
                "participant_video": true,
                "mute_upon_entry": false,
                "auto_recording": "none",
            }
        });

        // Make the API call
        let url = format!("{}/users/me/meetings", ZOOM_API_BASE);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send Zoom API request: {}", e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unable to get error response".into());
            error!("Zoom API error: {} - {}", status, error_text);
            return Err(anyhow!("Zoom API error ({}): {}", status, error_text));
        }

        // Parse response
        let meeting: ZoomMeetingResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse Zoom API response: {}", e))?;

        info!("Successfully created Zoom meeting: {}", meeting.id);
        Ok(meeting)
    }

    #[allow(dead_code)]
    pub async fn delete_meeting(&mut self, meeting_id: u64) -> Result<()> {
        debug!("Deleting Zoom meeting: {}", meeting_id);
        
        // Get access token
        let token = self.credentials.get_access_token().await?;
        
        // Make the API call
        let url = format!("{}/meetings/{}", ZOOM_API_BASE, meeting_id);
        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send Zoom API request: {}", e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unable to get error response".into());
            error!("Zoom API error: {} - {}", status, error_text);
            return Err(anyhow!("Zoom API error ({}): {}", status, error_text));
        }

        info!("Successfully deleted Zoom meeting: {}", meeting_id);
        Ok(())
    }

    #[allow(dead_code)]
    async fn make_request(&mut self, endpoint: &str, method: &str, body: Option<&str>) -> Result<String> {
        let token = self.credentials.get_access_token().await?;
        let url = format!("{}{}", ZOOM_API_BASE, endpoint);
        
        let mut request = self.client
            .request(reqwest::Method::from_str(method)?, &url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json");
            
        if let Some(body_str) = body {
            request = request.body(body_str.to_string());
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unable to get error response".into());
            return Err(anyhow!("API error ({}): {}", status, error_text));
        }
        
        response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))
    }
}

// Utility function to sanitize inputs to Zoom API
fn sanitize_zoom_field(input: &str, max_length: usize) -> String {
    let filtered: String = input
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
        .trim()
        .chars()
        .take(max_length)
        .collect();
    
    filtered
}

// Helper function to convert calendar date/time to Zoom format
pub fn format_zoom_time(date: &str, time: &str) -> Result<String> {
    // Parse the date and time
    let dt = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| anyhow!("Invalid date format"))?
        .and_time(
            chrono::NaiveTime::parse_from_str(time, "%H:%M")
                .map_err(|_| anyhow!("Invalid time format"))?
        );
    
    // Format for Zoom API: "2023-10-24T14:30:00Z"
    Ok(dt.format("%Y-%m-%dT%H:%M:00Z").to_string())
}

// Helper function to calculate meeting duration in minutes
pub fn calculate_meeting_duration(start_time: &str, end_time: &str) -> Result<u32> {
    let start = chrono::NaiveTime::parse_from_str(start_time, "%H:%M")
        .map_err(|_| anyhow!("Invalid start time format"))?;
    
    let end = chrono::NaiveTime::parse_from_str(end_time, "%H:%M")
        .map_err(|_| anyhow!("Invalid end time format"))?;
    
    // Calculate duration in minutes
    let duration_minutes = if end > start {
        (end - start).num_minutes() as u32
    } else {
        // If end time is earlier than start time, assume it's the next day
        (end.signed_duration_since(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()) + 
         chrono::NaiveTime::from_hms_opt(24, 0, 0).unwrap().signed_duration_since(start)).num_minutes() as u32
    };
    
    // Minimum duration is 15 minutes
    Ok(std::cmp::max(duration_minutes, 15))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_zoom_time() {
        let result = format_zoom_time("2023-12-25", "14:30").unwrap();
        assert_eq!(result, "2023-12-25T14:30:00Z");
    }

    #[test]
    fn test_calculate_meeting_duration() {
        // Test normal duration
        let result = calculate_meeting_duration("14:30", "15:45").unwrap();
        assert_eq!(result, 75); // 1h15m = 75 minutes

        // Test short duration gets minimum duration
        let result = calculate_meeting_duration("14:30", "14:40").unwrap();
        assert_eq!(result, 15); // Should use minimum 15 minutes
    }
    
    #[test]
    fn test_sanitize_zoom_field() {
        // Test normal input
        let result = sanitize_zoom_field("Test Meeting", 100);
        assert_eq!(result, "Test Meeting");
        
        // Test trimming
        let result = sanitize_zoom_field("  Test Meeting  ", 100);
        assert_eq!(result, "Test Meeting");
        
        // Test control character removal
        let input = "Test\u{007F}Meeting\u{0000}";
        let result = sanitize_zoom_field(input, 100);
        assert_eq!(result, "TestMeeting");
        
        // Test max length - should truncate cleanly at word boundary
        let input = "This is a very long meeting name that exceeds the limit";
        let result = sanitize_zoom_field(input, 19);
        assert_eq!(result.len(), 19);
        assert_eq!(result, "This is a very long");
    }
}