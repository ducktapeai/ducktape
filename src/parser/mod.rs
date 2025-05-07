pub mod command;
pub mod deepseek;
pub mod grok;
pub mod natural_language;
pub mod terminal;
/// DuckTape Parser module
///
/// This module implements different parsers for DuckTape.
pub mod traits;
pub mod utils;

use crate::config::{Config, LLMProvider};
use anyhow::Result;
use log::{debug, info};
// Re-export ParseResult so it's accessible as parser::ParseResult
pub use traits::{ParseResult, Parser};

/// Factory for creating appropriate parsers
pub struct ParserFactory;

impl ParserFactory {
    /// Create a parser instance based on project configuration
    pub fn create_parser() -> Result<Box<dyn Parser + Send + Sync>> {
        let config = Config::load()?;

        match config.language_model.provider {
            Some(LLMProvider::Grok) => {
                info!("Creating Grok parser");
                let parser = grok::GrokParser::new()?;
                Ok(Box::new(parser))
            }
            Some(LLMProvider::DeepSeek) => {
                info!("Creating DeepSeek parser");
                let parser = deepseek::DeepSeekParser::new()?;
                Ok(Box::new(parser))
            }
            None => {
                info!("Creating Terminal parser");
                terminal::create_terminal_parser()
            }
        }
    }
}

/// Centralized function to sanitize NLP-generated commands
pub fn sanitize_nlp_command(command: &str) -> String {
    if let Some(provider) = Config::load().ok().and_then(|c| c.language_model.provider) {
        match provider {
            LLMProvider::Grok => {
                debug!("Using Grok command sanitizer");
                // First apply command mapping to normalize "create" to "calendar create", etc.
                let normalized = natural_language::command_mapping::normalize_command(command);
                if normalized != command {
                    debug!("Normalized command: '{}'", normalized);
                    // If normalization happened, use the normalized command
                    crate::parser::natural_language::grok::utils::sanitize_nlp_command(&normalized)
                } else {
                    // Otherwise use the original command
                    crate::parser::natural_language::grok::utils::sanitize_nlp_command(command)
                }
            }
            LLMProvider::DeepSeek => {
                debug!("Using DeepSeek command sanitizer");
                // Apply command mapping for DeepSeek also for consistency
                let normalized = natural_language::command_mapping::normalize_command(command);
                if normalized != command {
                    debug!("Normalized command: '{}'", normalized);
                    format!("ducktape {}", normalized)
                } else {
                    // Use the DeepSeek-specific sanitizer when available
                    format!("ducktape {}", natural_language::utils::sanitize_user_input(command))
                }
            }
        }
    } else {
        // Default sanitizer for when no specific provider is set
        // Still apply command mapping for consistency
        let normalized = natural_language::command_mapping::normalize_command(command);
        if normalized != command {
            debug!("Normalized command (default): '{}'", normalized);
            format!("ducktape {}", normalized)
        } else {
            format!("ducktape {}", command)
        }
    }
}
