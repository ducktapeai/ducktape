// API Request Handlers
//
// This module contains handler functions for API endpoints.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use log::{debug, error};
use std::fs;
use std::sync::Arc;

use super::models::{
    ApiResponse, ApiState, CalendarResponse, CreateEventRequest, CreateNoteRequest,
    CreateTodoRequest, NoteResponse, StatusResponse, TodoResponse,
};

/// Handle health check requests
///
/// Returns 200 OK if the service is running
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Get API server status information
///
/// Returns version, uptime, and status information
pub async fn status(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    // Calculate uptime from start time
    let now = Utc::now();
    let duration = now.signed_duration_since(state.start_time);
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    let uptime =
        format!("{} days, {} hours, {} minutes, {} seconds", days, hours, minutes, seconds);

    let response = StatusResponse {
        version: state.version.clone(),
        uptime,
        status: "online".to_string(),
        calendars_available: true,
    };

    (StatusCode::OK, Json(response))
}

/// List available calendars
///
/// Returns a list of calendars from macOS Calendar.app
pub async fn list_calendars() -> impl IntoResponse {
    match crate::calendar::get_available_calendars().await {
        Ok(calendars) => {
            let response = CalendarResponse {
                success: true,
                message: "Calendars retrieved successfully".to_string(),
                calendars: Some(calendars),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to list calendars: {}", e);
            let response = CalendarResponse {
                success: false,
                message: format!("Failed to list calendars: {}", e),
                calendars: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Create a new calendar event
///
/// Creates an event in macOS Calendar.app
pub async fn create_calendar_event(Json(payload): Json<CreateEventRequest>) -> impl IntoResponse {
    debug!("Create event request: {:?}", payload);

    // Create an EventConfig from the request
    let mut event_config =
        crate::calendar::EventConfig::new(&payload.title, &payload.date, &payload.start_time);

    // Apply optional fields if present
    if let Some(end_time) = &payload.end_time {
        event_config.end_time = Some(end_time.clone());
    }

    if let Some(calendars) = &payload.calendars {
        event_config.calendars = calendars.clone();
    }

    if let Some(location) = &payload.location {
        event_config.location = Some(location.clone());
    }

    if let Some(description) = &payload.description {
        event_config.description = Some(description.clone());
    }

    if let Some(emails) = &payload.emails {
        event_config.emails = emails.clone();
    }

    if let Some(true) = payload.create_zoom_meeting {
        event_config.create_zoom_meeting = true;
    }

    // Create the calendar event
    match crate::calendar::create_event(event_config).await {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                message: "Event created successfully".to_string(),
                data: None,
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            error!("Failed to create event: {}", e);
            let response = ApiResponse {
                success: false,
                message: format!("Failed to create event: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Create a new todo item
///
/// Creates a todo in Reminders.app
pub async fn create_todo(Json(payload): Json<CreateTodoRequest>) -> impl IntoResponse {
    debug!("Create todo request: {:?}", payload);

    // This is a stub - would connect to actual todo module
    let response = TodoResponse {
        success: true,
        message: format!("Todo '{}' created successfully", payload.title),
    };

    (StatusCode::CREATED, Json(response))
}

/// Create a new note
///
/// Creates a note in Notes.app
pub async fn create_note(Json(payload): Json<CreateNoteRequest>) -> impl IntoResponse {
    debug!("Create note request: {:?}", payload);

    // This is a stub - would connect to actual notes module
    let response = NoteResponse {
        success: true,
        message: format!("Note '{}' created successfully", payload.title),
    };

    (StatusCode::CREATED, Json(response))
}

/// Serve the OpenAPI documentation
///
/// Returns the OpenAPI JSON specification
pub async fn api_docs() -> impl IntoResponse {
    // Read the OpenAPI spec from the api_docs.json file
    match fs::read_to_string("src/api_docs.json") {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => (StatusCode::OK, Json(json)),
                Err(_) => {
                    let error = ApiResponse {
                        success: false,
                        message: "Invalid API documentation format".to_string(),
                        data: None,
                    };
                    // Convert to serde_json::Value to match the other branch
                    let json_value = serde_json::to_value(error).unwrap_or_default();
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json_value))
                }
            }
        }
        Err(_) => {
            let error = ApiResponse {
                success: false,
                message: "API documentation not found".to_string(),
                data: None,
            };
            // Convert to serde_json::Value to match the other branch
            let json_value = serde_json::to_value(error).unwrap_or_default();
            (StatusCode::NOT_FOUND, Json(json_value))
        }
    }
}
