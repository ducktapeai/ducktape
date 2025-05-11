#!/bin/bash
# Test script for timezone support in Ducktape time parser
# Run this script to test timezone handling interactively

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}=== Testing Ducktape Time Parser Timezone Support ===${RESET}"
echo "This script will test the timezone support in the time parser"
echo

echo -e "${BLUE}Building timezone test program...${RESET}"
cat > test_timezone.rs << 'EOF'
use chrono::{DateTime, Local, TimeZone, Utc};
use chrono_tz::Tz;
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, BufRead};

// Simplified timezone map for testing
fn get_timezone_map() -> HashMap<&'static str, Tz> {
    let mut map = HashMap::new();
    map.insert("PST", Tz::America__Los_Angeles);
    map.insert("PDT", Tz::America__Los_Angeles);
    map.insert("MST", Tz::America__Denver);
    map.insert("MDT", Tz::America__Denver);
    map.insert("CST", Tz::America__Chicago);
    map.insert("CDT", Tz::America__Chicago);
    map.insert("EST", Tz::America__New_York);
    map.insert("EDT", Tz::America__New_York);
    map.insert("GMT", Tz::Etc__GMT);
    map.insert("UTC", Tz::UTC);
    map
}

// Parse time with timezone
fn parse_time_with_timezone(time_str: &str) -> Option<(u32, u32, Option<Tz>)> {
    let timezone_map = get_timezone_map();
    
    // Split time string into parts
    let parts: Vec<&str> = time_str.trim().split_whitespace().collect();
    
    // Last part might be a timezone abbreviation
    let mut timezone = None;
    let mut time_parts = parts.clone();
    
    if parts.len() > 1 {
        let last_part = parts.last().unwrap().to_uppercase();
        if timezone_map.contains_key(last_part.as_str()) {
            timezone = Some(timezone_map[last_part.as_str()]);
            time_parts.pop(); // Remove timezone part
        }
    }
    
    // Parse the time part (simplified for this example)
    let time_re = Regex::new(r"(?i)^(\d{1,2})(?::(\d{2}))?\s*([ap]\.?m\.?)?$").unwrap();
    let time_only = time_parts.join(" ");
    
    if let Some(caps) = time_re.captures(&time_only) {
        let hour_str = caps.get(1).unwrap().as_str();
        let minute_str = caps.get(2).map_or("0", |m| m.as_str());
        let ampm = caps.get(3).map(|m| m.as_str().to_lowercase());
        
        if let (Ok(h), Ok(m)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            // Convert to 24-hour format
            let hour = if let Some(ampm_str) = ampm {
                if ampm_str.starts_with('p') && h < 12 {
                    h + 12
                } else if ampm_str.starts_with('a') && h == 12 {
                    0
                } else {
                    h
                }
            } else if time_only.to_lowercase().contains("pm") && h < 12 {
                h + 12
            } else if time_only.to_lowercase().contains("am") && h == 12 {
                0
            } else {
                h
            };
            
            return Some((hour, m, timezone));
        }
    }
    
    None
}

// Adjust time for timezone
fn adjust_time_for_timezone(hour: u32, minute: u32, source_tz: Tz) -> (u32, u32) {
    // Get today's date
    let today = Local::now().date_naive();
    
    // Create time in source timezone
    let time = chrono::NaiveTime::from_hms_opt(hour, minute, 0).unwrap_or_default();
    let naive_dt = chrono::NaiveDateTime::new(today, time);
    
    // Convert to source timezone
    if let Some(source_dt) = source_tz.from_local_datetime(&naive_dt).single() {
        // Convert to UTC
        let utc_dt = source_dt.with_timezone(&Utc);
        
        // Convert to local timezone
        let local_dt = utc_dt.with_timezone(&Local);
        
        return (local_dt.hour(), local_dt.minute());
    }
    
    // Fallback
    (hour, minute)
}

fn main() {
    println!("Time Parser Timezone Test");
    println!("Your local timezone: {}", Local::now().format("%Z"));
    println!("\nEnter time expressions with timezone (e.g., '8pm PST' or '15:30 EST')");
    println!("Type 'exit' to quit\n");
    
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(input) = line {
            if input.trim().to_lowercase() == "exit" {
                break;
            }
            
            // Basic command parser
            println!("Input: {}", input);
            
            // Parse time with timezone
            if let Some((hour, minute, timezone)) = parse_time_with_timezone(&input) {
                println!("Parsed time: {:02}:{:02} (24-hour format)", hour, minute);
                
                // If timezone provided, adjust to local time
                if let Some(tz) = timezone {
                    println!("Timezone: {} ({})", tz.name(), tz);
                    let (adjusted_hour, adjusted_minute) = adjust_time_for_timezone(hour, minute, tz);
                    println!("Adjusted to local time: {:02}:{:02}", adjusted_hour, adjusted_minute);
                } else {
                    println!("No timezone specified, assuming local time");
                }
            } else {
                println!("Could not parse time from input");
            }
            
            println!();
        }
    }
}
EOF

# Install required dependencies if not already installed
echo "Checking for required dependencies..."
if ! cargo install chrono-tz regex --list | grep -q "chrono-tz"; then
    echo -e "${BLUE}Installing chrono-tz and regex crates...${RESET}"
    cargo install chrono-tz regex
fi

# Build and run the test program
echo -e "${BLUE}Building timezone test program...${RESET}"
rustc test_timezone.rs -L ~/.cargo/registry/src --extern chrono=~/.cargo/registry/src/github.com-*/chrono-*/libchrono.rlib --extern chrono_tz=~/.cargo/registry/src/github.com-*/chrono-tz-*/libchrono_tz.rlib --extern regex=~/.cargo/registry/src/github.com-*/regex-*/libregex.rlib

echo -e "${GREEN}Test program built successfully!${RESET}"
echo -e "${BLUE}Try expressions like:${RESET}"
echo "  8pm PST"
echo "  3:30pm EST"
echo "  10:00am GMT"
echo "  7pm"
echo "  Type 'exit' to quit"
echo

./test_timezone
