// API Routes
//
// This module sets up routes for the API server, including both
// REST API endpoints and WebSocket handlers.

use axum::{
    extract::State,
    http::Method,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use super::handlers;
use super::models::ApiState;
use super::websocket::websocket_handler;

/// Create application routes with proper CORS configuration
pub fn create_routes(state: Arc<ApiState>) -> Router {
    // Configure CORS for web and mobile clients
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Define routes with proper handler functions
    Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health))
        
        // API status endpoint
        .route("/status", get(handlers::status))
        
        // Calendar APIs
        .route("/calendars", get(handlers::list_calendars))
        .route("/calendar/event", post(handlers::create_calendar_event))
        
        // Todo API
        .route("/todo", post(handlers::create_todo))
        
        // Notes API
        .route("/note", post(handlers::create_note))
        
        // WebSocket endpoint for real-time communications
        .route("/chat", get(websocket_handler))
        
        // API docs
        .route("/api-docs", get(handlers::api_docs))
        
        // Apply CORS middleware
        .layer(cors)
        
        // Attach shared application state
        .with_state(state)
}