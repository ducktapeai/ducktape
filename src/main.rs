mod api_server;
mod app;
mod calendar;
mod calendar_legacy;
mod command_parser;
mod commands;
mod config;
mod contact_groups;
mod deepseek_parser;
mod deepseek_reasoning;
mod env_debug;
mod event_search;
mod file_search;
mod grok_parser;
mod notes;
mod openai_parser;
mod reminders;
mod state;
mod todo;
mod zoom;

use anyhow::Result;
use app::Application;
use config::Config;
use std::env;

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

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Check command line flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--api-server" => {
                // Load config and start API server only
                let config = Config::load()?;
                api_server::start_api_server(config).await?;
                return Ok(());
            }
            "--full" => {
                // Start both terminal and API server (original behavior)
                let app = Application::new();
                app.run().await?;
                return Ok(());
            }
            _ => {
                println!("Usage: ducktape [--api-server|--full]");
                println!("  --api-server  Start in API server mode only");
                println!("  --full        Start both terminal and API server");
                println!("  (no flags)    Start in terminal mode only");
                return Ok(());
            }
        }
    }

    // Default: Run in terminal-only mode
    let app = Application::new();
    app.run_terminal_only().await
}
