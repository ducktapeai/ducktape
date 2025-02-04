# DuckTape

A command-line calendar management tool for macOS that interfaces with Apple Calendar.app and Reminders.app.
All commands use the 'ducktape' prefix for consistency and clarity.

## Features

- Calendar event management
- Todo and reminder management
- Persistent state storage
- Multiple calendar support
- Rich command-line interface

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ducktape.git
cd ducktape

# Build and run
cargo build --release
cargo run
```

## Usage

Get help on available commands:
```bash
>> ducktape --help
# or
>> ducktape -h
```

All commands must be prefixed with 'ducktape'. For example:

### Calendar Management

List available calendars:
```bash
>> ducktape calendars
```

Create a calendar event:
```bash
>> ducktape calendar "Meeting" 2025-02-21 14:30 "Work" --location "Room 1" --description "Weekly sync"
```

List all calendar events:
```bash
>> ducktape list-events
```

### Todo Management

Create a todo item:
```bash
>> ducktape todo "Buy groceries" --notes "Milk, Eggs" --lists "Personal" --reminder-time "2025-02-05 11:00"
```

List all todos:
```bash
>> ducktape list-todos
```

### Command Options

Calendar options:
- `--all-day` - Create an all-day event
- `--location "<location>"` - Set event location
- `--description "<desc>"` - Set event description
- `--email "<email>"` - Add attendee
- `--reminder <minutes>` - Set reminder before event

Todo options:
- `--notes "<notes>"` - Add notes to the todo
- `--lists "<list1,list2>"` - Add to specific lists
- `--reminder-time "YYYY-MM-DD HH:MM"` - Set reminder time

### State Management

All data is automatically persisted to:
- `~/.ducktape/todos.json` - Todo items
- `~/.ducktape/events.json` - Calendar events

## Requirements

- macOS 10.13 or later
- Rust toolchain
- Calendar.app and Reminders.app with proper permissions

## Permissions

The application requires access to Calendar.app and Reminders.app. You may need to grant permissions in System Preferences > Security & Privacy > Privacy > Calendar/Reminders.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - See LICENSE file for details.
