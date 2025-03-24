use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Create a persistent environment store that can be accessed from multiple modules
pub static ENV_STORE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Important environment variables to track
pub static IMPORTANT_ENV_VARS: &[&str] = &[
    "XAI_API_KEY",
    "OPENAI_API_KEY",
    "DEEPSEEK_API_KEY",
    "ZOOM_ACCOUNT_ID",
    "ZOOM_CLIENT_ID",
    "ZOOM_CLIENT_SECRET",
    "GOOGLE_CALENDAR_CREDENTIALS",
];

// Helper function to get a stored environment variable
pub fn get_env_var(name: &str) -> Option<String> {
    // First try process environment
    if let Ok(value) = std::env::var(name) {
        return Some(value);
    }

    // Then try our persistent store
    if let Ok(store) = ENV_STORE.lock() {
        if let Some(value) = store.get(name) {
            return Some(value.clone());
        }
    }

    None
}

// Helper function to set an environment variable in both the process and store
pub fn set_env_var(name: &str, value: &str) {
    // Set in process environment
    std::env::set_var(name, value);

    // Store in persistent store
    if let Ok(mut store) = ENV_STORE.lock() {
        store.insert(name.to_string(), value.to_string());
    }
}

// Helper function to restore all environment variables from store
pub fn restore_env_vars() {
    if let Ok(store) = ENV_STORE.lock() {
        for (key, value) in store.iter() {
            // Only set if not already in process environment
            if std::env::var(key).is_err() {
                std::env::set_var(key, value);
            }
        }
    }
}
