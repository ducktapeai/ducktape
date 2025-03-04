use anyhow::Result;

use crate::commands::{
    calendar::CalendarCommand,
    config::ConfigCommand,
    help::HelpCommand,
    notes::NotesCommand,
    todo::TodoCommand,
    utilities::UtilitiesCommand,
    CommandArgs,
    CommandExecutor,
};
use crate::config::{Config, LLMProvider};
use rustyline::DefaultEditor;

pub struct Application {
    command_executors: Vec<Box<dyn CommandExecutor>>,
}

impl Application {
    pub fn new() -> Self {
        let executors: Vec<Box<dyn CommandExecutor>> = vec![
            Box::new(HelpCommand),
            Box::new(CalendarCommand),
            Box::new(TodoCommand),
            Box::new(NotesCommand),
            Box::new(ConfigCommand),
            Box::new(UtilitiesCommand),
        ];

        Self {
            command_executors: executors,
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Initialize logging
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format(|buf, record| {
                use chrono::Local;
                use std::io::Write;
                writeln!(
                    buf,
                    "{} [{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .init();

        log::info!("Starting DuckTape Terminal");
        let _config = Config::load()?;
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
                },
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                },
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn process_input(&self, input: &str) -> Result<()> {
        // First check if it's a ducktape command or natural language
        if !input.trim().to_lowercase().starts_with("ducktape") && !input.trim().is_empty() {
            // Process as natural language
            return self.process_natural_language(input).await;
        }
        
        // Process as a structured command
        match CommandArgs::parse(input) {
            Ok(args) => self.execute_command(args).await,
            Err(e) => {
                println!("Error parsing command: {}", e);
                Ok(())
            }
        }
    }

    async fn process_natural_language(&self, input: &str) -> Result<()> {
        let config = Config::load()?;
        
        let response = match config.language_model.provider {
            LLMProvider::OpenAI => crate::openai_parser::parse_natural_language(input).await,
            LLMProvider::Grok => crate::grok_parser::parse_natural_language(input).await,
            LLMProvider::DeepSeek => crate::deepseek_parser::parse_natural_language(input).await,
        };

        match response {
            Ok(parsed_command) => {
                if parsed_command.to_lowercase().contains("please provide") {
                    println!("{}", parsed_command);
                    let mut rl = rustyline::DefaultEditor::new()?;
                    let additional = rl.readline(">> Additional details: ")?;
                    let combined = format!("{} {}", input, additional);
                    
                    let new_response = match config.language_model.provider {
                        LLMProvider::OpenAI => crate::openai_parser::parse_natural_language(&combined).await?,
                        LLMProvider::Grok => crate::grok_parser::parse_natural_language(&combined).await?,
                        LLMProvider::DeepSeek => crate::deepseek_parser::parse_natural_language(&combined).await?,
                    };
                    
                    println!("{}", new_response);
                    let future = self.process_input(&new_response);
                    Box::pin(future).await
                } else {
                    println!("{}", parsed_command);
                    let future = self.process_input(&parsed_command);
                    Box::pin(future).await
                }
            }
            Err(e) => {
                println!("Error processing natural language: {}", e);
                Ok(())
            }
        }
    }

    async fn execute_command(&self, args: CommandArgs) -> Result<()> {
        for executor in &self.command_executors {
            if executor.can_handle(&args.command) {
                return executor.execute(args).await;
            }
        }

        println!("Unknown command '{}'. Type 'ducktape help' for available commands.", args.command);
        Ok(())
    }
}