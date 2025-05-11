//! Time parser integration module
//!
//! This module connects the fixed time parser with the main natural language processing
//! functionality to handle time expressions in natural language input.

use crate::parser::natural_language::grok::utils;
use crate::parser::natural_language::time_parser_fix;
use log::debug;

/// Process natural language input for time-related expressions and update
/// calendar commands accordingly.
///
/// This function serves as the main integration point between the fixed time parser
/// and the natural language processing pipeline.
///
/// # Arguments
///
/// * `command` - The original calendar command with default times
/// * `input` - The natural language input text to parse for time expressions
///
/// # Returns
///
/// * `String` - The updated command with extracted time information
pub fn process_time_expressions(command: &str, input: &str) -> String {
    debug!("Processing time expressions in: '{}'", input);

    // Skip if not a calendar command
    if !command.contains("calendar create") {
        return command.to_string();
    }

    // Process the command using our fixed time parser
    let processed_command = time_parser_fix::process_time_in_command(command, input);

    // If the command was updated, ensure end time formatting is correct
    if processed_command != command {
        debug!("Time expression processed: '{}'", processed_command);
        return utils::fix_calendar_end_time_format(&processed_command);
    }

    // If no time processing occurred, return the original command
    command.to_string()
}

/// Helper function to check if a string contains a time expression
///
/// # Arguments
///
/// * `text` - The text to check for time expressions
///
/// # Returns
///
/// * `bool` - True if the text contains a time expression
pub fn contains_time_expression(text: &str) -> bool {
    // Enhanced regex to detect common time patterns, with optional timezone abbreviations
    let time_regex =
        regex::Regex::new(r"(?i)\b(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?)(?:\s+[A-Z]{3,4})?)\b")
            .unwrap();
    time_regex.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_time_expressions() {
        // Test with time expressions
        let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
        let input = "Schedule a meeting at 8pm";
        let processed = process_time_expressions(command, input);
        assert!(processed.contains("20:00"));
        assert!(processed.contains("21:00"));

        // Test with PM time
        let command = "ducktape calendar create \"Call\" today 00:00 01:00 \"Personal\"";
        let input = "Set up a call for 3:30pm";
        let processed = process_time_expressions(command, input);
        assert!(processed.contains("15:30"));
        assert!(processed.contains("16:30"));

        // Test with AM time
        let command = "ducktape calendar create \"Breakfast\" today 00:00 01:00 \"Personal\"";
        let input = "Schedule breakfast for 8am tomorrow";
        let processed = process_time_expressions(command, input);
        assert!(processed.contains("08:00"));
        assert!(processed.contains("09:00"));
    }

    #[test]
    fn test_contains_time_expression() {
        assert!(contains_time_expression("Meeting at 8pm"));
        assert!(contains_time_expression("Call at 3:30pm"));
        assert!(contains_time_expression("Meet at 9:00 AM"));
        assert!(!contains_time_expression("Meeting tomorrow"));
        assert!(!contains_time_expression("No time here"));
    }

    #[test]
    fn test_process_time_expressions_with_timezone() {
        // Test with timezone expressions
        let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
        let input = "Schedule a meeting at 8pm PST";
        let processed = process_time_expressions(command, input);

        // We can't predict exact time in test since it depends on the local timezone,
        // but we can check that it was processed (changed from default)
        assert!(processed != command);
        assert!(!processed.contains("00:00"));
        assert!(!processed.contains("01:00"));

        // Test with eastern timezone
        let command = "ducktape calendar create \"Call\" today 00:00 01:00 \"Personal\"";
        let input = "Set up a call for 3:30pm EST";
        let processed = process_time_expressions(command, input);
        assert!(processed != command);
        assert!(!processed.contains("00:00"));
        assert!(!processed.contains("01:00"));
    }
}
