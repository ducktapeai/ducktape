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

    // Collect all command line args
    let all_args: Vec<String> = std::env::args().collect();
    // Create input string properly from arguments
    let input = all_args.iter().skip(1).map(|s| s.as_str()).collect::<Vec<_>>().join(" ");

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

    // Handle the ai subcommand for natural language input
    if let Some(cli::Commands::Ai { nl_command }) = &cli.command {
        let nl_input = nl_command.join(" ");
        if nl_input.trim().is_empty() {
            println!("Error: No natural language command provided to 'ai' subcommand.");
            std::process::exit(1);
        }
        return app.process_natural_language(&nl_input).await;
    }

    if cli.full {
        // Start both terminal and API server (original behavior)
        return app.run().await;
    }

    // If we have command line arguments, process them directly with our new method
    // that supports natural language detection
    if all_args.len() > 1 {
        return app.execute_from_args(all_args).await;
    }

    // No command specified, run in terminal-only mode
    app.run_terminal_only().await
}
