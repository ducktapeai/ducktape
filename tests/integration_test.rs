use anyhow::Result;
use ducktape::state::{CalendarItem, StateManager, TodoItem};
use std::env;
use std::fs;
use tempfile::tempdir;

fn setup_test_env() -> Result<(tempfile::TempDir, StateManager)> {
    // Create a temporary directory for state
    let temp_dir = tempdir()?;

    // Set up the home directory for the test
    env::set_var("HOME", temp_dir.path());

    // Create .ducktape directory
    let ducktape_dir = temp_dir.path().join(".ducktape");
    fs::create_dir_all(&ducktape_dir)?;

    // Initialize state manager
    let state_manager = StateManager::new()?;

    Ok((temp_dir, state_manager))
}

#[test]
fn test_calendar_operations() -> Result<()> {
    let (_temp_dir, state_manager) = setup_test_env()?;

    // Create a test event
    let event = CalendarItem {
        title: "Meeting Title".to_string(),
        date: "2024-02-21".to_string(),
        time: "14:30".to_string(),
        calendars: vec!["Test Calendar".to_string()],
        all_day: false,
        location: None,
        description: None,
        email: None,
        reminder: None,
    };

    // Save the event and verify it was saved
    state_manager.add(event.clone())?;
    let events = state_manager.load::<CalendarItem>()?;
    assert!(!events.is_empty(), "No events found after adding one");
    assert_eq!(events[0].title, "Meeting Title");
    assert_eq!(events[0].date, "2024-02-21");

    Ok(())
}

#[test]
fn test_todo_operations() -> Result<()> {
    let (_temp_dir, state_manager) = setup_test_env()?;

    // Create a test todo
    let todo = TodoItem {
        title: "Buy Groceries".to_string(),
        notes: Some("Milk, bread, eggs".to_string()),
        lists: vec!["Shopping".to_string()],
        reminder_time: None,
    };

    // Save the todo and verify it was saved
    state_manager.add(todo.clone())?;
    let todos = state_manager.load::<TodoItem>()?;
    assert!(!todos.is_empty(), "No todos found after adding one");
    let first_todo = &todos[0];
    assert_eq!(first_todo.title, "Buy Groceries");
    assert!(
        !first_todo.lists.is_empty(),
        "Todo should have at least one list"
    );
    assert_eq!(first_todo.lists[0], "Shopping");

    Ok(())
}
