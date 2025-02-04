# DuckTape

A command-line calendar management tool for macOS that focuses on productivity without the ClickProdOps.
For example this has the ability to add an event to multiple apps within your Calendar leveraging the command line.

## Features

- Create and manage calendar events
- Create and track todo items
- Persistent state storage
- Multiple calendar support
- Todo list management
- Reminder settings

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
>> calendar "Meeting Title" 2025-02-21 14:30 "Calendar Name" --location "Conference Room" --description "Meeting details"
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

All todos and calendar events are automatically saved to:
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

### View Available Calendar Properties

```bash
>> calendar-props
```

### Search Files (Utility Function)

```bash
>> search <path> <pattern>
```

### Help

```bash
>> help
```

## Requirements

- macOS 10.13 or later
- Rust toolchain
- Calendar.app with proper permissions

## Permissions

The application requires access to Calendar.app. You may need to grant permission in System Preferences > Security & Privacy > Privacy > Calendar.

## Examples

1. Create a team meeting in multiple calendars:
```bash
>> calendar "Team Meeting" 2025-02-21 14:30 "Work" "Personal" --location "Conference Room" --description "Weekly sync" --email "team@company.com"
```

2. Create an all-day holiday in both work and personal calendars:
```bash
>> calendar "Company Holiday" 2025-02-21 "Work" "Personal" --all-day --description "Office Closed"
```

3. Create an event with attendees in multiple calendars:
```bash
>> calendar "Project Review" 2025-02-21 15:00 "Work" "Projects" --location "Conference Room" --description "Monthly review" --email "team@company.com"
```

4. List available calendars:
```bash
>> calendars
```

## License

MIT License

Copyright (c) 2024 [Shaun Stuart]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.
