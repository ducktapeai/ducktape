mod app;
mod calendar;
mod calendar_legacy;
mod commands;
mod config;
mod contact_groups;
mod deepseek_parser;
mod deepseek_reasoning;
mod event_search;
mod file_search;
mod grok_parser;
mod notes;
mod openai_parser;
mod reminders;
mod state;
mod todo;
mod zoom;
mod api_server;
mod command_parser;

use anyhow::Result;
use app::Application;
use config::Config;
use std::env;
use command_parser::{UserMessage, process_command};
use serde_json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if we should start in API server mode
    if args.len() > 1 && args[1] == "--api-server" {
        // Load config and start API server
        let config = Config::load()?;
        api_server::start_api_server(config).await?;
        return Ok(());
    }
    
    // Otherwise, run the CLI application
    let app = Application::new();
    app.run().await
}
