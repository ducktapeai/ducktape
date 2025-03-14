use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Command line arguments structure
#[derive(Debug)]
pub struct CommandArgs {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, Option<String>>,
}

impl CommandArgs {
    pub fn parse(input: &str) -> Result<Self> {
        // Normalize input by replacing non-breaking spaces and multiple spaces with a single space
        let normalized_input = input
            .replace('\u{a0}', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        log::debug!("Normalized input: {}", normalized_input);

        // Handle exit commands
        if normalized_input.eq_ignore_ascii_case("exit") || 
           normalized_input.eq_ignore_ascii_case("quit") || 
           normalized_input.eq_ignore_ascii_case("ducktape exit") ||
           normalized_input.eq_ignore_ascii_case("ducktape quit") {
            return Ok(CommandArgs {
                command: "exit".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        // Special case for help commands
        if normalized_input.eq_ignore_ascii_case("help") || 
           normalized_input.eq_ignore_ascii_case("ducktape help") ||
           normalized_input.eq_ignore_ascii_case("ducktape --help") ||
           normalized_input.eq_ignore_ascii_case("ducktape -h") ||
           normalized_input.eq_ignore_ascii_case("ducktape --h") {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = normalized_input.chars().peekable();
        let mut escaped = false;

        while let Some(c) = chars.next() {
            match c {
                '\\' if !escaped => {
                    escaped = true;
                }
                '"' if !escaped => {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                ' ' if !in_quotes && !escaped => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    if escaped && c != '"' {
                        current.push('\\');
                    }
                    current.push(c);
                    escaped = false;
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            return Err(anyhow::anyhow!("No command provided"));
        }

        log::debug!("Parsed parts after normalization: {:?}", parts);

        // Special case for help command
        if parts.len() == 1 && (parts[0].eq_ignore_ascii_case("--help") || parts[0].eq_ignore_ascii_case("-h")) {
            return Ok(CommandArgs {
                command: "help".to_string(),
                args: vec![],
                flags: HashMap::new(),
            });
        }

        // Check for and remove "ducktape" prefix, being more lenient with case and whitespace
        let first_part = parts[0].trim();
        if !first_part.eq_ignore_ascii_case("ducktape") {
            log::debug!("First part '{}' does not match 'ducktape'", first_part);
            return Err(anyhow::anyhow!("Commands must start with 'ducktape'"));
        }
        parts.remove(0); // Remove "ducktape"

        if parts.is_empty() {
            return Err(anyhow::anyhow!("No command provided after 'ducktape'"));
        }

        let command = parts.remove(0).to_lowercase();
        let mut args = Vec::new();
        let mut flags = HashMap::new();
        let mut i = 0;

        while i < parts.len() {
            if parts[i].starts_with("--") {
                let flag = parts[i].clone();
                if i + 1 < parts.len() && !parts[i + 1].starts_with("--") {
                    flags.insert(flag, Some(parts[i + 1].clone()));
                    i += 1;
                } else {
                    flags.insert(flag, None);
                }
            } else {
                args.push(parts[i].clone());
            }
            i += 1;
        }

        log::debug!("Parsed command: {:?}, args: {:?}, flags: {:?}", command, args, flags);

        Ok(CommandArgs {
            command,
            args,
            flags,
        })
    }
}

// Command executor trait for handling commands
pub trait CommandExecutor {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>>;
    fn can_handle(&self, command: &str) -> bool;
}

// Re-export submodules
pub mod calendar;
pub mod config;
pub mod contacts;
pub mod help;
pub mod notes;
pub mod todo;
pub mod utilities;

// Public function to print help
pub fn print_help() -> Result<()> {
    help::print_help()
}