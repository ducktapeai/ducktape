pub mod api_server;
pub mod app;
pub mod calendar;
pub mod cli;
#[deprecated(since = "0.14.0", note = "Use parser module instead")]
pub mod command_parser;
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
pub mod parser_reexport; // Re-exports from parser module for backward compatibility
pub mod reminder;
pub mod reminders;
pub mod state;
pub mod storage;
pub mod todo;
pub mod utils;
pub mod validation;
pub mod zoom;

// Deprecated modules - will be removed after migration
#[deprecated(since = "0.13.0", note = "Use crate::parser::command module instead")]
pub mod command_parser;
#[deprecated(since = "0.13.0", note = "Use crate::parser::deepseek module instead")]
pub mod deepseek_parser;
#[deprecated(since = "0.13.0", note = "Use crate::parser::grok module instead")]
pub mod grok_parser;
#[deprecated(since = "0.13.0", note = "Use crate::parser::openai module instead")]
pub mod openai_parser;
#[deprecated(since = "0.13.0", note = "Use crate::parser::traits module instead")]
pub mod parser_trait;
#[deprecated(since = "0.13.0", note = "Use crate::parser::terminal module instead")]
pub mod terminal_parser;

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
