/// Grok Parser - DEPRECATED
/// 
/// This module is deprecated. Use crate::parser::grok module instead.
///
/// This module is kept for backward compatibility and forwards all calls to the new module structure.

// Re-export only what's needed for backward compatibility
pub use crate::parser::traits::{ParseResult, Parser};
pub use crate::parser::grok::GrokParser;

// Note: The following functions don't exist in the new module structure yet
// Functions will be implemented there or this file updated when they're available
