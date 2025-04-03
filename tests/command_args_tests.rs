use ducktape::command_processor::CommandArgs;

#[test]
fn test_command_args_parse() {
    // Test basic command
    let args = CommandArgs::parse("ducktape calendar create \"Test Event\" 2023-01-01 09:00 10:00")
        .unwrap();
    assert_eq!(args.command, "calendar");
    assert_eq!(args.args, vec!["create", "Test Event", "2023-01-01", "09:00", "10:00"]);
    assert!(args.flags.is_empty());

    // Test with flags
    let args = CommandArgs::parse(
        "ducktape calendar create \"Test Event\" 2023-01-01 09:00 10:00 --location \"Office\"",
    )
    .unwrap();
    assert_eq!(args.command, "calendar");
    assert_eq!(args.args, vec!["create", "Test Event", "2023-01-01", "09:00", "10:00"]);
    assert_eq!(args.flags.get("--location").unwrap().as_ref().unwrap(), "Office");

    // Test basic command with quotes
    let args = CommandArgs::parse(
        "ducktape calendar create \"Weekly Team Standup\" 2025-04-03 15:00 16:00 \"Work\"",
    )
    .unwrap();
    assert_eq!(args.command, "calendar");
    assert_eq!(
        args.args,
        vec!["create", "Weekly Team Standup", "2025-04-03", "15:00", "16:00", "Work"]
    );
    assert!(args.flags.is_empty());

    // Test with multiple quoted arguments
    let args = CommandArgs::parse(
        "ducktape calendar create \"Project Review\" 2025-04-03 15:00 16:00 \"Work\" --location \"Conference Room A\"",
    )
    .unwrap();
    assert_eq!(args.command, "calendar");
    assert_eq!(
        args.args,
        vec!["create", "Project Review", "2025-04-03", "15:00", "16:00", "Work"]
    );
    assert_eq!(args.flags.get("--location").unwrap().as_ref().unwrap(), "Conference Room A");
}

#[test]
fn test_command_args_parse_errors() {
    // Test unclosed quotes
    assert!(CommandArgs::parse("ducktape calendar create \"Unterminated Quote").is_err());

    // Test empty command
    assert!(CommandArgs::parse("").is_err());

    // Test missing ducktape prefix
    assert!(CommandArgs::parse("invalid command").is_err());
}
