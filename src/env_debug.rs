use log::{error, info};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

pub fn force_set_api_key() -> bool {
    // If environment variable is not set, try to set it from hardcoded value
    if env::var("XAI_API_KEY").is_err() {
        let api_key =
            "xai-vLxMy9tYMTkU4z1XwRc0LE74eUEg7acbvb9NL95oNqz3KlKJxAOtrPaCOtbA1estp0Z4foPGycmV0X8P";
        env::set_var("XAI_API_KEY", api_key);
        info!(
            "🔑 Forced XAI_API_KEY to be set (length: {})",
            api_key.len()
        );
        return true;
    }

    false
}
