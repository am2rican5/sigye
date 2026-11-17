//! Shared rendering context passed to display modes.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

use ratatui::style::Color;
use sigye_config::Config;
use sigye_core::{AnimationSpeed, AnimationStyle, BackgroundStyle, ColorTheme, TimeFormat};
use sigye_fonts::FontRegistry;

/// Shared state that all display modes need for rendering.
pub struct RenderContext {
    pub time_format: TimeFormat,
    pub color_theme: ColorTheme,
    pub animation_style: AnimationStyle,
    pub animation_speed: AnimationSpeed,
    pub colon_blink: bool,
    pub show_seconds: bool,
    pub background_style: BackgroundStyle,
    pub current_font: String,
    pub font_registry: FontRegistry,
    pub config: Config,
    /// Where `save_config` writes `config`.
    pub config_path: PathBuf,
    pub animation_start: Instant,
    pub flash_intensity: f32,
    pub flash_start: Option<Instant>,
    pub screensaver_mode: bool,
    pub on_complete_command: Option<String>,
    pub sunrise_sunset: Option<(String, String)>,
}

impl RenderContext {
    /// Get the static color for the current theme.
    pub fn color(&self) -> Color {
        self.color_theme.color()
    }

    /// Get the dimmed theme color for secondary text.
    pub fn dim_color(&self) -> Color {
        self.color_theme.secondary_color()
    }

    /// Get the muted theme color for slightly brighter secondary text.
    pub fn muted_color(&self) -> Color {
        self.color_theme.muted_color()
    }

    /// Get animation elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.animation_start.elapsed().as_millis() as u64
    }

    /// Trigger a flash effect.
    pub fn trigger_flash(&mut self, intensity: f32) {
        self.flash_intensity = intensity;
        self.flash_start = Some(Instant::now());
    }

    /// Decay flash over time. Call once per frame.
    pub fn decay_flash(&mut self) {
        if let Some(flash_start) = self.flash_start {
            let decay_ms = self.animation_speed.flash_decay_ms();
            let flash_elapsed = flash_start.elapsed().as_millis() as f32;
            let decay_progress = (flash_elapsed / decay_ms as f32).min(1.0);
            self.flash_intensity *= 1.0 - decay_progress;

            if self.flash_intensity < 0.01 {
                self.flash_intensity = 0.0;
                self.flash_start = None;
            }
        }
    }

    /// Save `config` to `config_path`.
    pub fn save_config(&self) -> Result<(), sigye_config::ConfigError> {
        self.config.save_to(&self.config_path)
    }

    /// Ring terminal bell if sound is enabled.
    pub fn ring_bell(&self) {
        if self.config.pomodoro_sound {
            print!("\x07");
        }
    }

    /// Run the on-complete shell command in a background thread.
    pub fn run_on_complete(&self) {
        self.run_command(&self.on_complete_command);
    }

    /// Run an arbitrary shell command in a background thread (for lifecycle hooks).
    pub fn run_command(&self, cmd: &Option<String>) {
        if let Some(cmd) = cmd {
            let mut command = hook_command(cmd);
            // Wait for the command so the finished child is reaped, not left a zombie.
            std::thread::spawn(move || command.status());
        }
    }

    /// Send a desktop notification if enabled.
    pub fn send_notification(&self, title: &str, body: &str) {
        if self.config.desktop_notifications {
            let _ = notify_rust::Notification::new()
                .summary(title)
                .body(body)
                .show();
        }
    }

    /// A context built from the default config, with bell and desktop notifications
    /// off so tests can run timers to completion quietly, and saves going to a
    /// temporary file instead of the user's config.
    #[cfg(test)]
    pub fn for_test() -> Self {
        let config = Config {
            pomodoro_sound: false,
            desktop_notifications: false,
            ..Config::default()
        };
        Self {
            time_format: config.time_format,
            color_theme: config.color_theme,
            animation_style: config.animation_style,
            animation_speed: config.animation_speed,
            colon_blink: config.colon_blink,
            show_seconds: config.show_seconds,
            background_style: config.background_style,
            current_font: config.font_name.clone(),
            font_registry: FontRegistry::new(),
            on_complete_command: None,
            config,
            config_path: test_config_path(),
            animation_start: Instant::now(),
            flash_intensity: 0.0,
            flash_start: None,
            screensaver_mode: false,
            sunrise_sunset: None,
        }
    }
}

/// Build the shell command for a hook. Its output is discarded: anything it printed
/// would land on top of the TUI.
fn hook_command(cmd: &str) -> Command {
    #[cfg(windows)]
    let mut command = {
        use std::os::windows::process::CommandExt;
        // `arg` would quote the string again and mangle any quotes inside it;
        // `cmd /C` expects the hook verbatim.
        let mut command = Command::new("cmd");
        command.arg("/C").raw_arg(cmd);
        command
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut command = Command::new("sh");
        command.arg("-c").arg(cmd);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn hooks_run_in_a_shell_with_output_discarded() {
        let out = std::env::temp_dir().join(format!("sigye-hook-{}", std::process::id()));
        let status = hook_command(&format!("echo hidden; echo kept > '{}'", out.display()))
            .output()
            .unwrap();

        assert!(status.status.success());
        assert!(status.stdout.is_empty());
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "kept\n");
        std::fs::remove_file(&out).unwrap();
    }
}

/// A scratch path for config saves made by tests. Tests only ever write it, so
/// concurrent runs sharing it are harmless.
#[cfg(test)]
pub fn test_config_path() -> PathBuf {
    std::env::temp_dir().join("sigye-test-config.toml")
}
