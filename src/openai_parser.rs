/// OpenAI Parser - DEPRECATED
/// 
/// This module is deprecated. Use crate::parser::openai module instead.
///
/// This module is kept for backward compatibility and forwards all calls to the new module structure.

// Re-export the main parser type
#[deprecated(since = "0.12.0", note = "Use crate::parser::openai::OpenAIParser instead")]
pub use crate::parser::openai::OpenAIParser;

// Re-export the main parsing function
#[deprecated(since = "0.12.0", note = "Use crate::parser::openai::parse_natural_language instead")]
pub use crate::parser::openai::parse_natural_language;

// Re-export the utility functions from the OpenAI module for backward compatibility
#[deprecated(since = "0.12.0", note = "Use crate::parser::openai module instead")]
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
