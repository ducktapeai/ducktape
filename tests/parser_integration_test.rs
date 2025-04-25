//! Integration tests for the parser module
//!
//! This file contains tests that verify the parser module works correctly
//! with the rest of the application.

use anyhow::Result;
use ducktape::parser::grok::GrokParser;
use ducktape::parser::traits::{ParseResult, Parser, ParserFactory};

#[tokio::test]
async fn test_grok_parser_factory() -> Result<()> {
    // Test that the parser factory can create a grok parser
    let parser = ParserFactory::create_parser_by_name("grok")?;

    // Use a test phrase that should be recognized as a calendar command
    let result = parser
        .parse_input("Create a meeting tomorrow at 3pm called Project Review")
        .await?;

    match result {
        ParseResult::CommandString(cmd) => {
            assert!(cmd.contains("calendar create"));
            assert!(cmd.contains("Project Review"));
            // The parser should add quotes around the title
            assert!(cmd.contains("\"Project Review\""));
            Ok(())
        }
        ParseResult::StructuredCommand(_) => {
            // It's also acceptable if the parser returns a structured command
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_terminal_parser() -> Result<()> {
    // Test that the terminal parser works correctly with direct commands
    let parser = ParserFactory::create_parser_by_name("terminal")?;

    // Use a direct command string
    let result = parser
        .parse_input("calendar create \"Team Meeting\" 2025-04-25 14:00 15:00 \"Work\"")
        .await?;

    match result {
        ParseResult::CommandString(cmd) => {
            assert!(cmd.contains("calendar create"));
            assert!(cmd.contains("Team Meeting"));
            assert!(cmd.contains("2025-04-25"));
            Ok(())
        }
        ParseResult::StructuredCommand(args) => {
            assert_eq!(args.command, "calendar");
            assert!(args.args.contains(&"create".to_string()));
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_command_parser() -> Result<()> {
    // Test that the command parser works correctly for structured commands
    let parser = ParserFactory::create_parser_by_name("command")?;

    // Use a direct command string
    let result = parser
        .parse_input("calendar create \"Team Meeting\" 2025-04-25 14:00 15:00 \"Work\"")
        .await?;

    // Command parser should always return StructuredCommand
    if let ParseResult::StructuredCommand(args) = result {
        assert_eq!(args.command, "calendar");
        assert!(args.args.contains(&"create".to_string()));

        // Check that positional arguments are parsed correctly
        assert!(args.args.len() >= 4);
        assert!(args.args.contains(&"Team Meeting".to_string()));
        assert!(args.args.contains(&"2025-04-25".to_string()));
        assert!(args.args.contains(&"14:00".to_string()));
        assert!(args.args.contains(&"15:00".to_string()));
        assert!(args.args.contains(&"Work".to_string()));

        Ok(())
    } else {
        panic!("Expected StructuredCommand result from command parser");
    }
}

#[tokio::test]
async fn test_backward_compatibility() -> Result<()> {
    // Test that the parser API works through directly instantiating the GrokParser

    // Use the GrokParser directly
    let parser = GrokParser::new()?;

    // Parse a test phrase
    let result = parser.parse_input("Remind me to buy groceries tomorrow").await?;

    match result {
        ParseResult::CommandString(cmd) => {
            // Should contain either todo or calendar command
            assert!(cmd.contains("todo") || cmd.contains("calendar"));
            Ok(())
        }
        ParseResult::StructuredCommand(_) => {
            // Also acceptable
            Ok(())
        }
    }
}

// This test is marked as ignore because it requires the XAI_API_KEY environment variable
#[tokio::test]
#[ignore]
async fn test_live_grok_api_integration() -> Result<()> {
    // This test requires a valid XAI_API_KEY environment variable
    // Only run this test when specifically testing the live API integration

    let parser = ParserFactory::create_parser_by_name("grok")?;

    // Use a test phrase that should be recognized as a calendar command
    let result = parser.parse_input("Schedule a team meeting for tomorrow at 2pm").await?;

    match result {
        ParseResult::CommandString(cmd) => {
            println!("Generated command: {}", cmd);
            assert!(cmd.contains("calendar create"));
            Ok(())
        }
        ParseResult::StructuredCommand(args) => {
            println!("Structured command: {:?}", args);
            assert_eq!(args.command, "calendar");
            Ok(())
        }
    }
}
