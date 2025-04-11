use ducktape::command_processor::CommandArgs;

/// Test function to validate our quoted string parsing solution
fn parse_with_shell_words(input: &str) -> anyhow::Result<Vec<String>> {
    // Use the shell-words crate to parse shell-like commands with proper quote handling
    shell_words::split(input).map_err(|e| anyhow::anyhow!("Failed to parse command: {}", e))
}

#[test]
fn test_quoted_strings_parsing() {
    // Test the shell-words based parsing with a command containing quoted strings
    let test_cmd = r#"ducktape calendar create "Team Meeting" 2025-04-15 10:00 11:00 "Work""#;
    let args = parse_with_shell_words(test_cmd).unwrap();

    // Verify correct parsing
    assert_eq!(args[0], "ducktape");
    assert_eq!(args[1], "calendar");
    assert_eq!(args[2], "create");
    assert_eq!(args[3], "Team Meeting"); // Should be preserved as one argument
    assert_eq!(args[4], "2025-04-15");
    assert_eq!(args[5], "10:00");
    assert_eq!(args[6], "11:00");
    assert_eq!(args[7], "Work");
}
