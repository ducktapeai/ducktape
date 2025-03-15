use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
    extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::net::SocketAddr;
use anyhow::{Result, anyhow};
use log::{info, debug, error};
use tokio::net::TcpListener;
use uuid::Uuid;
// Remove these unused imports:
// use std::sync::Mutex;
// use tokio::sync::mpsc;
// use crate::app::Application;

use crate::calendar::{EventConfig, create_event, get_available_calendars, validate_email};
use crate::todo::{TodoConfig, create_todo};
use crate::notes::{NoteConfig, create_note};
use crate::config::Config;
use crate::grok_parser;
// Remove this unused import:
// use crate::commands;

// API state that will be shared across handlers
pub struct ApiState {
    pub config: Config,
}

// Request and response types for calendar events
#[derive(Debug, Deserialize)] // Added Debug trait
pub struct CreateEventRequest {
    pub title: String,
    pub date: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub calendars: Option<Vec<String>>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub emails: Option<Vec<String>>,
    pub reminder: Option<i32>,
    pub create_zoom_meeting: Option<bool>,
}

#[derive(Serialize)]
pub struct CalendarResponse {
    pub success: bool,
    pub message: String,
    pub calendars: Option<Vec<String>>,
}

// Request and response types for todos
#[derive(Debug, Deserialize)] // Added Debug trait
pub struct CreateTodoRequest {
    pub title: String,
    pub lists: Option<Vec<String>>,
    pub reminder_time: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct TodoResponse {
    pub success: bool,
    pub message: String,
}

// Request and response types for notes
#[derive(Debug, Deserialize)] // Added Debug trait
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub folder: Option<String>,
}

#[derive(Serialize)]
pub struct NoteResponse {
    pub success: bool,
    pub message: String,
}

// General API response
#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

// Create an event handler
async fn create_event_handler(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<CreateEventRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    // Convert request to EventConfig
    let mut event_config = EventConfig::new(
        &request.title,
        &request.date,
        &request.start_time,
    );

    if let Some(end_time) = request.end_time {
        event_config.end_time = Some(end_time);
    }

    if let Some(calendars) = request.calendars {
        event_config.calendars = calendars;
    }

    if let Some(location) = request.location {
        event_config.location = Some(location);
    }

    if let Some(description) = request.description {
        event_config.description = Some(description);
    }

    if let Some(emails) = request.emails {
        // Validate emails
        for email in &emails {
            if !validate_email(email) {
                return Err((
                    StatusCode::BAD_REQUEST, 
                    Json(ApiResponse { 
                        success: false, 
                        message: format!("Invalid email format: {}", email),
                    })
                ));
            }
        }
        event_config.emails = emails;
    }

    if let Some(reminder) = request.reminder {
        event_config.reminder = Some(reminder);
    }

    if let Some(create_zoom) = request.create_zoom_meeting {
        event_config.create_zoom_meeting = create_zoom;
    }

    // Create the event
    match create_event(event_config).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            message: "Event created successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to create event: {}", e),
            }),
        )),
    }
}

// List available calendars
async fn list_calendars_handler() -> Result<Json<CalendarResponse>, (StatusCode, Json<CalendarResponse>)> {
    match get_available_calendars().await {
        Ok(calendars_list) => Ok(Json(CalendarResponse {
            success: true,
            message: "Calendars retrieved successfully".to_string(),
            calendars: Some(calendars_list),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CalendarResponse {
                success: false,
                message: format!("Failed to list calendars: {}", e),
                calendars: None,
            }),
        )),
    }
}

// Create a todo handler
async fn create_todo_handler(
    Json(request): Json<CreateTodoRequest>,
) -> Result<Json<TodoResponse>, (StatusCode, Json<TodoResponse>)> {
    // Create a new TodoConfig with the title
    let mut todo_config = TodoConfig::new(&request.title);
    
    // Convert String lists to &str lists (using temporary storage)
    let string_lists: Vec<String> = request.lists.unwrap_or_default();
    let str_refs: Vec<&str> = string_lists.iter().map(AsRef::as_ref).collect();
    todo_config.lists = str_refs;
    
    if let Some(reminder_time) = request.reminder_time.as_deref() {
        todo_config.reminder_time = Some(reminder_time);
    }
    
    if let Some(notes) = request.notes {
        todo_config.notes = Some(notes);
    }
    
    match create_todo(todo_config) {
        Ok(_) => Ok(Json(TodoResponse {
            success: true,
            message: "Todo created successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TodoResponse {
                success: false,
                message: format!("Failed to create todo: {}", e),
            }),
        )),
    }
}

// Create a note handler
async fn create_note_handler(
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<NoteResponse>, (StatusCode, Json<NoteResponse>)> {
    let config = NoteConfig {
        title: &request.title,
        content: &request.content,
        folder: request.folder.as_deref(),
    };
    
    match create_note(config) {
        Ok(_) => Ok(Json(NoteResponse {
            success: true,
            message: "Note created successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(NoteResponse {
                success: false,
                message: format!("Failed to create note: {}", e),
            }),
    )),
    }
}

// Health check endpoint
async fn health_check() -> &'static str {
    "DuckTape API is running"
}

// WebSocket handler for chat
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    info!("New WebSocket upgrade request received");
    ws.on_upgrade(handle_socket)
}

// Define WebSocket message types for communication
#[derive(Debug, Deserialize)]
#[serde(tag = "action", content = "data")]
enum WebSocketRequest {
    CreateEvent(()),
    GetCalendars,
    CreateTodo(()),
    CreateNote(()),
    Ping,
}

#[derive(Debug, Serialize)]
struct WebSocketResponse {
    success: bool,
    message: String,
    data: Option<Value>,
}

// Add these new message types to handle Swift client messages
#[derive(Debug, Deserialize)]
struct SwiftMessage {
    #[serde(rename = "type")]
    message_type: Option<String>,
    action: Option<String>,
    data: Option<serde_json::Value>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SwiftEventData {
    title: String,
    date: String,
    start_time: String,
    end_time: String,
    location: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct SwiftChatMessage {
    sender: String,
    content: String,
    timestamp: String,
    // Add type field that the Swift client may be looking for
    #[serde(rename = "type")]
    message_type: String,
}

#[derive(Debug, Serialize)]
struct SwiftEventResponse {
    #[serde(rename = "type")]
    message_type: String,
    status: String,
    message: String,
    event_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct SwiftErrorResponse {
    #[serde(rename = "type")]
    message_type: String,
    message: String,
}

// Handle the WebSocket connection
async fn handle_socket(mut socket: WebSocket) {
    let connection_id = Uuid::new_v4();
    info!("WebSocket[{}]: Connection established", connection_id);
    
    // Send a welcome message
    let welcome_message = SwiftChatMessage {
        sender: "system".to_string(), // Added .to_string()
        content: "Connected to DuckTape. You can now send messages and create events.".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        message_type: "chat".to_string(),  // Add message type
    };
    
    if let Ok(json) = serde_json::to_string(&welcome_message) {
        if let Err(e) = socket.send(Message::Binary(json.into_bytes())).await {
            error!("WebSocket[{}]: Error sending welcome message: {}", connection_id, e);
        }
    }
    
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("WebSocket[{}]: Received text message ({} bytes)", connection_id, text.len());
                debug!("WebSocket[{}]: Message content: {}", connection_id, text);
                
                process_message(connection_id, text, &mut socket).await;
            },
            Ok(Message::Binary(bin)) => {
                info!("WebSocket[{}]: Received binary message of {} bytes", connection_id, bin.len());
                
                match String::from_utf8(bin) {
                    Ok(text) => {
                        debug!("WebSocket[{}]: Decoded binary content: {}", connection_id, text);
                        process_message(connection_id, text, &mut socket).await;
                    },
                    Err(e) => {
                        error!("WebSocket[{}]: Failed to decode binary as UTF-8: {}", connection_id, e);
                        // Send error response
                        let response = WebSocketResponse {
                            success: false,
                            message: "Could not decode binary data as UTF-8".to_string(),
                            data: None,
                        };
                        
                        if let Ok(json) = serde_json::to_string(&response) {
                            if let Err(e) = socket.send(Message::Text(json)).await {
                                error!("WebSocket[{}]: Error sending error response: {}", connection_id, e);
                            }
                        }
                    }
                }
            },
            Ok(Message::Close(reason)) => {
                if let Some(r) = reason {
                    info!("WebSocket[{}]: Connection closed by client with code {} and reason: {}", 
                          connection_id, r.code, r.reason);
                } else {
                    // Fix: Add the connection_id to the format string
                    info!("WebSocket[{}]: Connection closed by client", connection_id);
                }
                break;
            },
            Err(e) => {
                error!("WebSocket[{}]: Communication error: {}", connection_id, e);
                break;
            },
            _ => {
                debug!("WebSocket[{}]: Received ping/pong message", connection_id);
            }
        }
    }
    
    info!("WebSocket[{}]: Connection closed", connection_id);
}

// Process messages using direct function calls to the parser
async fn process_message(connection_id: Uuid, message: String, socket: &mut WebSocket) {
    match serde_json::from_str::<SwiftMessage>(&message) {
        Ok(swift_message) => {
            // Check if it's a chat message with natural language command
            if let Some(content) = swift_message.content {
                info!("WebSocket[{}]: Received text command: {}", connection_id, content);
                
                // Process as a command if it looks like one
                if is_command_message(&content) {
                    info!("WebSocket[{}]: Processing as DuckTape command", connection_id);
                    
                    // Use grok_parser directly
                    match grok_parser::parse_natural_language(&content).await {
                        Ok(command) => {
                            info!("WebSocket[{}]: Parsed command: {}", connection_id, command);
                            
                            // Extract calendar event details from the command
                            if command.contains("calendar create") {
                                info!("WebSocket[{}]: Executing calendar create command", connection_id);
                                
                                // Parse the command into arguments
                                match crate::commands::CommandArgs::parse(&command) {
                                    Ok(args) => {
                                        // Log the parsed args to help debug
                                        info!("WebSocket[{}]: Parsed args: command={}, args={:?}, flags={:?}", 
                                              connection_id, args.command, args.args, args.flags);
                                        
                                        // For calendar create commands, we need to handle them specially
                                        if args.command == "calendar" && args.args.get(0).map(|s| s.as_str()) == Some("create") {
                                            // Skip "create" (which is args[0]) and process the rest of the args
                                            if args.args.len() >= 4 { // Needs at least title, date, start_time
                                                let title = &args.args[1]; // "title" is the second arg
                                                let date = &args.args[2];  // Date is the third arg
                                                let start_time = &args.args[3]; // Start time is the fourth arg
                                                
                                                // End time and calendar are optional
                                                let end_time = args.args.get(4).map(|s| s.as_str());
                                                let calendar = args.args.get(5).map(|s| s.as_str());
                                                
                                                info!("WebSocket[{}]: Creating event: {} on {} at {}", 
                                                      connection_id, title.trim_matches('"'), date, start_time);
                                                
                                                // Create the event config
                                                let mut config = crate::calendar::EventConfig::new(
                                                    title, date, start_time
                                                );
                                                
                                                // Set optional fields
                                                if let Some(end) = end_time {
                                                    config.end_time = Some(end.to_string());
                                                }
                                                
                                                if let Some(cal) = calendar {
                                                    let cal_str = cal.trim_matches('"');
                                                    config.calendars = vec![cal_str.to_string()];
                                                }
                                                
                                                // Execute the calendar creation
                                                match crate::calendar::create_event(config).await {
                                                    Ok(_) => {
                                                        info!("WebSocket[{}]: Event created successfully", connection_id);
                                                        let response = SwiftChatMessage {
                                                            sender: "ducktape".to_string(),
                                                            content: format!(
                                                                "✅ Created event \"{}\" for {} at {}", 
                                                                title.trim_matches('"'), date, start_time
                                                            ),
                                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                                            message_type: "chat".to_string(),  // Add message type
                                                        };
                                                        send_response(socket, response).await;
                                                    },
                                                    Err(e) => {
                                                        error!("WebSocket[{}]: Failed to create event: {}", connection_id, e);
                                                        let response = SwiftChatMessage {
                                                            sender: "ducktape".to_string(),
                                                            content: format!("❌ Failed to create event: {}", e),
                                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                                            message_type: "error".to_string(),  // Add error message type
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
                                                    message_type: "error".to_string(),  // Add error message type
                                                };
                                                send_response(socket, response).await;
                                            }
                                        } else {
                                            // For other command types
                                            let response = SwiftChatMessage {
                                                sender: "ducktape".to_string(),
                                                content: format!("Command parsed but not executed: {}", command),
                                                timestamp: chrono::Utc::now().to_rfc3339(),
                                                message_type: "chat".to_string(),  // Add message type
                                            };
                                            send_response(socket, response).await;
                                        }
                                    },
                                    Err(e) => {
                                        error!("WebSocket[{}]: Failed to parse command arguments: {}", connection_id, e);
                                        let response = SwiftChatMessage {
                                            sender: "ducktape".to_string(),
                                            content: format!("❌ Failed to parse command: {}. Raw command was: {}", e, command),
                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                            message_type: "error".to_string(),  // Add error message type
                                        };
                                        send_response(socket, response).await;
                                    }
                                }
                            } else {
                                // For other command types
                                let response = SwiftChatMessage {
                                    sender: "ducktape".to_string(),
                                    content: format!("Command parsed but not executed: {}", command),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    message_type: "chat".to_string(),  // Add message type
                                };
                                send_response(socket, response).await;
                            }
                        },
                        Err(e) => {
                            error!("WebSocket[{}]: Failed to parse command: {}", connection_id, e);
                            let response = SwiftChatMessage {
                                sender: "ducktape".to_string(),
                                content: format!("❌ Error: {}", e),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                message_type: "error".to_string(),  // Add error message type
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
                    message_type: "chat".to_string(),  // Add message type
                };
                send_response(socket, response).await;
            } else if let (Some(message_type), Some(_action), Some(data)) = (&swift_message.message_type, &swift_message.action, &swift_message.data) {
                // Check if it's an event creation request
                if message_type == "create" {
                    info!("WebSocket[{}]: Received event creation request", connection_id);
                    match serde_json::from_value::<SwiftEventData>(data.clone()) {
                        Ok(event_data) => {
                            info!("WebSocket[{}]: Creating event: {}", connection_id, event_data.title);
                            
                            // Create EventConfig
                            let mut event_config = EventConfig::new(
                                &event_data.title,
                                &event_data.date,
                                &event_data.start_time,
                            );
                            
                            if let Some(end_time) = Some(event_data.end_time) {
                                event_config.end_time = Some(end_time);
                            }
                            
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
                                        event_id: Some(uuid::Uuid::new_v4().to_string()),
                                    };
                                    send_response(socket, response).await;
                                },
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
                        },
                        Err(e) => {
                            error!("WebSocket[{}]: Failed to parse event data: {}", connection_id, e);
                            send_error_response(socket, &format!("Invalid event data format: {}", e)).await;
                        }
                    }
                    return;
                }
                
                // If we got here, it's an unknown message type
                error!("WebSocket[{}]: Unknown message format", connection_id);
                debug!("WebSocket[{}]: Message: {:?}", connection_id, swift_message);
                send_error_response(socket, "Unknown message format").await;
            }
        },
        Err(e) => {
            error!("WebSocket[{}]: Failed to parse message: {}", connection_id, e);
            send_error_response(socket, &format!("Failed to parse message: {}", e)).await;
        }
    }
}

// Check if a message looks like a command
fn is_command_message(message: &str) -> bool {
    // Simple heuristic: any message with action words is a command
    let command_words = [
        "create", "add", "schedule", "set", "make", "remind", "note", "meeting", "event",
        "calendar", "todo", "zoom", "invite", "tomorrow", "today", "monday", "tuesday",
        "wednesday", "thursday", "friday", "saturday", "sunday", "am", "pm"      
    ];
    
    for word in command_words.iter() {
        if message.to_lowercase().contains(word) {
            return true;
        }
    }
    
    false
}

async fn send_response<T: Serialize>(socket: &mut WebSocket, response: T) {
    match serde_json::to_string(&response) {
        Ok(json) => {
            info!("Sending response: {}", json);
            
            // Try to send as binary first (which your Swift client seems to expect)
            if let Err(e) = socket.send(Message::Binary(json.clone().into_bytes())).await {
                // Clone the json string before converting to bytes
                error!("Error sending binary response: {}", e);
                
                // Fall back to text if binary fails
                if let Err(e2) = socket.send(Message::Text(json)).await {
                    error!("Error sending text response: {}", e2);
                }
            }
        },
        Err(e) => {
            error!("Failed to serialize response: {}", e);
        }
    }
}

async fn send_error_response(socket: &mut WebSocket, message: &str) {
    let error_response = SwiftErrorResponse {
        message_type: "error".to_string(),
        message: message.to_string(),
    };
    
    match serde_json::to_string(&error_response) {
        Ok(json) => {
            if let Err(e) = socket.send(Message::Binary(json.into_bytes())).await {
                error!("Error sending error response: {}", e);
            }
        },
        Err(e) => {
            error!("Failed to serialize error response: {}", e);
        }
    }
}

// Create and start the API server
pub async fn start_api_server(config: Config) -> Result<()> {
    // Create shared state
    let state = Arc::new(ApiState { config });
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/calendars", get(list_calendars_handler))
        .route("/calendar/event", post(create_event_handler))
        .route("/todo", post(create_todo_handler))
        .route("/note", post(create_note_handler))
        .route("/chat", get(websocket_handler)) // Add WebSocket endpoint
        .layer(cors)
        .with_state(state);
    
    // Run our app with the correct syntax for axum 0.7
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("API server starting on http://{}", addr);
    info!("WebSocket endpoint available at ws://127.0.0.1:3000/chat");
    
    // Create a TcpListener first, then pass it to serve
    let listener = TcpListener::bind(addr).await
        .map_err(|e| anyhow!("Failed to bind to address: {}", e))?;
    
    info!("API server successfully bound to {}. Waiting for connections...", addr);
    
    axum::serve(listener, app).await
        .map_err(|e| anyhow!("Failed to start API server: {}", e))?;
    
    Ok(())
} // Fixed: removed extra closing brace