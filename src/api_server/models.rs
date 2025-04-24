// API data models
//
// This module contains data structures for API requests and responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Shared application state for the API server
#[derive(Clone)]
pub struct ApiState {
    /// Application configuration
    pub config: crate::config::Config,
    /// Application version from Cargo.toml
    pub version: String,
    /// Server start time for uptime calculation
    pub start_time: DateTime<Utc>,
}

/// Generic API response
#[derive(Serialize)]
pub struct ApiResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
    /// Optional data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Status response with server information
#[derive(Serialize)]
pub struct StatusResponse {
    /// API server version
    pub version: String,
    /// Server uptime in days, hours, minutes, seconds
    pub uptime: String,
    /// Server status (online/offline)
    pub status: String,
    /// Whether calendars are available
    pub calendars_available: bool,
}

/// Calendar listing response
#[derive(Serialize)]
pub struct CalendarResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
    /// List of available calendars
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calendars: Option<Vec<String>>,
}

/// Create event request
#[derive(Deserialize, Debug)]
pub struct CreateEventRequest {
    /// Event title
    pub title: String,
    /// Event date in YYYY-MM-DD format
    pub date: String,
    /// Start time in HH:MM format
    pub start_time: String,
    /// End time in HH:MM format (optional)
    #[serde(default)]
    pub end_time: Option<String>,
    /// Calendars to create the event in
    #[serde(default)]
    pub calendars: Option<Vec<String>>,
    /// Event location (optional)
    #[serde(default)]
    pub location: Option<String>,
    /// Event description (optional)
    #[serde(default)]
    pub description: Option<String>,
    /// Email addresses for attendees (optional)
    #[serde(default)]
    pub emails: Option<Vec<String>>,
    /// Reminder time in minutes before event (optional)
    #[serde(default)]
    pub reminder: Option<i32>,
    /// Whether to create a Zoom meeting for this event (optional)
    #[serde(default)]
    pub create_zoom_meeting: Option<bool>,
}

/// Create todo request
#[derive(Deserialize, Debug)]
pub struct CreateTodoRequest {
    /// Todo item title
    pub title: String,
    /// Lists to add the todo item to
    #[serde(default)]
    pub lists: Option<Vec<String>>,
    /// Reminder time in YYYY-MM-DD HH:MM format
    #[serde(default)]
    pub reminder_time: Option<String>,
    /// Additional notes
    #[serde(default)]
    pub notes: Option<String>,
}

/// Todo response
#[derive(Serialize)]
pub struct TodoResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
}

/// Create note request
#[derive(Deserialize, Debug)]
pub struct CreateNoteRequest {
    /// Note title
    pub title: String,
    /// Note content
    pub content: String,
    /// Folder to save the note in
    #[serde(default)]
    pub folder: Option<String>,
}

/// Note response
#[derive(Serialize)]
pub struct NoteResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Response message
    pub message: String,
}

/// Generic WebSocket message format
#[derive(Debug, Deserialize, Serialize)]
pub struct SwiftMessage {
    /// Message type (e.g., "chat", "command", "create")
    #[serde(default)]
    pub message_type: Option<String>,
    /// Action to perform (e.g., "event", "todo")
    #[serde(default)]
    pub action: Option<String>,
    /// Text content for chat messages
    #[serde(default)]
    pub content: Option<String>,
    /// Structured data payload
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Event data for WebSocket event creation
#[derive(Debug, Deserialize)]
pub struct SwiftEventData {
    /// Event title
    pub title: String,
    /// Event date (YYYY-MM-DD)
    pub date: String,
    /// Start time (HH:MM)
    pub start_time: String,
    /// End time (HH:MM)
    pub end_time: String,
    /// Optional location
    #[serde(default)]
    pub location: Option<String>,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// WebSocket chat message
#[derive(Debug, Serialize)]
pub struct SwiftChatMessage {
    /// Sender identifier
    pub sender: String,
    /// Message content
    pub content: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Message type
    pub message_type: String,
}

/// WebSocket event response
#[derive(Debug, Serialize)]
pub struct SwiftEventResponse {
    /// Message type (always "event")
    pub message_type: String,
    /// Status ("success" or "error")
    pub status: String,
    /// Response message
    pub message: String,
    /// Event ID if created successfully
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

/// WebSocket error response
#[derive(Debug, Serialize)]
pub struct SwiftErrorResponse {
    /// Message type (always "error")
    pub message_type: String,
    /// Error message
    pub message: String,
}
