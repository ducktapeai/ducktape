use anyhow::Result;
use ducktape::command_processor::{CommandArgs, CommandProcessor};
use ducktape::parser::natural_language::NaturalLanguageParser;
use ducktape::parser::natural_language::grok::GrokParser;
use log::{debug, info};
use std::env;
use std::path::Path;

/// Flag for natural language processing mode
const NL_FLAG: &str = "--nl";
/// Alternative flag for natural language processing mode
const NATURAL_FLAG: &str = "--natural";
/// Command for interactive mode
const INTERACTIVE_CMD: &str = "interactive";

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
    
    // Handle different command modes based on first argument
    if !raw_args.is_empty() {
        match raw_args[0].as_str() {
            // Natural Language Mode
            NL_FLAG | NATURAL_FLAG => {
                return process_natural_language(&raw_args[1..]).await;
            },
            // Interactive Mode
            INTERACTIVE_CMD => {
                return start_interactive_mode().await;
            },
            // Otherwise continue with regular command processing
            _ => {}
        }
    }

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

/// Process natural language input and convert it to a structured command
///
/// # Arguments
///
/// * `args` - Natural language argument string split into parts
///
/// # Returns
///
/// * `Result<()>` - Success if command was processed, or error
async fn process_natural_language(args: &[String]) -> Result<()> {
    // Ensure we have some text to process
    if args.is_empty() {
        return Err(anyhow::anyhow!("No natural language query provided. Usage: ducktape --nl \"your request here\""));
    }
    
    // Join all remaining arguments as the natural language query
    let nl_query = args.join(" ");
    
    info!("Processing natural language query: {}", nl_query);
    
    // Initialize the NLP parser
    let parser = GrokParser::new()?;
    
    // Sanitize user input
    let sanitized_input = ducktape::parser::natural_language::utils::sanitize_user_input(&nl_query);
    
    // Parse the natural language into a structured command
    let structured_command = parser.parse_natural_language(&sanitized_input).await?;
    
    debug!("Converted to structured command: {}", structured_command);
    
    // Parse the structured command and execute it
    match CommandArgs::parse(&structured_command) {
        Ok(args) => {
            debug!("Parsed args: {:?}", args);
            let processor = CommandProcessor::new();
            processor.execute(args).await?;
        }
        Err(e) => {
            eprintln!("Error parsing generated command: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Start the interactive shell mode for Ducktape
///
/// # Returns
///
/// * `Result<()>` - Success if the interactive mode completes normally
///
/// # Errors
///
/// Returns an error if there's a problem with the interactive session
async fn start_interactive_mode() -> Result<()> {
    use rustyline::error::ReadlineError;
    use rustyline::DefaultEditor;

    println!("Ducktape Interactive Mode");
    println!("Type your requests in natural language or use structured commands");
    println!("Type 'exit' or 'quit' to exit");
    
    let mut rl = DefaultEditor::new()?;
    let parser = GrokParser::new()?;
    let processor = CommandProcessor::new();

    loop {
        let readline = match rl.readline("ducktape> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting interactive mode");
                break;
            }
            Err(err) => {
                return Err(anyhow::anyhow!("Interactive mode error: {}", err));
            }
        };

        if readline.trim().is_empty() {
            continue;
        }

        let input = readline.trim();
        if input == "exit" || input == "quit" {
            println!("Exiting interactive mode");
            break;
        }

        // Try to parse as natural language first
        match parser.parse_natural_language(input).await {
            Ok(structured_command) => {
                debug!("Interactive mode generated command: {}", structured_command);
                
                // Show the user what command will be executed
                println!("Executing: {}", structured_command);
                
                // Execute the command
                match CommandArgs::parse(&structured_command) {
                    Ok(args) => {
                        if let Err(e) = processor.execute(args).await {
                            eprintln!("Error executing command: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing generated command: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing input: {}", e);
            }
        }
    }

    Ok(())
}
