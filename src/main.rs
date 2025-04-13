mod api_server;
mod app;
mod calendar;
mod calendar_legacy;
mod cli;
mod command_parser;
mod command_processor;
mod config;
mod contact_groups;
// Temporarily commenting out problematic parsers to get the build working
// mod deepseek_parser;
// mod deepseek_reasoning;
mod env_debug;
mod event_search;
mod file_search;
// mod grok_parser;
mod notes;
// mod openai_parser;
mod parser_trait; // Adding parser_trait directly to main for terminal mode
mod reminders;
mod state;
mod todo;
mod zoom;

use crate::command_processor::{CommandArgs, CommandProcessor, preprocess_input, resolve_contacts};
use anyhow::Result;
use app::Application;
use clap::Parser;
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

    // Parse command line arguments using Clap
    let cli = cli::Cli::parse();

    // Create application instance early so we can use it for commands
    let app = Application::new();

    // Handle special flags
    if cli.api_server {
        // Load config and start API server only
        let config = Config::load()?;
        return api_server::start_api_server(config).await;
    }

    if cli.full {
        // Start both terminal and API server (original behavior)
        return app.run().await;
    }

    // If no special flags, check for commands
    if let Some(command_args) = cli::convert_to_command_args(&cli) {
        // Use the CommandProcessor with the extracted arguments
        let processor = CommandProcessor::new();
        return processor.execute(command_args).await;
    }

    // No command specified, run in terminal-only mode
    app.run_terminal_only().await
}

/// Unified command processing pipeline
fn process_command(input: &str, mode: Mode) -> Result<()> {
    let preprocessed = preprocess_input(input);
    let contacts = resolve_contacts(&preprocessed);

    match mode {
        Mode::Terminal => {
            // Directly execute the command
            let args = CommandArgs::parse(&preprocessed)?;
            let processor = CommandProcessor::new();
            tokio::runtime::Runtime::new()?.block_on(processor.execute(args))
        }
        Mode::NaturalLanguage => {
            // Translate natural language to structured command
            let translated_command = translate_to_command(&preprocessed)?;
            let args = CommandArgs::parse(&translated_command)?;
            let processor = CommandProcessor::new();
            tokio::runtime::Runtime::new()?.block_on(processor.execute(args))
        }
    }
}

/// Enum to represent the mode of operation
enum Mode {
    Terminal,
    NaturalLanguage,
}

/// Function to detect the mode based on configuration
fn detect_mode(config: &Config) -> Mode {
    if config.language_model.provider.is_some() { Mode::NaturalLanguage } else { Mode::Terminal }
}

/// Function to translate natural language input to structured command
fn translate_to_command(input: &str) -> Result<String> {
    // Stub implementation for translate_to_command
    Ok(input.to_string())
}

// The print_version and print_help functions are no longer needed as Clap handles them
// But we'll keep them for backward compatibility with other parts of the code that might use them
/// Prints version information for the application
fn print_version() {
    println!("{} version {}", APP_NAME, VERSION);
}

/// Prints help information for the application
fn print_help() {
    println!(
        "{} - AI-powered terminal tool for Apple Calendar, Reminders and Notes",
        APP_NAME
    );
    println!("\nUSAGE:");
    println!("  ducktape [COMMAND] [FLAGS]");
    println!("\nCOMMANDS:");
    println!("  help        Display this help information");
    println!("  version     Display the current version");
    println!("  calendar    Manage calendar events");
    println!("  todo        Manage reminders/todos");
    println!("  note        Manage notes");
    println!("  config      View or modify configuration");
    println!("\nFLAGS:");
    println!("  --api-server  Start in API server mode only");
    println!("  --full        Start both terminal and API server");
    println!("  (no flags)    Start in terminal mode only");
    println!("\nEXAMPLES:");
    println!("  ducktape                      Start interactive terminal mode");
    println!("  ducktape calendar list        List available calendars");
    println!("  ducktape todo lists           List available reminder lists");
    println!("  ducktape note list            List recent notes");
    println!("  ducktape --api-server         Start API server only");
}
