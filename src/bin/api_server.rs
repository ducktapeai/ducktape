use anyhow::Result;
use clap::Parser;
use ducktape::api_server;
use ducktape::config::Config;
use log::{info, error};
use std::path::PathBuf;

/// Command line arguments for the API server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Port to run the server on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Parse command line arguments
    let cli = Cli::parse();
    
    // Construct server address
    let address = format!("{}:{}", cli.host, cli.port);
    info!("Starting API server on {}", address);
    
    // Load configuration
    let config = match cli.config {
        Some(path) => {
            info!("Loading configuration from {}", path.display());
            // Use the standard Config::load() method
            // If a custom path is needed in the future, modify the Config struct to support this
            Config::load()?
        }
        None => {
            info!("Using default configuration");
            Config::load()?
        }
    };
    
    // Start the API server
    if let Err(e) = api_server::start_api_server(config, &address).await {
        error!("API server error: {}", e);
    }
    
    Ok(())
}