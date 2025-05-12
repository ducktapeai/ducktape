#!/bin/bash
# Alternate test script for Ducktape time parser integration
# This script simulates the parsing process without executing the CLI

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}=== Testing Ducktape Time Parser Integration ===${RESET}"
echo "This script will verify natural language commands manually"
echo

echo -e "${BLUE}Building simple test program...${RESET}"
cat > test_time_stdin.rs << 'EOF'
use std::io::{self, BufRead};

// Simplified version of our parse_time_with_ampm function
fn parse_time_with_ampm(time_str: &str) -> Option<(u32, u32)> {
    let time_lower = time_str.to_lowercase();
    
    // Extract hour and minute parts
    let parts: Vec<&str> = if time_lower.contains(':') {
        time_lower.split(':').collect()
    } else {
        // For formats like "8pm" with no colon
        let mut hour_str = "";
        for (i, c) in time_lower.chars().enumerate() {
            if !c.is_digit(10) {
                hour_str = &time_lower[0..i];
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
    let is_pm = time_lower.contains("pm") || time_lower.contains("p.m");
    let is_am = time_lower.contains("am") || time_lower.contains("a.m");
    
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
        return Some((hour_24, m_val));
    }
    
    None
}

fn extract_time_from_command(command: &str) -> Option<String> {
    let patterns = ["at ", "tonight at ", "for ", "from "];
    
    for pattern in &patterns {
        if let Some(idx) = command.find(pattern) {
            let after_pattern = &command[idx + pattern.len()..];
            let words: Vec<&str> = after_pattern.split_whitespace().collect();
            
            for word in words {
                if word.to_lowercase().contains("am") || word.to_lowercase().contains("pm") {
                    if let Some((hour, minute)) = parse_time_with_ampm(word) {
                        return Some(format!("{:02}:{:02}", hour, minute));
                    }
                }
            }
        }
    }
    
    None
}

fn main() {
    println!("Type natural language commands with time expressions, one per line.");
    println!("Type 'exit' to quit.");
    println!();
    
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(command) = line {
            if command.trim().to_lowercase() == "exit" {
                break;
            }
            
            println!("Command: {}", command);
            
            if let Some(parsed_time) = extract_time_from_command(&command) {
                println!("Extracted time: {} (24-hour format)", parsed_time);
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

echo -e "${BLUE}Test program built. You can now enter natural language commands to test.${RESET}"
echo -e "${BLUE}Try commands like:${RESET}"
echo "  create an event called Team Meeting tonight at 7pm"
echo "  schedule a meeting called Review at 3:30pm"
echo "  create an event called Breakfast at 9am"
echo "  Type 'exit' to quit"
echo

./test_time_stdin
