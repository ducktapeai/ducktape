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

[Your chosen license]

## Contributing

[Your contribution guidelines]
