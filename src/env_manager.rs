use log::info;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

/// Required environment variables that the application needs
pub const REQUIRED_ENV_VARS: &[&str] = &[
    "XAI_API_KEY",
    "OPENAI_API_KEY",
    "DEEPSEEK_API_KEY",
    "ZOOM_ACCOUNT_ID",
    "ZOOM_CLIENT_ID",
    "ZOOM_CLIENT_SECRET",
];

/// Names of optional environment variables
pub const OPTIONAL_ENV_VARS: &[&str] = &["DUCKTAPE_LOG_LEVEL", "DUCKTAPE_CONFIG_PATH"];

/// Placeholder key for development/testing purposes only
const FALLBACK_XAI_API_KEY: &str = "xai-placeholder-development-key-not-for-production-use";

/// Checks if all required environment variables are set
/// Returns true if all required vars are present, false otherwise
pub fn check_env_vars() -> bool {
    let mut all_present = true;
    
    for var in REQUIRED_ENV_VARS {
        match env::var(var) {
            Ok(val) if !val.trim().is_empty() => (),
            _ => {
                if var == &"XAI_API_KEY" {
                    info!("Setting placeholder XAI_API_KEY for development");
                    env::set_var("XAI_API_KEY", FALLBACK_XAI_API_KEY);
                } else {
                    println!("❌ Missing required environment variable: {}", var);
                    all_present = false;
                }
            }
        }
    }
    
    all_present
}

/// Loads environment variables from a .env file if it exists
pub fn load_env_file() -> io::Result<()> {
    // Try to load from .env file
    match dotenvy::dotenv() {
        Ok(path) => {
            info!("Loaded environment from {:?}", path);
            Ok(())
        }
        Err(e) => {
            info!("No .env file found or error loading it: {}", e);
            create_env_template()
        }
    }
}

/// Creates a template .env file with required variables
fn create_env_template() -> io::Result<()> {
    let env_path = PathBuf::from(".env");
    
    // Don't overwrite existing .env file
    if env_path.exists() {
        return Ok(());
    }
    
    let mut file = File::create(env_path)?;
    
    // Write required variables
    for var in REQUIRED_ENV_VARS {
        writeln!(file, "{}=", var)?;
    }
    
    // Write optional variables with comments
    for var in OPTIONAL_ENV_VARS {
        writeln!(file, "# {}=", var)?;
    }
    
    Ok(())
}

/// Gets an environment variable, with fallback for development
pub fn get_env_var(name: &str) -> String {
    match env::var(name) {
        Ok(value) => {
            if name == "XAI_API_KEY" && value.trim().is_empty() {
                return FALLBACK_XAI_API_KEY.to_string();
            }
            value
        }
        Err(_) => {
            if name == "XAI_API_KEY" {
                return FALLBACK_XAI_API_KEY.to_string();
            }
            String::new()
        }
    }
}

// Save environment variables to .env file
pub fn save_environment(
    variables: &std::collections::HashMap<String, String>,
) -> io::Result<PathBuf> {
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
