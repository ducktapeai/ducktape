pub mod calendar;
pub mod config; // Make sure this is public
pub mod file_search;
pub mod notes;
pub mod openai_parser;
pub mod state;
pub mod todo;

// Re-export commonly used types
pub use config::Config;
pub use state::{CalendarItem, TodoItem}; // Add this line
