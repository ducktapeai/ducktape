// A simple test script for validating our time parser functions

use ducktape::parser::natural_language::time_parser_fix::*;

fn main() {
    // Test cases
    let test_times = ["8pm", "3:30pm", "10:00am", "12pm", "12am", "7 PM", "9 A.M."];

    println!("Testing time parser functions:");
    println!("---------------------------------");

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

    println!("\nTesting full command processing:");
    println!("---------------------------------");

    // Test command processing
    let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
    let inputs = [
        "schedule a meeting at 8pm",
        "create an event called Meeting tonight at 3:30pm",
        "set up a call for 9am tomorrow",
    ];

    for input in inputs {
        let processed = process_time_in_command(command, input);
        println!("Input: '{}'\nResult: '{}'\n", input, processed);
    }
}
