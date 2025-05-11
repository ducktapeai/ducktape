// Manual test script for time parser integration
// To run: `cargo run --bin test_time_parser_integration`

use ducktape::parser::natural_language::{
    grok::utils::sanitize_nlp_command, time_parser_fix::parse_time_with_ampm,
    time_parser_integration::process_time_expressions,
};

fn main() {
    println!("Testing Time Parser Integration");
    println!("==============================");

    // Test basic time parsing
    println!("\n1. Basic time parsing test:");
    let test_times = ["8pm", "3:30pm", "10:00am", "12pm", "12am", "7 PM", "9 A.M."];

    for time_str in test_times {
        match parse_time_with_ampm(time_str) {
            Some((hour, minute)) => {
                println!("'{}' -> {:02}:{:02} (24-hour format)", time_str, hour, minute);
            }
            None => {
                println!("'{}' -> Failed to parse", time_str);
            }
        }
    }

    // Test process_time_expressions function
    println!("\n2. Testing process_time_expressions function:");
    let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
    let inputs = [
        "schedule a meeting at 8pm",
        "create an event called Meeting tonight at 3:30pm",
        "set up a call for 9am tomorrow",
    ];

    for input in inputs {
        let processed = process_time_expressions(command, input);
        println!("Input: '{}'\nResult: '{}'\n", input, processed);
    }

    // Test sanitize_nlp_command function
    println!("\n3. Testing sanitize_nlp_command function:");
    let natural_language_inputs = [
        "create an event called Team Meeting tonight at 7pm",
        "schedule a meeting called Review at 3:30pm",
        "create an event called Breakfast at 9am",
        "schedule a meeting called Early call at 6:45am tomorrow",
        "create an event called Midnight Party at 12am",
        "create an event called Lunch at 12pm",
    ];

    for input in natural_language_inputs {
        let sanitized = sanitize_nlp_command(input);
        println!("Input: '{}'\nParsed to: '{}'\n", input, sanitized);
    }
}
