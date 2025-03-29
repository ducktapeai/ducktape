use crate::commands::calendar;
use crate::commands::config;
use crate::commands::contacts;
use crate::commands::help;
use crate::commands::notes;
use crate::commands::todo;
use crate::commands::utilities;
use crate::commands::version;
use crate::commands::{CommandArgs, CommandExecutor};
use crate::config::{Config, LLMProvider};
use anyhow::{Result, anyhow};
use rustyline::DefaultEditor;

pub struct Application {
    command_executors: Vec<Box<dyn CommandExecutor>>,
}

impl Application {
    pub fn new() -> Self {
        let executors: Vec<Box<dyn CommandExecutor>> = vec![
            Box::new(help::HelpCommand),
            Box::new(calendar::CalendarCommand),
            Box::new(todo::TodoCommand),
            Box::new(notes::NotesCommand),
            Box::new(config::ConfigCommand),
            Box::new(utilities::UtilitiesCommand),
            Box::new(contacts::ContactGroupsCommand),
            Box::new(version::VersionCommand),
        ];

        Self { command_executors: executors }
    }

    pub async fn run(&self) -> Result<()> {
        log::info!("Starting DuckTape Terminal");
        let config = Config::load()?;

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
                    if let Err(err) = self.process_input(&line).await {
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

        let mut rl = DefaultEditor::new()?;

        println!("Welcome to DuckTape Terminal! Type 'help' for commands.");
        let prompt = "🦆 ";

        loop {
            match rl.readline(prompt) {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    if let Err(err) = self.process_input(&line).await {
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

    async fn process_input(&self, input: &str) -> Result<()> {
        if input.starts_with("ducktape") {
            let args = CommandArgs::parse(input)?;
            if args.command == "exit" || args.command == "quit" {
                std::process::exit(0);
            }
            self.execute_command(args).await
        } else {
            // If input doesn't start with "ducktape", treat as natural language
            self.process_natural_language(input).await
        }
    }

    /// Process a direct command string - now public for CLI use
    pub async fn process_command(&self, input: &str) -> Result<()> {
        log::info!("Processing command: {}", input);

        // Add "ducktape" prefix if missing for consistency
        let normalized_input = if !input.trim().starts_with("ducktape") {
            format!("ducktape {}", input)
        } else {
            input.to_string()
        };

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
                LLMProvider::OpenAI => {
                    match crate::openai_parser::parse_natural_language(&normalized_input).await {
                        Ok(command) => command,
                        Err(e) => return Err(anyhow!("OpenAI parser error: {}", e)),
                    }
                }
                LLMProvider::Grok => {
                    match crate::grok_parser::parse_natural_language(&normalized_input).await {
                        Ok(command) => command,
                        Err(e) => return Err(anyhow!("Grok parser error: {}", e)),
                    }
                }
                LLMProvider::DeepSeek => {
                    match crate::deepseek_parser::parse_natural_language(&normalized_input).await {
                        Ok(command) => command,
                        Err(e) => return Err(anyhow!("DeepSeek parser error: {}", e)),
                    }
                }
            }
        };

        log::info!("Processed command: {}", processed_input);

        // Parse the processed command into arguments
        match CommandArgs::parse(&processed_input) {
            Ok(args) => self.execute_command(args).await,
            Err(e) => Err(anyhow!("Failed to parse command: {}", e)),
        }
    }

    async fn execute_command(&self, args: CommandArgs) -> Result<()> {
        for executor in &self.command_executors {
            if executor.can_handle(&args.command) {
                return executor.execute(args).await;
            }
        }

        println!("Unrecognized command. Type 'help' for a list of available commands.");
        Ok(())
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
                    self.execute_command(args).await
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
