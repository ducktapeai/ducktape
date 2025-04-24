use crate::command_processor::{CommandArgs, CommandProcessor};
use crate::config::{Config, LLMProvider};
use crate::parser::{ParseResult, ParserFactory};
use anyhow::{Result, anyhow};
use clap::Parser as ClapParser;
use log::info;
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

    async fn process_input(&self, input: &str, use_natural_language: bool) -> Result<()> {
        log::debug!("Inside process_input: use_natural_language = {}", use_natural_language);

        // Check for direct exit command regardless of mode
        let preprocessed = crate::command_processor::preprocess_input(input);
        if preprocessed == "exit"
            || preprocessed == "quit"
            || preprocessed == "ducktape exit"
            || preprocessed == "ducktape quit"
        {
            log::info!("Exit command detected, bypassing language processing");
            // Create command args for exit command
            let command_args = crate::command_processor::CommandArgs::new(
                "exit".to_string(),
                vec![],
                std::collections::HashMap::new(),
            );
            return self.command_processor.execute(command_args).await;
        }

        if !use_natural_language {
            log::info!("Skipping natural language processing as Terminal Mode is enabled");
            println!(
                "Note: To enable natural language processing, update and enable the 'provider' field in the 'language_model' section of your config.toml file."
            );
            return self.process_command(input).await;
        }

        log::info!("Proceeding with natural language processing");

        // Proceed with natural language processing if enabled
        self.process_natural_language(input).await
    }

    /// Process a direct command string - now public for CLI use
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

    /// Process the input from natural language
    async fn process_natural_language(&self, input: &str) -> Result<()> {
        info!("Proceeding with natural language processing");
        println!("Processing natural language: '{}'", input);

        // Check for common event creation patterns directly before parsing
        let input_lower = input.to_lowercase();
        if input_lower.contains("create an event")
            || input_lower.contains("schedule a meeting")
            || (input_lower.contains("create") && input_lower.contains("event"))
            || (input_lower.contains("schedule") && input_lower.contains("meeting"))
        {
            log::debug!("Detected calendar event creation intent directly in input");

            // Extract title if available
            let mut title = "Event";
            if input_lower.contains("called") {
                if let Some(pos) = input.find("called") {
                    let rest = &input[pos + 6..];
                    if let Some(end_pos) = rest.find(" at ") {
                        title = &rest[..end_pos].trim();
                    } else if let Some(end_pos) = rest.find(" and ") {
                        title = &rest[..end_pos].trim();
                    } else {
                        title = rest.trim();
                    }
                }
            }

            // Determine date and time
            let now = chrono::Local::now();
            let date = now.format("%Y-%m-%d").to_string();

            // Default times
            let mut start_time = "09:00";
            let mut end_time = "10:00";

            // Check for time patterns
            if input_lower.contains("tonight") && input_lower.contains("10pm") {
                start_time = "22:00";
                end_time = "23:00";
            } else if input_lower.contains(" at ") {
                if let Some(time_pos) = input_lower.find(" at ") {
                    let time_part = &input_lower[time_pos + 4..];
                    if time_part.contains("10pm") || time_part.contains("10 pm") {
                        start_time = "22:00";
                        end_time = "23:00";
                    }
                }
            }

            // Check for contacts
            let mut contact_arg = String::new();
            if input_lower.contains("with") || input_lower.contains("invite") {
                if let Some(contact_name) = self.extract_contact_name(input) {
                    contact_arg = format!(" --contacts \"{}\"", contact_name);
                }
            }

            // Create calendar command directly
            let calendar_cmd = format!(
                "ducktape calendar create \"{}\" {} {} {} \"Work\"{}",
                title, date, start_time, end_time, contact_arg
            );

            log::info!("Created direct calendar command: {}", calendar_cmd);

            // Execute the command
            let command_args = match self.parse_command_string(&calendar_cmd) {
                Ok(args) => args,
                Err(_) => {
                    // Fall back to legacy parser if needed
                    CommandArgs::parse(&calendar_cmd)?
                }
            };

            return self.command_processor.execute(command_args).await;
        }

        // Continue with normal processing for other cases
        let parser = ParserFactory::create_parser()?;
        let parse_result = parser.parse_input(input).await?;

        // Rest of the function remains the same
        match parse_result {
            ParseResult::CommandString(command) => {
                println!("Translated to command: {}", command);

                // Make sure the command starts with "ducktape"
                let sanitized_command = if !command.starts_with("ducktape") {
                    format!("ducktape {}", command)
                } else {
                    command
                };

                println!("Sanitized command: {}", sanitized_command);

                // Try to convert to a proper calendar or todo command if needed
                if sanitized_command.contains("calendar") || sanitized_command.contains("todo") {
                    // Handle specific command patterns
                    if sanitized_command.contains("create an event")
                        || sanitized_command.contains("schedule")
                        || sanitized_command.contains("add event")
                    {
                        // Extract details from command
                        let mut title = "Event";
                        if let Some(pos) = sanitized_command.find("called") {
                            let rest = &sanitized_command[pos + 6..];
                            if let Some(end_pos) = rest.find(" at ") {
                                title = &rest[..end_pos].trim();
                            } else if let Some(end_pos) = rest.find(" on ") {
                                title = &rest[..end_pos].trim();
                            }
                        }

                        // Create a proper calendar command
                        let now = chrono::Local::now();
                        let date = now.format("%Y-%m-%d").to_string();
                        let tomorrow =
                            (now + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

                        // Extract time if available
                        let mut start_time = "09:00";
                        let mut end_time = "10:00";
                        if sanitized_command.contains(" at ") {
                            if sanitized_command.contains("tonight")
                                || sanitized_command.contains(" at 10pm")
                            {
                                start_time = "22:00";
                                end_time = "23:00";
                            }
                        }

                        // Determine date
                        let event_date =
                            if sanitized_command.contains("tomorrow") { tomorrow } else { date };

                        // Handle contacts
                        let mut contacts_flag = String::new();
                        if sanitized_command.contains("with")
                            || sanitized_command.contains("invite")
                        {
                            // Try to extract contact name - this is simplistic but works for testing
                            if let Some(contact_name) =
                                self.extract_contact_name(&sanitized_command)
                            {
                                contacts_flag = format!(" --contacts \"{}\"", contact_name);
                            }
                        }

                        // Create reformatted command
                        let calendar_cmd = format!(
                            "ducktape calendar create \"{}\" {} {} {} \"Work\"{}",
                            title, event_date, start_time, end_time, contacts_flag
                        );

                        // Try to parse with Clap
                        match self.parse_command_string(&calendar_cmd) {
                            Ok(args) => {
                                log::debug!(
                                    "Parsed calendar command as structured command: {:?}",
                                    args
                                );
                                return self.command_processor.execute(args).await;
                            }
                            Err(_) => {
                                // Fall back to legacy parser
                                let args = CommandArgs::parse(&calendar_cmd)?;
                                log::debug!(
                                    "Parsed calendar command with legacy parser: {:?}",
                                    args
                                );
                                return self.command_processor.execute(args).await;
                            }
                        }
                    } else if sanitized_command.contains("reminder")
                        || sanitized_command.contains("todo")
                    {
                        // Similar logic for todos...
                    }
                }

                // If we get here, try to parse the original command
                if sanitized_command.starts_with("ducktape") {
                    match self.parse_command_string(&sanitized_command) {
                        Ok(args) => self.command_processor.execute(args).await,
                        Err(_) => {
                            // Fall back to legacy parser if Clap fails
                            let mut args = CommandArgs::parse(&sanitized_command)?;

                            // Further sanitize individual arguments to remove any remaining quotes
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
                        sanitized_command
                    );
                    Ok(())
                }
            }
            ParseResult::StructuredCommand(args) => {
                log::debug!("Got pre-parsed structured command: {:?}", args);
                println!("Processed command structure from natural language");

                // Execute directly with the structured command
                self.command_processor.execute(args).await
            }
        }
    }

    /// Helper function to extract contact names from natural language input
    fn extract_contact_name(&self, input: &str) -> Option<String> {
        // Simple extraction logic - look for patterns like "with Person" or "invite Person"
        if let Some(pos) = input.find(" with ") {
            return Some(input[pos + 6..].trim().to_string());
        }

        if let Some(pos) = input.find(" invite ") {
            return Some(input[pos + 8..].trim().to_string());
        }

        None
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
