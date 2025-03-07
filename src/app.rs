use crate::commands::{CommandArgs, CommandExecutor};
use crate::commands::{calendar, config, help, notes, todo, utilities};
use anyhow::Result;
use rustyline::DefaultEditor;
use crate::config::Config;

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
        let args = CommandArgs::parse(input)?;

        if args.command == "exit" || args.command == "quit" {
            std::process::exit(0);
        }

        self.execute_command(args).await
    }

    async fn execute_command(&self, args: CommandArgs) -> Result<()> {
        for executor in &self.command_executors {
            if executor.can_handle(&args.command) {
                return executor.execute(args).await;
            }
        }
        
        // If no executor found, try natural language processing
        self.process_natural_language(&args.command).await
    }

    async fn process_natural_language(&self, _input: &str) -> Result<()> {
        println!("Unrecognized command. Type 'help' for a list of available commands.");
        Ok(())
    }
}