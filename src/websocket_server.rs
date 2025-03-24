use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::{ws::Message, Filter};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use futures_util::{SinkExt, StreamExt};
use anyhow::Result;
use serde_json::{json, Value};
use uuid::Uuid;

// Shared state for keeping track of connected clients
type Clients = Arc<Mutex<HashMap<String, mpsc::Sender<Message>>>>;

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
    let client_id = Uuid::new_v4().to_string();
    
    // Create a channel for this client
    let (tx, mut rx) = mpsc::channel(100);
    
    // Register client
    if let Ok(mut guard) = clients.lock() {
        guard.insert(client_id.clone(), tx.clone());
    } else {
        log::error!("Failed to acquire lock for clients");
        return;
    }
    
    // Send welcome message
    let welcome = json!({
        "type": "welcome",
        "client_id": client_id
    });
    
    if let Err(e) = send_json_message(&tx, &welcome).await {
        log::error!("Failed to send welcome message: {}", e);
        return;
    }
    
    // Task to forward messages from the channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = client_ws_tx.send(message).await {
                log::error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });
    
    // Process incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = client_ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        match handle_message(&text, &tx).await {
                            Ok(_) => (),
                            Err(e) => log::error!("Error handling message: {}", e),
                        }
                    }
                },
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });
    
    // If either task completes, cancel the other one
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    // Client disconnected
    log::info!("Client disconnected: {}", client_id);
    if let Ok(mut guard) = clients.lock() {
        guard.remove(&client_id);
    }
}

async fn send_json_message(tx: &mpsc::Sender<Message>, value: &Value) -> Result<()> {
    let json_str = serde_json::to_string(value)?;
    tx.send(Message::Text(json_str)).await?;
    Ok(())
}

async fn handle_message(text: &str, tx: &mpsc::Sender<Message>) -> Result<()> {
    let message: Value = serde_json::from_str(text)?;
    
    match message.get("type").and_then(Value::as_str) {
        Some("command") => {
            if let Some(command) = message.get("command").and_then(Value::as_str) {
                let response = process_command(command).await?;
                send_json_message(tx, &response).await?;
            }
        }
        Some("ping") => {
            let response = json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            send_json_message(tx, &response).await?;
        }
        _ => {
            let response = json!({
                "type": "error",
                "message": "Unknown message type"
            });
            send_json_message(tx, &response).await?;
        }
    }
    Ok(())
}

async fn process_command(command: &str) -> Result<Value> {
    // Validate command to prevent injection
    if !is_safe_command(command) {
        return Ok(json!({
            "type": "error",
            "message": "Invalid command format"
        }));
    }

    // Process the command...
    Ok(json!({
        "type": "response",
        "message": format!("Processed command: {}", command)
    }))
}

fn is_safe_command(command: &str) -> bool {
    // Add validation logic here
    // For example, check for suspicious characters, maximum length, etc.
    !command.contains(';') && !command.contains('&') && !command.contains('|')
        && command.len() < 1000
}
