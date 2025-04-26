# DuckTape 🦆 - Your Personal TimeAI

DuckTape is an AI-powered command-line interface that simplifies managing your Apple Calendar, Reminders, and Notes. With DuckTape, you can use two distinct approaches: **Natural Language Mode** and **Terminal Mode**.

> **Note**: DuckTape currently only works on macOS and requires Apple Calendar, Reminders, and Notes to be properly configured on your system. [Learn how to set up Apple Calendar](https://support.apple.com/guide/calendar/set-up-icloud-calendar-icl1022/mac).
> DuckTape will use your native Apple capabilities, for example Apple Contacts, Apple Reminders and so forth. Please ensure these Applications are properly setup and configured.

**📚 Full Documentation:** [ducktapeai.com/docs](https://ducktapeai.com/docs)

---

## Features

- **Natural Language Processing**: Use everyday language to create events, reminders, and notes
- **Command-Line Interface**: Execute precise commands for advanced control
- **AI Model Support**: Integrates with OpenAI, Grok (X.AI), and DeepSeek for natural language understanding
- **Zoom Integration**: Schedule Zoom meetings directly from the terminal
- **Apple Contacts Integration**: Automatically add attendees to events using Apple Contacts
- **Reminder Management**: Create and manage reminders with due dates and notes
- **Recurring Events**: Create daily, weekly, monthly, or yearly recurring events
- **Improved Time Parsing**: Enhanced detection of time expressions in natural language commands
- **Smart Contact Detection**: Better recognition of contact information in event creation

---

## Installation

### Using Homebrew (Recommended)

Install DuckTape via Homebrew:
```bash
brew install ducktapeai/ducktape/ducktape
```

To update to the latest version:
```bash
brew upgrade ducktapeai/ducktape/ducktape
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

---

## Getting Started

DuckTape offers two modes of operation:

### 1. Natural Language Mode (Requires API Key)

In this mode, DuckTape uses AI language models to interpret natural language commands. This requires setting up API keys for one of the supported AI providers.

#### Setting Up API Keys

Choose at least one provider and set the corresponding environment variable:

```bash
# For OpenAI
export OPENAI_API_KEY='your-openai-api-key-here'

# For Grok (X.AI)
export XAI_API_KEY='your-xai-api-key-here'

# For DeepSeek
export DEEPSEEK_API_KEY='your-deepseek-api-key-here'
```

To make these changes persistent, add them to your shell profile (e.g., `~/.zshrc` or `~/.bashrc`).

#### Running in Natural Language Mode

1. Open your terminal.
2. Type `ducktape` and press enter to start the interactive Natural Language terminal:
   ```bash
   ducktape
   ```
3. You'll see a welcome message. Then type your request using natural language:
   ```
   🦆 create a zoom event today at 10am called Team Check in and invite Joe Duck
   ```
4. DuckTape will process your natural language request and execute the appropriate command:
   ```
   Processing natural language: 'create a zoom event today at 10am called Team Check in and invite Joe Duck'
   Translated to command: ducktape calendar create "Team Check in" 2025-04-26 10:00 11:00 "Work" --email "joe.duck@example.com" --contacts "Joe Duck" --zoom
   ```

The power of Natural Language Mode is that it automatically interprets dates, times, and contacts, saving you time and effort.

### 2. Terminal Mode (No API Key Required)

In this mode, DuckTape operates as a traditional command-line interface (CLI) where you directly execute structured commands without requiring any API keys.

```bash
ducktape calendar create "Team Meeting" 2025-04-26 13:00 14:00 "Work" --contacts "Joe Duck" --zoom
```

This command explicitly specifies all parameters: event title, date, start time, end time, calendar name, contacts, and the zoom flag to create a meeting link.

---

## Command Examples

### Natural Language Examples

#### Calendar Events
- `create an event Team Meeting with Joe Duck for this coming Tuesday`
- `create a zoom event today at 10am called Team Check in and invite Joe Duck`
- `schedule a meeting with Joe Duck tomorrow at 2pm about project review`
- `create a weekly team meeting every Tuesday at 10am`

#### Reminders
- `create a reminder today at 11pm called Check if Ducks are swimming`
- `add a reminder to buy groceries next Monday morning`
- `remind me to call Joe Duck on Friday at 3pm`
- `set a reminder for tomorrow at noon to review documents`

#### Notes
- `create a note titled "Meeting Ideas" with content about product planning`
- `add a note about the new marketing strategy`

### Terminal Command Examples

#### Calendar Commands
- List all calendars:
  ```bash
  ducktape calendar list
  ```
- Create a calendar event:
  ```bash
  ducktape calendar create "Project-Review" 2025-04-28 15:00 16:00 "Work"
  ```
- Create an event with a Zoom meeting and contacts:
  ```bash
  ducktape calendar create "Team Meeting" 2025-04-27 13:00 14:00 "Work" --contacts "Joe Duck" --zoom
  ```

#### Reminder Commands
- Create a reminder:
  ```bash
  ducktape reminder create "Buy groceries" --remind "2025-04-28 18:00"
  ```
- List reminders:
  ```bash
  ducktape reminder list
  ```
- Create a reminder in a specific list:
  ```bash
  ducktape reminder create "Call Joe Duck" "Work" --remind "tomorrow at 3pm"
  ```

> **Note**: For backward compatibility, the `todo` command is also supported and maps to the reminder functionality.

#### Notes Commands
- Create a note:
  ```bash
  ducktape note create "Project ideas" "Content for the note"
  ```
- List notes:
  ```bash
  ducktape note list
  ```

#### Utility Commands
- Show version:
  ```bash
  ducktape --version
  ```
- Show help:
  ```bash
  ducktape --help
  ```
- Exit the application:
  ```bash
  exit
  ```

For more detailed command documentation, visit [ducktapeai.com/docs](https://ducktapeai.com/docs).

---

## Configuration

DuckTape uses a `config.toml` file to manage its settings:

```toml
[language_model]
provider = "Grok"  # Options: "OpenAI", "Grok", "DeepSeek", or leave empty for Terminal Mode

[calendar]
default_calendar = "Work"
default_reminder_minutes = 15
default_duration_minutes = 60

[reminder]
default_list = "Reminders"
default_reminder = true

[notes]
default_folder = "Notes"
```

### Viewing and Editing Configuration
- To view the current configuration:
  ```bash
  ducktape config show all
  ```
- To change settings via command line:
  ```bash
  ducktape config set language_model.provider "grok"
  ```

For complete configuration details, see [ducktapeai.com/docs/config.html](https://ducktapeai.com/docs/config.html).

---

## Advanced Features

### Zoom Integration
DuckTape can create Zoom meetings directly from both Terminal Mode and Natural Language Mode.

Set up Zoom integration with:
```bash
export ZOOM_ACCOUNT_ID='your-zoom-account-id'
export ZOOM_CLIENT_ID='your-zoom-client-id'
export ZOOM_CLIENT_SECRET='your-zoom-client-secret'
```

For more details on Zoom integration, see [ducktapeai.com/docs/zoom.html](https://ducktapeai.com/docs/zoom.html).

### Contact Integration

DuckTape integrates with Apple Contacts to automatically look up email addresses:

```bash
ducktape calendar create "Project Discussion" 2025-04-28 14:00 15:00 "Work" --contacts "Joe Duck, Jane Doe"
```

Or in Natural Language Mode:
```
🦆 schedule a meeting with Joe Duck and Jane Doe tomorrow at 2pm
```

The latest version (0.16.11) provides enhanced contact extraction with support for the "and invite" pattern in natural language commands.

---

## Troubleshooting

### Common Issues
- **Missing API Keys**: Ensure you have set the required environment variables for your chosen language model provider.
- **Invalid Calendar Name**: Use `ducktape calendar list` to see available calendars.
- **Contact Not Found**: Verify that the contact exists in your Apple Contacts app.
- **Zoom Integration Issues**: Check that your Zoom API credentials are correct and have the necessary permissions.
- **Time Parsing Problems**: If times aren't recognized correctly, try to be more explicit with AM/PM designations.

DuckTape provides detailed logging information that can help diagnose issues:

```
[2025-04-26T20:04:10Z INFO ducktape::calendar] Creating Zoom meeting for event: Team Check in
[2025-04-26T20:04:11Z INFO ducktape::zoom] Successfully created Zoom meeting: 84349352425
```

For more troubleshooting tips, visit our documentation at [ducktapeai.com/docs](https://ducktapeai.com/docs).

---

## Recent Updates (v0.16.11)

- Fixed time parsing in natural language commands with specific time expressions
- Improved AM/PM to 24-hour time conversion for calendar events
- Enhanced contact extraction functionality for calendar events
- Added support for "and invite" pattern in natural language commands
- Improved error handling and stability for calendar operations
- Fixed compilation issues in parser modules

---

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to get started.

---

## Security

DuckTape takes security seriously. We follow Rust security best practices and regularly update dependencies for security patches. For more information, see our [Security Policy](SECURITY.md).

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.