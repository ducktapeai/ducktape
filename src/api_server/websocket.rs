// WebSocket handlers for real-time communication
//
// This module provides WebSocket functionality for the API server,
// allowing real-time commands and notifications.

use axum::{
    extract::WebSocketUpgrade,
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
};
use clap::Parser; // Add this missing import for try_parse_from
use log::{debug, error, info};
use serde::Serialize;
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

use crate::calendar::{EventConfig, create_event, import_csv_events, import_ics_events};
use crate::cli;
use crate::command_processor::CommandArgs;
use crate::parser;
use std::path::Path;

use super::models::{
    SwiftChatMessage, SwiftErrorResponse, SwiftEventData, SwiftEventResponse, SwiftMessage,
};

/// WebSocket handler for chat interface
///
/// Upgrades an HTTP request to a WebSocket connection
pub async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    info!("New WebSocket upgrade request received");
    ws.on_upgrade(handle_socket)
}

/// Handle an active WebSocket connection
///
/// Processes messages and maintains the connection with the client
async fn handle_socket(mut socket: WebSocket) {
    let connection_id = Uuid::new_v4();
    info!("WebSocket[{}]: Connection established", connection_id);

    // Send a welcome message
    let welcome_message = SwiftChatMessage {
        sender: "system".to_string(),
        content: "Connected to DuckTape. You can now send messages and create events.".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        message_type: "chat".to_string(),
    };

    if let Ok(json) = serde_json::to_string(&welcome_message) {
        if let Err(e) = socket.send(Message::Binary(json.into_bytes())).await {
            error!("WebSocket[{}]: Error sending welcome message: {}", connection_id, e);
        }
    }

    // Set up a heartbeat timer using socket.ping()
    let mut interval = interval(Duration::from_secs(45));

    loop {
        tokio::select! {
            // Periodically send pings to ensure connection stays alive
            _ = interval.tick() => {
                debug!("WebSocket[{}]: Sending ping", connection_id);
                if let Err(e) = socket.send(Message::Ping(Vec::new())).await {
                    error!("WebSocket[{}]: Failed to send ping: {}", connection_id, e);
                    break;
                }
            }

            // Handle incoming messages
            msg_result = socket.recv() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        info!("WebSocket[{}]: Received text message ({} bytes)", connection_id, text.len());
                        debug!("WebSocket[{}]: Message content: {}", connection_id, text);

                        process_message(connection_id, text, &mut socket).await;
                    },
                    Some(Ok(Message::Binary(bin))) => {
                        info!("WebSocket[{}]: Received binary message of {} bytes", connection_id, bin.len());

                        match String::from_utf8(bin) {
                            Ok(text) => {
                                debug!("WebSocket[{}]: Decoded binary content: {}", connection_id, text);
                                process_message(connection_id, text, &mut socket).await;
                            },
                            Err(e) => {
                                error!("WebSocket[{}]: Failed to decode binary as UTF-8: {}", connection_id, e);
                                let response = SwiftErrorResponse {
                                    message_type: "error".to_string(),
                                    message: "Could not decode binary data as UTF-8".to_string(),
                                };
                                send_response(&mut socket, response).await; // Fixed: Added &mut
                            }
                        }
                    },
                    Some(Ok(Message::Ping(data))) => {
                        debug!("WebSocket[{}]: Received ping, sending pong", connection_id);
                        if let Err(e) = socket.send(Message::Pong(data)).await {
                            error!("WebSocket[{}]: Failed to send pong: {}", connection_id, e);
                        }
                    },
                    Some(Ok(Message::Pong(_))) => {
                        debug!("WebSocket[{}]: Received pong", connection_id);
                    },
                    Some(Ok(Message::Close(reason))) => {
                        if let Some(r) = reason {
                            info!("WebSocket[{}]: Connection closed by client with code {} and reason: {}",
                                  connection_id, r.code, r.reason);
                        } else {
                            info!("WebSocket[{}]: Connection closed by client", connection_id);
                        }
                        break;
                    },
                    Some(Err(e)) => {
                        error!("WebSocket[{}]: Communication error: {}", connection_id, e);
                        break;
                    },
                    None => {
                        info!("WebSocket[{}]: Connection closed (no more messages)", connection_id);
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket[{}]: Connection closed", connection_id);
}

/// Process received WebSocket messages
///
/// Handles both natural language commands and structured JSON messages
async fn process_message(connection_id: Uuid, message: String, socket: &mut WebSocket) {
    match serde_json::from_str::<SwiftMessage>(&message) {
        Ok(swift_message) => {
            // Check if it's a chat message with natural language command
            if let Some(content) = swift_message.content {
                info!("WebSocket[{}]: Received text command: {}", connection_id, content);

                // Process as a command if it looks like one
                if is_command_message(&content) {
                    info!("WebSocket[{}]: Processing as DuckTape command", connection_id);

                    // Use parser module instead of grok_parser
                    match parser::openai::parse_natural_language(&content).await {
                        Ok(command) => {
                            info!("WebSocket[{}]: Parsed command: {}", connection_id, command);
                            handle_parsed_command(connection_id, command, socket).await;
                        }
                        Err(e) => {
                            error!("WebSocket[{}]: Failed to parse command: {}", connection_id, e);
                            let response = SwiftChatMessage {
                                sender: "ducktape".to_string(),
                                content: format!("❌ Error: {}", e),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                message_type: "error".to_string(),
                            };
                            send_response(socket, response).await;
                        }
                    }
                    return;
                }

                // Otherwise just echo back the message as before
                let response = SwiftChatMessage {
                    sender: "bot".to_string(),
                    content: format!("You said: {}", content),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "chat".to_string(),
                };
                send_response(socket, response).await;
            } else if let (Some(message_type), Some(action), Some(data)) =
                (&swift_message.message_type, &swift_message.action, &swift_message.data)
            {
                // Check if it's an event creation request
                if message_type == "create" && action == "event" {
                    handle_event_creation(connection_id, data.clone(), socket).await;
                } else {
                    // If we got here, it's an unknown message type
                    error!("WebSocket[{}]: Unknown message format", connection_id);
                    debug!("WebSocket[{}]: Message: {:?}", connection_id, swift_message);
                    send_error_response(socket, "Unknown message format").await;
                }
            }
        }
        Err(e) => {
            error!("WebSocket[{}]: Failed to parse message: {}", connection_id, e);
            send_error_response(socket, &format!("Failed to parse message: {}", e)).await;
        }
    }
}

/// Handle event creation from structured WebSocket messages
async fn handle_event_creation(
    connection_id: Uuid,
    data: serde_json::Value,
    socket: &mut WebSocket,
) {
    info!("WebSocket[{}]: Received event creation request", connection_id);
    match serde_json::from_value::<SwiftEventData>(data) {
        Ok(event_data) => {
            info!("WebSocket[{}]: Creating event: {}", connection_id, event_data.title);

            // Create EventConfig
            let mut event_config =
                EventConfig::new(&event_data.title, &event_data.date, &event_data.start_time);

            event_config.end_time = Some(event_data.end_time);

            if let Some(location) = event_data.location {
                event_config.location = Some(location);
            }

            if let Some(description) = event_data.description {
                event_config.description = Some(description);
            }

            // Create the event
            match create_event(event_config).await {
                Ok(_) => {
                    info!("WebSocket[{}]: Event created successfully", connection_id);
                    let response = SwiftEventResponse {
                        message_type: "event".to_string(),
                        status: "success".to_string(),
                        message: "Event created successfully".to_string(),
                        event_id: Some(Uuid::new_v4().to_string()),
                    };
                    send_response(socket, response).await;
                }
                Err(e) => {
                    error!("WebSocket[{}]: Failed to create event: {}", connection_id, e);
                    let response = SwiftEventResponse {
                        message_type: "event".to_string(),
                        status: "error".to_string(),
                        message: format!("Failed to create event: {}", e),
                        event_id: None,
                    };
                    send_response(socket, response).await;
                }
            }
        }
        Err(e) => {
            error!("WebSocket[{}]: Failed to parse event data: {}", connection_id, e);
            send_error_response(socket, &format!("Invalid event data format: {}", e)).await;
        }
    }
}

/// Handle parsed commands from natural language input
async fn handle_parsed_command(connection_id: Uuid, command: String, socket: &mut WebSocket) {
    // Parse the command into arguments using Clap first
    match parse_command_string(&command) {
        Ok(args) => {
            // Log the parsed args to help debug
            info!(
                "WebSocket[{}]: Parsed args: command={}, args={:?}, flags={:?}",
                connection_id, args.command, args.args, args.flags
            );

            handle_websocket_command(connection_id, args, socket).await;
        }
        Err(_) => {
            // Fall back to legacy parser if Clap fails
            match CommandArgs::parse(&command) {
                Ok(args) => {
                    info!(
                        "WebSocket[{}]: Parsed args (legacy): command={}, args={:?}, flags={:?}",
                        connection_id, args.command, args.args, args.flags
                    );

                    handle_websocket_command(connection_id, args, socket).await;
                }
                Err(e) => {
                    error!(
                        "WebSocket[{}]: Failed to parse command arguments: {}",
                        connection_id, e
                    );
                    let response = SwiftChatMessage {
                        sender: "ducktape".to_string(),
                        content: format!(
                            "❌ Failed to parse command: {}. Raw command was: {}",
                            e, command
                        ),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        message_type: "error".to_string(),
                    };
                    send_response(socket, response).await;
                }
            }
        }
    }
}

/// Check if a message looks like a command
fn is_command_message(message: &str) -> bool {
    // Simple heuristic: any message with action words is a command
    let command_words = [
        "create",
        "add",
        "schedule",
        "set",
        "make",
        "remind",
        "note",
        "meeting",
        "event",
        "calendar",
        "todo",
        "zoom",
        "invite",
        "tomorrow",
        "today",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "am",
        "pm",
    ];

    for word in &command_words {
        if message.to_lowercase().contains(word) {
            return true;
        }
    }

    false
}

/// Send a serializable response to the WebSocket client
async fn send_response<T: Serialize>(socket: &mut WebSocket, response: T) {
    match serde_json::to_string(&response) {
        Ok(json) => {
            debug!("Sending response: {}", json);

            // Try to send as binary first (which Swift clients typically expect)
            if let Err(e) = socket.send(Message::Binary(json.clone().into_bytes())).await {
                error!("Error sending binary response: {}", e);

                // Fall back to text if binary fails
                if let Err(e2) = socket.send(Message::Text(json)).await {
                    error!("Error sending text response: {}", e2);
                }
            }
        }
        Err(e) => {
            error!("Failed to serialize response: {}", e);
        }
    }
}

/// Send an error response to the WebSocket client
async fn send_error_response(socket: &mut WebSocket, message: &str) {
    let error_response =
        SwiftErrorResponse { message_type: "error".to_string(), message: message.to_string() };

    match serde_json::to_string(&error_response) {
        Ok(json) => {
            if let Err(e) = socket.send(Message::Binary(json.into_bytes())).await {
                error!("Error sending error response: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to serialize error response: {}", e);
        }
    }
}

/// Helper function to parse commands using Clap instead of deprecated CommandArgs::parse
fn parse_command_string(input: &str) -> Result<CommandArgs, anyhow::Error> {
    use anyhow::{Result, anyhow};

    // Format the input into argv style for clap
    let args = shell_words::split(input).map_err(|e| anyhow!("Failed to parse command: {}", e))?;

    // Check if we have any arguments
    if args.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    // Parse using Clap - corrected the parsing approach
    let cli = match cli::Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // This is likely not a structured command but a natural language input
            return Err(anyhow!("Not a structured command: {}", e));
        }
    };

    // Convert from Clap command to CommandArgs
    cli::convert_to_command_args(&cli)
        .ok_or_else(|| anyhow!("Failed to convert parsed command to CommandArgs"))
}

/// Function to handle websocket commands
async fn handle_websocket_command(connection_id: Uuid, args: CommandArgs, socket: &mut WebSocket) {
    if args.command == "calendar" {
        // Handle different calendar subcommands
        match args.args.get(0).map(|s| s.as_str()) {
            Some("create") => {
                handle_calendar_create(connection_id, args, socket).await;
            }
            Some("import") => {
                handle_calendar_import(connection_id, args, socket).await;
            }
            Some(cmd) => {
                // Handle other calendar commands (list, delete, etc.)
                let response = SwiftChatMessage {
                    sender: "ducktape".to_string(),
                    content: format!(
                        "Command '{}' parsed but not yet implemented in WebSocket server",
                        cmd
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "chat".to_string(),
                };
                send_response(socket, response).await;
            }
            None => {
                let response = SwiftChatMessage {
                    sender: "ducktape".to_string(),
                    content: "❌ Invalid calendar command format".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "error".to_string(),
                };
                send_response(socket, response).await;
            }
        }
    } else {
        // For other command types (todo, notes, etc.)
        let response = SwiftChatMessage {
            sender: "ducktape".to_string(),
            content: format!(
                "Command '{}' parsed but not yet implemented in WebSocket server",
                args.command
            ),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: "chat".to_string(),
        };
        send_response(socket, response).await;
    }
}

/// Handle calendar create command
async fn handle_calendar_create(connection_id: Uuid, args: CommandArgs, socket: &mut WebSocket) {
    // Skip "create" (which is args[0]) and process the rest of the args
    if args.args.len() >= 4 {
        // Needs at least title, date, start_time
        let title = &args.args[1]; // "title" is the second arg
        let date = &args.args[2]; // Date is the third arg
        let start_time = &args.args[3]; // Start time is the fourth arg

        // End time and calendar are optional
        let end_time = args.args.get(4).map(|s| s.as_str());
        let calendar = args.args.get(5).map(|s| s.as_str());

        info!(
            "WebSocket[{}]: Creating event: {} on {} at {}",
            connection_id,
            title.trim_matches('"'),
            date,
            start_time
        );

        // Create the event config
        let mut config = crate::calendar::EventConfig::new(title, date, start_time);

        // Set optional fields
        if let Some(end) = end_time {
            config.end_time = Some(end.to_string());
        }

        if let Some(cal) = calendar {
            let cal_str = cal.trim_matches('"');
            config.calendars = vec![cal_str.to_string()];
        }

        // Handle the email flag
        if let Some(Some(emails_str)) = args.flags.get("email") {
            let emails: Vec<String> =
                emails_str.split(',').map(|e| e.trim().trim_matches('"').to_string()).collect();

            if !emails.is_empty() {
                info!("WebSocket[{}]: Adding email attendees: {:?}", connection_id, emails);
                config.emails = emails;
            }
        }

        // Handle the zoom flag
        if args.flags.contains_key("zoom") {
            info!("WebSocket[{}]: Enabling Zoom meeting creation", connection_id);
            config.create_zoom_meeting = true;
        }

        // Execute the calendar creation
        match crate::calendar::create_event(config).await {
            Ok(_) => {
                info!("WebSocket[{}]: Event created successfully", connection_id);
                let response = SwiftChatMessage {
                    sender: "ducktape".to_string(),
                    content: format!(
                        "✅ Created event \"{}\" for {} at {}",
                        title.trim_matches('"'),
                        date,
                        start_time
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "chat".to_string(),
                };
                send_response(socket, response).await;
            }
            Err(e) => {
                error!("WebSocket[{}]: Failed to create event: {}", connection_id, e);
                let response = SwiftChatMessage {
                    sender: "ducktape".to_string(),
                    content: format!("❌ Failed to create event: {}", e),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    message_type: "error".to_string(),
                };
                send_response(socket, response).await;
            }
        }
    } else {
        error!("WebSocket[{}]: Invalid command format - not enough arguments", connection_id);
        let response = SwiftChatMessage {
            sender: "ducktape".to_string(),
            content: "❌ Invalid command format".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: "error".to_string(),
        };
        send_response(socket, response).await;
    }
}

/// Handle calendar import command
async fn handle_calendar_import(connection_id: Uuid, args: CommandArgs, socket: &mut WebSocket) {
    info!("WebSocket[{}]: Processing calendar import command", connection_id);

    if args.args.len() < 2 {
        let response = SwiftChatMessage {
            sender: "ducktape".to_string(),
            content: "❌ Usage: calendar import \"<file_path>\" [--format csv|ics] [--calendar \"<calendar_name>\"]".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: "error".to_string(),
        };
        send_response(socket, response).await;
        return;
    }

    // Get the file path and expand it if needed
    let mut file_path_str = args.args[1].clone();
    file_path_str = file_path_str.trim_matches('"').to_string();

    // Expand tilde to home directory
    if file_path_str.starts_with('~') {
        if let Some(home_dir) = dirs::home_dir() {
            file_path_str = file_path_str.replacen("~", home_dir.to_string_lossy().as_ref(), 1);
        }
    }

    let file_path = Path::new(&file_path_str);

    if !file_path.exists() {
        let response = SwiftChatMessage {
            sender: "ducktape".to_string(),
            content: format!("❌ File not found: {}", file_path_str),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: "error".to_string(),
        };
        send_response(socket, response).await;
        return;
    }

    // Get format from --format flag, default to csv
    let format = args
        .flags
        .get("format")
        .and_then(|f| f.as_ref())
        .map(|f| f.to_lowercase())
        .unwrap_or_else(|| "csv".to_string());

    if !["csv", "ics"].contains(&format.as_str()) {
        let response = SwiftChatMessage {
            sender: "ducktape".to_string(),
            content: "❌ Unsupported format. Use --format csv or --format ics".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(), // Fixed: removed .to
            message_type: "error".to_string(),
        };
        send_response(socket, response).await;
        return;
    }

    // Get target calendar if specified
    let calendar = args
        .flags
        .get("calendar")
        .and_then(|c| c.as_ref())
        .map(|c| c.trim_matches('"').to_string());

    info!(
        "WebSocket[{}]: Importing {} file: {} to calendar: {:?}",
        connection_id, format, file_path_str, calendar
    );

    // Call the appropriate import function
    let result = match format.as_str() {
        "csv" => import_csv_events(file_path, calendar).await,
        "ics" => import_ics_events(file_path, calendar).await,
        _ => unreachable!(),
    };

    match result {
        Ok(_) => {
            let response = SwiftChatMessage {
                sender: "ducktape".to_string(),
                content: format!("✅ Successfully imported events from {}", file_path_str),
                timestamp: chrono::Utc::now().to_rfc3339(),
                message_type: "chat".to_string(),
            };
            send_response(socket, response).await;
        }
        Err(e) => {
            error!("WebSocket[{}]: Failed to import events: {}", connection_id, e);
            let response = SwiftChatMessage {
                sender: "ducktape".to_string(),
                content: format!("❌ Failed to import events: {}", e),
                timestamp: chrono::Utc::now().to_rfc3339(), // Fixed from .to.rfc3339()
                message_type: "error".to_string(),
            };
            send_response(socket, response).await;
        }
    }
}
