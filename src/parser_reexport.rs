//! Parser interface for DuckTape
//!
//! This module serves as a entry point for the parser functionality,
//! re-exporting the parser trait and implementations from the modular structure.
//!
//! This module is kept for backward compatibility and forwards all calls to the new module structure.

// Re-export the parser trait and associated types
pub use crate::parser_trait::{ParseResult, Parser};

// Re-export the parser factory
pub use crate::parser::traits::ParserFactory;

// Re-export parser implementations for backward compatibility
pub use crate::parser::command::CommandParser;
pub use crate::parser::deepseek::DeepSeekParser;
pub use crate::parser::grok::GrokParser;
pub use crate::parser::terminal::TerminalParser;

// Re-export core functionality
pub use crate::parser::command::parse_with_clap;

// OpenAI parser functionality has been removed
// Utility functions removed
