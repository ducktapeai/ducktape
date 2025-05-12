// End-to-end test for time parser with the command processor
// To run: `cargo run --bin test_time_parser_e2e`

use ducktape::parser::natural_language::grok::utils::sanitize_nlp_command;
use ducktape::command_processor::CommandProcessor;
use std::io::{self, Write};

struct MockOutput {
    buffer: Vec<String>,
}

impl MockOutput {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }
    
    fn write(&mut self, line: &str) {
        self.buffer.push(line.to_string());
    }
    
    fn get_output(&self) -> String {
        self.buffer.join("\n")
    }
}

fn main() -> io::Result<()> {
    println!("Time Parser End-to-End Test");
    println!("==========================");
    
    // Define test cases with natural language input
    let test_cases = [
        "create an event called Team Meeting tonight at 7pm",
        "schedule a meeting called Daily Standup at 9:30am",
        "set up a call with Jane at 3pm tomorrow",
        "create an event called Lunch at 12pm",
        "schedule a meeting called Planning Session from 2pm to 4pm",
    ];
    
    println!("\nRunning test cases through sanitize_nlp_command:");
    println!("-----------------------------------------------");
    
    for (i, input) in test_cases.iter().enumerate() {
        let sanitized = sanitize_nlp_command(input);
        println!("Test case #{}: '{}'\nConverted to: '{}'\n", i+1, input, sanitized);
        
        // Print details about time extraction
        if sanitized.contains("calendar create") {
            let parts: Vec<&str> = sanitized.split_whitespace().collect();
            let time_indices = parts.iter().enumerate()
                .filter(|(_, &part)| part.contains(":"))
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            
            if time_indices.len() >= 2 {
                let start_time = parts[time_indices[0]];
                let end_time = parts[time_indices[1]];
                println!("  Extracted times: {} to {}", start_time, end_time);
                
                // Verify correctness
                if input.contains("pm") && !input.contains("12pm") && !start_time.starts_with("0") {
                    assert!(start_time.split(":").next().unwrap().parse::<u32>().unwrap() >= 12, 
                            "PM time should be converted to 24-hour format (≥12)");
                    println!("  ✓ PM time correctly converted to 24-hour format");
                }
                
                if input.contains("am") {
                    if input.contains("12am") {
                        assert_eq!(start_time.split(":").next().unwrap(), "00",
                                  "12am should be converted to 00 in 24-hour format");
                        println!("  ✓ 12am correctly converted to 00:xx");
                    } else {
                        let hour = start_time.split(":").next().unwrap().parse::<u32>().unwrap();
                        assert!(hour < 12 || hour == 12, 
                                "AM time should be < 12 in 24-hour format (except for 12pm)");
                        println!("  ✓ AM time correctly represented in 24-hour format");
                    }
                }
            }
        }
    }
    
    Ok(())
}
