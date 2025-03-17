mod app;
mod calendar;
mod calendar_legacy;
mod commands;
mod config;
mod contact_groups;
mod deepseek_parser;
mod deepseek_reasoning;
mod event_search;
mod file_search;
mod grok_parser;
mod notes;
mod openai_parser;
mod reminders;
mod state;
mod todo;
mod zoom;
mod api_server;
mod command_parser;

use anyhow::Result;
use app::Application;
use config::Config;
use std::env;
use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::Response,
};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;
use command_parser::{UserMessage, process_command};
use serde_json;

// Simple logging function
fn log(msg: &str) {
    println!("[SERVER] {}", msg);
}

async fn handle_socket(mut socket: WebSocket) {
    log("New WebSocket connection established!");

    if let Err(e) = socket.send(Message::Text("Connected to Rust server!".to_string())).await {
        log(&format!("Error sending welcome message: {}", e));
        return;
    }

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                log(&format!("Received text message: {}", text));
                handle_command(&mut socket, text).await;
            }
            Ok(Message::Binary(bin)) => {
                log("Received binary message");
                if let Ok(text) = String::from_utf8(bin.clone()) {
                    log(&format!("Binary content: {}", text));
                    handle_command(&mut socket, text).await;
                } else {
                    // Echo the binary message back
                    if let Err(e) = socket.send(Message::Binary(bin)).await {
                        log(&format!("Error sending response: {}", e));
                    }
                }
            }
            Ok(Message::Ping(_)) => {
                if let Err(e) = socket.send(Message::Pong(vec![])).await {
                    log(&format!("Error sending pong: {}", e));
                }
            }
            Ok(Message::Pong(_)) => {
                // Ignore pong messages
            }
            Ok(Message::Close(_)) => {
                log("Client disconnected");
                break;
            }
            Err(e) => {
                log(&format!("Error receiving message: {}", e));
                break;
            }
        }
    }
}

async fn handle_command(socket: &mut WebSocket, text: String) {
    match serde_json::from_str::<UserMessage>(&text) {
        Ok(message) => {
            log(&format!("Processing command: {}", message.content));
            let response = process_command(message);
            
            match serde_json::to_string(&response) {
                Ok(json) => {
                    if let Err(e) = socket.send(Message::Text(json)).await {
                        log(&format!("Error sending response: {}", e));
                    }
                },
                Err(e) => {
                    log(&format!("Error serializing response: {}", e));
                    if let Err(e) = socket.send(Message::Text("Error processing command".to_string())).await {
                        log(&format!("Error sending error message: {}", e));
                    }
                }
            }
        },
        Err(e) => {
            log(&format!("Error parsing user message: {}", e));
            if let Err(e) = socket.send(Message::Text("Invalid message format".to_string())).await {
                log(&format!("Error sending error message: {}", e));
            }
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    log("WebSocket upgrade request received");
    ws.on_upgrade(handle_socket)
}

async fn health_check() -> &'static str {
    "Server is running!"
}

#[tokio::main]
async fn main() {
    log("Starting server...");
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/chat", get(ws_handler))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    log(&format!("Server starting on {}", addr));
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    log(&format!("Listening on {}", addr));
    axum::serve(listener, app).await.unwrap();
}
