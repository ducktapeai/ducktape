use std::env;
use std::fs::{self, OpenOptions, File};
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use log::{info, error, debug};

// Define important environment variables
pub const IMPORTANT_VARS: &[&str] = &[
    "XAI_API_KEY", "OPENAI_API_KEY", "DEEPSEEK_API_KEY",
    "ZOOM_ACCOUNT_ID", "ZOOM_CLIENT_ID", "ZOOM_CLIENT_SECRET",
    "GOOGLE_CALENDAR_CREDENTIALS"
];

// Hardcoded fallback API key - use yours from the environment export
const FALLBACK_XAI_API_KEY: &str = "xai-vLxMy9tYMTkU4z1XwRc0LE74eUEg7acbvb9NL95oNqz3KlKJxAOtrPaCOtbA1estp0Z4foPGycmV0X8P";

// Get all possible locations for .env file
fn get_env_file_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Current directory
    paths.push(PathBuf::from(".env"));
    
    // Home directory
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".env"));
    }
    
    // Project root directory
    if let Ok(current_dir) = env::current_dir() {
        paths.push(current_dir.join(".env"));
    }
    
    // Add absolute path for the user's specific directory
    paths.push(PathBuf::from("/Users/shaunstuart/RustroverProjects/ducktape/.env"));
    
    paths
}

// Initialize environment variables from all possible sources
pub fn init_environment() {
    // First try dotenv crate
    match dotenv::dotenv() {
        Ok(path) => info!("Loaded environment from dotenv: {:?}", path),
        Err(e) => debug!("Dotenv error: {}", e),
    }
    
    // Then try manual loading from all possible locations
    let paths = get_env_file_paths();
    for path in &paths {
        if path.exists() {
            info!("Found .env file at {:?}", path);
            if let Err(e) = load_env_file(path) {
                error!("Error loading env file at {:?}: {}", path, e);
            } else {
                info!("Successfully loaded environment from {:?}", path);
            }
        }
    }
    
    // Apply fallbacks for critical variables
    ensure_critical_vars();
    
    // Log current environment state
    info!("Environment variables after initialization:");
    for var in IMPORTANT_VARS {
        let is_set = env::var(var).is_ok();
        info!("  {}: {}", var, if is_set { "SET" } else { "NOT SET" });
    }
}

// Load environment variables from a file
fn load_env_file(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        
        // Parse KEY=VALUE format
        if let Some(pos) = line.find('=') {
            let (key, value) = line.split_at(pos);
            let key = key.trim();
            // Skip the '=' character
            let value = value[1..].trim();
            
            // Only set if not already set in environment
            if env::var(key).is_err() {
                env::set_var(key, value);
                debug!("Set environment variable: {}", key);
            }
        }
    }
    
    Ok(())
}

// Ensure critical variables have values
fn ensure_critical_vars() {
    // Make sure XAI_API_KEY is set
    if env::var("XAI_API_KEY").is_err() {
        info!("Setting fallback XAI_API_KEY");
        env::set_var("XAI_API_KEY", FALLBACK_XAI_API_KEY);
    }
}

// Get API key with fallback
pub fn get_api_key(name: &str) -> String {
    match env::var(name) {
        Ok(value) => value,
        Err(_) => {
            if name == "XAI_API_KEY" {
                return FALLBACK_XAI_API_KEY.to_string();
            }
            String::new()
        }
    }
}

// Save environment variables to .env file
pub fn save_environment(variables: &std::collections::HashMap<String, String>) -> io::Result<PathBuf> {
    // Choose appropriate location - prefer current directory
    let env_path = PathBuf::from(".env");
    
    // Create or update file
    let mut content = String::new();
    
    // Add each variable
    for (key, value) in variables {
        if !value.is_empty() {
            content.push_str(&format!("{}={}\n", key, value));
        }
    }
    
    // Write to file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&env_path)?;
    
    file.write_all(content.as_bytes())?;
    
    // Also update process environment
    for (key, value) in variables {
        if !value.is_empty() {
            env::set_var(key, value);
        }
    }
    
    Ok(env_path)
}
