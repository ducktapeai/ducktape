use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

pub mod calendar_handler;
pub mod config_handler;
pub mod contact_groups_handler;
pub mod exit_handler;
pub mod help_handler;
pub mod notes_handler;
pub mod reminder_handler;
pub mod todo_handler;
pub mod utilities_handler;
pub mod version_handler;

/// Command line arguments structure
#[derive(Debug, Clone)]
pub struct CommandArgs {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, Option<String>>,
}

impl CommandArgs {
    pub fn new(command: String, args: Vec<String>, flags: HashMap<String, Option<String>>) -> Self {
        Self { command, args, flags }
    }

    /// Legacy method for parsing command arguments from a string
    /// This is deprecated in favor of using the Clap-based command line parser
    #[deprecated(note = "Use the Clap-based command line parser instead")]
    pub fn parse(input: &str) -> Result<Self> {
        let normalized_input = input.replace('\u{a0}', " ");
        debug!("Normalized input: {}", normalized_input);
        let tokens = shell_words::split(&normalized_input)
            .map_err(|e| anyhow!("Tokenization error: {}", e))?;
        debug!("Tokenized input: {:?}", tokens);
        if tokens.is_empty() {
            return Err(anyhow!("No command provided"));
        }
        let mut tokens_iter = tokens.into_iter();
        let first_token = tokens_iter.next().unwrap();
        let command = if first_token.eq_ignore_ascii_case("ducktape") {
            tokens_iter
                .next()
                .ok_or_else(|| anyhow!("No command provided after 'ducktape'"))?
                .to_lowercase()
        } else {
            first_token.to_lowercase()
        };
        let mut args = Vec::new();
        let mut flags = HashMap::new();
        let mut current_flag: Option<String> = None;
        for token in tokens_iter {
            if token.starts_with("--") {
                if let Some(flag_name) = current_flag.take() {
                    flags.insert(flag_name, None);
                }
                current_flag = Some(token[2..].to_string());
                debug!("Found flag: --{}", current_flag.as_ref().unwrap());
            } else if let Some(flag_name) = current_flag.take() {
                debug!("Flag --{} has value: '{}'", flag_name, token);
                flags.insert(flag_name, Some(token));
            } else {
                args.push(token);
            }
        }
        if let Some(flag_name) = current_flag {
            flags.insert(flag_name, None);
        }
        debug!("Final parsed command: {:?}, args: {:?}, flags: {:?}", command, args, flags);
        Ok(CommandArgs { command, args, flags })
    }
}

/// Standardized input preprocessing function
pub fn preprocess_input(input: &str) -> String {
    input.trim().to_lowercase()
}

pub trait CommandHandler: Debug + Send + Sync {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>>;
    fn can_handle(&self, command: &str) -> bool;
}

#[derive(Debug)]
pub struct CommandProcessor {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn CommandHandler>> = vec![
            Box::new(calendar_handler::CalendarHandler),
            Box::new(todo_handler::TodoHandler),
            Box::new(notes_handler::NotesHandler),
            Box::new(config_handler::ConfigHandler),
            Box::new(utilities_handler::UtilitiesHandler),
            Box::new(contact_groups_handler::ContactGroupsHandler),
            Box::new(version_handler::VersionHandler),
            Box::new(help_handler::HelpHandler),
            Box::new(exit_handler::ExitHandler),
            Box::new(reminder_handler::ReminderHandler),
        ];
        Self { handlers }
    }
    pub async fn execute(&self, args: CommandArgs) -> Result<()> {
        debug!("Attempting to execute command: {}", args.command);
        debug!("Parsed arguments: {:?}", args.args);
        debug!("Parsed flags: {:?}", args.flags);
        let command_name = args.command.clone();
        let args_debug = format!("{:?}", args.args);
        for handler in &self.handlers {
            if handler.can_handle(&command_name) {
                info!("Executing command '{}' with arguments: {}", command_name, args_debug);
                let args_to_use = args.clone();
                match handler.execute(args_to_use).await {
                    Ok(()) => {
                        debug!("Command '{}' executed successfully", command_name);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Failed to execute command '{}': {:?}", command_name, e);
                        return Err(e);
                    }
                }
            }
        }
        warn!("Unrecognized command: {}", command_name);
        println!("Unrecognized command. Type 'help' for a list of available commands.");
        Ok(())
    }
}

impl Default for CommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}
