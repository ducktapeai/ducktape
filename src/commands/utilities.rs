use anyhow::{anyhow, Result};
use crate::commands::{CommandArgs, CommandExecutor};
use std::future::Future;
use std::pin::Pin;
use std::io::{self, Write};
use crate::state::StateManager;

pub struct UtilitiesCommand;

impl CommandExecutor for UtilitiesCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.command.as_str() {
                "cleanup" => handle_cleanup(args),
                "zoom-setup" => handle_zoom_setup(args),
                _ => {
                    println!("Unknown utility command");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "cleanup" | "zoom-setup")
    }
}

fn handle_cleanup(_args: CommandArgs) -> Result<()> {
    let manager = StateManager::new()?;
    manager.vacuum()?;
    println!("✅ Storage cleaned up and compacted.");
    Ok(())
}

fn handle_zoom_setup(_args: CommandArgs) -> Result<()> {
    println!("Zoom API Setup");
    println!("==============");
    println!("Enter your Zoom Server-to-Server OAuth credentials to enable Zoom meeting creation.");
    println!("These values will be set as environment variables.\n");

    // Handle Account ID
    print!("Account ID: ");
    io::stdout().flush().map_err(|e| anyhow!("Failed to flush stdout: {}", e))?;
    let mut account_id = String::new();
    io::stdin()
        .read_line(&mut account_id)
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;
    let account_id = account_id.trim().to_string();

    if account_id.is_empty() {
        return Err(anyhow!("Account ID cannot be empty"));
    }

    // Handle Client ID
    print!("Client ID: ");
    io::stdout().flush().map_err(|e| anyhow!("Failed to flush stdout: {}", e))?;
    let mut client_id = String::new();
    io::stdin()
        .read_line(&mut client_id)
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;
    let client_id = client_id.trim().to_string();

    if client_id.is_empty() {
        return Err(anyhow!("Client ID cannot be empty"));
    }

    // Handle Client Secret
    print!("Client Secret: ");
    io::stdout().flush().map_err(|e| anyhow!("Failed to flush stdout: {}", e))?;
    let mut client_secret = String::new();
    io::stdin()
        .read_line(&mut client_secret)
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;
    let client_secret = client_secret.trim().to_string();

    if client_secret.is_empty() {
        return Err(anyhow!("Client Secret cannot be empty"));
    }

    println!("\nTo use these credentials, set the following environment variables:");
    println!("  export ZOOM_ACCOUNT_ID='{}'", account_id);
    println!("  export ZOOM_CLIENT_ID='{}'", client_id);
    println!("  export ZOOM_CLIENT_SECRET='{}'", client_secret);
    println!("\nYou can add these lines to your shell's startup file (e.g., .bashrc, .zshrc)");
    println!("to make them persistent across terminal sessions.");

    Ok(())
}

// Helper function to parse duration string like "1h30m" into minutes
#[allow(dead_code)]
pub fn parse_duration_to_minutes(duration: &str) -> Result<i32> {
    let mut total_minutes = 0;
    let mut current_number = String::new();
    
    for c in duration.chars() {
        if c.is_digit(10) {
            current_number.push(c);
        } else if c == 'h' || c == 'H' {
            if current_number.is_empty() {
                return Err(anyhow!("Invalid duration format: missing number before 'h'"));
            }
            let hours = current_number.parse::<i32>()?;
            total_minutes += hours * 60;
            current_number.clear();
        } else if c == 'm' || c == 'M' {
            if current_number.is_empty() {
                return Err(anyhow!("Invalid duration format: missing number before 'm'"));
            }
            let minutes = current_number.parse::<i32>()?;
            total_minutes += minutes;
            current_number.clear();
        } else if !c.is_whitespace() {
            return Err(anyhow!("Invalid character in duration: '{}'", c));
        }
    }
    
    // If we have a number without a unit at the end, assume it's minutes
    if !current_number.is_empty() {
        let minutes = current_number.parse::<i32>()?;
        total_minutes += minutes;
    }
    
    if total_minutes == 0 {
        return Err(anyhow!("Duration must be greater than 0 minutes"));
    }
    
    Ok(total_minutes)
}