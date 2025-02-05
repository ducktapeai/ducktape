use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

const STATE_DIR: &str = ".ducktape";
const TODOS_FILE: &str = "todos.json";
const EVENTS_FILE: &str = "events.json";
const NOTES_FILE: &str = "notes.json";

// Trait for items that can be persisted
pub trait Persistent: Sized + Serialize + for<'de> Deserialize<'de> {
    fn filename() -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoItem {
    pub title: String,
    pub notes: Option<String>,
    pub lists: Vec<String>,
    pub reminder_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarItem {
    pub title: String,
    pub date: String,
    pub time: String,
    pub calendars: Vec<String>,
    pub all_day: bool,
    pub location: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub reminder: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteItem {
    pub title: String,
    pub content: String,
    pub folder: Option<String>,
    pub created_at: String,
}

impl Persistent for TodoItem {
    fn filename() -> &'static str {
        TODOS_FILE
    }
}

impl Persistent for CalendarItem {
    fn filename() -> &'static str {
        EVENTS_FILE
    }
}

impl Persistent for NoteItem {
    fn filename() -> &'static str {
        NOTES_FILE
    }
}

pub struct StateManager {
    state_dir: PathBuf,
}

impl StateManager {
    pub fn new() -> Result<Self> {
        let mut state_dir = dirs::home_dir().expect("Could not find home directory");
        state_dir.push(STATE_DIR);
        std::fs::create_dir_all(&state_dir)?;
        Ok(Self { state_dir })
    }

    pub fn load<T: Persistent>(&self) -> Result<Vec<T>> {
        let path = self.state_dir.join(T::filename());
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            Ok(serde_json::from_reader(reader)?)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn save<T: Persistent>(&self, items: &[T]) -> Result<()> {
        let path = self.state_dir.join(T::filename());
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, items)?;
        Ok(())
    }

    pub fn add<T: Persistent>(&self, item: T) -> Result<()> {
        let mut items = self.load::<T>()?;
        items.push(item);
        self.save(&items)
    }
}

// Convenience functions for backward compatibility
pub fn load_todos() -> Result<Vec<TodoItem>> {
    StateManager::new()?.load()
}

// Remove unused save_events function since it's handled by StateManager
pub fn load_events() -> Result<Vec<CalendarItem>> {
    StateManager::new()?.load()
}

// Add convenience function for notes
pub fn load_notes() -> Result<Vec<NoteItem>> {
    StateManager::new()?.load()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::env;

    #[test]
    fn test_state_manager() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        env::set_var("HOME", temp_dir.path());

        let manager = StateManager::new()?;

        // Test todos
        let todo = TodoItem {
            title: "Test Todo".to_string(),
            notes: Some("Test Notes".to_string()),
            lists: vec!["Test List".to_string()],
            reminder_time: None,
        };
        manager.add(todo)?;

        let todos: Vec<TodoItem> = manager.load()?;
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Test Todo");

        // Test calendar events
        let event = CalendarItem {
            title: "Test Event".to_string(),
            date: "2024-02-21".to_string(),
            time: "14:30".to_string(),
            calendars: vec!["Test Calendar".to_string()],
            all_day: false,
            location: None,
            description: None,
            email: None,
            reminder: None,
        };
        manager.add(event)?;

        let events: Vec<CalendarItem> = manager.load()?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Test Event");

        Ok(())
    }
}

