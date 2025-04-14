use ducktape::api_server;
use ducktape::app::Application;
use ducktape::cli;
use ducktape::command_processor::{
    CommandArgs, CommandProcessor, preprocess_input, resolve_contacts,
};
use ducktape::config::Config;
use ducktape::env_debug;

use anyhow::{Result, anyhow};
use clap::Parser;
use log::debug;
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
        return api_server::start_api_server(config).await;
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

/// Unified command processing pipeline
fn process_command(input: &str, mode: Mode) -> Result<()> {
    let preprocessed = preprocess_input(input);

    // We don't use contacts directly here, but store for later use
    let _contacts = resolve_contacts(&preprocessed);

    match mode {
        Mode::Terminal => {
            // Format the input into argv style for clap
            let args = shell_words::split(&preprocessed)
                .map_err(|e| anyhow!("Failed to parse command: {}", e))?;

            // Check if we have any arguments
            if args.is_empty() {
                return Err(anyhow!("Empty command"));
            }

            // Try using Clap to parse the command
            match cli::Cli::try_parse_from(&args) {
                Ok(cli) => {
                    // Convert from Clap command to CommandArgs
                    if let Some(command_args) = cli::convert_to_command_args(&cli) {
                        let processor = CommandProcessor::new();
                        tokio::runtime::Runtime::new()?.block_on(processor.execute(command_args))
                    } else {
                        // If there's no command, just return Ok
                        Ok(())
                    }
                }
                Err(_) => {
                    // Fall back to legacy parser if Clap fails
                    // This is useful for backward compatibility
                    let args = CommandArgs::parse(&preprocessed)?;
                    let processor = CommandProcessor::new();
                    tokio::runtime::Runtime::new()?.block_on(processor.execute(args))
                }
            }
        }
        Mode::NaturalLanguage => {
            // Translate natural language to structured command
            let translated_command = translate_to_command(&preprocessed)?;

            // Try parsing with Clap first
            let args = shell_words::split(&translated_command)
                .map_err(|e| anyhow!("Failed to parse translated command: {}", e))?;

            if !args.is_empty() {
                match cli::Cli::try_parse_from(&args) {
                    Ok(cli) => {
                        // Convert from Clap command to CommandArgs
                        if let Some(command_args) = cli::convert_to_command_args(&cli) {
                            let processor = CommandProcessor::new();
                            return tokio::runtime::Runtime::new()?
                                .block_on(processor.execute(command_args));
                        }
                    }
                    Err(_) => {
                        // Fall back to legacy parser
                    }
                }
            }

            // Fall back to legacy parser
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
