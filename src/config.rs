use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::Result;

/// RGB color values
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Theme color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    /// Muted/secondary text color
    pub muted: Rgb,
    /// Success indicators (checkmarks, completed counts)
    pub success: Rgb,
    /// Warning indicators (in-progress, medium priority)
    pub warning: Rgb,
    /// Error/high priority indicators
    pub error: Rgb,
    /// Info indicators (notes, in-progress counts)
    pub info: Rgb,
    /// Pending task indicators
    pub pending: Rgb,
    /// Starred item indicator
    pub starred: Rgb,
}

impl Default for ThemeColors {
    fn default() -> Self {
        // Default theme - readable on most terminals
        Self {
            muted: Rgb::new(140, 140, 140),
            success: Rgb::new(134, 239, 172),
            warning: Rgb::new(253, 224, 71),
            error: Rgb::new(252, 129, 129),
            info: Rgb::new(147, 197, 253),
            pending: Rgb::new(216, 180, 254),
            starred: Rgb::new(253, 224, 71),
        }
    }
}

impl ThemeColors {
    /// Catppuccin Macchiato theme
    pub fn catppuccin_macchiato() -> Self {
        Self {
            muted: Rgb::new(165, 173, 203),    // Subtext0
            success: Rgb::new(166, 218, 149),  // Green
            warning: Rgb::new(238, 212, 159),  // Yellow
            error: Rgb::new(237, 135, 150),    // Red
            info: Rgb::new(138, 173, 244),     // Blue
            pending: Rgb::new(198, 160, 246),  // Mauve
            starred: Rgb::new(238, 212, 159),  // Yellow
        }
    }

    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            muted: Rgb::new(166, 173, 200),    // Subtext0
            success: Rgb::new(166, 227, 161),  // Green
            warning: Rgb::new(249, 226, 175),  // Yellow
            error: Rgb::new(243, 139, 168),    // Red
            info: Rgb::new(137, 180, 250),     // Blue
            pending: Rgb::new(203, 166, 247),  // Mauve
            starred: Rgb::new(249, 226, 175),  // Yellow
        }
    }

    /// Catppuccin Frappe theme
    pub fn catppuccin_frappe() -> Self {
        Self {
            muted: Rgb::new(165, 173, 206),    // Subtext0
            success: Rgb::new(166, 209, 137),  // Green
            warning: Rgb::new(229, 200, 144),  // Yellow
            error: Rgb::new(231, 130, 132),    // Red
            info: Rgb::new(140, 170, 238),     // Blue
            pending: Rgb::new(202, 158, 230),  // Mauve
            starred: Rgb::new(229, 200, 144),  // Yellow
        }
    }

    /// Catppuccin Latte theme (light theme)
    pub fn catppuccin_latte() -> Self {
        Self {
            muted: Rgb::new(108, 111, 133),    // Subtext0
            success: Rgb::new(64, 160, 43),    // Green
            warning: Rgb::new(223, 142, 29),   // Yellow
            error: Rgb::new(210, 15, 57),      // Red
            info: Rgb::new(30, 102, 245),      // Blue
            pending: Rgb::new(136, 57, 239),   // Mauve
            starred: Rgb::new(223, 142, 29),   // Yellow
        }
    }

    /// High contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            muted: Rgb::new(200, 200, 200),
            success: Rgb::new(0, 255, 0),
            warning: Rgb::new(255, 255, 0),
            error: Rgb::new(255, 0, 0),
            info: Rgb::new(0, 255, 255),
            pending: Rgb::new(255, 0, 255),
            starred: Rgb::new(255, 255, 0),
        }
    }

    /// Get theme by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace(['-', '_', ' '], "") {
            s if s == "default" => Some(Self::default()),
            s if s == "catppuccinmacchiato" => Some(Self::catppuccin_macchiato()),
            s if s == "catppuccinmocha" => Some(Self::catppuccin_mocha()),
            s if s == "catppuccinfrappe" => Some(Self::catppuccin_frappe()),
            s if s == "catppuccinlatte" => Some(Self::catppuccin_latte()),
            s if s == "highcontrast" => Some(Self::high_contrast()),
            _ => None,
        }
    }
}

/// Theme configuration - either a preset name or custom colors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThemeConfig {
    /// Preset theme name
    Preset(String),
    /// Custom color configuration
    Custom(ThemeColors),
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig::Preset("default".to_string())
    }
}

impl ThemeConfig {
    /// Resolve to actual theme colors
    pub fn resolve(&self) -> ThemeColors {
        match self {
            ThemeConfig::Preset(name) => {
                ThemeColors::from_name(name).unwrap_or_default()
            }
            ThemeConfig::Custom(colors) => colors.clone(),
        }
    }
}

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

    #[serde(default)]
    pub theme: ThemeConfig,
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
            theme: ThemeConfig::default(),
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
    #[allow(dead_code)]
    pub fn get_taskbook_directory(&self) -> PathBuf {
        Self::format_taskbook_dir(&self.taskbook_directory)
    }
}
