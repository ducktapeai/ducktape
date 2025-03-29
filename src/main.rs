mod api_server;
mod app;
mod calendar;
mod calendar_legacy;
mod command_parser;
mod commands;
mod config;
mod contact_groups;
mod deepseek_parser;
mod deepseek_reasoning;
mod env_debug;
mod event_search;
mod file_search;
mod grok_parser;
mod notes;
mod openai_parser;
mod reminders;
mod state;
mod todo;
mod zoom;

use anyhow::Result;
use app::Application;
use config::Config;
use std::env;

/// Name of the application used in help and version output
const APP_NAME: &str = "DuckTape";
/// Current version of the application
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load environment variables at startup
    if let Err(e) = dotenvy::dotenv() {
        println!("Warning: Failed to load .env file: {}", e);
    }

    // Force set the API key
    env_debug::force_set_api_key();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Create application instance early so we can use it for commands
    let app = Application::new();

    // Check command line flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--api-server" => {
                // Load config and start API server only
                let config = Config::load()?;
                api_server::start_api_server(config).await?;
                return Ok(());
            }
            "--full" => {
                // Start both terminal and API server (original behavior)
                app.run().await?;
                return Ok(());
            }
            "version" | "--version" | "-v" => {
                // Display version information
                print_version();
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                // Display help information
                print_help();
                return Ok(());
            }
            "calendars" => {
                // Handle calendars command directly
                calendar::list_calendars().await?;
                return Ok(());
            }
            "calendar" => {
                // Handle calendar subcommands
                if args.len() > 2 {
                    let subcommand = args[2].as_str();
                    match subcommand {
                        "list" => {
                            calendar::list_calendars().await?;
                            return Ok(());
                        }
                        "props" | "properties" => {
                            calendar::list_event_properties().await?;
                            return Ok(());
                        }
                        "create" | "add" => {
                            // For calendar create/add, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "delete" | "remove" => {
                            // For calendar delete/remove, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "import" => {
                            // For calendar import, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "set-default" => {
                            // For calendar set-default, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        _ => {
                            // For other calendar subcommands, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                    }
                } else {
                    // If just "calendar" with no subcommand, show help
                    println!("Usage: ducktape calendar <subcommand>");
                    println!("Subcommands:");
                    println!("  list      - List available calendars");
                    println!("  props     - List available event properties");
                    println!("  create    - Create a new calendar event");
                    println!("  delete    - Delete a calendar event");
                    println!("  import    - Import events from a file");
                    println!("  set-default - Set the default calendar");
                    return Ok(());
                }
            }
            "todo" | "todos" => {
                // Handle todo subcommands
                if args.len() > 2 {
                    let subcommand = args[2].as_str();
                    match subcommand {
                        "list" => {
                            // List todos directly
                            let full_command = format!("ducktape todo list");
                            return app.process_command(&full_command).await;
                        }
                        "lists" => {
                            // List todo lists directly
                            let full_command = format!("ducktape todo lists");
                            return app.process_command(&full_command).await;
                        }
                        "create" | "add" => {
                            // For todo create/add, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "complete" | "done" => {
                            // For todo complete, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "delete" | "remove" => {
                            // For todo delete, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "set-list" | "set-default" => {
                            // For todo set-list, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        _ => {
                            // For other todo subcommands, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                    }
                } else {
                    // If just "todo" with no subcommand, show todos
                    let full_command = format!("ducktape todo list");
                    return app.process_command(&full_command).await;
                }
            }
            "note" | "notes" => {
                // Handle note subcommands
                if args.len() > 2 {
                    let subcommand = args[2].as_str();
                    match subcommand {
                        "list" => {
                            // List notes directly
                            let full_command = format!("ducktape note list");
                            return app.process_command(&full_command).await;
                        }
                        "create" | "add" | "new" => {
                            // For note create, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "search" | "find" => {
                            // For note search, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "delete" | "remove" => {
                            // For note delete, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        _ => {
                            // For other note subcommands, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                    }
                } else {
                    // If just "note" with no subcommand, show notes
                    let full_command = format!("ducktape note list");
                    return app.process_command(&full_command).await;
                }
            }
            "config" => {
                // Handle config subcommands
                if args.len() > 2 {
                    let subcommand = args[2].as_str();
                    match subcommand {
                        "show" | "list" | "get" => {
                            // For config show, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        "set" => {
                            // For config set, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                        _ => {
                            // For other config subcommands, process the full command
                            let full_command = args.join(" ");
                            return app.process_command(&full_command).await;
                        }
                    }
                } else {
                    // If just "config" with no subcommand, show config
                    let full_command = format!("ducktape config show");
                    return app.process_command(&full_command).await;
                }
            }
            _ => {
                // Check if this might be a command with arguments
                if args.len() >= 2 {
                    // Reconstruct full command including "ducktape"
                    let full_command = args.join(" ");

                    // Try to process as a command
                    match app.process_command(&full_command).await {
                        Ok(_) => return Ok(()),
                        Err(_) => {
                            // If command processing fails, just show help
                            print_help();
                            return Ok(());
                        }
                    }
                }

                // Default: show help for unknown commands
                print_help();
                return Ok(());
            }
        }
    }

    // Default: Run in terminal-only mode
    app.run_terminal_only().await
}

/// Prints version information for the application
fn print_version() {
    println!("{} version {}", APP_NAME, VERSION);
}

/// Prints help information for the application
fn print_help() {
    println!(
        "{} - AI-powered terminal tool for Apple Calendar, Reminders and Notes",
        APP_NAME
    );
    println!("\nUSAGE:");
    println!("  ducktape [COMMAND] [FLAGS]");
    println!("\nCOMMANDS:");
    println!("  help        Display this help information");
    println!("  version     Display the current version");
    println!("  calendar    Manage calendar events");
    println!("  todo        Manage reminders/todos");
    println!("  note        Manage notes");
    println!("  config      View or modify configuration");
    println!("\nFLAGS:");
    println!("  --api-server  Start in API server mode only");
    println!("  --full        Start both terminal and API server");
    println!("  (no flags)    Start in terminal mode only");
    println!("\nEXAMPLES:");
    println!("  ducktape                      Start interactive terminal mode");
    println!("  ducktape calendar list        List available calendars");
    println!("  ducktape todo lists           List available reminder lists");
    println!("  ducktape note list            List recent notes");
    println!("  ducktape --api-server         Start API server only");
}
