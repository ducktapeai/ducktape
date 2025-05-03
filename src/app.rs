use crate::command_processor::{CommandArgs, CommandProcessor};
use crate::config::{Config, LLMProvider};
use crate::parser::{ParseResult, Parser, ParserFactory};
use anyhow::{Result, anyhow};
use clap::Parser as ClapParser;
use rustyline::DefaultEditor;

pub struct Application {
    command_processor: CommandProcessor,
}

impl Application {
    pub fn new() -> Self {
        Self { command_processor: CommandProcessor::new() }
    }

    pub async fn run(&self) -> Result<()> {
        log::info!("Starting DuckTape Terminal");
        let config = Config::load()?;

        let use_natural_language = config.language_model.provider.is_some();
        log::debug!(
            "Provider: {:?}, use_natural_language: {}",
            config.language_model.provider,
            use_natural_language
        );

        match Config::load()?.language_model.provider {
            Some(LLMProvider::Grok) => {
                log::info!("Using Grok for natural language processing");
            }
            Some(LLMProvider::DeepSeek) => {
                log::info!("Using DeepSeek for natural language processing");
            }
            None => {
                log::info!("Terminal Mode enabled (no API key required)");
            }
        }

        // Start the API server in a background thread
        log::info!("Starting API server on port 3000");
        let config_clone = config.clone();
        let api_handle = tokio::spawn(async move {
            if let Err(e) =
                crate::api_server::start_api_server(config_clone, "127.0.0.1:3000").await
            {
                log::error!("API server error: {:?}", e);
            }
        });

        let mut rl = DefaultEditor::new()?;

        println!("Welcome to DuckTape Terminal! Type 'help' for commands.");
        let prompt = "🦆 ";

        loop {
            match rl.readline(prompt) {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    if let Err(err) = self.process_input(&line, use_natural_language).await {
                        log::error!("Failed to process command: {:?}", err);
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Signal API server to shutdown if needed
        api_handle.abort();

        Ok(())
    }

    pub async fn run_terminal_only(&self) -> Result<()> {
        log::info!("Starting DuckTape Terminal");

        let config = Config::load()?;
        let use_natural_language = config.language_model.provider.is_some();
        log::debug!(
            "Provider: {:?}, use_natural_language: {}",
            config.language_model.provider,
            use_natural_language
        );

        let mut rl = DefaultEditor::new()?;

        println!("Welcome to DuckTape! How can I assist you today?");
        println!("Example: schedule a meeting with Siya tomorrow at 3pm about project review");

        let prompt = "🦆 ";

        loop {
            match rl.readline(prompt) {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    if let Err(err) = self.process_input(&line, use_natural_language).await {
                        log::error!("Failed to process command: {:?}", err);
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_input(&self, input: &str, _use_natural_language: bool) -> Result<()> {
        log::debug!("Hybrid input mode: auto-detecting command vs natural language");

        // Check for direct exit command regardless of mode
        let preprocessed = crate::command_processor::preprocess_input(input);
        if preprocessed == "exit"
            || preprocessed == "quit"
            || preprocessed == "ducktape exit"
            || preprocessed == "ducktape quit"
        {
            log::info!("Exit command detected, bypassing language processing");
            let command_args = crate::command_processor::CommandArgs::new(
                "exit".to_string(),
                vec![],
                std::collections::HashMap::new(),
            );
            return self.command_processor.execute(command_args).await;
        }

        // List of known DuckTape commands (expanded from README)
        const KNOWN_COMMANDS: &[&str] = &[
            "calendar",
            "reminder",
            "todo",
            "note",
            "config",
            "contact",
            "help",
            "exit",
            "version",
            "--help",
            "--version",
        ];
        let trimmed = preprocessed.trim_start();
        let is_direct_command = KNOWN_COMMANDS.iter().any(|cmd| {
            trimmed.starts_with(cmd) || trimmed.starts_with(&format!("ducktape {}", cmd))
        });

        if is_direct_command {
            log::info!("Detected direct DuckTape command: {}", trimmed);
            return self.process_command(input).await;
        }

        log::info!("Detected natural language input, routing to NLP pipeline");
        self.process_natural_language(input).await
    }

    /// Process a direct DuckTape command string.
    ///
    /// # Security
    /// - Only call this with validated, preprocessed input.
    /// - Do not expose this method to external crates except for CLI entry points.
    /// - All input should be sanitized and validated before execution.
    ///
    /// # Errors
    /// Returns an error if the command is invalid or execution fails.
    pub async fn process_command(&self, input: &str) -> Result<()> {
        log::info!("Processing command: {}", input);

        // Preprocess the input for normalization
        let preprocessed_input = crate::command_processor::preprocess_input(input);

        if Config::load()?.language_model.provider.is_none() {
            log::info!("Terminal Mode: Direct command processing only");
            // Try to parse with Clap first
            let command_args = match self.parse_command_string(&preprocessed_input) {
                Ok(args) => args,
                Err(_) => {
                    // Fall back to the legacy parser if Clap parsing fails
                    // This is useful for backward compatibility
                    CommandArgs::parse(&preprocessed_input)?
                }
            };
            return self.command_processor.execute(command_args).await;
        }

        // Create appropriate parser using factory
        let parser = ParserFactory::create_parser()?;

        // Process input through parser
        match parser.parse_input(&preprocessed_input).await? {
            crate::parser::ParseResult::CommandString(cmd) => {
                log::debug!("Processed command string: {}", cmd);

                // Try to parse with Clap first
                let command_args = match self.parse_command_string(&cmd) {
                    Ok(args) => args,
                    Err(_) => {
                        // Fall back to legacy parser
                        CommandArgs::parse(&cmd)?
                    }
                };

                // Execute the command
                self.command_processor.execute(command_args).await
            }
            crate::parser::ParseResult::StructuredCommand(args) => {
                log::debug!("Got pre-parsed command arguments: {:?}", args);

                // Execute directly with the structured command
                self.command_processor.execute(args).await
            }
        }
    }

    /// Process a natural language command string.
    ///
    /// # Security
    /// - Only call this with validated, preprocessed input.
    /// - Do not expose this method to external crates except for CLI entry points.
    /// - All input should be sanitized and validated before execution.
    ///
    /// # Errors
    /// Returns an error if the command is invalid or execution fails.
    pub async fn process_natural_language(&self, input: &str) -> Result<()> {
        println!("Processing natural language: '{}'", input);

        // Direct handling for note creation commands
        let input_lower = input.to_lowercase();
        let is_note_creation = input_lower.contains("create a note")
            || input_lower.contains("add a note")
            || input_lower.contains("note called")
            || input_lower.contains("create note");

        if is_note_creation {
            log::debug!("Detected note creation intent: {}", input);
            // Use the parse_natural_language_to_command function directly
            match crate::parser::utils::parse_natural_language_to_command(input) {
                Ok(command) => {
                    log::debug!("Generated note command: {}", command);
                    println!("Sanitized command: {}", command);

                    // Execute the generated command directly
                    match self.parse_command_string(&command) {
                        Ok(args) => {
                            log::debug!("Final parsed arguments for note command: {:?}", args);
                            return self.command_processor.execute(args).await;
                        }
                        Err(_) => {
                            // Fallback to legacy parsing
                            let mut args = CommandArgs::parse(&command)?;
                            args.args = args
                                .args
                                .into_iter()
                                .map(|arg| arg.trim_matches('"').to_string())
                                .collect();
                            log::debug!(
                                "Final parsed arguments for note command (legacy): {:?}",
                                args
                            );
                            return self.command_processor.execute(args).await;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse note creation command: {}", e);
                    // Continue with regular processing
                }
            }
        }

        // Create appropriate parser using factory
        let parser = ParserFactory::create_parser()?;

        // Process input through parser
        match parser.parse_input(input).await {
            Ok(crate::parser::ParseResult::CommandString(command)) => {
                println!("Translated to command: {}", command);

                // Always treat LLM output as natural language and run through sanitizer
                let mut sanitized_command = crate::parser::sanitize_nlp_command(&command);
                log::debug!("After sanitize_nlp_command: '{}'", sanitized_command);
                // Guarantee --zoom is present if needed
                let zoom_keywords =
                    ["zoom", "video call", "video meeting", "virtual meeting", "online meeting"];
                if zoom_keywords.iter().any(|kw| input.to_lowercase().contains(kw))
                    && !sanitized_command.contains("--zoom")
                {
                    sanitized_command.push_str(" --zoom");
                }
                println!("Sanitized command: {}", sanitized_command);
                log::debug!("Sanitized NLP command: {}", sanitized_command);

                // Apply enhancements for contacts and other special cases
                let mut enhanced_command = sanitized_command.clone();

                // Add contacts if this is a calendar command
                if enhanced_command.contains("calendar create") {
                    let contacts =
                        crate::parser::natural_language::utils::extract_contact_names(input);
                    if !contacts.is_empty() {
                        log::debug!("Found contacts in natural language input: {:?}", contacts);
                        let contacts_str = contacts.join(",");
                        if !enhanced_command.contains("--contacts") {
                            enhanced_command =
                                format!("{} --contacts \"{}\"", enhanced_command, contacts_str);
                            log::debug!("Enhanced command with contacts: {}", enhanced_command);
                        }
                    }
                }

                // Check if the generated command starts with ducktape
                if enhanced_command.starts_with("ducktape") {
                    match self.parse_command_string(&enhanced_command) {
                        Ok(args) => {
                            log::debug!("Final parsed arguments: {:?}", args);
                            self.command_processor.execute(args).await
                        }
                        Err(_) => {
                            let mut args = CommandArgs::parse(&enhanced_command)?;
                            args.args = args
                                .args
                                .into_iter()
                                .map(|arg| arg.trim_matches('"').to_string())
                                .collect();
                            log::debug!("Final parsed arguments (legacy): {:?}", args);
                            self.command_processor.execute(args).await
                        }
                    }
                } else {
                    println!(
                        "Generated command doesn't start with 'ducktape': {}",
                        enhanced_command
                    );
                    Ok(())
                }
            }
            Ok(crate::parser::ParseResult::StructuredCommand(args)) => {
                log::debug!("Got pre-parsed structured command: {:?}", args);
                println!("Processed command structure from natural language");
                self.command_processor.execute(args).await
            }
            Err(e) => {
                println!("Error processing natural language: {}", e);
                println!("Type 'help' for a list of available commands or try rephrasing.");
                Ok(())
            }
        }
    }

    /// Helper method to parse a command string using Clap instead of the deprecated CommandArgs::parse
    fn parse_command_string(&self, input: &str) -> Result<CommandArgs> {
        // Format the input into argv style for clap
        let args =
            shell_words::split(input).map_err(|e| anyhow!("Failed to parse command: {}", e))?;

        // Check if we have any arguments
        if args.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        // Parse using Clap
        let cli = match crate::cli::Cli::try_parse_from(&args) {
            Ok(cli) => cli,
            Err(e) => {
                // This is likely not a structured command but a natural language input
                return Err(anyhow!("Not a structured command: {}", e));
            }
        };

        // Convert from Clap command to CommandArgs
        crate::cli::convert_to_command_args(&cli)
            .ok_or_else(|| anyhow!("Failed to convert parsed command to CommandArgs"))
    }

    /// Called by the CLI entry point to execute a command from args
    pub async fn execute_from_args(&self, mut args: Vec<String>) -> Result<()> {
        // Check for natural language intent when the first argument isn't a recognized command
        if args.len() > 1 {
            let first_arg = &args[1].to_lowercase(); // args[0] is the program name

            // List of known DuckTape commands
            const KNOWN_COMMANDS: &[&str] = &[
                "calendar",
                "reminder",
                "todo",
                "note",
                "config",
                "contact",
                "help",
                "exit",
                "version",
                "ai",
                "--help",
                "--version",
            ];

            // Check if it's not a known command and not starting with a dash
            let is_unknown_command =
                !KNOWN_COMMANDS.iter().any(|cmd| first_arg == *cmd) && !first_arg.starts_with('-');

            // If not a recognized command, treat the entire input as natural language
            if is_unknown_command {
                // Reconstruct the natural language input (skip program name)
                let nl_input = args[1..].join(" ");
                log::info!("Unknown command detected, treating as natural language: {}", nl_input);
                return self.process_natural_language(&nl_input).await;
            }
        }

        // Normal command line argument processing
        let input = args.join(" ");
        self.process_command(&input).await
    }
}

#[allow(dead_code)] // Kept for future use when logging is expanded
pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}
