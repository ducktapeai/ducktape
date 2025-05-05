pub mod api_server;
pub mod app;
pub mod calendar;
pub mod cli;
#[deprecated(since = "0.14.0", note = "Use parser module instead")]
// pub mod command_parser; // Removed: use parser::command instead
pub mod command_processor;
pub mod config;
pub mod contact_groups;
// pub mod contacts;  // Commented out if it doesn't exist
// Removed deepseek_reasoning module
pub mod env_debug;
pub mod env_loader;
pub mod env_manager;
pub mod env_store;
pub mod event_search;
pub mod file_search;
pub mod notes;
pub mod parser; // New modular parser module
pub mod reminder;
pub mod reminders;
pub mod state;
pub mod storage;
// todo module removed in version 0.17.0, use reminder module instead
pub mod utils;
// pub mod validation; // Removed in cleanup
pub mod zoom;

// Deprecated modules - will be removed after migration
// command_parser already declared above with deprecation notice
// pub mod deepseek_parser; // Removed: use parser::deepseek instead
#[deprecated(since = "0.13.0", note = "Use crate::parser::traits module instead")]
pub mod parser_trait;
// pub mod terminal_parser; // Deprecated: use parser::terminal instead
// pub mod openai_parser; // Removed: use parser::openai instead

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
pub use state::CalendarItem;
pub use state::ReminderItem;
// TodoItem type removed in version 0.17.0, use ReminderItem instead

// Re-export parser types for convenience
pub use parser::ParserFactory;
pub use parser::traits::{ParseResult, Parser};
