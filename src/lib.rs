pub mod api_server;
pub mod app;
pub mod calendar;
// Removed calendar_legacy module reference as it was moved to calendar directory
pub mod cli;
#[deprecated(since = "0.14.0", note = "Use parser module instead")]
pub mod command_parser;
pub mod command_processor;
// pub mod commands; // Removed commands module
pub mod config;
pub mod contact_groups;
// pub mod contacts;  // Commented out if it doesn't exist
// Removed deepseek_parser module
// Removed deepseek_reasoning module
pub mod env_debug; // Add this line to expose the env_debug module
pub mod env_loader; // Add this line
pub mod env_store; // Add this line
pub mod event_search;
pub mod file_search;
// Removed deprecated grok_parser module (now using parser::natural_language::grok)
pub mod notes;
// Removed openai_parser module
pub mod parser; // New modular parser system
pub mod parser_reexport;
#[deprecated(since = "0.14.0", note = "Use parser::traits module instead")]
pub mod parser_trait; // Kept for backward compatibility // Backward compatibility layer
// pub mod parsing_utils;  // Commented out if it doesn't exist
pub mod env_manager;
pub mod reminder; // New module for Apple Reminders functionality
pub mod reminders;
pub mod state;
pub mod storage; // Add storage module
#[deprecated(since = "0.14.0", note = "Use parser::command module instead")]
pub mod terminal_parser; // Kept for backward compatibility
pub mod todo; // Kept for backward compatibility
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

// Re-export parser types for convenience
pub use parser::traits::{ParseResult, Parser, ParserFactory};
