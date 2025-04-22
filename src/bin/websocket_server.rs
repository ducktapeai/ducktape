use anyhow::Result;
use ducktape::{api_server, config::Config};
use log::{error, info};
use tokio::sync::mpsc;

/// API server entry point for DuckTape
///
/// This binary starts a web server that provides:
/// 1. REST API endpoints for CRUD operations
/// 2. WebSocket connections for real-time updates
/// 3. A JSON-based interface for frontend applications
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("Starting WebSocket server");

    // Load configuration
    let config = Config::load()?;

    // Create a channel for communication between websocket and API server
    let (command_tx, mut command_rx) = mpsc::channel::<String>(100);

    // Start the API server with websocket support
    let api_server_handle = {
        let config = config.clone();
        tokio::spawn(async move {
            // Start the API server on localhost:3000
            const API_ADDRESS: &str = "127.0.0.1:3000";
            if let Err(e) = api_server::start_api_server(config, API_ADDRESS).await {
                error!("API server error: {}", e);
            }
        })
    };

    // Process incoming websocket commands
    while let Some(command) = command_rx.recv().await {
        info!("Received command: {}", command);
        // Process the command...
    }

    // Wait for the API server to finish (which it shouldn't unless there's an error)
    api_server_handle.await?;

    Ok(())
}
