use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

const STATE_DIR: &str = ".ducktape";
const TODOS_FILE: &str = "todos.json";
const EVENTS_FILE: &str = "events.json";
const NOTES_FILE: &str = "notes.json";
// Maximum allowed size for state files to prevent DoS attacks (10MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

// Trait for items that can be persisted
pub trait Persistent: Sized + Serialize + for<'de> Deserialize<'de> {
    fn filename() -> &'static str;
}

// Make the structs public and cloneable
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub title: String,
    pub notes: Option<String>,
    pub lists: Vec<String>,
    pub reminder_time: Option<String>,
}

// Make the structs public and cloneable
#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        let mut state_dir = home_dir;
        state_dir.push(STATE_DIR);
        std::fs::create_dir_all(&state_dir)?;
        Ok(Self { state_dir })
    }

    pub fn load<T: Persistent>(&self) -> Result<Vec<T>> {
        let path = self.state_dir.join(T::filename());
        if path.exists() {
            // Check file size before loading to prevent DoS attacks
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() > MAX_FILE_SIZE {
                return Err(anyhow!("File size exceeds security limits"));
            }

            let file = File::open(path)?;
            let reader = BufReader::new(file);

            // Use the from_reader function with proper security limits
            let json_value: serde_json::Value = serde_json::from_reader(reader)
                .map_err(|e| anyhow!("Failed to parse JSON data: {}", e))?;

            // Count elements to prevent DoS attacks
            if let Some(array) = json_value.as_array() {
                if array.len() > 10000 {
                    return Err(anyhow!("Too many items in file (maximum 10000)"));
                }
            }

            // Convert to the desired type
            let items: Vec<T> = serde_json::from_value(json_value)
                .map_err(|e| anyhow!("Failed to deserialize data: {}", e))?;

            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn save<T: Persistent>(&self, items: &[T]) -> Result<()> {
        let path = self.state_dir.join(T::filename());
        let file = OpenOptions::new().write(true).create(true).truncate(true).open(path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, items)?;
        Ok(())
    }

    pub fn add<T: Persistent>(&self, item: T) -> Result<()> {
        let mut items = self.load::<T>()?;
        items.push(item);
        self.save(&items)
    }

    #[allow(dead_code)]
    pub fn cleanup_old_items(&self) -> Result<()> {
        // Clean up old calendar events
        let mut events: Vec<CalendarItem> = self.load()?;
        let now = Local::now();
        events.retain(|event| {
            if let Ok(event_date) = DateTime::parse_from_str(
                &format!("{} {}", event.date, event.time),
                "%Y-%m-%d %H:%M",
            ) {
                event_date > now
            } else {
                true // Keep events with invalid dates
            }
        });
        self.save(&events)?;

        // Clean up old todos
        let todos: Vec<TodoItem> = self.load()?;
        let one_month_ago = now - Duration::days(30);
        let todos: Vec<_> = todos
            .into_iter()
            .filter(|todo| {
                if let Some(time) = &todo.reminder_time {
                    if let Ok(todo_date) = DateTime::parse_from_str(time, "%Y-%m-%d %H:%M") {
                        return todo_date > one_month_ago;
                    }
                }
                true // Keep todos without dates
            })
            .collect();
        self.save(&todos)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn vacuum(&self) -> Result<()> {
        // Compact JSON files by removing whitespace
        for filename in &[TODOS_FILE, EVENTS_FILE, NOTES_FILE] {
            let path = self.state_dir.join(filename);
            if path.exists() {
                // Check file size before loading to prevent DoS attacks
                let metadata = std::fs::metadata(&path)?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(anyhow!("File {} exceeds security limits", filename));
                }

                // Read file content with size limits
                let mut file = File::open(&path)?;
                let mut content = Vec::with_capacity(metadata.len() as usize);
                file.read_to_end(&mut content)?;

                // Use a depth limit for JSON parsing
                let items: serde_json::Value = serde_json::from_slice(&content)
                    .map_err(|e| anyhow!("Error parsing {}: {}", filename, e))?;

                // Count elements to prevent DoS attacks
                if let Some(array) = items.as_array() {
                    if array.len() > 10000 {
                        return Err(anyhow!("Too many items in file {} (maximum 10000)", filename));
                    }
                }

                let file =
                    OpenOptions::new().write(true).truncate(true).create(true).open(&path)?;
                serde_json::to_writer(file, &items)?;
            }
        }
        Ok(())
    }
}

// Convenience functions for backward compatibility
#[allow(dead_code)]
pub fn load_todos() -> Result<Vec<TodoItem>> {
    let manager = StateManager::new()?;
    manager.cleanup_old_items()?;
    manager.load()
}

#[allow(dead_code)]
pub fn load_events() -> Result<Vec<CalendarItem>> {
    let manager = StateManager::new()?;
    manager.cleanup_old_items()?;
    manager.load()
}

#[allow(dead_code)] // Add this attribute since we might use this function later
pub fn load_notes() -> Result<Vec<NoteItem>> {
    StateManager::new()?.load()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

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
