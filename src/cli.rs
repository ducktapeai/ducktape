use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// DuckTape - AI-powered terminal tool for Apple Calendar, Reminders and Notes
#[derive(Debug, Parser)]
#[command(name = "ducktape")]
#[command(about = "AI-powered terminal tool for Apple Calendar, Reminders and Notes", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Command to execute (if not specified, enters interactive terminal mode)
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Start in API server mode only
    #[arg(long = "api-server", conflicts_with = "full")]
    pub api_server: bool,

    /// Start both terminal and API server
    #[arg(long = "full", conflicts_with = "api_server")]
    pub full: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage calendar events
    Calendar {
        #[command(subcommand)]
        action: CalendarActions,
    },

    /// Manage reminders/todos
    #[command(alias = "todos")]
    Todo {
        #[command(subcommand)]
        action: TodoActions,
    },

    /// Manage notes
    #[command(alias = "notes")]
    Note {
        #[command(subcommand)]
        action: NoteActions,
    },

    /// View or modify configuration
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },

    /// Manage contact groups
    #[command(alias = "contacts")]
    Contact {
        #[command(subcommand)]
        action: ContactActions,
    },

    /// Utility commands
    #[command(alias = "utils")]
    Utility {
        #[command(subcommand)]
        action: UtilityActions,
    },
}

#[derive(Debug, Subcommand)]
pub enum CalendarActions {
    /// List available calendars
    List,

    /// List available event properties
    #[command(alias = "properties")]
    Props,

    /// Create a new calendar event
    #[command(alias = "add")]
    Create {
        /// Event title
        #[arg(required = true)]
        title: String,

        /// Event date (YYYY-MM-DD)
        #[arg(required = true)]
        date: String,

        /// Start time (HH:MM)
        #[arg(required = true)]
        start_time: String,

        /// End time (HH:MM)
        #[arg(required = true)]
        end_time: String,

        /// Calendar name
        #[arg(default_value = "Work")]
        calendar: String,

        /// Contact names to invite
        #[arg(long, value_delimiter = ',')]
        contacts: Option<Vec<String>>,

        /// Email addresses to invite
        #[arg(long, value_delimiter = ',')]
        email: Option<Vec<String>>,

        /// Event location
        #[arg(long)]
        location: Option<String>,

        /// Event notes/description
        #[arg(long)]
        notes: Option<String>,

        /// Create a Zoom meeting for this event
        #[arg(long)]
        zoom: bool,

        /// Recurrence frequency (daily, weekly, monthly, yearly)
        #[arg(long, alias = "recurring")]
        repeat: Option<RecurrenceFreq>,

        /// Recurrence interval (e.g., every 2 weeks)
        #[arg(long)]
        interval: Option<u32>,

        /// End date for recurrence (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,

        /// Number of occurrences
        #[arg(long)]
        count: Option<u32>,

        /// Days of week (0=Sun, 1=Mon, etc.)
        #[arg(long, value_delimiter = ',')]
        days: Option<Vec<u8>>,
    },

    /// Delete a calendar event
    #[command(alias = "remove")]
    Delete {
        /// Event ID or title to delete
        #[arg(required = true)]
        event_id: String,

        /// Calendar name
        #[arg(default_value = "Work")]
        calendar: String,
    },

    /// Import events from a file
    Import {
        /// File to import
        #[arg(required = true)]
        file: PathBuf,

        /// Calendar name
        #[arg(default_value = "Work")]
        calendar: String,

        /// File format (ics, csv)
        #[arg(long, default_value = "ics")]
        format: String,
    },

    /// Set the default calendar
    SetDefault {
        /// Calendar name
        #[arg(required = true)]
        calendar: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum RecurrenceFreq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Subcommand)]
pub enum TodoActions {
    /// List available reminder lists
    Lists,

    /// List reminders
    List {
        /// List name
        list: Option<String>,
    },

    /// Create a new reminder
    #[command(alias = "add")]
    Create {
        /// Reminder title
        #[arg(required = true)]
        title: String,

        /// List name(s)
        #[arg(value_delimiter = ',')]
        lists: Vec<String>,

        /// Set a reminder time
        #[arg(long)]
        remind: Option<String>,

        /// Notes for the reminder
        #[arg(long)]
        notes: Option<String>,
    },

    /// Mark a reminder as completed
    #[command(alias = "done")]
    Complete {
        /// Reminder ID or title
        #[arg(required = true)]
        reminder_id: String,

        /// List name
        list: Option<String>,
    },

    /// Delete a reminder
    #[command(alias = "remove")]
    Delete {
        /// Reminder ID or title
        #[arg(required = true)]
        reminder_id: String,

        /// List name
        list: Option<String>,
    },

    /// Set the default reminder list
    #[command(alias = "set-default")]
    SetList {
        /// List name
        #[arg(required = true)]
        list: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum NoteActions {
    /// List notes
    List {
        /// Folder name
        folder: Option<String>,
    },

    /// Create a new note
    #[command(aliases = ["add", "new"])]
    Create {
        /// Note title
        #[arg(required = true)]
        title: String,

        /// Note content
        content: Option<String>,

        /// Folder name
        folder: Option<String>,
    },

    /// Search for notes
    #[command(alias = "find")]
    Search {
        /// Search query
        #[arg(required = true)]
        query: String,

        /// Folder name
        folder: Option<String>,
    },

    /// Delete a note
    #[command(alias = "remove")]
    Delete {
        /// Note title or ID
        #[arg(required = true)]
        note_id: String,

        /// Folder name
        folder: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigActions {
    /// Show configuration
    #[command(aliases = ["list", "get"])]
    Show {
        /// Key to show (use "all" for all settings)
        key: Option<String>,
    },

    /// Set configuration value
    Set {
        /// Configuration key
        #[arg(required = true)]
        key: String,

        /// Configuration value
        #[arg(required = true)]
        value: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ContactActions {
    /// Create a new contact group
    Create {
        /// Group name
        #[arg(required = true)]
        group_name: String,

        /// Email addresses
        #[arg(required = true, num_args = 1..)]
        emails: Vec<String>,
    },

    /// List available contact groups
    List,

    /// Show contact group members
    Show {
        /// Group name
        #[arg(required = true)]
        group_name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum UtilityActions {
    /// Display current date
    Date,

    /// Display current time
    Time,

    /// Display current date and time
    DateTime,
}

/// Convert a Clap command to a CommandArgs representation
pub fn convert_to_command_args(cli: &Cli) -> Option<crate::command_processor::CommandArgs> {
    use crate::command_processor::CommandArgs;
    use std::collections::HashMap;

    // If no command specified, return None to handle terminal mode
    let command = match &cli.command {
        Some(cmd) => cmd,
        None => return None,
    };

    let mut command_str = String::new();
    let mut args = Vec::new();
    let mut flags = HashMap::new();

    match command {
        Commands::Calendar { action } => {
            command_str = "calendar".to_string();
            match action {
                CalendarActions::List => {
                    args.push("list".to_string());
                }
                CalendarActions::Props => {
                    args.push("props".to_string());
                }
                CalendarActions::Create {
                    title,
                    date,
                    start_time,
                    end_time,
                    calendar,
                    contacts,
                    email,
                    location,
                    notes,
                    zoom,
                    repeat,
                    interval,
                    until,
                    count,
                    days,
                } => {
                    args.push("create".to_string());
                    args.push(title.clone());
                    args.push(date.clone());
                    args.push(start_time.clone());
                    args.push(end_time.clone());
                    args.push(calendar.clone());

                    // Handle optional flags
                    if let Some(contact_list) = contacts {
                        flags.insert("contacts".to_string(), Some(contact_list.join(",")));
                    }
                    if let Some(email_list) = email {
                        flags.insert("email".to_string(), Some(email_list.join(",")));
                    }
                    if let Some(loc) = location {
                        flags.insert("location".to_string(), Some(loc.clone()));
                    }
                    if let Some(note) = notes {
                        flags.insert("notes".to_string(), Some(note.clone()));
                    }
                    if *zoom {
                        flags.insert("zoom".to_string(), None);
                    }
                    if let Some(repeat_val) = repeat {
                        let repeat_str = match repeat_val {
                            RecurrenceFreq::Daily => "daily",
                            RecurrenceFreq::Weekly => "weekly",
                            RecurrenceFreq::Monthly => "monthly",
                            RecurrenceFreq::Yearly => "yearly",
                        };
                        flags.insert("repeat".to_string(), Some(repeat_str.to_string()));
                    }
                    if let Some(interval_val) = interval {
                        flags.insert("interval".to_string(), Some(interval_val.to_string()));
                    }
                    if let Some(until_date) = until {
                        flags.insert("until".to_string(), Some(until_date.clone()));
                    }
                    if let Some(count_val) = count {
                        flags.insert("count".to_string(), Some(count_val.to_string()));
                    }
                    if let Some(day_list) = days {
                        let days_str =
                            day_list.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(",");
                        flags.insert("days".to_string(), Some(days_str));
                    }
                }
                CalendarActions::Delete { event_id, calendar } => {
                    args.push("delete".to_string());
                    args.push(event_id.clone());
                    args.push(calendar.clone());
                }
                CalendarActions::Import { file, calendar, format } => {
                    args.push("import".to_string());
                    args.push(file.to_string_lossy().to_string());
                    args.push(calendar.clone());
                    flags.insert("format".to_string(), Some(format.clone()));
                }
                CalendarActions::SetDefault { calendar } => {
                    args.push("set-default".to_string());
                    args.push(calendar.clone());
                }
            }
        }
        Commands::Todo { action } => {
            command_str = "todo".to_string();
            match action {
                TodoActions::Lists => {
                    args.push("lists".to_string());
                }
                TodoActions::List { list } => {
                    args.push("list".to_string());
                    if let Some(list_name) = list {
                        args.push(list_name.clone());
                    }
                }
                TodoActions::Create { title, lists, remind, notes } => {
                    args.push("create".to_string());
                    args.push(title.clone());
                    args.extend(lists.iter().cloned());

                    if let Some(remind_time) = remind {
                        flags.insert("remind".to_string(), Some(remind_time.clone()));
                    }
                    if let Some(note_text) = notes {
                        flags.insert("notes".to_string(), Some(note_text.clone()));
                    }
                }
                TodoActions::Complete { reminder_id, list } => {
                    args.push("complete".to_string());
                    args.push(reminder_id.clone());
                    if let Some(list_name) = list {
                        args.push(list_name.clone());
                    }
                }
                TodoActions::Delete { reminder_id, list } => {
                    args.push("delete".to_string());
                    args.push(reminder_id.clone());
                    if let Some(list_name) = list {
                        args.push(list_name.clone());
                    }
                }
                TodoActions::SetList { list } => {
                    args.push("set-list".to_string());
                    args.push(list.clone());
                }
            }
        }
        Commands::Note { action } => {
            command_str = "note".to_string();
            match action {
                NoteActions::List { folder } => {
                    args.push("list".to_string());
                    if let Some(folder_name) = folder {
                        args.push(folder_name.clone());
                    }
                }
                NoteActions::Create { title, content, folder } => {
                    args.push("create".to_string());
                    args.push(title.clone());
                    if let Some(content_text) = content {
                        args.push(content_text.clone());
                    }
                    if let Some(folder_name) = folder {
                        args.push(folder_name.clone());
                    }
                }
                NoteActions::Search { query, folder } => {
                    args.push("search".to_string());
                    args.push(query.clone());
                    if let Some(folder_name) = folder {
                        args.push(folder_name.clone());
                    }
                }
                NoteActions::Delete { note_id, folder } => {
                    args.push("delete".to_string());
                    args.push(note_id.clone());
                    if let Some(folder_name) = folder {
                        args.push(folder_name.clone());
                    }
                }
            }
        }
        Commands::Config { action } => {
            command_str = "config".to_string();
            match action {
                ConfigActions::Show { key } => {
                    args.push("show".to_string());
                    if let Some(key_name) = key {
                        args.push(key_name.clone());
                    }
                }
                ConfigActions::Set { key, value } => {
                    args.push("set".to_string());
                    args.push(key.clone());
                    args.push(value.clone());
                }
            }
        }
        Commands::Contact { action } => {
            command_str = "contacts".to_string();
            match action {
                ContactActions::Create { group_name, emails } => {
                    args.push("create".to_string());
                    args.push(group_name.clone());
                    args.extend(emails.iter().cloned());
                }
                ContactActions::List => {
                    args.push("list".to_string());
                }
                ContactActions::Show { group_name } => {
                    args.push("show".to_string());
                    args.push(group_name.clone());
                }
            }
        }
        Commands::Utility { action } => {
            command_str = "utility".to_string();
            match action {
                UtilityActions::Date => {
                    args.push("date".to_string());
                }
                UtilityActions::Time => {
                    args.push("time".to_string());
                }
                UtilityActions::DateTime => {
                    args.push("datetime".to_string());
                }
            }
        }
    }

    Some(CommandArgs { command: command_str, args, flags })
}
