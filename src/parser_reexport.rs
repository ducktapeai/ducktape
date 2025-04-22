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
pub use crate::parser::openai::OpenAIParser;
pub use crate::parser::grok::GrokParser;
pub use crate::parser::terminal::TerminalParser;
pub use crate::parser::deepseek::DeepSeekParser;
pub use crate::parser::command::CommandParser;

// Re-export core functionality
pub use crate::parser::command::parse_with_clap;
pub use crate::parser::openai::parse_natural_language;

// Re-export utility functions for backward compatibility
pub use crate::parser::openai::{
    enhance_command_with_recurrence,
    enhance_command_with_contacts,
    enhance_command_with_zoom,
    extract_contact_names,
    extract_emails,
    sanitize_nlp_command,
    sanitize_user_input,
    validate_calendar_command,
};