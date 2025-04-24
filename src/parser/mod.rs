//! Parser module for DuckTape
//!
//! This module provides a unified interface for parsing various
//! types of input including natural language and structured commands.

pub mod command;
pub mod deepseek;
pub mod grok;
pub mod terminal;
pub mod traits;
pub mod utils;

// Re-export core types for easier access
pub use self::traits::{ParseResult, Parser, ParserFactory};

// Re-export important utility functions
pub use self::command::parse_with_clap;
pub use self::utils::sanitize_nlp_command;
