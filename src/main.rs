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

use anyhow::Result;
use app::Application;
use config::Config;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Check if we should start in API server mode
    let args: Vec<String> = env::args().collect();
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

// Add a wrapper to redirect stdout for command capturing
#[cfg(not(test))]
pub fn with_captured_stdout<F, R>(f: F) -> (R, String)
where
    F: FnOnce() -> R,
{
    // Execute the function directly without trying to capture stdout
    let result = f();
    (result, String::new())
}
