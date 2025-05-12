// A standalone test file for our time parser
// To run: rustc test_time_parser.rs -L ~/.cargo/registry/src --extern regex=~/.cargo/registry/src/github.com-*/regex-*/libregex.rlib && ./test_time_parser

extern crate regex;
use regex::Regex;

fn parse_time_with_ampm(time_str: &str) -> Option<(u32, u32)> {
    // Create regex pattern to extract hour, minute, and am/pm
    let re = Regex::new(r"(?i)^(\d{1,2})(?::(\d{2}))?\s*([ap]\.?m\.?)?$").unwrap();
    
    let time_lower = time_str.to_lowercase();
    let am_pm_present = time_lower.contains("am") || time_lower.contains("pm");
    
    if let Some(caps) = re.captures(&time_lower) {
        let hour_str = caps.get(1).unwrap().as_str();
        let minute_str = caps.get(2).map_or("0", |m| m.as_str()); // Default to 0 if no minutes
        let ampm_str_opt = caps.get(3).map(|m| m.as_str().to_lowercase());
        
        if let (Ok(h_val), Ok(m_val)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            // Convert to 24-hour format
            let hour_24 = if let Some(ampm) = ampm_str_opt {
                if ampm.starts_with('p') && h_val < 12 { 
                    h_val + 12
                } else if ampm.starts_with('a') && h_val == 12 { 
                    0
                } else {
                    h_val
                }
            } else if am_pm_present {
                // If am/pm is present in string but not captured by regex
                if time_lower.contains("pm") && h_val < 12 {
                    h_val + 12
                } else if time_lower.contains("am") && h_val == 12 {
                    0
                } else {
                    h_val
                }
            } else {
                h_val
            };
            
            // Return parsed time if valid
            if hour_24 < 24 && m_val < 60 {
                return Some((hour_24, m_val));
            }
        }
    }
    
    None
}

fn extract_time_info(input: &str) -> Option<(String, String, String)> {
    // Look for patterns like "8pm", "8:30pm", etc.
    let re_time = Regex::new(r"(?i)\b(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?))\b").unwrap();
    
    if let Some(time_match) = re_time.find(input) {
        let time_str = time_match.as_str();
        
        // Parse the time
        if let Some((hour, minute)) = parse_time_with_ampm(time_str) {
            // Create date string (today)
            let today = "2023-05-15"; // Fixed date for testing
            
            // Create formatted start and end times
            let start_time = format!("{:02}:{:02}", hour, minute);
            
            // Set end time 1 hour later
            let end_hour = if hour == 23 { 0 } else { hour + 1 };
            let end_time = format!("{:02}:{:02}", end_hour, minute);
            
            return Some((today.to_string(), start_time, end_time));
        }
    }
    
    None
}

fn process_time_in_command(command: &str, input: &str) -> String {
    if let Some((date, start_time, end_time)) = extract_time_info(input) {
        // Replace placeholders in command
        let mut processed = command.to_string();
        
        // If the command contains "today", replace it with the date
        if processed.contains("today") {
            processed = processed.replace("today", &date);
        }
        
        // Replace time placeholders (assumes 00:00 and 01:00 are placeholders)
        if processed.contains("00:00") {
            processed = processed.replace("00:00", &start_time);
            
            // Also replace end time if present
            if processed.contains("01:00") {
                processed = processed.replace("01:00", &end_time);
            }
        }
        
        return processed;
    }
    
    // Return original command if no time info found
    command.to_string()
}

fn main() {
    // Test cases
    let test_times = [
        "8pm", "3:30pm", "10:00am", "12pm", "12am", "7 PM", "9 A.M."
    ];
    
    println!("Testing time parser functions:");
    println!("---------------------------------");
    
    for time_str in test_times {
        match parse_time_with_ampm(time_str) {
            Some((hour, minute)) => {
                println!("'{}' -> {:02}:{:02} (24-hour format)", time_str, hour, minute);
            },
            None => {
                println!("'{}' -> Failed to parse", time_str);
            }
        }
    }
    
    println!("\nTesting full command processing:");
    println!("---------------------------------");
    
    // Test command processing
    let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
    let inputs = [
        "schedule a meeting at 8pm",
        "create an event called Meeting tonight at 3:30pm",
        "set up a call for 9am tomorrow"
    ];
    
    for input in inputs {
        let processed = process_time_in_command(command, input);
        println!("Input: '{}'\nResult: '{}'\n", input, processed);
    }
}
