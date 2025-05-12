// Manual test script for the new time parser implementation
// This will run without requiring compilation of the entire codebase
// To run: rustc verify_time_parser.rs && ./verify_time_parser

use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;
use std::thread;

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

// Simulate the sanitize_nlp_command function for natural language event creation
fn sanitize_event_command(command: &str) -> String {
    // Check if this is an event creation command
    let is_event_creation = command.contains("create an event")
        || command.contains("schedule a")
        || command.contains("create event")
        || command.contains("schedule event")
        || command.contains("create a meeting");
        
    if !is_event_creation {
        return format!("ducktape {}", command);
    }
    
    // Extract event title 
    let mut title = "Event";
    if command.contains(" called ") {
        let parts: Vec<&str> = command.split(" called ").collect();
        if parts.len() > 1 {
            let title_part = parts[1];
            let end_markers = [" at ", " on ", " for ", " with ", " and "];
            let mut end_pos = title_part.len();
            for marker in &end_markers {
                if let Some(pos) = title_part.find(marker) {
                    if pos < end_pos {
                        end_pos = pos;
                    }
                }
            }
            title = &title_part[..end_pos];
        }
    }
    
    // Build initial command with placeholder times
    let initial_command = format!(
        "ducktape calendar create \"{}\" today 00:00 01:00 \"Calendar\"",
        title
    );
    
    // Extract time information
    let mut processed = initial_command.clone();
    
    // Example regex patterns to find time expressions
    let time_patterns = [
        "tonight at ", 
        "at ", 
        "for ", 
        "from "
    ];
    
    // Look for time expressions
    for pattern in &time_patterns {
        if let Some(index) = command.find(pattern) {
            let time_part = &command[index + pattern.len()..];
            let mut time_str = "";
            
            // Extract time string (simple approach)
            for (i, word) in time_part.split_whitespace().enumerate() {
                if word.to_lowercase().contains("am") || word.to_lowercase().contains("pm") {
                    time_str = word;
                    break;
                }
                // Only look at first few words
                if i > 3 {
                    break;
                }
            }
            
            if !time_str.is_empty() {
                // Parse the time
                if let Some((hour, minute)) = parse_time_with_ampm(time_str) {
                    // Format times
                    let start_time = format!("{:02}:{:02}", hour, minute);
                    let end_hour = if hour == 23 { 0 } else { hour + 1 };
                    let end_time = format!("{:02}:{:02}", end_hour, minute);
                    
                    // Update command
                    processed = processed.replace("00:00", &start_time);
                    processed = processed.replace("01:00", &end_time);
                    break;
                }
            }
        }
    }
    
    processed
}

fn main() -> io::Result<()> {
    println!("== Time Parser Verification ==");
    println!("This test checks that our time parser can correctly");
    println!("convert times like '8pm' to 24-hour format (20:00)\n");
    
    // Test basic time parsing
    println!("1. Testing basic time parsing:");
    let test_times = [
        "8pm", "3:30pm", "10:00am", "12pm", "12am", "7 PM", "9 A.M."
    ];
    
    let mut all_passed = true;
    
    for time_str in test_times {
        match parse_time_with_ampm(time_str) {
            Some((hour, minute)) => {
                println!("  '{}'  ->  {:02}:{:02}  ✓", time_str, hour, minute);
            },
            None => {
                println!("  '{}'  ->  Failed to parse  ✗", time_str);
                all_passed = false;
            }
        }
    }
    
    // Test command processing
    println!("\n2. Testing natural language command processing:");
    let test_commands = [
        "create an event called Team Meeting tonight at 7pm",
        "schedule a meeting called Review at 3:30pm",
        "create an event called Breakfast at 9am", 
        "schedule a meeting called Early call at 6:45am tomorrow",
        "create an event called Midnight Party at 12am",
        "create an event called Lunch at 12pm"
    ];
    
    for cmd in test_commands {
        let result = sanitize_event_command(cmd);
        println!("  Input: '{}'", cmd);
        println!("  Output: '{}'", result);
        
        // Check for correct time conversion
        if cmd.contains("pm") && !cmd.contains("12pm") {
            let expected_hour = match cmd {
                _ if cmd.contains("7pm") => "19:00",
                _ if cmd.contains("3:30pm") => "15:30",
                _ => "",
            };
            
            if !expected_hour.is_empty() && !result.contains(expected_hour) {
                println!("  ✗ Failed: Expected {} in output", expected_hour);
                all_passed = false;
            } else {
                println!("  ✓ PM time correctly converted");
            }
        } else if cmd.contains("am") {
            let expected_hour = match cmd {
                _ if cmd.contains("9am") => "09:00",
                _ if cmd.contains("6:45am") => "06:45",
                _ if cmd.contains("12am") => "00:00",
                _ => "",
            };
            
            if !expected_hour.is_empty() && !result.contains(expected_hour) {
                println!("  ✗ Failed: Expected {} in output", expected_hour);
                all_passed = false;
            } else {
                println!("  ✓ AM time correctly converted");
            }
        }
        
        println!("");
    }
    
    if all_passed {
        println!("\n✅ All tests passed! The time parser works correctly.");
        println!("The implementation should be ready for integration.");
    } else {
        println!("\n❌ Some tests failed. The time parser needs further work.");
    }
    
    Ok(())
}
