# DuckTape 🦆 ...

DuckTape is an AI-powered command-line interface that simplifies managing your Apple Calendar, Reminders, and Notes. With DuckTape, you can use two distinct approaches: **Natural Language Mode** and **Terminal Mode**.

> **Note**: DuckTape currently### Advanced Features

### Zoom Integration
DuckTape can create Zoom meetings directly from both Terminal Mode and Natural Language Mode. To enable this feature, set the following environment variables:

```bash
export ZOOM_ACCOUNT_ID='your-zoom-account-id'
export ZOOM_CLIENT_ID='your-zoom-client-id'
export ZOOM_CLIENT_SECRET='your-zoom-client-secret'
```

#### Creating Zoom Meetings

Using Terminal Mode:
```bash
ducktape calendar create "Team Meeting" 2025-04-15 13:00 14:00 "Work" --contacts "Joe Duck" --zoom
```

Using Natural Language Mode:
```
🦆 ducktape "create a zoom meeting called Team Meeting with Joe Duck for this coming Tuesday"
```s on macOS and requires Apple Calendar to be properly configured on your system. [Learn how to set up Apple Calendar](https://support.apple.com/guide/calendar/set-up-icloud-calendar-icl1022/mac).
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

#### Running in Natural Language Mode (API Key Required)

1. Open your terminal.
2. Type `ducktape` and press enter. This will place you in the interactive Natural Language terminal:
   ```bash
   ducktape
   ```
3. You'll see a welcome message. Then type your request using natural language:
   ```
   🦆 ducktape "create an event Team Meeting with Joe Duck for this coming Tuesday"
   ```
4. DuckTape will process your natural language request and execute the appropriate command:
   ```
   Processing natural language: 'ducktape "create an event Team Meeting with Joe Duck for this coming Tuesday"'
   Translated to command: ducktape calendar create "Team Meeting" 2025-04-22 11:00 12:00 "Work" --contacts "Joe Duck"
   ```

The power of the Natural Language Mode is that it automatically interprets dates, times, and contacts, saving you time and effort.
----
### Example Commands

### Natural Language Examples
- `ducktape "create an event Team Meeting with Joe Duck for this coming Tuesday"`
- `ducktape "create a zoom meeting called Team Meeting with Joe Duck for this coming Tuesday"`
- `ducktape "schedule a meeting with Joe Duck tomorrow at 2pm about project review"`
- `ducktape "create a weekly team meeting every Tuesday at 10am"`
- `ducktape "add a reminder to buy groceries next Monday morning"`

### 2. Terminal Mode (No API Key Required)

In this mode, DuckTape operates as a traditional command-line interface (CLI) where you directly execute structured commands without requiring any API keys.

#### Running in Terminal Mode

1. Open your terminal.
2. Use explicit commands with the appropriate syntax to interact with DuckTape:
   ```bash
   ducktape calendar create "Team Meeting" 2025-04-15 13:00 14:00 "Work" --contacts "Joe Duck" --zoom
   ```
   
This command explicitly specifies all parameters: event title, date, start time, end time, calendar name, contacts, and the zoom flag to create a meeting link.

---
### Terminal Command Examples

#### Calendar Commands
- List all calendars:
  ```bash
  ducktape calendar list
  ```
- Create a calendar event:
  ```bash
  ducktape calendar create "Project-Review" 2025-04-20 15:00 16:00 "Work"
  ```
- Add attendees by email:
  ```bash
  ducktape calendar create "Team Sync" 2025-04-15 10:00 11:00 "Work" --email "joe.Duck@example.com,jane.doe@example.com"
  ```
- Create an event with a Zoom meeting and contacts:
  ```bash
  ducktape calendar create "Team Meeting" 2025-04-15 13:00 14:00 "Work" --contacts "Joe Duck" --zoom
  ```
- Create a recurring event:
  ```bash
  ducktape calendar create "Weekly Standup" 2025-04-15 09:00 09:30 "Work" --repeat weekly
  ```

#### Todo Commands
- Create a todo:
  ```bash
  ducktape todo create "Buy groceries" "Reminders"
  ```
- List todos:
  ```bash
  ducktape todo list
  ```
- Delete a todo:
  ```bash
  ducktape todo delete "Buy groceries"
  ```

#### Notes Commands
- Create a note:
  ```bash
  ducktape note create "Project ideas" "Content for the note"
  ```
- List notes:
  ```bash
  ducktape note list
  ```
- Search notes:
  ```bash
  ducktape note search "project"
  ```
- Delete a note:
  ```bash
  ducktape note delete "Project ideas"
  ```

#### Configuration Commands
- Show configuration:
  ```bash
  ducktape config show all
  ```
- Set default calendar:
  ```bash
  ducktape config set calendar.default "Work"
  ```
- Set language model provider:
  ```bash
  ducktape config set language_model.provider "grok"
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

---

## Configuration

DuckTape uses a `config.toml` file located in the root of the repository to manage its settings. This file allows you to configure various aspects of the application, such as whether to use a language model (LLM) or operate in Terminal Mode.

## Example Configuration
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