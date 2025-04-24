use ducktape::api_server;
use ducktape::app::Application;
use ducktape::cli;
use ducktape::config::Config;
use ducktape::env_debug;

use anyhow::Result;
use clap::Parser;
use log::debug;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load environment variables at startup
    if let Err(e) = dotenvy::dotenv() {
        println!("Warning: Failed to load .env file: {}", e);
    }

    // Force set the API key
    env_debug::force_set_api_key();

    // Create a String from all command line args to preserve exact quoting
    let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");

    debug!("Raw input from command line: '{}'", input);

    // Parse command line arguments using Clap
    let cli = cli::Cli::parse();

    // Create application instance early so we can use it for commands
    let app = Application::new();

    // Handle special flags
    if cli.api_server {
        // Load config and start API server only
        let config = Config::load()?;
        let address = "127.0.0.1:3000";
        return api_server::start_api_server(config, &address).await;
    }

    if cli.full {
        // Start both terminal and API server (original behavior)
        return app.run().await;
    }

    // If we have command line arguments, process them directly
    if !input.trim().is_empty() {
        return app.process_command(&input).await;
    }

    // No command specified, run in terminal-only mode
    app.run_terminal_only().await
}
