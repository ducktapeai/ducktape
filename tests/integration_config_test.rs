use anyhow::Result;
use ducktape::command_processor::{CommandArgs, CommandHandler, ConfigHandler};
use std::collections::HashMap;

#[tokio::test]
async fn test_config_show_command() -> Result<()> {
    let handler = ConfigHandler;

    // Test 'show all' command
    let show_args = CommandArgs {
        command: "config".to_string(),
        args: vec!["show".to_string(), "all".to_string()],
        flags: HashMap::new(),
    };
    let result = handler.execute(show_args).await;
    assert!(result.is_ok(), "Failed to execute 'config show all' command");

    // Test 'get all' command (should behave the same as show)
    let get_args = CommandArgs {
        command: "config".to_string(),
        args: vec!["get".to_string(), "all".to_string()],
        flags: HashMap::new(),
    };
    let result = handler.execute(get_args).await;
    assert!(result.is_ok(), "Failed to execute 'config get all' command");

    // Test show specific key
    let show_key_args = CommandArgs {
        command: "config".to_string(),
        args: vec!["show".to_string(), "calendar.default".to_string()],
        flags: HashMap::new(),
    };
    let result = handler.execute(show_key_args).await;
    assert!(result.is_ok(), "Failed to execute 'config show calendar.default' command");

    Ok(())
}
