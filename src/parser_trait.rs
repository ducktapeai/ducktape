// Make this module public so it can be used from external crates
pub use crate::command_processor::CommandArgs;
pub use anyhow::Result;
pub use async_trait::async_trait;
pub use log::debug;

/// Trait defining the interface for all parsers in the system
/// This allows for a clean separation between terminal mode and natural language processing
#[async_trait]
pub trait Parser {
    /// Parse input text and return either command string or structured command args
    async fn parse_input(&self, input: &str) -> Result<ParseResult>;

    /// Create a new instance of the parser
    fn new() -> Result<Self>
    where
        Self: Sized;
}

/// Results from parsing user input
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// A command string that needs further parsing with CommandArgs::parse
    CommandString(String),

    /// Already structured command arguments that can bypass string parsing
    StructuredCommand(CommandArgs),
}

/// Factory for creating the appropriate parser based on configuration
pub struct ParserFactory;

impl ParserFactory {
    /// Create a parser based on the configured language model provider
    pub fn create_parser() -> Result<Box<dyn Parser + Send + Sync>> {
        // Temporarily always use terminal mode until the parser modules are fixed
        // This ensures proper handling of "--contacts" with multi-word names
        create_terminal_parser()

        // Original code commented out for now
        /*
        let config = crate::config::Config::load()?;

        // Create appropriate parser based on configuration
        match config.language_model.provider {
            Some(crate::config::LLMProvider::OpenAI) => {
                // Use OpenAI parser for natural language processing
                let parser = crate::openai_parser::OpenAIParser::new()?;
                Ok(Box::new(parser) as Box<dyn Parser + Send + Sync>)
            }
            Some(crate::config::LLMProvider::Grok) => {
                // Use Grok parser for natural language processing
                let parser = crate::grok_parser::GrokParser::new()?;
                Ok(Box::new(parser) as Box<dyn Parser + Send + Sync>)
            }
            Some(crate::config::LLMProvider::DeepSeek) => {
                // Use DeepSeek parser for natural language processing
                let parser = crate::deepseek_parser::DeepSeekParser::new()?;
                Ok(Box::new(parser) as Box<dyn Parser + Send + Sync>)
            }
            None => {
                // Terminal mode - create a dedicated terminal parser
                // that doesn't use natural language processing
                create_terminal_parser()
            }
        }
        */
    }
}

/// Creates a terminal parser for handling command-line inputs
fn create_terminal_parser() -> Result<Box<dyn Parser + Send + Sync>> {
    // Avoid direct use of TerminalParser to prevent module resolution issues
    struct SimpleTerminalParser;

    #[async_trait]
    impl Parser for SimpleTerminalParser {
        async fn parse_input(&self, input: &str) -> Result<ParseResult> {
            debug!("Terminal parser processing input: {}", input);

            // Normalize the input by adding "ducktape" prefix if missing
            let normalized_input = if !input.trim().starts_with("ducktape") {
                format!("ducktape {}", input)
            } else {
                input.to_string()
            };

            debug!("Terminal parser normalized input: {}", normalized_input);

            // Just return the command string to be parsed by CommandArgs::parse
            Ok(ParseResult::CommandString(normalized_input))
        }

        fn new() -> Result<Self> {
            Ok(Self)
        }
    }

    Ok(Box::new(SimpleTerminalParser) as Box<dyn Parser + Send + Sync>)
}
