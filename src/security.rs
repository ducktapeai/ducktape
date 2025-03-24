use std::process::Command;
use anyhow::{Result, anyhow};
use secrecy::{Secret, ExposeSecret};
use regex::Regex;

// Safe wrapper for AppleScript execution
pub fn execute_applescript(script: &str) -> Result<String> {
    // Validate script content
    if !is_safe_applescript(script) {
        return Err(anyhow!("Invalid AppleScript content"));
    }

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("AppleScript execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Validate AppleScript content
fn is_safe_applescript(script: &str) -> bool {
    // Basic safety checks
    if script.contains("do shell script") || 
       script.contains("system events") ||
       script.contains(";") ||
       script.contains("|") ||
       script.contains("&") {
        return false;
    }

    // Only allow specific AppleScript commands we use
    let allowed_patterns = vec![
        r"^tell application \"Calendar\".*end tell$",
        r"^tell application \"Reminders\".*end tell$",
        r"^tell application \"Notes\".*end tell$"
    ];

    for pattern in allowed_patterns {
        if Regex::new(pattern).unwrap().is_match(script) {
            return true;
        }
    }

    false
}

// Secure API key handling
pub struct ApiKey(Secret<String>);

impl ApiKey {
    pub fn new(key: String) -> Self {
        ApiKey(Secret::new(key))
    }

    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

// Secure environment variable handling
pub fn get_api_key(name: &str) -> Option<ApiKey> {
    std::env::var(name).ok().map(ApiKey::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_applescript() {
        // Valid scripts
        assert!(is_safe_applescript(
            "tell application \"Calendar\"\nget name of calendars\nend tell"
        ));
        
        // Invalid scripts
        assert!(!is_safe_applescript(
            "do shell script \"rm -rf /\""
        ));
        assert!(!is_safe_applescript(
            "tell application \"System Events\"\nclick button\nend tell"
        ));
    }
}