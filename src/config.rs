use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::Result;

/// Configuration settings for taskbook
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_taskbook_directory")]
    pub taskbook_directory: String,

    #[serde(default = "default_true")]
    pub display_complete_tasks: bool,

    #[serde(default = "default_true")]
    pub display_progress_overview: bool,
}

fn default_taskbook_directory() -> String {
    "~".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            taskbook_directory: default_taskbook_directory(),
            display_complete_tasks: true,
            display_progress_overview: true,
        }
    }
}

impl Config {
    /// Get the config file path (~/.taskbook.json)
    fn config_file_path() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".taskbook.json")
    }

    /// Ensure the config file exists, creating it with defaults if not
    fn ensure_config_file() -> Result<()> {
        let config_path = Self::config_file_path();
        if !config_path.exists() {
            let default_config = Config::default();
            let data = serde_json::to_string_pretty(&default_config)?;
            fs::write(&config_path, data)?;
        }
        Ok(())
    }

    /// Format a taskbook directory path, expanding ~ to home directory
    fn format_taskbook_dir(path: &str) -> PathBuf {
        if path.starts_with('~') {
            let home = dirs::home_dir().expect("Could not find home directory");
            let rest = path.trim_start_matches('~').trim_start_matches('/');
            if rest.is_empty() {
                home
            } else {
                home.join(rest)
            }
        } else {
            PathBuf::from(path)
        }
    }

    /// Load configuration from file, merging with defaults
    pub fn load() -> Result<Self> {
        Self::ensure_config_file()?;

        let config_path = Self::config_file_path();
        let content = fs::read_to_string(&config_path)?;
        let mut config: Config = serde_json::from_str(&content)?;

        // Expand ~ in taskbook_directory
        if config.taskbook_directory.starts_with('~') {
            config.taskbook_directory = Self::format_taskbook_dir(&config.taskbook_directory)
                .to_string_lossy()
                .to_string();
        }

        Ok(config)
    }

    /// Get the resolved taskbook directory path
    pub fn get_taskbook_directory(&self) -> PathBuf {
        Self::format_taskbook_dir(&self.taskbook_directory)
    }
}
