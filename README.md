# DuckTape 🦆 - Your Personal TimeAI

DuckTape is an AI-powered command-line interface that simplifies managing your Apple Calendar, Reminders, and Notes. DuckTape now supports a **hybrid CLI**: you can use direct commands, natural language in interactive mode, or run natural language directly from the terminal using the `ai` subcommand.

> **Note**: DuckTape currently only works on macOS and requires Apple Calendar, Reminders, and Notes to be properly configured on your system. [Learn how to set up Apple Calendar](https://support.apple.com/guide/calendar/set-up-icloud-calendar-icl1022/mac).
> DuckTape will use your native Apple capabilities, for example Apple Contacts, Apple Reminders and so forth. Please ensure these Applications are properly setup and configured.

**📚 Full Documentation:** [ducktapeai.com/docs](https://ducktapeai.com/docs)

---

## Features

- **Hybrid CLI**: Use direct commands, natural language in interactive mode, or natural language via the `ai` subcommand
- **Natural Language Processing**: Use everyday language to create events, reminders, and notes
- **Command-Line Interface**: Execute precise commands for advanced control
- **AI Model Support**: Integrates with OpenAI, Grok (X.AI), and DeepSeek for natural language understanding
- **Zoom Integration**: Schedule Zoom meetings directly from the terminal
- **Apple Contacts Integration**: Automatically add attendees to events using Apple Contacts
- **Reminder Management**: Create and manage reminders with due dates and notes
- **Recurring Events**: Create daily, weekly, monthly, or yearly recurring events
- **Enhanced Recurring Pattern Support**: Improved support for complex recurring event patterns
- **Advanced Time Parsing**: Better detection of time expressions and timezone handling
- **Smart Contact Detection**: Better recognition of contact information in event creation
- **WebSocket Stability**: Improved stability in WebSocket connections for API server

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

DuckTape now supports three ways to use commands:

### 1. Interactive Hybrid Mode (Recommended)

Type `ducktape` to enter interactive mode. You can then type either direct commands or natural language:

```bash
ducktape
```

**Examples in interactive mode:**
```
🦆 calendar list
🦆 schedule a meeting in 30 minutes with Joe
🦆 remind me to call Jane tomorrow at 2pm
🦆 ducktape note create "Project ideas" "Content for the note"
```

### 2. Direct CLI Commands (Terminal Mode)

Run structured commands directly from your shell:

```bash
ducktape calendar list
ducktape calendar create "Team Meeting" 2025-04-26 13:00 14:00 "Work" --contacts "Joe Duck" --zoom
ducktape note list
```

### 3. Natural Language via `ai` Subcommand

Run natural language commands directly from your shell using the `ai` subcommand:

```bash
ducktape ai schedule a meeting in 30 minutes with Joe
ducktape ai remind me to call Jane tomorrow at 2pm
ducktape ai create a zoom event today at 10am called Team Check in and invite Joe Duck
ducktape ai add a note about the new marketing strategy
```

---

## Command Examples

### Interactive Mode (Hybrid)
- `calendar list`
- `schedule a meeting in 30 minutes with Joe`
- `remind me to call Jane tomorrow at 2pm`
- `note create "Project ideas" "Content for the note"`

### Direct CLI Commands
- `ducktape calendar list`
- `ducktape calendar create "Project-Review" 2025-04-28 15:00 16:00 "Work"`
- `ducktape reminder create "Buy groceries" --remind "2025-04-28 18:00"`
- `ducktape note list`

### Natural Language via `ai` Subcommand
- `ducktape ai schedule a meeting in 30 minutes with Joe`
- `ducktape ai remind me to call Jane tomorrow at 2pm`
- `ducktape ai create a note titled "Meeting Ideas" with content about product planning`

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

## Recent Updates (v0.16.15)

- Enhanced support for recurring event patterns in calendar operations
- Improved handling of timezone conversions for international meetings
- Refined error messaging for better user experience
- Updated dependencies for better performance and security
- Fixed edge cases in time parsing for specific date formats
- Improved stability in WebSocket connections for API integrations

---

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to get started.

---

## Security

DuckTape takes security seriously. We follow Rust security best practices and regularly update dependencies for security patches. For more information, see our [Security Policy](SECURITY.md).

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.