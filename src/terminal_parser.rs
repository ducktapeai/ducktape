//! Terminal parser compatibility module (Deprecated)
//!
//! This module is kept for backward compatibility and redirects to the new modular structure.
//! Use the `crate::parser::terminal` module instead.

use anyhow::Result;
use async_trait::async_trait;
use log::debug;

use crate::parser::traits::{ParseResult, Parser};

#[deprecated(
    since = "0.13.0",
    note = "Use crate::parser::terminal::TerminalParser instead"
)]
pub struct TerminalParser;

#[async_trait]
impl Parser for TerminalParser {
    async fn parse_input(&self, input: &str) -> Result<ParseResult> {
        // Delegate to the new implementation
        let parser = crate::parser::terminal::TerminalParser;
        parser.parse_input(input).await
    }

    fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[deprecated(
    since = "0.13.0",
    note = "Use crate::parser::terminal::create_terminal_parser instead"
)]
pub fn create_terminal_parser() -> Result<Box<dyn Parser + Send + Sync>> {
    // Delegate to the new implementation
    crate::parser::terminal::create_terminal_parser()
}

