#[cfg(test)]
mod time_parser_tests {
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
    
    fn extract_time_from_input(input: &str) -> Option<(u32, u32)> {
        // Simple regex to find time strings like "8pm", "8:30pm", etc.
        let re_time = Regex::new(r"(?i)\b(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?))\b").unwrap();
        
        if let Some(time_match) = re_time.find(input) {
            let time_str = time_match.as_str();
            return parse_time_with_ampm(time_str);
        }
        
        None
    }
    
    fn process_time_in_command(command: &str, input: &str) -> String {
        if let Some((hour, minute)) = extract_time_from_input(input) {
            // Replace placeholders in command
            let mut processed = command.to_string();
            
            // Replace time placeholders (assumes 00:00 and 01:00 are placeholders)
            if processed.contains("00:00") {
                processed = processed.replace("00:00", &format!("{:02}:{:02}", hour, minute));
                
                // Also replace end time if present
                if processed.contains("01:00") {
                    let end_hour = if hour == 23 { 0 } else { hour + 1 };
                    processed = processed.replace("01:00", &format!("{:02}:{:02}", end_hour, minute));
                }
            }
            
            return processed;
        }
        
        // Return original command if no time info found
        command.to_string()
    }
    
    #[test]
    fn test_time_parsing() {
        let test_cases = vec![
            ("8pm", Some((20, 0))),
            ("8:30pm", Some((20, 30))),
            ("10:00 PM", Some((22, 0))),
            ("8am", Some((8, 0))),
            ("12pm", Some((12, 0))),
            ("12am", Some((0, 0))),
            ("23:45", Some((23, 45))),
            ("invalid", None),
        ];
        
        for (input, expected) in test_cases {
            let result = parse_time_with_ampm(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_time_extraction() {
        let test_cases = vec![
            ("Meeting at 8pm", Some((20, 0))),
            ("Let's meet at 8:30pm tomorrow", Some((20, 30))),
            ("Conference call at 10:00 PM", Some((22, 0))),
            ("Start at 8am sharp", Some((8, 0))),
            ("No time mentioned", None),
        ];
        
        for (input, expected) in test_cases {
            let result = extract_time_from_input(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_command_processing() {
        let test_cases = vec![
            (
                "calendar create \"Meeting\" today 00:00 01:00",
                "Meeting at 8pm",
                "calendar create \"Meeting\" today 20:00 21:00"
            ),
            (
                "calendar create \"Conference\" today 00:00 01:00",
                "Conference call at 10:00 PM",
                "calendar create \"Conference\" today 22:00 23:00"
            ),
            (
                "calendar create \"Breakfast\" today 00:00 01:00",
                "Breakfast at 8am",
                "calendar create \"Breakfast\" today 08:00 09:00"
            ),
            (
                "calendar create \"No Time\" today 00:00 01:00",
                "No time mentioned",
                "calendar create \"No Time\" today 00:00 01:00"
            ),
        ];
        
        for (command, input, expected) in test_cases {
            let result = process_time_in_command(command, input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}
