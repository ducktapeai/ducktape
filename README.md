# DuckTape 🦆

Time possible the most important commodity. At DuckTape we recognise that.
So we figured why dont we combine how we interact with time and mix it with a touch of AI.
Effectively you "TimeAI"
Our goal is ambitious, to interact naturally (-clicking in Calendars) and get to a place where suggestions happen.
Your time, your way, with what AI thinks can help your day run smoothly. Work towards a common goal you might have.

We have started this journey off as unified CLI (soon to be App) for Apple Calendar, Reminders, and Notes with natural language support.

So why wait, give it some TimeAI 🦆

## Description

DuckTape is an AI-powered command-line interface that makes it easy to manage your Apple Calendar, Reminders, and Notes. Just type what you want to do in natural language, and DuckTape's AI will understand and execute the appropriate command.

## Installation

1. Ensure you have Rust installed
2. Clone this repository

For OpenAI, export your API key:
```bash
export OPENAI_API_KEY='your-openai-api-key-here'
```

For Grok (XAI), export your XAI API key:
```bash
export XAI_API_KEY='your-xai-api-key-here'
```

For DeepSeek, export your DeepSeek API key:
```bash
export DEEPSEEK_API_KEY='your-deepseek-api-key-here'
```

3. Build and run:
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
- "create an event for 7pm tonight to my KIDS calendar inviting joe.blogs@gmail.com"
- "schedule a weekly team meeting every Tuesday at 10am with Jane and Bob"
- "create a monthly book club meeting on the first Friday until December"

## Command Reference

### Configuration Commands
```bash
# Switch language model provider (OpenAI, Grok, or DeepSeek)
ducktape config llm openai
ducktape config llm grok
ducktape config llm deepseek

# Show current configuration settings
ducktape config show
```

### Calendar Commands
```bash
# Create a calendar event
ducktape calendar create "<title>" <date> <start_time> <end_time> [calendar]

# Create event with attendee
ducktape calendar create "<title>" <date> <start_time> <end_time> [calendar] --email "attendee@example.com"

# Create recurring event
ducktape calendar create "<title>" <date> <start_time> <end_time> [calendar] --repeat daily

# Create recurring event with contacts
ducktape calendar create "<title>" <date> <start_time> <end_time> [calendar] --repeat weekly --contacts "Jane Doe"

# Delete events
ducktape calendar delete "<title>"

# Set the default calendar (if no calendar is specified in event creation, this calendar will be used)
ducktape calendar set-default "<calendar_name>"

# List available calendars
ducktape calendars

# List calendar properties
ducktape calendar-props

# List all events
ducktape list-events
```

### Calendar Options
- `--all-day` - Create an all-day event
- `--location "<location>"` - Set event location
- `--description "<desc>"` - Set event description
- `--email "<email>"` - Add attendee
- `--reminder <minutes>` - Set reminder (minutes before event)
- `--contacts "<name1,name2>"` - Add contacts by name (automatically looks up email addresses)

### Recurring Event Options
- `--repeat <daily|weekly|monthly|yearly>` - Set recurrence frequency
- `--recurring <daily|weekly|monthly|yearly>` - Alternative to --repeat
- `--interval <number>` - Set interval (e.g., every 2 weeks)
- `--until <YYYY-MM-DD>` - Set end date for recurrence
- `--count <number>` - Set number of occurrences
- `--days <0,1,2...>` - Set days of week (0=Sun, 1=Mon, etc.)

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

# Clean up old events and compact storage
ducktape cleanup

# Show help
ducktape help

# Exit application
ducktape exit
```

## Features

- Natural language command processing with multiple AI model support:
  - OpenAI
  - Grok (X.AI)
  - DeepSeek
- Smart date/time understanding ("tomorrow", "next Monday")
- Context-aware calendar selection
- Automatic email attendee addition
- Recurring events support (daily, weekly, monthly, yearly)
- Contact lookup for event attendees
- State persistence
- Calendar integration with Apple Calendar.app
- Modular, well-organized code architecture

## Architecture

DuckTape follows a modular architecture pattern:
- Command pattern for processing different command types
- Natural language processing adapters for different LLM providers
- Clear separation of concerns between UI, business logic, and state management

## State Files

DuckTape maintains state in the following files:
- `~/.ducktape/todos.json` - Todo items
- `~/.ducktape/events.json` - Calendar events
- `~/.ducktape/config.json` - Application configuration

## Requirements

- macOS with Calendar.app configured
- Rust toolchain
- API key for at least one supported language model provider (OpenAI, Grok, or DeepSeek)