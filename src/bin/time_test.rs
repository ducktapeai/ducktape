// Test program for time extraction
use chrono::Local;
use regex::Regex;
use std::time::SystemTime;

// Include our fixed time parser code
use ducktape::parser::natural_language::time_parser_fix;

fn main() {
    println!("Time Extraction Test");

    // Test time strings
    let test_times = vec!["8pm", "8:30pm", "10:00 PM", "8pm PST", "8:30 am", "12:00", "23:45"];

    for time_str in test_times {
        println!("\nTesting time string: \"{}\"", time_str);

        // Original regex method
        let re = Regex::new(r"(?i)^(\d{1,2})(?::(\d{2}))?\s*([ap]\.?m\.?)?$").unwrap();
        if let Some(caps) = re.captures(time_str) {
            println!("✓ Regex matched!");
            for i in 0..caps.len() {
                println!("  Group {}: {:?}", i, caps.get(i).map(|m| m.as_str()));
            }
        } else {
            println!("✗ Regex did NOT match!");
        }

        // Test our fixed implementation
        if let Some((hour, minute)) = time_parser_fix::parse_time_with_ampm(time_str) {
            println!("✓ New parser success: {}:{:02}", hour, minute);
        } else {
            println!("✗ New parser failed!");
        }

        // Test time extraction with command
        let input = format!("I want to have a meeting at {}", time_str);
        let command = "calendar create \"Test Meeting\" today 00:00 01:00";

        let processed = time_parser_fix::process_time_in_command(command, &input);
        println!("Command: {}", command);
        println!("Processed: {}", processed);
    }

    // Current time
    let now = Local::now();
    println!("\nCurrent local time: {}", now.format("%Y-%m-%d %H:%M:%S %z"));
}
