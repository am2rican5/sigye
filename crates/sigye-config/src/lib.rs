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

    /// Whether to show seconds in the clock display.
    #[serde(default = "default_show_seconds")]
    pub show_seconds: bool,

    /// Whether Clock mode hides all secondary UI chrome.
    #[serde(default)]
    pub minimal_mode: bool,

    /// Pomodoro work duration in minutes.
    #[serde(default = "default_pomodoro_work_mins")]
    pub pomodoro_work_mins: u32,

    /// Pomodoro short break duration in minutes.
    #[serde(default = "default_pomodoro_break_mins")]
    pub pomodoro_break_mins: u32,

    /// Pomodoro long break duration in minutes.
    #[serde(default = "default_pomodoro_long_break_mins")]
    pub pomodoro_long_break_mins: u32,

    /// Number of work sessions before a long break.
    #[serde(default = "default_pomodoro_sessions_until_long")]
    pub pomodoro_sessions_until_long: u32,

    /// Whether to play terminal bell on pomodoro phase transitions.
    #[serde(default = "default_pomodoro_sound")]
    pub pomodoro_sound: bool,

    /// Timer countdown duration in minutes.
    #[serde(default = "default_timer_duration_mins")]
    pub timer_duration_mins: u32,

    /// Whether to show desktop notifications on timer events.
    #[serde(default = "default_desktop_notifications")]
    pub desktop_notifications: bool,

    /// List of timezone entries for world clock mode.
    /// Each entry is "Label=Timezone" (e.g., "Tokyo=Asia/Tokyo").
    #[serde(default = "default_world_clock_zones")]
    pub world_clock_zones: Vec<String>,

    /// Shell command to execute on timer/pomodoro completion.
    #[serde(default)]
    pub on_complete: Option<String>,

    /// Number of completed pomodoro work sessions (persisted across runs).
    #[serde(default)]
    pub pomodoro_sessions_completed: u32,

    /// Total focus minutes accumulated across pomodoro sessions (persisted).
    #[serde(default)]
    pub pomodoro_total_focus_mins: u32,

    /// Shell command to execute when a pomodoro work phase starts.
    #[serde(default)]
    pub on_start: Option<String>,

    /// Shell command to execute when a pomodoro break phase starts.
    #[serde(default)]
    pub on_break: Option<String>,

    /// Life countdown events (e.g., birthdays, launches, anniversaries).
    #[serde(default)]
    pub countdown_events: Vec<CountdownEvent>,
}

/// A single life countdown event.
///
/// `target` is parsed by the countdown mode as either:
/// - An ISO 8601 date (`YYYY-MM-DD`), interpreted as midnight local time, or
/// - An ISO 8601 datetime (`YYYY-MM-DDTHH:MM:SS`) in local time, or
/// - An ISO 8601 datetime with offset (`YYYY-MM-DDTHH:MM:SS+09:00`).
///
/// When `since = false` (default) the event counts **down** to the target.
/// When `since = true` the event counts **up** from the target (e.g., anniversaries, sobriety).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownEvent {
    /// Display label (e.g., "Launch", "Birthday", "Sober").
    pub name: String,
    /// Target date/time as an ISO 8601 string.
    pub target: String,
    /// If true, count up from `target` instead of down to it.
    #[serde(default)]
    pub since: bool,
}

fn default_font() -> String {
    "Standard".to_string()
}

fn default_show_seconds() -> bool {
    true
}

fn default_pomodoro_work_mins() -> u32 {
    25
}

fn default_pomodoro_break_mins() -> u32 {
    5
}

fn default_pomodoro_long_break_mins() -> u32 {
    15
}

fn default_pomodoro_sessions_until_long() -> u32 {
    4
}

fn default_pomodoro_sound() -> bool {
    true
}

fn default_timer_duration_mins() -> u32 {
    5
}

fn default_desktop_notifications() -> bool {
    true
}

fn default_world_clock_zones() -> Vec<String> {
    vec![
        "New York=America/New_York".to_string(),
        "London=Europe/London".to_string(),
        "Tokyo=Asia/Tokyo".to_string(),
    ]
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
            show_seconds: default_show_seconds(),
            minimal_mode: false,
            pomodoro_work_mins: default_pomodoro_work_mins(),
            pomodoro_break_mins: default_pomodoro_break_mins(),
            pomodoro_long_break_mins: default_pomodoro_long_break_mins(),
            pomodoro_sessions_until_long: default_pomodoro_sessions_until_long(),
            pomodoro_sound: default_pomodoro_sound(),
            timer_duration_mins: default_timer_duration_mins(),
            desktop_notifications: default_desktop_notifications(),
            world_clock_zones: default_world_clock_zones(),
            on_complete: None,
            pomodoro_sessions_completed: 0,
            pomodoro_total_focus_mins: 0,
            on_start: None,
            on_break: None,
            countdown_events: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_minimal_mode_defaults_to_false() {
        let config: Config = toml::from_str("").unwrap();

        assert!(!config.minimal_mode);
    }

    #[test]
    fn minimal_mode_round_trips() {
        let config = Config {
            minimal_mode: true,
            ..Config::default()
        };

        let encoded = toml::to_string(&config).unwrap();
        let decoded: Config = toml::from_str(&encoded).unwrap();

        assert!(decoded.minimal_mode);
    }
}
