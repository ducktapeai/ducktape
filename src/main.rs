mod app;
mod calendar;
mod calendar_legacy;
mod commands;
mod config;
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

use anyhow::Result;
use app::Application;

#[tokio::main]
async fn main() -> Result<()> {
    // Create and run the application
    let app = Application::new();
    app.run().await
}
