# DuckTape

A command-line calendar productivity tool for macOS that integrates with Apple's native apps.
Duct tape fixes anything—just like this tool. No clicks, no distractions, just pure productivity. Stick with what works.

## Features

- Calendar event management with Apple Calendar.app
- Todo and reminder management with Apple Reminders.app
- Apple Notes integration
- Multiple calendar/list support
- Persistent state storage
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

### Todo Management

Create a todo item:
```bash
>> ducktape todo "Buy groceries" --notes "Milk, Eggs" --lists "Personal" --reminder-time "2025-02-05 11:00"
```

### Note Management

Create a note:
```bash
>> ducktape note "Meeting Notes" --content "Important points..." --folder "Work"
```

List all notes:
```bash
>> ducktape notes
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

Note options:
- `--content "<content>"` - Set note content
- `--folder "<folder>"` - Specify note folder (creates if doesn't exist)

### List and View Commands

- `ducktape list-todos` - Show all stored todos
- `ducktape list-events` - Show all calendar events with details
- `ducktape notes` - List all Apple Notes

### State Management

All data is automatically persisted to:
- `~/.ducktape/todos.json` - Todo items
- `~/.ducktape/events.json` - Calendar events

## Requirements

- macOS 10.13 or later
- Rust toolchain
- Calendar.app, Reminders.app, and Notes.app with proper permissions

## Permissions

The application requires access to Apple's native apps. Grant permissions in:
System Preferences > Security & Privacy > Privacy > Calendar/Reminders/Notes

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - See LICENSE file for details.
