use anyhow::Result;
use crate::commands::{CommandArgs, CommandExecutor};
use std::future::Future;
use std::pin::Pin;
use crate::config::{Config, LLMProvider};

pub struct ConfigCommand;

impl CommandExecutor for ConfigCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            handle_config_command(args)
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "config"
    }
}

fn handle_config_command(args: CommandArgs) -> Result<()> {
    if args.args.is_empty() {
        println!("Usage: ducktape config <setting> <value>");
        println!("Available settings:");
        println!("  llm - Set the language model provider");
        println!("\nExample: ducktape config llm grok");
        return Ok(());
    }
    
    match args.args[0].as_str() {
        "show" => {
            let config = Config::load()?;
            println!("\nCurrent Configuration:");
            println!("  Language Model Provider: {:?}", config.language_model.provider);
            println!("\nCalendar Settings:");
            println!("  Default Calendar: {}", config.calendar.default_calendar.as_deref().unwrap_or("None"));
            println!("  Default Reminder: {} minutes", config.calendar.default_reminder_minutes.unwrap_or(15));
            println!("  Default Duration: {} minutes", config.calendar.default_duration_minutes.unwrap_or(60));
            println!("\nTodo Settings:");
            println!("  Default List: {}", config.todo.default_list.as_deref().unwrap_or("None"));
            println!("  Default Reminder: {}", if config.todo.default_reminder { "Enabled" } else { "Disabled" });
            println!("\nNotes Settings:");
            println!("  Default Folder: {}", config.notes.default_folder.as_deref().unwrap_or("None"));
            return Ok(());
        },
        "llm" => {
            if args.args.len() < 2 {
                println!("Usage: ducktape config llm <provider>");
                println!("Available providers: openai, grok, deepseek");
                println!("\nExample: ducktape config llm grok");
                return Ok(());
            }
            
            let provider = match args.args[1].to_lowercase().as_str() {
                "openai" => LLMProvider::OpenAI,
                "grok" => LLMProvider::Grok,
                "deepseek" => LLMProvider::DeepSeek,
                _ => {
                    println!("Error: Invalid provider '{}'", args.args[1]);
                    println!("Available providers: openai, grok, deepseek");
                    println!("\nExample: ducktape config llm grok");
                    return Ok(());
                }
            };
            
            let mut config = Config::load()?;
            config.language_model.provider = provider;
            config.save()?;
            println!("Language model provider updated to: {}", args.args[1]);
            return Ok(());
        }
        _ => {
            println!("Unknown config setting '{}'. Available settings:", args.args[0]);
            println!("  llm - Set the language model provider");
            println!("  show - Display current configuration");
            println!("\nExample: ducktape config llm grok");
            return Ok(());
        }
    }
}