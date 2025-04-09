use crate::command_processor::{CommandArgs, CommandProcessor};
use crate::config::{Config, LLMProvider};
use anyhow::{Result, anyhow};
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
        let mut config = Config::load()?;

        let use_natural_language = config.language_model.provider.is_some();
        log::debug!(
            "Provider: {:?}, use_natural_language: {}",
            config.language_model.provider,
            use_natural_language
        );

        match Config::load()?.language_model.provider {
            Some(LLMProvider::OpenAI) => {
                log::info!("Using OpenAI for natural language processing");
            }
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
            if let Err(e) = crate::api_server::start_api_server(config_clone).await {
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

        if !use_natural_language {
            log::info!("Skipping natural language processing as Terminal Mode is enabled");
            println!("Note: To enable natural language processing, update and enable the 'provider' field in the 'language_model' section of your config.toml file.");
            return self.process_command(input).await;
        }

        log::info!("Proceeding with natural language processing");

        // Proceed with natural language processing if enabled
        self.process_natural_language(input).await
    }

    /// Process a direct command string - now public for CLI use
    pub async fn process_command(&self, input: &str) -> Result<()> {
        log::info!("Processing command: {}", input);

        if Config::load()?.language_model.provider.is_none() {
            log::info!("Terminal Mode: Direct command processing only");
            return self.command_processor.execute(CommandArgs::parse(input)?).await;
        }

        // Add "ducktape" prefix if missing for consistency
        let normalized_input = if !input.trim().starts_with("ducktape") {
            format!("ducktape {}", input)
        } else {
            input.to_string()
        };
        log::debug!("Normalized input: {}", normalized_input);

        // Determine if this is a natural language command that needs AI processing
        // or a direct command with parameters
        let processed_input = if normalized_input.starts_with("ducktape calendar")
            || normalized_input.starts_with("ducktape todo")
            || normalized_input.starts_with("ducktape note")
        {
            normalized_input
        } else {
            // For natural language, we need to process via one of the AI models
            match Config::load()?.language_model.provider {
                Some(LLMProvider::OpenAI) => {
                    match crate::openai_parser::parse_natural_language(&normalized_input).await {
                        Ok(command) => command,
                        Err(e) => return Err(anyhow!("OpenAI parser error: {}", e)),
                    }
                }
                Some(LLMProvider::Grok) => {
                    match crate::grok_parser::parse_natural_language(&normalized_input).await {
                        Ok(command) => command,
                        Err(e) => return Err(anyhow!("Grok parser error: {}", e)),
                    }
                }
                Some(LLMProvider::DeepSeek) => {
                    match crate::deepseek_parser::parse_natural_language(&normalized_input).await {
                        Ok(command) => command,
                        Err(e) => return Err(anyhow!("DeepSeek parser error: {}", e)),
                    }
                }
                None => {
                    return Err(anyhow!("No language model provider configured"));
                }
            }
        };

        log::info!("Processed command: {}", processed_input);

        // Ensure calendar event commands have proper end times
        let final_command = if processed_input.contains("calendar create")
            || processed_input.contains("calendar add")
        {
            // Check if it has a start time but no end time
            let parts: Vec<&str> = processed_input.split_whitespace().collect();
            let mut has_start_time = false;
            let mut has_end_time = false;
            let mut start_time_index = 0;

            // Find start time position and check if end time exists
            for (i, part) in parts.iter().enumerate() {
                if part.contains(':') && i > 2 {
                    // Potential time format (avoid matching in command prefix)
                    if !has_start_time {
                        has_start_time = true;
                        start_time_index = i;
                    } else if i == start_time_index + 1 {
                        has_end_time = true;
                        break;
                    }
                }
            }

            // If there's a start time but no end time, add an end time 1 hour later
            if has_start_time
                && !has_end_time
                && start_time_index > 0
                && start_time_index < parts.len()
            {
                let start_time = parts[start_time_index];
                if let Some((hours_str, minutes_str)) = start_time.split_once(':') {
                    if let (Ok(hours), Ok(_minutes)) =
                        (hours_str.parse::<u32>(), minutes_str.parse::<u32>())
                    {
                        // Calculate end time one hour later
                        let end_hours = (hours + 1) % 24;
                        let end_time = format!("{}:{}", end_hours, minutes_str);

                        let mut new_parts = parts.clone();
                        new_parts.insert(start_time_index + 1, &end_time);
                        let fixed_command = new_parts.join(" ");
                        log::info!("Added missing end time. Fixed command: {}", fixed_command);
                        fixed_command
                    } else {
                        processed_input.clone()
                    }
                } else {
                    processed_input.clone()
                }
            } else {
                processed_input.clone()
            }
        } else {
            processed_input.clone()
        };

        // Parse the processed command into arguments
        match CommandArgs::parse(&final_command) {
            Ok(args) => self.command_processor.execute(args).await,
            Err(e) => Err(anyhow!("Failed to parse command: {}", e)),
        }
    }

    async fn process_natural_language(&self, input: &str) -> Result<()> {
        use crate::grok_parser;

        println!("Processing natural language: '{}'", input);

        match grok_parser::parse_natural_language(input).await {
            Ok(command) => {
                println!("Translated to command: {}", command);

                // Check if the generated command starts with ducktape
                if command.starts_with("ducktape") {
                    let args = CommandArgs::parse(&command)?;
                    self.command_processor.execute(args).await
                } else {
                    println!("Generated command doesn't start with 'ducktape': {}", command);
                    Ok(())
                }
            }
            Err(e) => {
                println!("Error processing natural language: {}", e);
                println!("Type 'help' for a list of available commands or try rephrasing.");
                Ok(())
            }
        }
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
