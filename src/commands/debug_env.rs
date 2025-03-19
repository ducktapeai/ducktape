use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use std::env;
use log::info;

pub fn debug_environment_variables() -> Result<()> {
    println!("🔍 ENVIRONMENT VARIABLES DIAGNOSTIC");
    println!("==================================");
    
    // Check .env file
    let env_file_path = ".env";
    println!("\n📄 .env file:");
    if Path::new(env_file_path).exists() {
        println!("  ✅ Found at {}", env_file_path);
        
        // Read content
        match fs::read_to_string(env_file_path) {
            Ok(content) => {
                println!("  📝 Content:");
                for line in content.lines() {
                    if line.trim().starts_with("XAI_API_KEY=") || 
                       line.trim().starts_with("OPENAI_API_KEY=") ||
                       line.trim().starts_with("DEEPSEEK_API_KEY=") {
                        let parts: Vec<&str> = line.split('=').collect();
                        if parts.len() >= 2 {
                            let key = parts[0];
                            let value_length = parts[1].len();
                            println!("    - {}: {} characters", key, value_length);
                        } else {
                            println!("    - {}: malformed line", line);
                        }
                    } else {
                        println!("    - {}", line);
                    }
                }
            },
            Err(e) => println!("  ❌ Error reading file: {}", e),
        }
    } else {
        println!("  ❌ Not found");
    }
    
    // Check environment variables
    println!("\n🔐 Environment Variables:");
    let important_vars = [
        "XAI_API_KEY", "OPENAI_API_KEY", "DEEPSEEK_API_KEY",
        "ZOOM_ACCOUNT_ID", "ZOOM_CLIENT_ID", "ZOOM_CLIENT_SECRET"
    ];
    
    for var in important_vars {
        match env::var(var) {
            Ok(value) => println!("  ✅ {}: SET (length: {})", var, value.len()),
            Err(_) => println!("  ❌ {}: NOT SET", var),
        }
    }
    
    // Try API key getter
    println!("\n🔑 API Key Retrieval Test:");
    match crate::api_server::get_api_key("XAI_API_KEY") {
        Some(key) => println!("  ✅ XAI_API_KEY retrieved successfully (length: {})", key.len()),
        None => println!("  ❌ XAI_API_KEY could not be retrieved"),
    }
    
    Ok(())
}
