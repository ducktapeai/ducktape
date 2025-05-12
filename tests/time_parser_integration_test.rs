// Integration test for time parser functionality
use ducktape::parser::natural_language::grok::utils::sanitize_nlp_command;

#[test]
fn test_time_parser_integration() {
    // Test PM times
    let input = "create an event called Team Meeting tonight at 7pm";
    let result = sanitize_nlp_command(input);
    assert!(result.contains("19:00"), "Failed to parse '7pm': {}", result);
    assert!(result.contains("20:00"), "End time should be 1 hour after start: {}", result);

    let input = "schedule a meeting called Review at 3:30pm";
    let result = sanitize_nlp_command(input);
    assert!(result.contains("15:30"), "Failed to parse '3:30pm': {}", result);
    assert!(result.contains("16:30"), "End time should be 1 hour after start: {}", result);

    // Test AM times
    let input = "create an event called Breakfast at 9am";
    let result = sanitize_nlp_command(input);
    assert!(result.contains("09:00"), "Failed to parse '9am': {}", result);
    assert!(result.contains("10:00"), "End time should be 1 hour after start: {}", result);

    // Test with 12-hour edge cases
    let input = "create an event called Lunch at 12pm";
    let result = sanitize_nlp_command(input);
    assert!(result.contains("12:00"), "Failed to parse '12pm': {}", result);
    assert!(result.contains("13:00"), "End time should be 1 hour after start: {}", result);

    let input = "create an event called Midnight Party at 12am";
    let result = sanitize_nlp_command(input);
    assert!(result.contains("00:00"), "Failed to parse '12am': {}", result);
    assert!(result.contains("01:00"), "End time should be 1 hour after start: {}", result);

    // Test with different time formats
    let input = "schedule a meeting called Standup at 8:45 am tomorrow";
    let result = sanitize_nlp_command(input);
    assert!(result.contains("08:45"), "Failed to parse '8:45 am': {}", result);
    assert!(result.contains("09:45"), "End time should be 1 hour after start: {}", result);
}
