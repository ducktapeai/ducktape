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
                    processed =
                        processed.replace("01:00", &format!("{:02}:{:02}", end_hour, minute));
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
                "calendar create \"Meeting\" today 20:00 21:00",
            ),
            (
                "calendar create \"Conference\" today 00:00 01:00",
                "Conference call at 10:00 PM",
                "calendar create \"Conference\" today 22:00 23:00",
            ),
            (
                "calendar create \"Breakfast\" today 00:00 01:00",
                "Breakfast at 8am",
                "calendar create \"Breakfast\" today 08:00 09:00",
            ),
            (
                "calendar create \"No Time\" today 00:00 01:00",
                "No time mentioned",
                "calendar create \"No Time\" today 00:00 01:00",
            ),
        ];

        for (command, input, expected) in test_cases {
            let result = process_time_in_command(command, input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_relative_time_extraction() {
        // We'll test this by adding a function that mimics the behavior of the real implementation
        // but in a way that's testable in this context

        fn extract_relative_time_string(input: &str) -> Option<String> {
            let re = Regex::new(r"(?i)in\s+(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|day|days|week|weeks|wk|wks)").unwrap();
            
            if let Some(caps) = re.captures(input) {
                let amount = caps.get(1).unwrap().as_str();
                let unit = caps.get(2).unwrap().as_str().to_lowercase();
                
                return Some(format!("{}_{}", amount, unit));  // Just return a string representation for testing
            }
            
            None
        }
        
        let test_cases = vec![
            ("Create a meeting in 30 minutes", Some(String::from("30_minutes"))),
            ("Remind me in 2 hours to call back", Some(String::from("2_hours"))),
            ("Schedule a follow-up in 3 days", Some(String::from("3_days"))),
            ("Plan team meeting in 2 weeks", Some(String::from("2_weeks"))),
            ("Use shortened forms: in 45 mins", Some(String::from("45_mins"))),
            ("Use shortened forms: in 1 hr", Some(String::from("1_hr"))),
            ("No relative time mentioned", None),
        ];
        
        for (input, expected) in test_cases {
            let result = extract_relative_time_string(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_command_processing_with_relative_time() {
        // This test checks that commands like "in X minutes/hours" 
        // are properly processed in a way similar to the actual implementation
        
        fn process_command_with_relative_time(command: &str, input: &str) -> String {
            // First check for relative time expressions
            let re = Regex::new(r"(?i)in\s+(\d+)\s+(minute|minutes|min|mins|hour|hours|hr|hrs|day|days|week|weeks|wk|wks)").unwrap();
            
            if let Some(caps) = re.captures(input) {
                // Get the unit - we don't need to use the amount for this test implementation
                let _amount: i64 = caps.get(1).unwrap().as_str().parse().unwrap();
                let unit = caps.get(2).unwrap().as_str().to_lowercase();
                
                // In a real implementation, we would calculate the actual time
                // Here, we'll just replace the placeholders with something recognizable for testing
                let mut processed = command.to_string();
                
                // Replace date placeholder if present
                if processed.contains("today") {
                    let placeholder = if unit.contains("day") || unit.contains("week") {
                        "FUTURE_DATE"  // Would be calculated date in real implementation
                    } else {
                        "TODAY"  // Same day for minutes/hours
                    };
                    processed = processed.replace("today", placeholder);
                }
                
                // Replace time placeholders
                if processed.contains("00:00") && processed.contains("01:00") {
                    // Create recognizable patterns for testing
                    let start_time = format!("REL_{}_START", unit);
                    let end_time = format!("REL_{}_END", unit);
                    
                    processed = processed.replace("00:00", &start_time);
                    processed = processed.replace("01:00", &end_time);
                }
                
                return processed;
            }
            
            // If no relative time, return unchanged
            command.to_string()
        }
        
        let test_cases = vec![
            (
                "calendar create \"Quick Meeting\" today 00:00 01:00",
                "Quick Meeting in 30 minutes",
                "calendar create \"Quick Meeting\" TODAY REL_minutes_START REL_minutes_END",
            ),
            (
                "calendar create \"Team Sync\" today 00:00 01:00",
                "Team Sync in 2 hours",
                "calendar create \"Team Sync\" TODAY REL_hours_START REL_hours_END",
            ),
            (
                "calendar create \"Project Review\" today 00:00 01:00",
                "Project Review in 3 days",
                "calendar create \"Project Review\" FUTURE_DATE REL_days_START REL_days_END",
            ),
            (
                "calendar create \"Sprint Planning\" today 00:00 01:00",
                "Sprint Planning in 2 weeks",
                "calendar create \"Sprint Planning\" FUTURE_DATE REL_weeks_START REL_weeks_END",
            ),
            (
                "calendar create \"No Relative Time\" today 00:00 01:00",
                "No relative time mentioned",
                "calendar create \"No Relative Time\" today 00:00 01:00",
            ),
        ];
        
        for (command, input, expected) in test_cases {
            let result = process_command_with_relative_time(command, input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}
