use anyhow::Result;
use ducktape::command_processor::{CommandArgs, CommandProcessor};
use log::{debug, info};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load environment variables from .env file
    if let Err(e) = load_env_file() {
        eprintln!("Warning: {}", e);
        info!("Warning: {}", e);
    }

    // Parse command line arguments directly, preserving quoted strings
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    // Build input string with proper handling of quoted arguments
    let mut input = String::from("ducktape ");
    let mut i = 0;

    // Process all arguments and preserve quotes for multi-word values
    while i < raw_args.len() {
        let arg = &raw_args[i];

        // Special handling for flags that might have multi-word values
        if arg.starts_with("--") && i + 1 < raw_args.len() && !raw_args[i + 1].starts_with("--") {
            // This is a flag with a value
            let flag_name = arg;
            let flag_value = &raw_args[i + 1];

            // If the value contains spaces, wrap it in quotes
            if flag_value.contains(' ') {
                input.push_str(&format!("{} \"{}\" ", flag_name, flag_value));
            } else {
                input.push_str(&format!("{} {} ", flag_name, flag_value));
            }
            i += 2;
        } else {
            // Regular argument
            if arg.contains(' ') && !arg.starts_with('"') && !arg.ends_with('"') {
                input.push_str(&format!("\"{}\" ", arg));
            } else {
                input.push_str(&format!("{} ", arg));
            }
            i += 1;
        }
    }

    debug!("Processed command input: {}", input);

    // Parse the arguments using our command processor
    match CommandArgs::parse(&input) {
        Ok(args) => {
            debug!("Parsed args: {:?}", args);
            let processor = CommandProcessor::new();
            processor.execute(args).await?;
        }
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Helper function to load environment variables from the .env file
/// Prioritizes loading Zoom credentials needed for calendar events with Zoom integration
fn load_env_file() -> Result<()> {
    // Try to load from .env file in the current directory
    if Path::new(".env").exists() {
        dotenvy::from_path(".env")?;
        info!("Loaded environment variables from .env file in current directory");
    }
    // Then try to load from the project root directory
    else if Path::new("/Users/shaunstuart/RustroverProjects/ducktape/.env").exists() {
        dotenvy::from_path("/Users/shaunstuart/RustroverProjects/ducktape/.env")?;
        info!("Loaded environment variables from .env file in project root");
    }
    // Finally try any .env file in the path
    else {
        match dotenvy::dotenv() {
            Ok(_) => info!("Loaded environment variables from .env file"),
            Err(e) => return Err(anyhow::anyhow!("Failed to load .env file: {}", e)),
        }
    }

    // Verify that Zoom credentials are available in the environment
    if env::var("ZOOM_ACCOUNT_ID").is_err()
        || env::var("ZOOM_CLIENT_ID").is_err()
        || env::var("ZOOM_CLIENT_SECRET").is_err()
    {
        info!("One or more required Zoom credentials are missing from environment");
        return Err(anyhow::anyhow!(
            "One or more required Zoom credentials are missing in .env file"
        ));
    }

    info!("Successfully loaded Zoom credentials from environment");
    Ok(())
}

/// Helper function to properly process contact names from command string
/// Handles both comma-separated lists and multi-word contact names
fn process_contact_string(contacts_str: &str) -> Vec<&str> {
    // Check if the contact string contains spaces but no commas
    // This handles the case where a single contact name has multiple words
    if !contacts_str.contains(',') && contacts_str.contains(' ') {
        // Treat the entire string as one contact name if it has spaces but no commas
        vec![contacts_str.trim()]
    } else {
        // Otherwise, split by comma as usual for multiple contacts
        contacts_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    }
}
