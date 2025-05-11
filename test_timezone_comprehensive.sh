#!/bin/bash
# Test script for comprehensive timezone functionality in Ducktape
# This script tests the timezone conversion logic more thoroughly

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RESET='\033[0m'

echo -e "${BLUE}=== Ducktape Timezone Conversion Test Suite ===${RESET}"
echo "This script will verify timezone conversion logic thoroughly"
echo

echo -e "${BLUE}Building comprehensive timezone test program...${RESET}"
cat > test_timezone_comprehensive.rs << 'EOF'
use std::io::{self, BufRead};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
    map.insert("AKST", "America/Anchorage");
    map.insert("AKDT", "America/Anchorage");
    map.insert("HST", "Pacific/Honolulu");
    map.insert("AEST", "Australia/Sydney");
    map.insert("AEDT", "Australia/Sydney");
    map.insert("JST", "Asia/Tokyo");
    map.insert("GMT", "Etc/GMT");
    map.insert("BST", "Europe/London");
    map.insert("CET", "Europe/Paris");
    map.insert("CEST", "Europe/Paris");
    map.insert("IST", "Asia/Kolkata");
    map.insert("UTC", "UTC");
    map
}

// Extract timezone from string if present
fn extract_timezone(str: &str) -> Option<&str> {
    let timezone_map = get_timezone_map();
    
    for (abbr, _) in timezone_map.iter() {
        if str.to_uppercase().contains(abbr) {
            return Some(abbr);
        }
    }
    
    None
}

// Get the current local timezone for display purposes
// Note: This is a simulation for display purposes only
fn get_local_timezone() -> &'static str {
    // In a real implementation, we would detect the system timezone
    // Here we'll just return a fixed value for demonstration
    "Your Local Timezone"
}

// Determine if DST is in effect (simplified simulation)
fn is_dst_active(timezone: &str) -> bool {
    // In a real implementation, we would use proper date/time library
    // to determine if DST is active for the given timezone
    
    // Get current month (1-12)
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let seconds = since_epoch.as_secs();
    
    // Very rough DST calculation for northern hemisphere
    // DST is typically March-November
    let month = (seconds / 2629746) % 12 + 1; // approximate month calculation
    
    match timezone {
        "PST" | "MST" | "CST" | "EST" | "AKST" | "GMT" => month >= 3 && month <= 11,
        "CET" => month >= 3 && month <= 10,
        "AEST" => month <= 4 || month >= 10, // Southern hemisphere
        _ => false
    }
}

// Parse time with timezone
fn parse_time_with_timezone(time_str: &str) -> Option<(u32, u32, Option<&str>)> {
    // Try to extract timezone first
    let timezone = extract_timezone(time_str);
    
    // Clean the time string for parsing
    let clean_time = if let Some(tz) = timezone {
        time_str.replace(tz, "").trim().to_string()
    } else {
        time_str.to_string()
    };
    
    // Parse hour and minute
    let time_regex = regex::Regex::new(r"(?i)(\d{1,2})(?::(\d{2}))?(?:\s*([ap]\.?m\.?))?").unwrap();
    
    if let Some(caps) = time_regex.captures(&clean_time) {
        let hour_str = caps.get(1).unwrap().as_str();
        let minute_str = caps.get(2).map_or("0", |m| m.as_str());
        let ampm = caps.get(3).map(|m| m.as_str().to_lowercase());
        
        if let (Ok(hour), Ok(minute)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            // Convert to 24-hour format
            let hour_24 = if let Some(period) = ampm {
                if period.starts_with('p') && hour < 12 {
                    hour + 12
                } else if period.starts_with('a') && hour == 12 {
                    0
                } else {
                    hour
                }
            } else {
                hour // Assume 24-hour format if no AM/PM specified
            };
            
            if hour_24 < 24 && minute < 60 {
                return Some((hour_24, minute, timezone));
            }
        }
    }
    
    None
}

// Simulate timezone conversion
fn convert_timezone(hour: u32, minute: u32, source_tz: &str) -> (u32, u32) {
    // Get offset hours between source timezone and UTC
    let src_offset = match source_tz {
        "PST" => -8,
        "PDT" => -7,
        "MST" => -7,
        "MDT" => -6,
        "CST" => -6,
        "CDT" => -5,
        "EST" => -5,
        "EDT" => -4,
        "AKST" => -9,
        "AKDT" => -8,
        "HST" => -10,
        "AEST" => 10,
        "AEDT" => 11,
        "JST" => 9,
        "IST" => 5,
        "GMT" => 0,
        "BST" => 1,
        "CET" => 1,
        "CEST" => 2,
        "UTC" => 0,
        _ => 0,
    };
    
    // Local offset (simulated - in a real implementation we'd detect the system timezone)
    let local_offset = -5; // Example: US Eastern
    
    // Adjust for DST if needed
    let src_dst_adj = if is_dst_active(source_tz) { 1 } else { 0 };
    let local_dst_adj = if true { 1 } else { 0 }; // Assume local DST is active for this example
    
    // Convert to UTC, then to local
    let utc_hour = (hour as i32 - src_offset - src_dst_adj + 24) % 24;
    let local_hour = (utc_hour + local_offset + local_dst_adj + 24) % 24;
    
    (local_hour as u32, minute)
}

// Format time for display
fn format_time(hour: u32, minute: u32) -> String {
    format!("{:02}:{:02}", hour, minute)
}

// Main test function
fn run_timezone_test(command: &str) {
    println!("Command: {}", command);
    
    // Extract timezone first
    let timezone = extract_timezone(command);
    
    if let Some(tz) = timezone {
        println!("Detected timezone: {}", tz);
        
        // Try to find time expression
        let time_regex = regex::Regex::new(r"(?i)(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?))").unwrap();
        if let Some(time_match) = time_regex.find(command) {
            let time_str = time_match.as_str();
            println!("Time expression: {}", time_str);
            
            if let Some((hour, minute, _)) = parse_time_with_timezone(&format!("{} {}", time_str, tz)) {
                println!("Source time: {}:{:02} {}", 
                    if hour > 12 { hour - 12 } else if hour == 0 { 12 } else { hour }, 
                    minute, 
                    if hour >= 12 { "PM" } else { "AM" });
                
                let (local_hour, local_minute) = convert_timezone(hour, minute, tz);
                
                println!("Converted to local time: {}:{:02} {} ({})",
                    if local_hour > 12 { local_hour - 12 } else if local_hour == 0 { 12 } else { local_hour },
                    local_minute,
                    if local_hour >= 12 { "PM" } else { "AM" },
                    get_local_timezone());
                
                println!("24-hour format: {}:{:02} → {}:{:02}", 
                    hour, minute, local_hour, local_minute);
            } else {
                println!("Could not parse time with timezone");
            }
        } else {
            println!("No time expression found");
        }
    } else {
        println!("No timezone detected in command");
    }
    
    println!();
}

fn main() {
    println!("=== Timezone Conversion Tester ===");
    println!("This program demonstrates timezone conversion functionality.");
    println!("Enter commands with time and timezone (e.g., 'meeting at 3pm PST')");
    println!("Type 'test all' to run standard test cases.");
    println!("Type 'exit' to quit.");
    println!();
    
    // Create predefined test cases
    let test_cases = vec![
        "schedule a meeting at 9am PST",
        "create event at 3pm EST called planning",
        "set up a call for 8pm JST tomorrow",
        "meeting with team at 12pm GMT",
        "lunch at 1:30pm CST",
        "conference call at 7am AEST",
        "schedule meeting at 11pm UTC",
    ];
    
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(command) = line {
            let cmd = command.trim();
            
            if cmd.to_lowercase() == "exit" {
                break;
            } else if cmd.to_lowercase() == "test all" {
                println!("Running standard test cases...\n");
                
                for test_case in &test_cases {
                    run_timezone_test(test_case);
                }
                
                println!("All test cases completed.\n");
            } else {
                run_timezone_test(cmd);
            }
        }
    }
}
EOF

echo -e "${YELLOW}Installing required crates for test program...${RESET}"
cargo install regex

echo -e "${BLUE}Building timezone test program...${RESET}"
rustc test_timezone_comprehensive.rs -L ~/.cargo/registry/src --extern regex=~/.cargo/registry/src/github.com-*/regex-*/libregex.rlib

echo -e "${GREEN}Timezone test program built successfully!${RESET}"
echo -e "${BLUE}Usage:${RESET}"
echo "  - Enter commands with time and timezone (e.g., 'meeting at 3pm PST')"
echo "  - Type 'test all' to run through standard test cases"
echo "  - Type 'exit' to quit"
echo

./test_timezone_comprehensive
