# DuckTape 🦆

DuckTape is an AI-powered command-line interface that simplifies managing your Apple Calendar, Reminders, and Notes. With DuckTape, you can use two distinct approaches: **Natural Language Mode** and **Terminal Mode**.

> **Note**: DuckTape currently only works on macOS and requires Apple Calendar to be properly configured on your system. [Learn how to set up Apple Calendar](https://support.apple.com/guide/calendar/set-up-icloud-calendar-icl1022/mac).
> DuckTape will use your native Apple capabilities, for example Apple Contacts, Apple Todo and so forth. Please ensure these Applications are properly setup and configured as outlined above.

---

## Features

- **Natural Language Processing**: Use everyday language to create events, reminders, and notes.
- **Command-Line Interface**: Execute precise commands for advanced control.
- **AI Model Support**: Integrates with OpenAI, Grok (X.AI), and DeepSeek for natural language understanding.
- **Zoom Integration**: Schedule Zoom meetings directly from the terminal.
- **Recurring Events**: Create daily, weekly, monthly, or yearly recurring events.
- **Apple Contacts Integration**: Automatically add attendees to events using Apple Contacts.

---

## Installation

### Using Homebrew (Recommended)

Install DuckTape via Homebrew:
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

To make these changes persistent, add them to your shell profile (e.g., `~/.zshrc` or `~/.bashrc`):
```bash
echo "export OPENAI_API_KEY='your-openai-api-key-here'" >> ~/.zshrc
echo "export XAI_API_KEY='your-xai-api-key-here'" >> ~/.zshrc
echo "export DEEPSEEK_API_KEY='your-deepseek-api-key-here'" >> ~/.zshrc
source ~/.zshrc
```

#### Running in Natural Language Mode

1. Open your terminal.
2. Run DuckTape with a natural language command:
   ```bash
   ducktape "create an event roadtrip with Duck Tape for this coming Tuesday"
   ```

### 2. Terminal Mode (No API Key Required)

In this mode, DuckTape operates as a traditional command-line interface (CLI) where you can directly execute commands without requiring any API keys.

#### Running in Terminal Mode

1. Open your terminal.
2. Use explicit commands to interact with DuckTape:
   ```bash
   ducktape calendar create "Team Meeting" 2025-04-15 10:00 11:00 "Work"
   ```

---

## Example Commands

### Natural Language Examples
- "schedule a meeting with John tomorrow at 2pm"
- "create a weekly team meeting every Tuesday at 10am"
- "schedule a Zoom meeting with the team tomorrow at 3pm"
- "create an event for my dentist appointment next Friday at 2pm"

### Terminal Command Examples
- List all calendars:
  ```bash
  ducktape calendars
  ```
- Create a calendar event:
  ```bash
  ducktape calendar create "Project Review" 2025-04-20 15:00 16:00 "Work"
  ```
- Add attendees by email:
  ```bash
  ducktape calendar create "Team Sync" 2025-04-15 10:00 11:00 "Work" --email "john@example.com,jane@example.com"
  ```

---

## Configuration

DuckTape uses a `config.toml` file located in the root of the repository to manage its settings. This file allows you to configure various aspects of the application, such as whether to use a language model (LLM) or operate in Terminal Mode.

### Example Configuration
```toml
[language_model]
provider = "OpenAI"  # Options: "OpenAI", "Grok", "DeepSeek", or leave empty for Terminal Mode

[calendar]
default_calendar = "Work"
default_reminder_minutes = 15
default_duration_minutes = 60

[todo]
default_list = "Reminders"
default_reminder = true

[notes]
default_folder = "Notes"
```

### Viewing and Editing Configuration
- To view the current configuration, open the `config.toml` file in the root of the repository.
- To change settings, edit the file and save your changes.

### Key Settings
- **Language Model Provider**: Set the `provider` field under `[language_model]` to enable natural language processing. Leave it empty to use Terminal Mode.
- **Default Calendar**: Specify the default calendar for events under `[calendar]`.
- **Default Todo List**: Set the default list for todos under `[todo]`.
- **Default Notes Folder**: Define the folder for notes under `[notes]`.

For more details, refer to the documentation or examples in the `config.toml` file.

### Default Settings
You can configure default settings for DuckTape using the `config` command:

- Set the default calendar:
  ```bash
  ducktape config set calendar.default "Work"
  ```
- Set the default reminder time (in minutes):
  ```bash
  ducktape config set calendar.reminder 30
  ```
- Set the default event duration (in minutes):
  ```bash
  ducktape config set calendar.duration 60
  ```

### Viewing Configuration
To view your current configuration:
```bash
ducktape config show all
```

---

## Advanced Features

### Zoom Integration
DuckTape can create Zoom meetings directly from the terminal. To enable this feature, set the following environment variables:

```bash
export ZOOM_ACCOUNT_ID='your-zoom-account-id'
export ZOOM_CLIENT_ID='your-zoom-client-id'
export ZOOM_CLIENT_SECRET='your-zoom-client-secret'
```

### Recurring Events
Create recurring events with the `--repeat` flag:
```bash
ducktape calendar create "Weekly Standup" 2025-04-03 09:00 09:30 "Work" --repeat weekly
```

---

## Troubleshooting

### Common Issues
- **Missing API Keys**: Ensure you have set the required environment variables.
- **Invalid Calendar Name**: Use `ducktape calendars` to list available calendars.
- **Permission Denied**: Ensure DuckTape has executable permissions:
  ```bash
  chmod +x ducktape
  ```

---

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to get started.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.