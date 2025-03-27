use log::{error, info};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Prints the status of environment variables and .env file for debugging
pub fn print_env_status() {
    info!("📊 Environment Variables Status:");

    // Important variables to check
    let important_vars = [
        "XAI_API_KEY",
        "OPENAI_API_KEY",
        "DEEPSEEK_API_KEY",
        "ZOOM_ACCOUNT_ID",
        "ZOOM_CLIENT_ID",
        "ZOOM_CLIENT_SECRET",
    ];

    for var in important_vars {
        match env::var(var) {
            Ok(val) => info!("  ✅ {} is SET (length: {})", var, val.len()),
            Err(_) => info!("  ❌ {} is NOT SET", var),
        }
    }

    // Check .env file
    info!("📄 .env File Check:");
    let env_paths = [".env", "/Users/shaunstuart/RustroverProjects/ducktape/.env"];

    for path in env_paths {
        if Path::new(path).exists() {
            info!("  ✅ Found .env file at: {}", path);

            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                let mut found_vars = Vec::new();

                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.starts_with('#') || line.trim().is_empty() {
                            continue;
                        }

                        if let Some(pos) = line.find('=') {
                            let key = line[..pos].trim();
                            found_vars.push(key.to_string());
                        }
                    }
                }

                info!("  📋 Variables in .env file: {}", found_vars.join(", "));
            }
        } else {
            info!("  ❌ No .env file at: {}", path);
        }
    }
}

/// Sets a placeholder API key for development/testing only
/// 
/// # Warning
/// This function should only be used during development and testing,
/// never in production environments. Proper API keys should be set
/// through environment variables or the .env file.
pub fn force_set_api_key() -> bool {
    // If environment variable is not set, set a placeholder for development
    if env::var("XAI_API_KEY").is_err() {
        let api_key = "xai-placeholder-development-key-not-for-production-use";
        env::set_var("XAI_API_KEY", api_key);
        info!(
            "🔑 Set placeholder XAI_API_KEY for development (length: {})",
            api_key.len()
        );
        return true;
    }

    false
}
