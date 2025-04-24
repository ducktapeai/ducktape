//! OpenAI parser module for DuckTape
//!
//! This module provides natural language parsing functionality using OpenAI's models.

use crate::parser::traits::{ParseResult, Parser};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::debug;
use reqwest::Client;
use serde_json::{Value, json};
use std::env;

mod parser;
mod utils;

// Re-export the parser
pub use parser::OpenAIParser;

// Re-export the parse_natural_language function for use elsewhere
pub use parser::parse_natural_language;

// Re-export utility functions for backward compatibility
pub use utils::{
    enhance_command_with_contacts, enhance_command_with_recurrence, enhance_command_with_zoom,
    extract_contact_names, extract_emails, sanitize_nlp_command, sanitize_user_input,
    validate_calendar_command,
};
