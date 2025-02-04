use anyhow::Result;
use ducktape::state::{StateManager, TodoItem, CalendarItem};

#[test]
fn test_calendar_operations() -> Result<()> {
    let manager = StateManager::new()?;
    
    // Test calendar event creation and storage
    let event = CalendarItem {
        title: "Test Event".to_string(),
        date: "2025-02-21".to_string(),
        time: "14:30".to_string(),
        calendars: vec!["Calendar".to_string()],
        all_day: false,
        location: Some("Test Location".to_string()),
        description: Some("Test Description".to_string()),
        email: None,
        reminder: Some(30),
    };
    
    manager.add(event)?;
    
    let events: Vec<CalendarItem> = manager.load()?;
    assert!(!events.is_empty());
    assert_eq!(events[0].title, "Test Event");
    
    Ok(())
}

#[test]
fn test_todo_operations() -> Result<()> {
    let manager = StateManager::new()?;
    
    // Test todo creation and storage
    let todo = TodoItem {
        title: "Test Todo".to_string(),
        notes: Some("Test Notes".to_string()),
        lists: vec!["Test List".to_string()],
        reminder_time: Some("2025-02-21 14:30".to_string()),
    };
    
    manager.add(todo)?;
    
    let todos: Vec<TodoItem> = manager.load()?;
    assert!(!todos.is_empty());
    assert_eq!(todos[0].title, "Test Todo");
    
    Ok(())
}

// Remove the CommandArgs test since it's now an implementation detail
// and should be tested in the main.rs unit tests
