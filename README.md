# DuckTape 🦆

A unified CLI for Apple Calendar, Reminders, and Notes with natural language support.

## Description

DuckTape is an AI-powered command-line interface that makes it easy to manage your Apple Calendar, Reminders, and Notes. Just type what you want to do in natural language, and DuckTape's AI will understand and execute the appropriate command.

## Installation

1. Ensure you have Rust installed
2. Clone this repository
3. Export your OpenAI API key:
```bash
export OPENAI_API_KEY='your-api-key-here'
```
4. Build and run:
```bash
cargo build
cargo run
```

## Natural Language Examples

Just type what you want to do:
- "schedule a meeting with John tomorrow at 2pm"
- "remind me to buy groceries next Monday morning"
- "take notes about the project meeting"
- "add a todo about calling the bank"
- "create an event for 7pm tonight to my KIDS calendar inviting shaun.stuart@gmail.com"

## Command Reference

### Calendar Commands
```bash
# Create a calendar event
ducktape calendar create "<title>" <date> <start_time> <end_time> [calendar]

# Create event with attendee
ducktape calendar create "<title>" <date> <start_time> <end_time> [calendar] --email "attendee@example.com"

# Delete events
ducktape calendar delete "<title>"

# Set the default calendar (if no calendar is specified in event creation, this calendar will be used)
ducktape calendar set-default "<name>"

# List available calendars
ducktape calendars

# List calendar properties
ducktape calendar-props
```

### Calendar Options
- `--all-day` - Create an all-day event
- `--location "<location>"` - Set event location
- `--description "<desc>"` - Set event description
- `--email "<email>"` - Add attendee
- `--reminder <minutes>` - Set reminder (minutes before event)

### Todo & Reminders Commands
```bash
# Create a todo item
ducktape todo "<title>"

# List all stored todos
ducktape list-todos
```

### Todo Options
- `--notes "<notes>"` - Add notes to the todo
- `--lists "<list1,list2>"` - Add to specific lists
- `--reminder-time "YYYY-MM-DD HH:MM"` - Set reminder time

### Notes Commands
```bash
# Create a note
ducktape note "<title>" --content "<content>" [--folder "<folder>"]

# List all notes
ducktape notes
```

### Utility Commands
```bash
# Search for files
ducktape search <path> <pattern>

# List calendar properties
ducktape calendar-props

# Clean up old events and compact storage
ducktape cleanup

# Show help
ducktape --help

# Exit application
ducktape exit
```

## Features

- Natural language command processing
- Smart date/time understanding ("tomorrow", "next Monday")
- Context-aware calendar selection
- Automatic email attendee addition
- State persistence
- Calendar integration with Apple Calendar.app

## State Files

DuckTape maintains state in the following files:
- `~/.ducktape/todos.json` - Todo items
- `~/.ducktape/events.json` - Calendar events

## Requirements

- macOS with Calendar.app configured
- Rust toolchain
- OpenAI API key for natural language processing