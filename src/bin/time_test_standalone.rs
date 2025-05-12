// Test program for time extraction
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

fn process_time_in_command(command: &str, input: &str) -> String {
    // Simple regex to find time strings like "8pm", "8:30pm", etc.
    let re_time = Regex::new(r"(?i)\b(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?))\b").unwrap();

    if let Some(time_match) = re_time.find(input) {
        let time_str = time_match.as_str();

        // Parse the time
        if let Some((hour, minute)) = parse_time_with_ampm(time_str) {
            // Replace placeholders in command
            let mut processed = command.to_string();

            // Replace time placeholders (assumes 00:00 and 01:00 are placeholders)
            if processed.contains("00:00") {
                processed = processed.replace("00:00", &format!("{:02}:{:02}", hour, minute));

                // Also replace end time if present
                if processed.contains("01:00") {
                    let end_hour = if hour == 23 { 0 } else { hour + 1 };
                    processed =
                        processed.replace("01:00", &format!("{:02}:{:02}", end_hour, minute));
                }
            }

            return processed;
        }
    }

    // Return original command if no time info found
    command.to_string()
}

fn main() {
    println!("Time Parsing Test");

    // Test time strings
    let test_times = vec![
        "8pm", "8:30pm", "10:00 PM", "8pm PST", // This includes a timezone
        "8:30 am", "12:00", "23:45",
    ];

    for time_str in test_times {
        println!("\nTesting time string: \"{}\"", time_str);

        // Test with regex
        let re = Regex::new(r"(?i)^(\d{1,2})(?::(\d{2}))?\s*([ap]\.?m\.?)?$").unwrap();
        if let Some(caps) = re.captures(time_str) {
            println!("✓ Regex matched!");
            for i in 0..caps.len() {
                println!("  Group {}: {:?}", i, caps.get(i).map(|m| m.as_str()));
            }
        } else {
            println!("✗ Regex did NOT match!");
        }

        // Test our parser
        if let Some((hour, minute)) = parse_time_with_ampm(time_str) {
            println!("✓ Parser success: {}:{:02}", hour, minute);
        } else {
            println!("✗ Parser failed!");
        }

        // Test time extraction with command
        let input = format!("I want to have a meeting at {}", time_str);
        let command = "calendar create \"Test Meeting\" today 00:00 01:00";

        let processed = process_time_in_command(command, &input);
        println!("Command: {}", command);
        println!("Processed: {}", processed);
    }
}
