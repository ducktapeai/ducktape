//! Parser trait compatibility module (Deprecated)
//!
//! This module is kept for backward compatibility and redirects to the new modular structure.
//! Use the `crate::parser::traits` module instead.

// Re-export all types from the new module for backward compatibility
pub use crate::parser::ParserFactory;
pub use crate::parser::traits::{ParseResult, Parser};

#[deprecated(
    since = "0.13.0",
    note = "Use crate::parser::traits::create_parser function instead"
)]
pub async fn create_parser_by_name(name: &str) -> anyhow::Result<Box<dyn Parser + Send + Sync>> {
    // Delegate to the new implementation
    crate::parser::traits::create_parser(name)
}
