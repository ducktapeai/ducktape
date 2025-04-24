//! Parser trait compatibility module (Deprecated)
//!
//! This module is kept for backward compatibility and redirects to the new modular structure.
//! Use the `crate::parser::traits` module instead.

// Re-export all types from the new module for backward compatibility
pub use crate::parser::traits::{ParseResult, Parser, ParserFactory};

// Re-export required types that were previously imported directly
pub use crate::command_processor::CommandArgs;
pub use anyhow::Result;
pub use async_trait::async_trait;
pub use log::debug;
