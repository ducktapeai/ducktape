use ducktape::api_server; // Change from websocket to api_server since that appears to be the correct module

fn main() {
    // Initialize logging using proper env_logger approach
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Start the server
    println!("[SERVER] Starting server...");

    let config = ducktape::Config::load().unwrap();

    // Run the server with tokio
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        if let Err(e) = api_server::start_api_server(config).await {
            eprintln!("Error starting server: {}", e);
        }
    });
}
