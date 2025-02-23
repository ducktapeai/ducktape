use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CalendarConfig {
    pub default_calendar: Option<String>,
    pub default_reminder_minutes: Option<i32>,
    pub default_duration_minutes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TodoConfig {
    pub default_list: Option<String>,
    pub default_reminder: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NotesConfig {
    pub default_folder: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LanguageModelConfig {
    pub provider: LLMProvider,
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
            notes: NotesConfig {
                default_folder: None,
            },
            language_model: LanguageModelConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        // If config doesn't exist, create default
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }

        // Read and parse config file
        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config file")
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
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(
            config.calendar.default_calendar,
            Some("Calendar".to_string())
        );
        assert_eq!(config.calendar.default_reminder_minutes, Some(15));
        assert_eq!(config.todo.default_list, Some("Reminders".to_string()));
        assert!(matches!(config.language_model.provider, LLMProvider::OpenAI));
    }

    #[test]
    fn test_config_save_load() -> Result<()> {
        // Create temporary directory
        let temp_dir = tempdir()?;
        let _config_path = temp_dir.path().join("config.toml");

        // Set up temporary config directory
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        // Create and save config
        let config = Config::default();
        config.save()?;

        // Load config
        let loaded = Config::load()?;

        // Verify loaded config matches saved config
        assert_eq!(
            loaded.calendar.default_calendar,
            config.calendar.default_calendar
        );

        Ok(())
    }
}
