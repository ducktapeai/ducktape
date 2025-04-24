// API Server Module
//
// This module provides a complete HTTP and WebSocket API for frontend applications
// to interact with DuckTape's functionality.

mod handlers;
mod models;
mod routes;
mod server;
mod websocket;

// Re-export the main types and functions needed by consumers of this module
pub use models::ApiState;
pub use server::start_api_server;

#[cfg(test)]
mod tests {
    // API server tests will be implemented here
}
