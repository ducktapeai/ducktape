# DuckTape 🦆

## Description
DuckTape is an AI-powered command-line interface that makes it easy to manage your Apple Calendar, Reminders, and Notes. Just type what you want to do in natural language, and DuckTape's AI will understand and execute the appropriate command.
You can also just use the ducktape commands below if you dont want to leverage AI.
Plans underway to integrate as an Application.

> **Note**: DuckTape currently only works on macOS and requires Apple Calendar to be properly configured on your system. [Learn how to set up Apple Calendar](https://support.apple.com/guide/calendar/set-up-icloud-calendar-icl1022/mac)

### AI Models Currently in Use

DuckTape leverages the following language models for natural language understanding:

| Provider | Model | Status | Use Case | 
|----------|-------|:------:|----------|
| OpenAI   | GPT-4 Turbo | ✅ | Primary model for complex natural language processing |
| OpenAI   | GPT-3.5 Turbo | ✅ | Fallback model for faster processing of simple requests |  
| Grok (XAI) | Grok-2-latest | ✅ | Alternative model with specialized calendar understanding |
| DeepSeek | DeepSeek-Coder | 🚧 | In development for code-related tasks and reminders |

The default model is determined by your configuration settings. You can switch between models using:
```bash
ducktape config llm openai  # For OpenAI models
ducktape config llm grok    # For Grok-2-latest model
```

### Integration Status

| Integration      | Status      | Description                                        |
|------------------|:-----------:|----------------------------------------------------|
| macOS            | ✅          | Full support for Apple Calendar, Notes, Reminders  |
| Windows          | ❌          | Not currently supported                            |
| Linux            | ❌          | Not currently supported                            |
| OpenAI           | ✅          | Complete integration with API                      |
| Grok (XAI)       | ✅          | Complete integration with API                      |
| DeepSeek         | 🚧          | Integration in progress                            |
| Apple Contacts   | ✅          | Full contact lookup for event invitations          |
| Zoom             | ✅          | Meeting creation via Server-to-Server OAuth        |
| Native Calendar  | ✅          | Full read/write with Apple Calendar                |
| CSV Import       | ✅          | Full support for event importing                   |
| ICS Import       | ✅          | Full support for iCalendar format                  |
| WebSocket API    | ✅          | Complete API for desktop client connections        |


## Installation

### Using Homebrew (macOS and Linux)

You can now install Ducktape using Homebrew:

```bash
brew install ducktapeai/ducktape/ducktape
```

To update to the latest version:

```bash
brew upgrade ducktape
```

### Manual Installation

From crates.io:
```bash
cargo install ducktape
```

From source:
```bash
git clone https://github.com/DuckTapeAI/ducktape.git
cd ducktape
cargo install --path .
```

## Quick Start

1. Set up your environment variables in a `.env` file:
```bash
OPENAI_API_KEY=your_key_here  # Required if using OpenAI
GROK_API_KEY=your_key_here    # Required if using Grok
DEEPSEEK_API_KEY=your_key_here # Required if using DeepSeek
```

2. Run DuckTape:
```bash
ducktape
```

For API server mode:
```bash
ducktape --api-server
```

## Installation

1. Ensure you have Rust installed
2. Clone this repository
3. Set up required API keys (see [Environment Variables](#environment-variables) section below)
4. Build and run:
```bash
cargo build
cargo run
```

## Running Modes
DuckTape can be run in three different modes:

1. Terminal-only mode (default):
```bash
ducktape
```
Starts DuckTape in a terminal interface without the API server. Best for command-line usage.

2. API server-only mode:
```bash
ducktape --api-server
```
Starts only the API server on port 3000. Useful for integrating with the DuckTape Desktop app or other clients.

3. Full mode:
```bash
ducktape --full
```
Starts both the terminal interface and API server. Use this if you want to use both interfaces simultaneously.

## Environment Variables
DuckTape uses several environment variables to store sensitive configuration information:

### Required for AI Language Models:
Choose at least one of the following AI providers:

```bash
# For OpenAI
export OPENAI_API_KEY='your-openai-api-key-here'

# For Grok (XAI)
export XAI_API_KEY='your-xai-api-key-here'

# For DeepSeek
export DEEPSEEK_API_KEY='your-deepseek-api-key-here'
```

### Required for Zoom Integration:
```bash
# Zoom credentials
export ZOOM_ACCOUNT_ID='your-zoom-account-id'
export ZOOM_CLIENT_ID='your-zoom-client-id'
export ZOOM_CLIENT_SECRET='your-zoom-client-secret'
```

You can add these to your shell profile file (e.g., `.zshrc`, `.bashrc`, or `.bash_profile`) for persistence across terminal sessions.

## Using Natural Language Mode

DuckTape supports natural language commands for creating events, reminders, and notes. This mode requires API keys for supported AI providers.

### Setting Up Natural Language Mode

1. **Obtain API Keys**:
   - OpenAI: [Get your API key](https://platform.openai.com/)
   - Grok (X.AI): [Get your API key](https://x.ai/)
   - DeepSeek: [Get your API key](https://deepseek.ai/)

2. **Set Environment Variables**:
   Add the required API keys to your environment variables:
   ```bash
   export OPENAI_API_KEY='your-openai-api-key-here'
   export XAI_API_KEY='your-xai-api-key-here'
   export DEEPSEEK_API_KEY='your-deepseek-api-key-here'
   ```

   To make these changes persistent, add them to your shell profile (e.g., `~/.zshrc` or `~/.bashrc`):
   ```bash
   echo "export OPENAI_API_KEY='your-openai-api-key-here'" >> ~/.zshrc
   echo "export XAI_API_KEY='your-xai-api-key-here'" >> ~/.zshrc
   echo "export DEEPSEEK_API_KEY='your-deepseek-api-key-here'" >> ~/.zshrc
   source ~/.zshrc
   ```

3. **Run DuckTape**:
   Use natural language commands directly in the terminal:
   ```bash
   ducktape "create an event roadtrip with David Johnston for Tuesday and Wednesday"
   ```

   DuckTape will process the command and translate it into the appropriate calendar action.

### Example Commands

- "schedule a meeting with John tomorrow at 2pm"
- "create a weekly team meeting every Tuesday at 10am"
- "schedule a zoom meeting with the team tomorrow at 3pm"
- "create an event for my dentist appointment next Friday at 2pm"
- "set up a monthly book club meeting on the first Friday"

## Natural Language Examples

Just type what you want to do:
- "schedule a meeting with Shaun tomorrow at 2pm"
- "create a weekly team meeting every Tuesday at 10am"
- "schedule a zoom meeting with the team tomorrow at 3pm"
- "create an event for my dentist appointment next Friday at 2pm"
- "set up a monthly book club meeting on the first Friday"

## Command Reference

### Configuration Commands
```bash
# Switch language model provider
ducktape config llm [openai|grok]  # deepseek support coming soon

# Show current configuration
ducktape config show all

# Show a specific configuration setting
ducktape config show calendar.default

# Set default calendar
ducktape config set calendar.default "<calendar_name>"

# Set default reminder time (in minutes before events)
ducktape config set calendar.reminder 30

# Set default event duration (in minutes)
ducktape config set calendar.duration 60

# Set language model provider
ducktape config set language_model.provider "grok"
```

### Calendar Commands

```bash
# List all available calendars
ducktape calendars

# Create a new calendar event
ducktape calendar create "Meeting_Title" 2025-04-10 13:00 14:30 "Work"

# Create with email invites
ducktape calendar create "Team_Meeting" 2025-04-15 10:00 11:00 "Work" --email "colleague@example.com,manager@example.com"

# Create with contact invites (will look up email addresses automatically)
ducktape calendar create "Project_Review" 2025-04-20 15:00 16:00 "Work" --contacts "John Smith,Jane Doe"

# Create recurring events
ducktape calendar create "Weekly_Standup" 2025-04-03 09:00 09:30 "Work" --repeat weekly

# Create event with Zoom meeting
ducktape calendar create "Client_Call" 2025-04-12 14:00 15:00 "Work" --zoom
```

> **Important: When using the terminal directly, event titles with spaces must be enclosed in quotes.This will be fixed to allow spaces in a coming release**
> 
> When quotes are not properly handled by your terminal, use underscores or hyphens instead:
> ```bash
> # Use underscores for spaces
> ducktape calendar create Weekly_Team_Standup 2025-04-03 16:00 17:00 Work
> 
> # Or use hyphens
> ducktape calendar create Project-Review 2025-04-20 15:00 16:00 Work
> ```

### Calendar Options
- `--all-day` - Create an all-day event
- `--location "<location>"` - Set event location
- `--description "<desc>"` - Set event description
- `--email "<email>"` - Add attendee(s), separate multiple with commas
- `--reminder <minutes>` - Set reminder time in minutes before event
- `--contacts "<name1,name2>"` - Add contacts by name (uses Apple Contacts)
- `--zoom` - Create a Zoom meeting for the event

### Recurring Event Options
- `--repeat [daily|weekly|monthly|yearly]` - Set recurrence frequency
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

## Zoom Integration
DuckTape provides seamless integration with Zoom for creating meeting links directly in your calendar events. There are two ways to set up Zoom integration:

### Option 1: Using the Interactive Setup
```bash
# Run the interactive setup
ducktape zoom-setup
```

This command will guide you through the process of entering your Zoom credentials and save them securely.

### Option 2: Using Environment Variables
Set the following environment variables before running DuckTape:
```bash
export ZOOM_ACCOUNT_ID='your-zoom-account-id'
export ZOOM_CLIENT_ID='your-zoom-client-id'
export ZOOM_CLIENT_SECRET='your-zoom-client-secret'
```

### Creating a Server-to-Server OAuth App in Zoom
To get the required credentials:

1. Sign in to the [Zoom App Marketplace](https://marketplace.zoom.us/)
2. Click "Develop" in the top-right corner
3. Select "Build App"
4. Choose "Server-to-Server OAuth" app type
5. Enter app name and other required information
6. Under "Scopes", add the following permissions:
   - `meeting:write:admin` or `meeting:write`
   - `user:read:admin` or `user:read`
7. Create the app and collect the Account ID, Client ID, and Client Secret
8. Your app might need to be activated or submitted for approval depending on your Zoom account type

### Using Zoom in Calendar Events
Once configured, you can add Zoom meetings to calendar events using:

```bash
# Command-line approach
ducktape calendar create "Meeting Title" 2023-06-15 14:00 15:00 --zoom

# Or using natural language
"schedule a zoom meeting with the team tomorrow at 3pm"
```

## Apple Contacts Integration

DuckTape seamlessly integrates with the Apple Contacts app to simplify adding attendees to your calendar events.

### How Contact Lookup Works

- **Name Matching**: When you specify contact names using the `--contacts` flag or natural language commands, DuckTape looks up these names in your Apple Contacts app.
- **Exact Matching**: Contact names must match exactly as they appear in Apple Contacts for successful lookup.
- **Multiple Email Handling**: If a contact has multiple email addresses, DuckTape will use the all the email address.

### Using Contacts in Commands

```bash
# Add contacts by name (names must match Apple Contacts entries)
ducktape calendar create "Team Meeting" 2024-04-01 10:00 11:00 --contacts "Jane Smith,John Doe"

# Using natural language
ducktape "schedule a meeting with Jane Smith and John Doe tomorrow at 2pm"
```

### Tips for Contact Usage

- Ensure contact names are spelled exactly as they appear in the Apple Contacts app
- If a contact isn't being found, check your Contacts app to verify the name format
- For contacts with the same name, consider using their email address directly with the `--email` flag
- Contact groups from the Apple Contacts app are not currently supported

## Calendar Import
DuckTape supports importing calendar events from CSV and ICS files. 

### CSV Format Requirements
The CSV file should have the following columns:
- title (required): Event title
- date (required): YYYY-MM-DD format
- start_time (required): HH:MM format (24-hour)
- end_time (required): HH:MM format (24-hour)
- calendar (optional): Calendar name to add event to
- description (optional): Event description
- location (optional): Event location
- attendees (optional): Comma-separated email addresses

Example CSV:
```csv
title,date,start_time,end_time,calendar,description,location,attendees
Team Meeting,2024-02-15,14:00,15:00,Work,Weekly sync,Conference Room A,john@example.com;jane@example.com
```

### ICS Format
DuckTape supports standard iCalendar (.ics) files following the RFC 5545 specification. When importing ICS files, DuckTape will:
- Preserve all event properties including attendees, location, and description
- Handle recurring events with their full recurrence rules
- Import reminders and alerts
- Maintain event categories and classifications

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
- Zoom meeting integration with automatic meeting creation
- State persistence
- Calendar integration with Apple Calendar.app
- Modular, well-organized code architecture

## Security Features
- Command injection prevention through proper input sanitization and escaping
- Secure API key handling via environment variables rather than hardcoded values
- Validation of all user inputs before processing
- Size and data limits on JSON parsing to prevent DoS attacks
- Proper error handling to prevent information leakage
- Memory-safe mutex handling for concurrent operations
- Path traversal prevention in file operations
- Automatic security checks with `security-check.sh` script

## Running Security Checks
DuckTape includes a comprehensive security check script:

```bash
# Run the security checks
chmod +x security-check.sh
./security-check.sh
```

The script checks for:
- Dependency vulnerabilities using cargo-audit
- License compliance and additional dependency checks with cargo-deny
- Code quality issues with Clippy security lints
- Common vulnerable patterns like unwrap(), expect(), and unsafe blocks
- Potential command injection vulnerabilities
- Secure handling of sensitive data

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
- (Optional) Zoom API credentials for meeting integration

## Security Best Practices
- Make sure you always keep your API keys confidential and use environment variables
- Regularly run `cargo update` and `cargo audit` to keep dependencies up to date and secure
- Never edit the generated JSON state files manually
- Consider using application-specific passwords for calendar access if using iCloud
- Store Zoom API credentials securely in your system keychain

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to get started.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.