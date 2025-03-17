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
                // Echo the message back to the client
                if let Err(e) = socket.send(Message::Text(text)).await {
                    log(&format!("Error sending response: {}", e));
                }
            }
            Ok(Message::Binary(bin)) => {
                log("Received binary message");
                // Echo the binary message back
                if let Err(e) = socket.send(Message::Binary(bin)).await {
                    log(&format!("Error sending response: {}", e));
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
