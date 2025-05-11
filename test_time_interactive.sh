#!/bin/bash
# Alternate test script for Ducktape time parser integration
# This script simulates the parsing process without executing the CLI
# Updated to include timezone support testing

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RESET='\033[0m'

echo -e "${BLUE}=== Testing Ducktape Time Parser Integration with Timezone Support ===${RESET}"
echo "This script will verify natural language commands with timezone handling"
echo

echo -e "${BLUE}Building enhanced test program with timezone support...${RESET}"
cat > test_time_stdin.rs << 'EOF'
use std::io::{self, BufRead};
use std::collections::HashMap;

// Timezone abbreviation mapping
fn get_timezone_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("PST", "America/Los_Angeles");
    map.insert("PDT", "America/Los_Angeles");
    map.insert("MST", "America/Denver");
    map.insert("MDT", "America/Denver");
    map.insert("CST", "America/Chicago");
    map.insert("CDT", "America/Chicago");
    map.insert("EST", "America/New_York");
    map.insert("EDT", "America/New_York");
    map.insert("GMT", "GMT");
    map.insert("UTC", "UTC");
    map
}

// Extract timezone from string if present
fn extract_timezone(str: &str) -> Option<&str> {
    let timezone_map = get_timezone_map();
    
    for (abbr, _) in timezone_map.iter() {
        if str.contains(abbr) {
            return Some(abbr);
        }
    }
    
    None
}

// Simplified version of our parse_time_with_ampm function
fn parse_time_with_ampm(time_str: &str) -> Option<(u32, u32, Option<&str>)> {
    let time_lower = time_str.to_lowercase();
    
    // Check for timezone
    let timezone = extract_timezone(time_str);
    
    // Remove timezone part for easier parsing
    let clean_time_str = match timezone {
        Some(tz) => time_lower.replace(tz.to_lowercase().as_str(), "").trim().to_string(),
        None => time_lower
    };
    
    // Extract hour and minute parts
    let parts: Vec<&str> = if clean_time_str.contains(':') {
        clean_time_str.split(':').collect()
    } else {
        // For formats like "8pm" with no colon
        let mut hour_str = "";
        for (i, c) in clean_time_str.chars().enumerate() {
            if !c.is_digit(10) {
                hour_str = &clean_time_str[0..i];
                break;
            }
        }
        vec![hour_str, "0"] // Default minute to 0
    };
    
    if parts.len() < 1 || parts[0].is_empty() {
        return None;
    }
    
    // Parse hour
    let h_val = match parts[0].parse::<u32>() {
        Ok(h) => h,
        Err(_) => return None,
    };
    
    // Parse minute
    let m_val = if parts.len() > 1 {
        let minute_part = parts[1].trim_end_matches(|c: char| !c.is_digit(10));
        match minute_part.parse::<u32>() {
            Ok(m) => m,
            Err(_) => 0, // Default to 0 if can't parse
        }
    } else {
        0 // Default to 0 if no minute part
    };
    
    // Check for AM/PM
    let is_pm = clean_time_str.contains("pm") || clean_time_str.contains("p.m");
    let is_am = clean_time_str.contains("am") || clean_time_str.contains("a.m");
    
    // Convert to 24-hour format
    let hour_24 = if is_pm && h_val < 12 {
        h_val + 12
    } else if is_am && h_val == 12 {
        0
    } else {
        h_val
    };
    
    // Return parsed time if valid
    if hour_24 < 24 && m_val < 60 {
        return Some((hour_24, m_val, timezone));
    }
    
    None
}

fn extract_time_from_command(command: &str) -> Option<(String, Option<&str>)> {
    let patterns = ["at ", "tonight at ", "for ", "from "];
    let timezone_map = get_timezone_map();
    let mut found_timezone = None;
    
    // First, check for timezone anywhere in the command
    for (abbr, _) in timezone_map.iter() {
        if command.contains(abbr) {
            found_timezone = Some(*abbr);
            break;
        }
    }
    
    for pattern in &patterns {
        if let Some(idx) = command.find(pattern) {
            let after_pattern = &command[idx + pattern.len()..];
            let words: Vec<&str> = after_pattern.split_whitespace().collect();
            
            for word in words {
                if word.to_lowercase().contains("am") || word.to_lowercase().contains("pm") {
                    // Try first with the timezone if found
                    if found_timezone.is_some() {
                        let full_time = format!("{} {}", word, found_timezone.unwrap());
                        if let Some((hour, minute, _)) = parse_time_with_ampm(&full_time) {
                            return Some((format!("{:02}:{:02}", hour, minute), found_timezone));
                        }
                    }
                    
                    // Try without timezone (or as fallback)
                    if let Some((hour, minute, tz)) = parse_time_with_ampm(word) {
                        // If timezone was found in the time expression itself
                        if tz.is_some() {
                            found_timezone = tz;
                        }
                        return Some((format!("{:02}:{:02}", hour, minute), found_timezone));
                    }
                }
            }
        }
    }
    
    None
}

// Simulated timezone conversion
fn timezone_adjusted_time(time: &str, timezone: Option<&str>) -> String {
    if timezone.is_none() {
        return format!("{}  (local time)", time);
    }
    
    // Parse the time
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return format!("{}  (from {} - parsing error)", time, timezone.unwrap());
    }
    
    let hour: u32 = parts[0].parse().unwrap_or(0);
    let minute: u32 = parts[1].parse().unwrap_or(0);
    
    // Simplified timezone adjustment - this is just a simulation!
    // In a real implementation, we'd use chrono-tz for proper conversion
    let hour_offset = match timezone.unwrap() {
        "PST" => 5,  // PST is UTC-8, assuming local is UTC-3 (simplified example)
        "MST" => 4,  // MST is UTC-7, assuming local is UTC-3
        "CST" => 3,  // CST is UTC-6, assuming local is UTC-3
        "EST" => 2,  // EST is UTC-5, assuming local is UTC-3
        _ => 0,      // Default no adjustment
    };
    
    // Adjust the hour based on the simulated timezone difference
    let mut local_hour = (hour + hour_offset) % 24;
    
    // Format the result
    format!("{}:{:02}  (converted from {} to local time)", 
            local_hour, minute, timezone.unwrap())
}

fn main() {
    println!("Type natural language commands with time expressions, one per line.");
    println!("Type 'exit' to quit.");
    println!();
    
    println!("Current simulation assumes your local timezone is 'US Eastern' for display purposes.");
    println!("In the actual implementation, your system's local timezone will be used.");
    println!();
    
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(command) = line {
            if command.trim().to_lowercase() == "exit" {
                break;
            }
            
            println!("Command: {}", command);
            
            if let Some((parsed_time, timezone)) = extract_time_from_command(&command) {
                println!("Extracted time: {} (24-hour format)", parsed_time);
                
                if timezone.is_some() {
                    println!("Detected timezone: {}", timezone.unwrap());
                    println!("Local time would be: {}", timezone_adjusted_time(&parsed_time, timezone));
                } else {
                    println!("No timezone detected, assuming local time");
                }
            } else {
                println!("No time expression found");
            }
            println!();
        }
    }
}
EOF

rustc test_time_stdin.rs
chmod +x test_time_stdin

echo -e "${BLUE}Enhanced test program built with timezone support. You can now enter natural language commands to test.${RESET}"
echo -e "${BLUE}Try commands like:${RESET}"
echo "  create an event called Team Meeting tonight at 7pm"
echo "  schedule a meeting called Review at 3:30pm PST"
echo "  schedule a zoom event at 9pm EST called check in"
echo "  create an event called Breakfast at 9am CDT"
echo "  Type 'exit' to quit"
echo

./test_time_stdin
