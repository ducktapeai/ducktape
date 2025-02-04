# DuckTape

A command-line productivity management tool for macOS. Duct tape fixes anything—just like this tool. No clicks, no distractions, just pure productivity. Stick with what works.

## Features

- Create and manage calendar events across multiple calendars
- Create and track todo items in multiple lists
- Persistent state storage
- Rich command-line interface
- Reminder settings for both events and todos
- Detailed event and todo listing

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ducktape.git
cd ducktape

# Build the project
cargo build --release

# Run the application
cargo run
```

## Usage

### Calendar Management

List available calendars:
```bash
>> calendars
```

Create a calendar event:
```bash
>> calendar "Meeting Title" 2025-02-21 14:30 "Work" --location "Conference Room" --description "Meeting details" --email "attendee@example.com"
```

List all calendar events with details:
```bash
>> list-events
```

### Todo Management

Create a todo item:
```bash
>> todo "Buy groceries" --notes "Milk, Eggs, Bread" --lists "Personal,Shopping" --reminder-time "2025-02-05 11:00"
```

List all stored todos:
```bash
>> list-todos
```

### State Management

All data is automatically persisted to:
- `~/.ducktape/todos.json` - Todo items
- `~/.ducktape/events.json` - Calendar events

### Command Options

Calendar options:
- `--all-day` - Create an all-day event
- `--location "<location>"` - Set event location
- `--description "<desc>"` - Set event description
- `--email "<email>"` - Add attendee
- `--reminder <minutes>` - Set reminder (minutes before event)

Todo options:
- `--notes "<notes>"` - Add notes to the todo
- `--lists "<list1,list2>"` - Add to specific lists
- `--reminder-time "YYYY-MM-DD HH:MM"` - Set reminder time

## Requirements

- macOS 10.13 or later
- Rust toolchain
- Calendar.app with proper permissions
- Reminders.app with proper permissions

## Permissions

The application requires access to Calendar.app and Reminders.app. You may need to grant permissions in System Preferences > Security & Privacy > Privacy > Calendar/Reminders.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - See LICENSE file for details.
