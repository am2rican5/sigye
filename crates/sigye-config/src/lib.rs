//! Configuration management for the sigye clock application.

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use sigye_core::{AnimationSpeed, AnimationStyle, BackgroundStyle, ColorTheme, TimeFormat};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Current font name.
    #[serde(default = "default_font")]
    pub font_name: String,

    /// Color theme.
    #[serde(default)]
    pub color_theme: ColorTheme,

    /// Time format (12h or 24h).
    #[serde(default)]
    pub time_format: TimeFormat,

    /// Animation style.
    #[serde(default)]
    pub animation_style: AnimationStyle,

    /// Animation speed.
    #[serde(default)]
    pub animation_speed: AnimationSpeed,

    /// Whether colon blinks.
    #[serde(default)]
    pub colon_blink: bool,

    /// Background animation style.
    #[serde(default)]
    pub background_style: BackgroundStyle,

    /// Weather location for dynamic weather background (empty = auto-detect via IP).
    #[serde(default)]
    pub weather_location: String,
}

fn default_font() -> String {
    "Standard".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_name: default_font(),
            color_theme: ColorTheme::default(),
            time_format: TimeFormat::default(),
            animation_style: AnimationStyle::default(),
            animation_speed: AnimationSpeed::default(),
            colon_blink: false,
            background_style: BackgroundStyle::default(),
            weather_location: String::new(),
        }
    }
}

impl Config {
    /// Load configuration from file, or return defaults if not found.
    pub fn load() -> Self {
        let config_path = Self::config_file_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config file: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read config file: {e}");
                }
            }
        }

        Self::default()
    }

    /// Save configuration to file.
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir).map_err(|e| ConfigError::Io(e.to_string()))?;

        let config_path = Self::config_file_path();
        let contents =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Serialize(e.to_string()))?;

        fs::write(&config_path, contents).map_err(|e| ConfigError::Io(e.to_string()))?;

        Ok(())
    }

    /// Get the configuration directory path.
    pub fn config_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "sigye", "sigye") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            // Fallback to home directory
            dirs_fallback().join(".config").join("sigye")
        }
    }

    /// Get the configuration file path.
    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Get the custom fonts directory path.
    pub fn fonts_dir() -> PathBuf {
        Self::config_dir().join("fonts")
    }
}

/// Fallback to get home directory if ProjectDirs fails.
fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Configuration error types.
#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Serialize(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(msg) => write!(f, "IO error: {msg}"),
            ConfigError::Serialize(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}
