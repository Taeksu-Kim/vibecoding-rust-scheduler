use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default time block size in minutes
    #[serde(default = "default_time_block")]
    pub default_time_block: u32,

    /// Color theme
    #[serde(default = "default_theme")]
    pub theme: Theme,

    /// Notification settings
    #[serde(default)]
    pub notifications: NotificationSettings,

    /// Daemon settings
    #[serde(default)]
    pub daemon: DaemonSettings,
}

fn default_time_block() -> u32 {
    30
}

fn default_theme() -> Theme {
    Theme::Green
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Green,
    Blue,
    Purple,
    Cyan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    #[serde(default = "default_true")]
    pub task_start_reminder: bool,

    #[serde(default = "default_true")]
    pub task_end_reminder: bool,

    #[serde(default = "default_reminder_minutes")]
    pub reminder_minutes: u32,
}

fn default_true() -> bool {
    true
}

fn default_reminder_minutes() -> u32 {
    5
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            task_start_reminder: true,
            task_end_reminder: true,
            reminder_minutes: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    #[serde(default = "default_update_interval")]
    pub update_interval_seconds: u64,

    #[serde(default = "default_true")]
    pub auto_start: bool,
}

fn default_update_interval() -> u64 {
    60
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            update_interval_seconds: 60,
            auto_start: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_time_block: 30,
            theme: Theme::Green,
            notifications: NotificationSettings::default(),
            daemon: DaemonSettings::default(),
        }
    }
}

impl Config {
    /// Get config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("scheduler")
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                .join(".config")
                .join("scheduler")
        };

        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Get theme color
    pub fn theme_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.theme {
            Theme::Green => Color::Green,
            Theme::Blue => Color::Blue,
            Theme::Purple => Color::Magenta,
            Theme::Cyan => Color::Cyan,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_time_block, 30);
        assert!(config.notifications.task_start_reminder);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml).unwrap();
        assert_eq!(deserialized.default_time_block, config.default_time_block);
    }
}
