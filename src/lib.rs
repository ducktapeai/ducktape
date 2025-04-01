pub mod api_server;
pub mod app;
pub mod calendar;
pub mod calendar_legacy;
// pub mod cli;  // Commented out if it doesn't exist
pub mod command_parser;
pub mod command_processor; // New command processor module
// pub mod commands; // Removed commands module
pub mod config;
pub mod contact_groups;
// pub mod contacts;  // Commented out if it doesn't exist
pub mod deepseek_parser;
pub mod deepseek_reasoning;
pub mod env_loader; // Add this line
pub mod env_store; // Add this line
pub mod event_search;
pub mod file_search;
pub mod grok_parser;
pub mod notes;
pub mod openai_parser;
// pub mod parsing_utils;  // Commented out if it doesn't exist
pub mod env_manager;
pub mod reminders;
pub mod state;
pub mod storage; // Add storage module
pub mod todo;
pub mod utils;
pub mod validation;
pub mod zoom; // New module

use anyhow::Result;
use log::*;
use std::path::PathBuf;

pub async fn run(_config_path: Option<PathBuf>) -> Result<()> {
    // Create and run the application
    let app = app::Application::new();
    info!("Initializing DuckTape application");
    app.run().await
}

pub fn init_logger() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();
}

// Re-export commonly used types
pub use config::Config;
pub use state::{CalendarItem, TodoItem};
