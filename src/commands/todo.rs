use crate::commands::{CommandArgs, CommandExecutor};
use crate::state::{self, TodoItem};
use crate::todo;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

pub struct TodoCommand;

impl CommandExecutor for TodoCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            match args.command.as_str() {
                "todo" => handle_todo_command(args),
                "list-todos" => list_todos(),
                _ => {
                    println!("Unknown todo command");
                    Ok(())
                }
            }
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        matches!(command, "todo" | "list-todos")
    }
}

fn list_todos() -> Result<()> {
    let todos = state::load_todos()?;
    println!("Stored Todo Items:");
    for item in todos {
        println!(
            "  - {} [{}]",
            item.title,
            item.reminder_time.as_deref().unwrap_or("No reminder")
        );
    }
    Ok(())
}

fn handle_todo_command(args: CommandArgs) -> Result<()> {
    if args.args.is_empty() {
        println!("Usage: todo \"<task title>\" [--notes \"<task notes>\"] [--lists \"list1,list2,...\"] [--reminder-time \"YYYY-MM-DD HH:MM\"]");
        return Ok(());
    }
    let mut config = todo::TodoConfig::new(&args.args[0]);
    if let Some(notes) = args.flags.get("--notes") {
        config.notes = notes.clone();
    }
    if let Some(lists) = args.flags.get("--lists") {
        let list_names: Vec<&str> = lists
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        config.lists = list_names;
    }
    if let Some(reminder) = args.flags.get("--reminder-time") {
        config.reminder_time = reminder.as_deref();
    }
    todo::create_todo(config)?;

    // Create new todo item and save using StateManager
    let todo_item = TodoItem {
        title: args.args[0].clone(),
        notes: args.flags.get("--notes").and_then(|n| n.clone()),
        lists: args
            .flags
            .get("--lists")
            .map(|l| {
                l.as_deref()
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .collect()
            })
            .unwrap_or(vec!["Reminders".to_owned()]),
        reminder_time: args.flags.get("--reminder-time").and_then(|r| r.clone()),
    };

    // Use StateManager to save the todo
    state::StateManager::new()?.add(todo_item)?;

    Ok(())
}
