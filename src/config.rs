use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub calendar: CalendarConfig,
    #[serde(default)]
    pub todo: TodoConfig,
    #[serde(default)]
    pub notes: NotesConfig,
    #[serde(default)]
    pub language_model: LanguageModelConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CalendarConfig {
    pub default_calendar: Option<String>,
    pub default_reminder_minutes: Option<i32>,
    pub default_duration_minutes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TodoConfig {
    pub default_list: Option<String>,
    pub default_reminder: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NotesConfig {
    pub default_folder: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LLMProvider {
    OpenAI,
    Grok,
    DeepSeek,
}

impl Default for LLMProvider {
    fn default() -> Self {
        LLMProvider::OpenAI
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LanguageModelConfig {
    pub provider: Option<LLMProvider>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            calendar: CalendarConfig {
                default_calendar: Some("Calendar".to_string()),
                default_reminder_minutes: Some(15),
                default_duration_minutes: Some(60),
            },
            todo: TodoConfig {
                default_list: Some("Reminders".to_string()),
                default_reminder: true,
            },
            notes: NotesConfig { default_folder: None },
            language_model: LanguageModelConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        log::info!("Using configuration file at: {:?}", config_path);

        // If config doesn't exist, create default
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            log::info!("Default config created at: {:?}", config_path);
            return Ok(default_config);
        }

        // Read and parse config file
        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
        let mut config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        // Handle missing provider field explicitly
        if config.language_model.provider.is_none() {
            log::info!("Defaulting to Terminal Mode as no provider is set");
        }

        // Debug log the loaded configuration
        log::debug!("Loaded configuration: {:?}", config);

        // Check if the provider field is set to a valid value
        if let Some(provider) = &config.language_model.provider {
            match provider {
                LLMProvider::OpenAI | LLMProvider::Grok | LLMProvider::DeepSeek => {
                    log::info!("Natural Language Mode detected: provider is {:?}", provider);
                }
            }
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize and save config
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }
}

fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "ducktape", "ducktape")
        .context("Failed to determine config directory")?;

    Ok(proj_dirs.config_dir().join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.calendar.default_calendar, Some("Calendar".to_string()));
        assert_eq!(config.calendar.default_reminder_minutes, Some(15));
        assert_eq!(config.todo.default_list, Some("Reminders".to_string()));
        assert!(matches!(config.language_model.provider, None));
    }

    #[test]
    fn test_config_save_load() -> Result<()> {
        // Create a test config directly
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.toml");

        // Create the test config
        let test_config = Config {
            calendar: CalendarConfig {
                default_calendar: Some("TestCalendar".to_string()),
                default_reminder_minutes: Some(30),
                default_duration_minutes: Some(45),
            },
            todo: TodoConfig {
                default_list: Some("TestList".to_string()),
                default_reminder: false,
            },
            notes: NotesConfig { default_folder: Some("TestFolder".to_string()) },
            language_model: LanguageModelConfig { provider: Some(LLMProvider::Grok) },
        };

        // Serialize and write directly to file
        let content = toml::to_string_pretty(&test_config)?;
        fs::write(&config_path, content)?;

        // Read it back using TOML parser
        let file_content = fs::read_to_string(&config_path)?;
        let loaded_config: Config = toml::from_str(&file_content)?;

        // Verify the loaded config matches what we saved
        assert_eq!(loaded_config.calendar.default_calendar, test_config.calendar.default_calendar);
        assert_eq!(
            loaded_config.calendar.default_reminder_minutes,
            test_config.calendar.default_reminder_minutes
        );
        assert_eq!(
            loaded_config.calendar.default_duration_minutes,
            test_config.calendar.default_duration_minutes
        );
        assert_eq!(loaded_config.todo.default_list, test_config.todo.default_list);
        assert_eq!(loaded_config.todo.default_reminder, test_config.todo.default_reminder);
        assert_eq!(loaded_config.notes.default_folder, test_config.notes.default_folder);

        // Test that different LLM providers are correctly serialized/deserialized
        assert!(matches!(loaded_config.language_model.provider, Some(LLMProvider::Grok)));

        Ok(())
    }
}
