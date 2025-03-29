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

/// Name of the application used in help and version output
const APP_NAME: &str = "DuckTape";
/// Current version of the application
const VERSION: &str = env!("CARGO_PKG_VERSION");

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
            "version" | "--version" | "-v" => {
                // Display version information
                print_version();
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                // Display help information
                print_help();
                return Ok(());
            }
            _ => {
                // For other commands, we could either:
                // 1. Print help and return error
                // 2. Try to process as a domain command
                // For now, display help for unknown commands
                print_help();
                return Ok(());
            }
        }
    }

    // Default: Run in terminal-only mode
    let app = Application::new();
    app.run_terminal_only().await
}

/// Prints version information for the application
fn print_version() {
    println!("{} version {}", APP_NAME, VERSION);
}

/// Prints help information for the application
fn print_help() {
    println!("{} - AI-powered terminal tool for Apple Calendar, Reminders and Notes", APP_NAME);
    println!("\nUSAGE:");
    println!("  ducktape [COMMAND] [FLAGS]");
    println!("\nCOMMANDS:");
    println!("  help        Display this help information");
    println!("  version     Display the current version");
    println!("\nFLAGS:");
    println!("  --api-server  Start in API server mode only");
    println!("  --full        Start both terminal and API server");
    println!("  (no flags)    Start in terminal mode only");
    println!("\nEXAMPLES:");
    println!("  ducktape                 Start interactive terminal mode");
    println!("  ducktape --api-server    Start API server only");
    println!("  ducktape version         Display version information");
}
