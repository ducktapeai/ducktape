pub mod app;
pub mod calendar;
pub mod calendar_legacy;
pub mod commands;
pub mod config;
pub mod contact_groups;
pub mod deepseek_parser;
pub mod deepseek_reasoning;
pub mod event_search;
pub mod file_search;
pub mod grok_parser;
pub mod notes;
pub mod openai_parser;
pub mod reminders;
pub mod state;
pub mod todo;
pub mod utils;
pub mod validation;
pub mod zoom;
pub mod api_server;

use log::*;
use std::path::PathBuf;
use anyhow::Result;

pub async fn run(_config_path: Option<PathBuf>) -> Result<()> {
    // Initialize error handling
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();

    // Create and run the application
    let app = app::Application::new();
    info!("Initializing DuckTape application");
    app.run().await
}

// Re-export commonly used types
pub use config::Config;
pub use state::{CalendarItem, TodoItem};
