// A standalone test program for our time parser
// This version doesn't use regex for simplicity

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

fn main() {
    println!("Simple Time Parser Test");
    println!("-----------------------");
    
    let test_cases = [
        "8pm", "3:30pm", "10:00am", "12pm", "12am", "7 PM", "9 A.M."
    ];
    
    for time_str in test_cases {
        match parse_time_with_ampm(time_str) {
            Some((hour, minute)) => {
                println!("'{}' -> {:02}:{:02} (24-hour format)", time_str, hour, minute);
            },
            None => {
                println!("'{}' -> Failed to parse", time_str);
            }
        }
    }
}
