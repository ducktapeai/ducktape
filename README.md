# DuckTape

A command-line calendar management tool for macOS that interfaces with Apple Calendar.app.

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

DuckTape provides several commands for managing calendar events:

### List Available Calendars

```bash
>> calendars
```

### Create a Calendar Event

Regular timed event:
```bash
>> calendar "Meeting Title" 2024-02-21 14:30 "Calendar Name" --location "Conference Room" --description "Meeting details" --email "attendee@example.com"
```

All-day event:
```bash
>> calendar "Company Holiday" 2024-02-21 "Calendar Name" --all-day --description "Office Closed"
```

### Command Options

- `--all-day`: Create an all-day event
- `--location "Location"`: Add a location to the event
- `--description "Description"`: Add a description to the event
- `--email "email@example.com"`: Add an attendee to the event

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

1. Create a team meeting:
```bash
>> calendar "Team Meeting" 2024-02-21 14:30 "Work" --location "Conference Room" --description "Weekly sync" --email "team@company.com"
```

2. Create an all-day holiday:
```bash
>> calendar "Company Holiday" 2024-02-21 "Work" --all-day --description "Office Closed"
```

3. List available calendars:
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
