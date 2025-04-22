// API Server implementation
//
// This module handles starting and configuring the API server.

use axum::serve;
use chrono::Utc;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;

use super::models::ApiState;
use super::routes::create_routes;

/// Start the API server on the specified address
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `address` - Socket address to bind to (e.g., "127.0.0.1:3000")
///
/// # Returns
///
/// Result indicating success or error
pub async fn start_api_server(config: crate::config::Config, address: &str) -> anyhow::Result<()> {
    // Parse the address
    let addr: SocketAddr = address.parse()?;

    // Create the shared application state
    let state = Arc::new(ApiState {
        config,
        version: env!("CARGO_PKG_VERSION").to_string(),
        start_time: Utc::now(),
    });

    // Create the application with routes
    let app = create_routes(state.clone());

    info!("API server starting on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await.map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
