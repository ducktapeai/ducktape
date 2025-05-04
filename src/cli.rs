//! CLI module for parsing command line arguments
//!
//! This module defines the command-line interface for the DuckTape application
//! using the clap crate for argument parsing.

use crate::command_processor::CommandArgs;
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;
use std::path::PathBuf;

/// DuckTape - AI-powered terminal tool for Apple Calendar, Reminders and Notes
#[derive(Debug, Parser)]
#[command(name = "ducktape")]
#[command(about = "AI-powered terminal tool for Apple Calendar, Reminders and Notes", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
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

    /// Manage reminders
    #[command(alias = "todos", alias = "reminders")]
    Reminder {
        #[command(subcommand)]
        action: ReminderActions,
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

    /// Run a natural language command using AI
    Ai {
        /// The natural language command
        #[arg(required = true, num_args = 1.., value_delimiter = ' ', trailing_var_arg = true)]
        nl_command: Vec<String>,
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
        /// Check for calendar conflicts before creating the event
        #[arg(long)]
        check_conflicts: bool,
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
pub enum ReminderActions {
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
        #[arg(required = true, num_args = 1.., value_delimiter = ' ')]
        title: Vec<String>,

        /// Note content
        #[arg(long)]
        content: Option<String>,

        /// Folder name
        #[arg(long)]
        folder: Option<String>,
    },

    /// Search for notes
    #[command(alias = "find")]
    Search {
        /// Search query
        #[arg(required = true, num_args = 1.., value_delimiter = ' ')]
        query: Vec<String>,

        /// Folder name
        #[arg(long)]
        folder: Option<String>,
    },

    /// Delete a note
    #[command(alias = "remove")]
    Delete {
        /// Note title or ID
        #[arg(required = true, num_args = 1.., value_delimiter = ' ')]
        note_id: Vec<String>,

        /// Folder name
        #[arg(long)]
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

/// Convert a Cli object to CommandArgs for use with the command processor
///
/// This function extracts relevant information from the Cli struct and
/// converts it to a CommandArgs struct that can be used by the command processor.
pub fn convert_to_command_args(cli: &Cli) -> Option<CommandArgs> {
    match &cli.command {
        Some(cmd) => match cmd {
            Commands::Calendar { action } => {
                let mut args = Vec::new();
                let mut flags = HashMap::new();

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
                        check_conflicts,
                    } => {
                        args.push("create".to_string());
                        args.push(title.clone());
                        args.push(date.clone());
                        args.push(start_time.clone());
                        args.push(end_time.clone());
                        args.push(calendar.clone());

                        if let Some(loc) = location {
                            flags.insert("location".to_string(), Some(loc.clone()));
                        }
                        if let Some(n) = notes {
                            flags.insert("notes".to_string(), Some(n.clone()));
                        }
                        if *zoom {
                            flags.insert("zoom".to_string(), Some("true".to_string()));
                        }
                        if let Some(r) = repeat {
                            flags.insert("repeat".to_string(), Some(format!("{:?}", r)));
                        }
                        if let Some(i) = interval {
                            flags.insert("interval".to_string(), Some(i.to_string()));
                        }
                        if let Some(u) = until {
                            flags.insert("until".to_string(), Some(u.clone()));
                        }
                        if let Some(c) = count {
                            flags.insert("count".to_string(), Some(c.to_string()));
                        }
                        if let Some(d) = days {
                            let days_str = d
                                .iter()
                                .map(|&day| day.to_string())
                                .collect::<Vec<String>>()
                                .join(",");
                            flags.insert("days".to_string(), Some(days_str));
                        }
                        if let Some(c) = contacts {
                            let contacts_str = c.join(",");
                            flags.insert("contacts".to_string(), Some(contacts_str));
                        }
                        if let Some(e) = email {
                            let email_str = e.join(",");
                            flags.insert("email".to_string(), Some(email_str));
                        }
                        if *check_conflicts {
                            flags.insert("check_conflicts".to_string(), Some("true".to_string()));
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

                Some(CommandArgs { command: "calendar".to_string(), args, flags })
            }
            Commands::Reminder { action } => {
                let mut args = Vec::new();
                let mut flags = HashMap::new();

                match action {
                    ReminderActions::Lists => {
                        args.push("lists".to_string());
                    }
                    ReminderActions::List { list } => {
                        args.push("list".to_string());
                        if let Some(l) = list {
                            args.push(l.clone());
                        }
                    }
                    ReminderActions::Create { title, lists, remind, notes } => {
                        args.push("create".to_string());
                        args.push(title.clone());
                        for list in lists {
                            args.push(list.clone());
                        }
                        if let Some(r) = remind {
                            flags.insert("remind".to_string(), Some(r.clone()));
                        }
                        if let Some(n) = notes {
                            flags.insert("notes".to_string(), Some(n.clone()));
                        }
                    }
                    ReminderActions::Complete { reminder_id, list } => {
                        args.push("complete".to_string());
                        args.push(reminder_id.clone());
                        if let Some(l) = list {
                            args.push(l.clone());
                        }
                    }
                    ReminderActions::Delete { reminder_id, list } => {
                        args.push("delete".to_string());
                        args.push(reminder_id.clone());
                        if let Some(l) = list {
                            args.push(l.clone());
                        }
                    }
                    ReminderActions::SetList { list } => {
                        args.push("set-list".to_string());
                        args.push(list.clone());
                    }
                }

                Some(CommandArgs { command: "reminder".to_string(), args, flags })
            }
            Commands::Note { action } => {
                let mut args = Vec::new();
                let mut flags = HashMap::new();

                match action {
                    NoteActions::List { folder } => {
                        args.push("list".to_string());
                        if let Some(f) = folder {
                            args.push(f.clone());
                        }
                    }
                    NoteActions::Create { title, content, folder } => {
                        args.push("create".to_string());
                        let title_str = title.join(" ");
                        args.push(title_str);

                        if let Some(folder_val) = folder {
                            flags.insert("folder".to_string(), Some(folder_val.clone()));
                        }
                        if let Some(content_val) = content {
                            flags.insert("content".to_string(), Some(content_val.clone()));
                        }
                    }
                    NoteActions::Search { query, folder } => {
                        args.push("search".to_string());
                        let query_str = query.join(" ");
                        args.push(query_str);

                        if let Some(f) = folder {
                            flags.insert("folder".to_string(), Some(f.clone()));
                        }
                    }
                    NoteActions::Delete { note_id, folder } => {
                        args.push("delete".to_string());
                        let id_str = note_id.join(" ");
                        args.push(id_str);

                        if let Some(f) = folder {
                            flags.insert("folder".to_string(), Some(f.clone()));
                        }
                    }
                }

                Some(CommandArgs { command: "note".to_string(), args, flags })
            }
            Commands::Config { action } => {
                let mut args = Vec::new();
                let flags = HashMap::new();

                match action {
                    ConfigActions::Show { key } => {
                        args.push("show".to_string());
                        if let Some(k) = key {
                            args.push(k.clone());
                        }
                    }
                    ConfigActions::Set { key, value } => {
                        args.push("set".to_string());
                        args.push(key.clone());
                        args.push(value.clone());
                    }
                }

                Some(CommandArgs { command: "config".to_string(), args, flags })
            }
            Commands::Contact { action } => {
                let mut args = Vec::new();
                let flags = HashMap::new();

                match action {
                    ContactActions::List => {
                        args.push("list".to_string());
                    }
                    ContactActions::Create { group_name, emails } => {
                        args.push("create".to_string());
                        args.push(group_name.clone());
                        for email in emails {
                            args.push(email.clone());
                        }
                    }
                    ContactActions::Show { group_name } => {
                        args.push("show".to_string());
                        args.push(group_name.clone());
                    }
                }

                Some(CommandArgs { command: "contact".to_string(), args, flags })
            }
            Commands::Utility { action } => {
                let mut args = Vec::new();
                let flags = HashMap::new();

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

                Some(CommandArgs { command: "utility".to_string(), args, flags })
            }
            Commands::Ai { nl_command } => {
                let mut args = Vec::new();
                let flags = HashMap::new();

                args.push("ai".to_string());
                for cmd in nl_command {
                    args.push(cmd.clone());
                }

                Some(CommandArgs { command: "ai".to_string(), args, flags })
            }
        },
        None => {
            // No command specified, enter interactive mode
            None
        }
    }
}
