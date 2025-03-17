use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::{ws::Message, Filter};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use futures_util::{SinkExt, StreamExt};
use anyhow::Result;

// Shared state for keeping track of connected clients
type Clients = Arc<Mutex<HashMap<String, UnboundedSender<Message>>>>;

// Request format from Swift client
#[derive(Deserialize, Debug)]
struct ClientRequest {
    request_type: String,  // "natural_language" or other types in the future
    text: String,
    client_id: String,
}

// Response format to Swift client
#[derive(Serialize)]
struct ClientResponse {
    status: String,        // "success" or "error"
    message: String,
    command: Option<String>,
    result: Option<String>,
}

pub async fn start_server() -> Result<()> {
    log::info!("Starting WebSocket server on 127.0.0.1:3000");
    
    // Keep track of connected clients
    let clients = Clients::default();
    let clients = warp::any().map(move || clients.clone());

    // WebSocket endpoint
    let websocket = warp::path("chat")
        .and(warp::ws())
        .and(clients)
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| handle_client_connection(socket, clients))
        });

    // HTTP health check endpoint
    let health = warp::path("health")
        .map(|| "DuckTape WebSocket Server is running");

    // Combined routes with CORS
    let routes = websocket
        .or(health)
        .with(warp::cors().allow_any_origin());

    // Start the server in a background task
    tokio::spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
    });
    
    log::info!("WebSocket server started successfully");
    Ok(())
}

async fn handle_client_connection(websocket: warp::ws::WebSocket, clients: Clients) {
    let (mut client_ws_tx, mut client_ws_rx) = websocket.split();
    
    // Generate a unique client ID
    let client_id = uuid::Uuid::new_v4().to_string();
    
    // Create a channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Task to forward messages from the channel to WebSocket
    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = client_ws_tx.send(message).await {
                log::error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });
    
    // Register client
    clients.lock().unwrap().insert(client_id.clone(), tx.clone());
    log::info!("New client connected: {}", client_id);
    
    // Send welcome message
    let welcome = ClientResponse {
        status: "success".to_string(),
        message: "Connected to DuckTape WebSocket server".to_string(),
        command: None,
        result: None,
    };
    
    if let Err(e) = tx.send(Message::text(serde_json::to_string(&welcome).unwrap())) {
        log::error!("Failed to send welcome message: {}", e);
    }
    
    // Process incoming messages
    while let Some(result) = client_ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    process_client_message(text, &client_id, &tx).await;
                }
            },
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    // Client disconnected
    log::info!("Client disconnected: {}", client_id);
    clients.lock().unwrap().remove(&client_id);
}

async fn process_client_message(text: &str, client_id: &str, tx: &UnboundedSender<Message>) {
    log::info!("Received message from client {}: {}", client_id, text);
    
    match serde_json::from_str::<ClientRequest>(text) {
        Ok(request) => {
            if request.request_type == "natural_language" {
                // Use the existing grok_parser to process natural language
                match crate::grok_parser::parse_natural_language(&request.text).await {
                    Ok(command) => {
                        log::info!("Translated to command: {}", command);
                        
                        // Execute the command (you can implement this part)
                        // For now, we'll just return the translated command
                        let response = ClientResponse {
                            status: "success".to_string(),
                            message: "Command processed".to_string(),
                            command: Some(command),
                            result: Some("Command was successfully translated".to_string()),
                        };
                        
                        if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                            log::error!("Failed to send response: {}", e);
                        }
                    },
                    Err(e) => {
                        log::error!("Error processing natural language: {}", e);
                        
                        let response = ClientResponse {
                            status: "error".to_string(),
                            message: format!("Error processing natural language: {}", e),
                            command: None,
                            result: None,
                        };
                        
                        if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                            log::error!("Failed to send error response: {}", e);
                        }
                    }
                }
            } else {
                // Unsupported request type
                let response = ClientResponse {
                    status: "error".to_string(),
                    message: format!("Unsupported request type: {}", request.request_type),
                    command: None,
                    result: None,
                };
                
                if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                    log::error!("Failed to send error response: {}", e);
                }
            }
        },
        Err(e) => {
            log::error!("Failed to parse client message: {}", e);
            
            let response = ClientResponse {
                status: "error".to_string(),
                message: format!("Invalid message format: {}", e),
                command: None,
                result: None,
            };
            
            if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                log::error!("Failed to send error response: {}", e);
            }
        }
    }
}
