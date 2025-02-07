# DuckTape 🦆

Your AI-powered command-line productivity assistant for macOS. DuckTape seamlessly integrates with Apple's native apps while providing natural language interaction.

We coined the term Productivity as AI, Smooth as a Duck "Unlock Productivity with AI—For Everyone, On Every Device" 

## Features

- **Natural Language Support** - Just type what you want in plain English
- **AI Command Processing** - Powered by OpenAI's GPT-4
- **Smart Date Understanding** - Handles relative dates like "tomorrow" or "next Monday"
- **Native App Integration**:
  - Apple Calendar.app
  - Apple Reminders.app
  - Apple Notes.app
- **Persistent State Storage**
- **Rich Command Line Interface**

## Examples

Natural language commands:
```bash
>> schedule a team meeting tomorrow at 2pm
🦆 Interpreting as: ducktape calendar create "Team Meeting" 2025-02-06 14:00 15:00 "Work"

>> remind me to buy groceries next Monday
🦆 Interpreting as: ducktape todo "Buy groceries" --reminder-time "2025-02-12 09:00"

>> take notes from the project meeting
🦆 Interpreting as: ducktape note "Project Meeting Notes" --content "Meeting notes" --folder "Work"
```

Or use traditional commands:
```bash
>> ducktape calendar "Meeting" 2025-02-06 14:00 "Work" --location "Room 1"
>> ducktape todo "Buy groceries" --lists "Personal" --reminder-time "2025-02-12 09:00"
>> ducktape note "Meeting Notes" --content "Important points" --folder "Work"
```

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/ducktape.git
cd ducktape

# Build and run
cargo build --release
cargo run
```

## Setup

1. Get an OpenAI API key from https://platform.openai.com/
2. Export your API key:
```bash
export OPENAI_API_KEY='your-api-key-here'
```

## Features

### Calendar Management
- Create events with natural language
- Support for multiple calendars
- Location, description, and attendee management
- Reminder settings

### Todo Management
- Create todos with natural language
- Multiple list support
- Notes and reminders
- Smart date parsing

### Note Management
- Create and organize notes
- Folder organization
- Quick capture of ideas

### AI Features
- Natural language processing
- Smart date/time understanding
- Context-aware command generation
- Automatic calendar/list selection

## Calendar Configuration

If no calendar name is specified in your EventConfig, DuckTape will add the event to the default calendar set in Calendar.app (usually named "Calendar").

## State Management

All data is automatically persisted to:
- `~/.ducktape/todos.json` - Todo items
- `~/.ducktape/events.json` - Calendar events
- `~/.ducktape/notes.json` - Notes

## Requirements

- macOS 10.13 or later
- Rust toolchain
- OpenAI API key
- Calendar.app, Reminders.app, and Notes.app with proper permissions

## Permissions

Grant permissions in System Preferences:
- Security & Privacy > Privacy > Calendar
- Security & Privacy > Privacy > Reminders
- Security & Privacy > Privacy > Notes

## Contributing

Contributions welcome! Feel free to submit issues or pull requests.

## License

MIT License - See LICENSE file for details.

## Commands

DuckTape - Your AI-Powered Command Line Productivity Duck 🦆

Description:
  A unified CLI for Apple Calendar, Reminders, and Notes with natural language support  
  Just type what you want to do - DuckTape's AI will understand!

Natural Language Examples:
  "schedule a meeting with John tomorrow at 2pm"  
  "remind me to buy groceries next Monday morning"  
  "take notes about the project meeting"  
  "add a todo about calling the bank"

Or use traditional commands:
  ducktape [command] [options]  
  ducktape --help | -h

## Command Groups

**Calendar:**
- ducktape calendar create "<title>" <start_date> <start_time> <end_time> [calendar] - Create event  
- ducktape calendar delete "<title>" - Delete matching events  
- ducktape calendars - List available calendars