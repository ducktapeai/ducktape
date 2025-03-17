use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::{ws::Message, Filter};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use futures_util::{SinkExt, StreamExt};
use anyhow::Result;

// Type for connected clients
type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>;

// Request from Swift app
#[derive(Deserialize, Debug)]
struct CommandRequest {
    text: String,  // The natural language text to process
}

// Response to Swift app
#[derive(Serialize)]
struct CommandResponse {
    status: String,
    command: String,
    message: String,
}

pub async fn start_server() -> Result<()> {
    let port = 3000;
    let addr = ([127, 0, 0, 1], port);
    
    log::info!("Starting WebSocket server on http://127.0.0.1:{}/chat", port);
    
    // Store connected clients
    let clients = Clients::default();
    let clients = warp::any().map(move || clients.clone());

    // WebSocket endpoint
    let chat = warp::path("chat")
        .and(warp::ws())
        .and(clients)
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    // Health check endpoint
    let health = warp::path("health")
        .map(|| "DuckTape WebSocket server is running");

    // Combined routes
    let routes = chat
        .or(health)
        .with(warp::cors().allow_any_origin());

    // Start the server in a separate task so it doesn't block
    tokio::spawn(async move {
        warp::serve(routes).run(addr).await;
    });

    log::info!("WebSocket server started successfully");
    Ok(())
}

async fn handle_connection(websocket: warp::ws::WebSocket, clients: Clients) {
    log::info!("New WebSocket connection established");
    
    // Split the socket into sender and receiver
    let (mut user_ws_tx, mut user_ws_rx) = websocket.split();
    
    let client_id = uuid::Uuid::new_v4().to_string();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Task to forward messages from the channel to the websocket
    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            if user_ws_tx.send(message).await.is_err() {
                break;
            }
        }
    });
    
    // Save the sender
    clients.lock().unwrap().insert(client_id.clone(), tx.clone());
    log::info!("Client connected: {}", client_id);
    
    // Send welcome message
    let welcome_msg = serde_json::json!({
        "status": "connected",
        "message": "Connected to DuckTape backend"
    }).to_string();
    
    if let Err(e) = tx.send(Message::text(welcome_msg)) {
        log::error!("Error sending welcome message: {}", e);
    }
    
    // Handle incoming messages
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Error receiving message: {}", e);
                break;
            }
        };
        
        if let Ok(text) = msg.to_str() {
            log::info!("Received message: {}", text);
            
            match serde_json::from_str::<CommandRequest>(text) {
                Ok(request) => {
                    // Process the natural language using your existing parser
                    match crate::grok_parser::parse_natural_language(&request.text).await {
                        Ok(command) => {
                            log::info!("Translated to command: {}", command);
                            
                            // Send the parsed command back to the client
                            let response = CommandResponse {
                                status: "success".to_string(),
                                command,
                                message: "Command processed successfully".to_string(),
                            };
                            
                            if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                                log::error!("Error sending response: {}", e);
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to parse natural language: {}", e);
                            
                            let response = CommandResponse {
                                status: "error".to_string(),
                                command: String::new(),
                                message: format!("Failed to parse command: {}", e),
                            };
                            
                            if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                                log::error!("Error sending error response: {}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to parse JSON request: {}", e);
                    
                    let response = CommandResponse {
                        status: "error".to_string(),
                        command: String::new(),
                        message: format!("Invalid request format: {}", e),
                    };
                    
                    if let Err(e) = tx.send(Message::text(serde_json::to_string(&response).unwrap())) {
                        log::error!("Error sending error response: {}", e);
                    }
                }
            }
        }
    }
    
    // Client disconnected
    log::info!("Client disconnected: {}", client_id);
    clients.lock().unwrap().remove(&client_id);
}
