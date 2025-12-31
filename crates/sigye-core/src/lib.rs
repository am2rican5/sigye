//! Core types for the sigye clock application.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// System resource metrics for reactive backgrounds.
///
/// All values are normalized to the range 0.0 - 1.0.
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// CPU usage as a percentage (0.0 - 1.0).
    pub cpu_usage: f32,
    /// Memory usage as a percentage (0.0 - 1.0).
    pub memory_usage: f32,
    /// Network receive rate, normalized (0.0 - 1.0).
    pub network_rx_rate: f32,
    /// Network transmit rate, normalized (0.0 - 1.0).
    pub network_tx_rate: f32,
    /// Disk read rate, normalized (0.0 - 1.0).
    pub disk_read_rate: f32,
    /// Disk write rate, normalized (0.0 - 1.0).
    pub disk_write_rate: f32,
    /// Battery level (0.0 - 1.0), None if no battery.
    pub battery_level: Option<f32>,
    /// Whether battery is charging, None if no battery.
    pub battery_charging: Option<bool>,
}

/// Time of day for weather-aware rendering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TimeOfDay {
    #[default]
    Day,
    Night,
    /// Civil twilight before sunrise (~30 min).
    Dawn,
    /// Civil twilight after sunset (~30 min).
    Dusk,
}

/// Time format for the clock display.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeFormat {
    #[default]
    TwentyFourHour,
    TwelveHour,
}

impl TimeFormat {
    /// Toggle between 12-hour and 24-hour format.
    pub fn toggle(&self) -> Self {
        match self {
            TimeFormat::TwentyFourHour => TimeFormat::TwelveHour,
            TimeFormat::TwelveHour => TimeFormat::TwentyFourHour,
        }
    }
}

/// Animation style for color themes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationStyle {
    #[default]
    None,
    Shifting,
    Pulsing,
    Wave,
    Reactive,
}

/// All animation styles for cycling.
const ALL_ANIMATION_STYLES: &[AnimationStyle] = &[
    AnimationStyle::None,
    AnimationStyle::Shifting,
    AnimationStyle::Pulsing,
    AnimationStyle::Wave,
    AnimationStyle::Reactive,
];

impl AnimationStyle {
    /// Cycle to the next animation style.
    pub fn next(&self) -> Self {
        let current_idx = ALL_ANIMATION_STYLES
            .iter()
            .position(|s| s == self)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % ALL_ANIMATION_STYLES.len();
        ALL_ANIMATION_STYLES[next_idx]
    }

    /// Cycle to the previous animation style.
    pub fn prev(&self) -> Self {
        let current_idx = ALL_ANIMATION_STYLES
            .iter()
            .position(|s| s == self)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            ALL_ANIMATION_STYLES.len() - 1
        } else {
            current_idx - 1
        };
        ALL_ANIMATION_STYLES[prev_idx]
    }

    /// Get display name for the animation style.
    pub fn display_name(self) -> &'static str {
        match self {
            AnimationStyle::None => "None",
            AnimationStyle::Shifting => "Shifting",
            AnimationStyle::Pulsing => "Pulsing",
            AnimationStyle::Wave => "Wave",
            AnimationStyle::Reactive => "Reactive",
        }
    }
}

/// Background animation style for the terminal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundStyle {
    #[default]
    None,
    Starfield,
    MatrixRain,
    GradientWave,
    // Winter theme backgrounds
    Snowfall,
    Frost,
    Aurora,
    // Weather theme backgrounds
    Sunny,
    Rainy,
    Stormy,
    Windy,
    Cloudy,
    Foggy,
    // Dynamic weather background based on real weather data
    Weather,
    // Twilight backgrounds for dawn/dusk
    TwilightDawn,
    TwilightDusk,
    // Reactive backgrounds that respond to system resource usage
    SystemPulse,
    ResourceWave,
    DataFlow,
    HeatMap,
}

/// All background styles for cycling.
const ALL_BACKGROUND_STYLES: &[BackgroundStyle] = &[
    BackgroundStyle::None,
    BackgroundStyle::Starfield,
    BackgroundStyle::MatrixRain,
    BackgroundStyle::GradientWave,
    BackgroundStyle::Snowfall,
    BackgroundStyle::Frost,
    BackgroundStyle::Aurora,
    BackgroundStyle::Sunny,
    BackgroundStyle::Rainy,
    BackgroundStyle::Stormy,
    BackgroundStyle::Windy,
    BackgroundStyle::Cloudy,
    BackgroundStyle::Foggy,
    BackgroundStyle::Weather,
    BackgroundStyle::TwilightDawn,
    BackgroundStyle::TwilightDusk,
    BackgroundStyle::SystemPulse,
    BackgroundStyle::ResourceWave,
    BackgroundStyle::DataFlow,
    BackgroundStyle::HeatMap,
];

impl BackgroundStyle {
    /// Cycle to the next background style.
    pub fn next(&self) -> Self {
        let current_idx = ALL_BACKGROUND_STYLES
            .iter()
            .position(|s| s == self)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % ALL_BACKGROUND_STYLES.len();
        ALL_BACKGROUND_STYLES[next_idx]
    }

    /// Cycle to the previous background style.
    pub fn prev(&self) -> Self {
        let current_idx = ALL_BACKGROUND_STYLES
            .iter()
            .position(|s| s == self)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            ALL_BACKGROUND_STYLES.len() - 1
        } else {
            current_idx - 1
        };
        ALL_BACKGROUND_STYLES[prev_idx]
    }

    /// Get display name for the background style.
    pub fn display_name(self) -> &'static str {
        match self {
            BackgroundStyle::None => "None",
            BackgroundStyle::Starfield => "Starfield",
            BackgroundStyle::MatrixRain => "Matrix",
            BackgroundStyle::GradientWave => "Gradient",
            BackgroundStyle::Snowfall => "Snowfall",
            BackgroundStyle::Frost => "Frost",
            BackgroundStyle::Aurora => "Aurora",
            BackgroundStyle::Sunny => "Sunny",
            BackgroundStyle::Rainy => "Rainy",
            BackgroundStyle::Stormy => "Stormy",
            BackgroundStyle::Windy => "Windy",
            BackgroundStyle::Cloudy => "Cloudy",
            BackgroundStyle::Foggy => "Foggy",
            BackgroundStyle::Weather => "Weather",
            BackgroundStyle::TwilightDawn => "Dawn",
            BackgroundStyle::TwilightDusk => "Dusk",
            BackgroundStyle::SystemPulse => "Sys Pulse",
            BackgroundStyle::ResourceWave => "Resource",
            BackgroundStyle::DataFlow => "Data Flow",
            BackgroundStyle::HeatMap => "Heat Map",
        }
    }

    /// Check if this background style requires system metrics (reactive).
    pub fn is_reactive(self) -> bool {
        matches!(
            self,
            BackgroundStyle::SystemPulse
                | BackgroundStyle::ResourceWave
                | BackgroundStyle::DataFlow
                | BackgroundStyle::HeatMap
        )
    }

    /// Check if this background style requires weather data.
    pub fn requires_weather(self) -> bool {
        matches!(self, BackgroundStyle::Weather)
    }
}

/// Animation speed setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationSpeed {
    Slow,
    #[default]
    Medium,
    Fast,
}

/// All animation speeds for cycling.
const ALL_ANIMATION_SPEEDS: &[AnimationSpeed] = &[
    AnimationSpeed::Slow,
    AnimationSpeed::Medium,
    AnimationSpeed::Fast,
];

impl AnimationSpeed {
    /// Cycle to the next speed.
    pub fn next(&self) -> Self {
        let current_idx = ALL_ANIMATION_SPEEDS
            .iter()
            .position(|s| s == self)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % ALL_ANIMATION_SPEEDS.len();
        ALL_ANIMATION_SPEEDS[next_idx]
    }

    /// Cycle to the previous speed.
    pub fn prev(&self) -> Self {
        let current_idx = ALL_ANIMATION_SPEEDS
            .iter()
            .position(|s| s == self)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            ALL_ANIMATION_SPEEDS.len() - 1
        } else {
            current_idx - 1
        };
        ALL_ANIMATION_SPEEDS[prev_idx]
    }

    /// Get display name for the speed.
    pub fn display_name(self) -> &'static str {
        match self {
            AnimationSpeed::Slow => "Slow",
            AnimationSpeed::Medium => "Medium",
            AnimationSpeed::Fast => "Fast",
        }
    }

    /// Get the cycle duration in milliseconds for shifting animation.
    pub fn shift_cycle_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 30_000,
            AnimationSpeed::Medium => 15_000,
            AnimationSpeed::Fast => 5_000,
        }
    }

    /// Get the pulse period in milliseconds.
    pub fn pulse_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 3_000,
            AnimationSpeed::Medium => 1_500,
            AnimationSpeed::Fast => 750,
        }
    }

    /// Get the wave period in milliseconds.
    pub fn wave_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 4_000,
            AnimationSpeed::Medium => 2_000,
            AnimationSpeed::Fast => 1_000,
        }
    }

    /// Get the flash decay duration in milliseconds for reactive animation.
    pub fn flash_decay_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 800,
            AnimationSpeed::Medium => 400,
            AnimationSpeed::Fast => 200,
        }
    }

    /// Get the star twinkle period in milliseconds.
    pub fn star_twinkle_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 500,
            AnimationSpeed::Medium => 300,
            AnimationSpeed::Fast => 150,
        }
    }

    /// Get the matrix rain fall speed multiplier.
    pub fn matrix_fall_speed(self) -> f32 {
        match self {
            AnimationSpeed::Slow => 0.5,
            AnimationSpeed::Medium => 1.0,
            AnimationSpeed::Fast => 2.0,
        }
    }

    /// Get the gradient scroll period in milliseconds.
    pub fn gradient_scroll_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 5000,
            AnimationSpeed::Medium => 3000,
            AnimationSpeed::Fast => 1500,
        }
    }

    /// Get the snowfall speed multiplier.
    pub fn snow_fall_speed(self) -> f32 {
        match self {
            AnimationSpeed::Slow => 0.3,
            AnimationSpeed::Medium => 0.6,
            AnimationSpeed::Fast => 1.0,
        }
    }

    /// Get the frost growth period in milliseconds.
    pub fn frost_growth_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 8000,
            AnimationSpeed::Medium => 5000,
            AnimationSpeed::Fast => 3000,
        }
    }

    /// Get the aurora wave period in milliseconds.
    pub fn aurora_wave_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 6000,
            AnimationSpeed::Medium => 4000,
            AnimationSpeed::Fast => 2000,
        }
    }

    // Weather animation timing methods

    /// Get the rain fall speed multiplier.
    pub fn rain_fall_speed(self) -> f32 {
        match self {
            AnimationSpeed::Slow => 0.8,
            AnimationSpeed::Medium => 1.5,
            AnimationSpeed::Fast => 2.5,
        }
    }

    /// Get the lightning flash interval range in milliseconds (min, max).
    pub fn lightning_interval_ms(self) -> (u64, u64) {
        match self {
            AnimationSpeed::Slow => (6000, 12000),
            AnimationSpeed::Medium => (4000, 8000),
            AnimationSpeed::Fast => (2000, 5000),
        }
    }

    /// Get the wind streak speed multiplier.
    pub fn wind_streak_speed(self) -> f32 {
        match self {
            AnimationSpeed::Slow => 0.5,
            AnimationSpeed::Medium => 1.0,
            AnimationSpeed::Fast => 2.0,
        }
    }

    /// Get the cloud drift period in milliseconds.
    pub fn cloud_drift_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 8000,
            AnimationSpeed::Medium => 5000,
            AnimationSpeed::Fast => 3000,
        }
    }

    /// Get the sun ray shimmer period in milliseconds.
    pub fn sun_shimmer_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 2000,
            AnimationSpeed::Medium => 1200,
            AnimationSpeed::Fast => 600,
        }
    }

    /// Get the fog pulse period in milliseconds.
    pub fn fog_pulse_period_ms(self) -> u64 {
        match self {
            AnimationSpeed::Slow => 6000,
            AnimationSpeed::Medium => 4000,
            AnimationSpeed::Fast => 2500,
        }
    }
}

/// Color theme for the clock display.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorTheme {
    #[default]
    Cyan,
    Green,
    White,
    Magenta,
    Yellow,
    Red,
    Blue,
    // Dynamic color themes
    Rainbow,
    RainbowVertical,
    GradientWarm,
    GradientCool,
    GradientOcean,
    GradientNeon,
    GradientFire,
    // Winter color themes
    GradientFrost,
    GradientAurora,
    GradientWinter,
}

/// All color themes in order for cycling.
const ALL_THEMES: &[ColorTheme] = &[
    ColorTheme::Cyan,
    ColorTheme::Green,
    ColorTheme::Magenta,
    ColorTheme::Yellow,
    ColorTheme::Red,
    ColorTheme::Blue,
    ColorTheme::White,
    ColorTheme::Rainbow,
    ColorTheme::RainbowVertical,
    ColorTheme::GradientWarm,
    ColorTheme::GradientCool,
    ColorTheme::GradientOcean,
    ColorTheme::GradientNeon,
    ColorTheme::GradientFire,
    ColorTheme::GradientFrost,
    ColorTheme::GradientAurora,
    ColorTheme::GradientWinter,
];

impl ColorTheme {
    /// Cycle to the next color theme.
    pub fn next(&self) -> Self {
        let current_idx = ALL_THEMES.iter().position(|t| t == self).unwrap_or(0);
        let next_idx = (current_idx + 1) % ALL_THEMES.len();
        ALL_THEMES[next_idx]
    }

    /// Cycle to the previous color theme.
    pub fn prev(&self) -> Self {
        let current_idx = ALL_THEMES.iter().position(|t| t == self).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            ALL_THEMES.len() - 1
        } else {
            current_idx - 1
        };
        ALL_THEMES[prev_idx]
    }

    /// Convert theme to Ratatui Color (for static themes).
    pub fn color(self) -> Color {
        match self {
            ColorTheme::Cyan => Color::Cyan,
            ColorTheme::Green => Color::Green,
            ColorTheme::White => Color::White,
            ColorTheme::Magenta => Color::Magenta,
            ColorTheme::Yellow => Color::Yellow,
            ColorTheme::Red => Color::Red,
            ColorTheme::Blue => Color::Blue,
            // Dynamic themes return a default color for backward compatibility
            ColorTheme::Rainbow | ColorTheme::RainbowVertical | ColorTheme::GradientNeon => {
                Color::Magenta
            }
            ColorTheme::GradientWarm | ColorTheme::GradientFire => Color::Red,
            ColorTheme::GradientCool | ColorTheme::GradientOcean => Color::Cyan,
            ColorTheme::GradientFrost | ColorTheme::GradientWinter => Color::Cyan,
            ColorTheme::GradientAurora => Color::Green,
        }
    }

    /// Check if this theme requires per-character coloring.
    pub fn is_dynamic(self) -> bool {
        matches!(
            self,
            ColorTheme::Rainbow
                | ColorTheme::RainbowVertical
                | ColorTheme::GradientWarm
                | ColorTheme::GradientCool
                | ColorTheme::GradientOcean
                | ColorTheme::GradientNeon
                | ColorTheme::GradientFire
                | ColorTheme::GradientFrost
                | ColorTheme::GradientAurora
                | ColorTheme::GradientWinter
        )
    }

    /// Get color at a specific position for dynamic themes.
    /// `x` is the horizontal position (column), `y` is the vertical position (row).
    /// `width` and `height` are the total dimensions for normalization.
    pub fn color_at_position(self, x: usize, y: usize, width: usize, height: usize) -> Color {
        match self {
            ColorTheme::Rainbow => {
                let colors = [
                    Color::Red,
                    Color::Rgb(255, 127, 0), // Orange
                    Color::Yellow,
                    Color::Green,
                    Color::Cyan,
                    Color::Blue,
                    Color::Magenta,
                ];
                let idx = if width > 0 {
                    (x * colors.len() / width.max(1)) % colors.len()
                } else {
                    0
                };
                colors[idx]
            }
            ColorTheme::RainbowVertical => {
                let colors = [
                    Color::Red,
                    Color::Rgb(255, 127, 0), // Orange
                    Color::Yellow,
                    Color::Green,
                    Color::Cyan,
                    Color::Blue,
                    Color::Magenta,
                ];
                let idx = if height > 0 {
                    (y * colors.len() / height.max(1)) % colors.len()
                } else {
                    0
                };
                colors[idx]
            }
            ColorTheme::GradientWarm => {
                // Red -> Orange -> Yellow
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.5 {
                    // Red to Orange
                    let g = (127.0 * (progress * 2.0)) as u8;
                    Color::Rgb(255, g, 0)
                } else {
                    // Orange to Yellow
                    let g = 127 + ((128.0 * ((progress - 0.5) * 2.0)) as u8);
                    Color::Rgb(255, g, 0)
                }
            }
            ColorTheme::GradientCool => {
                // Blue -> Cyan -> Green
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.5 {
                    // Blue to Cyan
                    let g = (255.0 * (progress * 2.0)) as u8;
                    Color::Rgb(0, g, 255)
                } else {
                    // Cyan to Green
                    let b = 255 - ((255.0 * ((progress - 0.5) * 2.0)) as u8);
                    Color::Rgb(0, 255, b)
                }
            }
            ColorTheme::GradientOcean => {
                // Dark blue -> Cyan -> Teal
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.5 {
                    // Dark blue to Cyan
                    let r = (100.0 * (progress * 2.0)) as u8;
                    let g = (150.0 + 105.0 * (progress * 2.0)) as u8;
                    Color::Rgb(r, g, 255)
                } else {
                    // Cyan to Teal
                    let b = 255 - ((127.0 * ((progress - 0.5) * 2.0)) as u8);
                    Color::Rgb(100, 255, b)
                }
            }
            ColorTheme::GradientNeon => {
                // Magenta -> Cyan (synthwave style)
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                let r = 255 - ((255.0 * progress) as u8);
                let g = (255.0 * progress) as u8;
                let b = 255;
                Color::Rgb(r, g, b)
            }
            ColorTheme::GradientFire => {
                // Dark red -> Red -> Orange -> Yellow (fire effect)
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.33 {
                    // Dark red to Red
                    let r = 128 + ((127.0 * (progress * 3.0)) as u8);
                    Color::Rgb(r, 0, 0)
                } else if progress < 0.66 {
                    // Red to Orange
                    let g = (165.0 * ((progress - 0.33) * 3.0)) as u8;
                    Color::Rgb(255, g, 0)
                } else {
                    // Orange to Yellow
                    let g = 165 + ((90.0 * ((progress - 0.66) * 3.0)) as u8);
                    Color::Rgb(255, g, 0)
                }
            }
            ColorTheme::GradientFrost => {
                // White -> Ice Blue -> Steel Blue
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.5 {
                    // White to Ice Blue
                    let t = progress * 2.0;
                    let r = 255 - ((255 - 176) as f32 * t) as u8;
                    let g = 255 - ((255 - 224) as f32 * t) as u8;
                    let b = 255 - ((255 - 230) as f32 * t) as u8;
                    Color::Rgb(r, g, b)
                } else {
                    // Ice Blue to Steel Blue
                    let t = (progress - 0.5) * 2.0;
                    let r = 176 - ((176 - 70) as f32 * t) as u8;
                    let g = 224 - ((224 - 130) as f32 * t) as u8;
                    let b = 230 - ((230 - 180) as f32 * t) as u8;
                    Color::Rgb(r, g, b)
                }
            }
            ColorTheme::GradientAurora => {
                // Green -> Cyan -> Blue -> Purple (aurora colors)
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.33 {
                    // Green to Cyan
                    let t = progress * 3.0;
                    let r = (0.0 + 0.0 * t) as u8;
                    let g = (255.0 - 128.0 * t) as u8;
                    let b = (127.0 + 128.0 * t) as u8;
                    Color::Rgb(r, g, b)
                } else if progress < 0.66 {
                    // Cyan to Blue
                    let t = (progress - 0.33) * 3.0;
                    let r = (0.0 + 65.0 * t) as u8;
                    let g = (127.0 - 22.0 * t) as u8;
                    let b = (255.0 - 30.0 * t) as u8;
                    Color::Rgb(r, g, b)
                } else {
                    // Blue to Purple
                    let t = (progress - 0.66) * 3.0;
                    let r = (65.0 + 73.0 * t) as u8;
                    let g = (105.0 - 62.0 * t) as u8;
                    let b = (225.0 + 1.0 * t) as u8;
                    Color::Rgb(r, g, b)
                }
            }
            ColorTheme::GradientWinter => {
                // Deep Blue -> Royal Blue -> Ice Blue
                let progress = if width > 0 {
                    (x as f32) / (width.max(1) as f32)
                } else {
                    0.0
                };
                if progress < 0.5 {
                    // Deep Blue to Royal Blue
                    let t = progress * 2.0;
                    let r = (25.0 + 40.0 * t) as u8;
                    let g = (25.0 + 80.0 * t) as u8;
                    let b = (112.0 + 113.0 * t) as u8;
                    Color::Rgb(r, g, b)
                } else {
                    // Royal Blue to Ice Blue
                    let t = (progress - 0.5) * 2.0;
                    let r = (65.0 + 70.0 * t) as u8;
                    let g = (105.0 + 101.0 * t) as u8;
                    let b = (225.0 + 25.0 * t) as u8;
                    Color::Rgb(r, g, b)
                }
            }
            // Static themes just return their color
            _ => self.color(),
        }
    }

    /// Get display name for the theme.
    pub fn display_name(self) -> &'static str {
        match self {
            ColorTheme::Cyan => "Cyan",
            ColorTheme::Green => "Green",
            ColorTheme::White => "White",
            ColorTheme::Magenta => "Magenta",
            ColorTheme::Yellow => "Yellow",
            ColorTheme::Red => "Red",
            ColorTheme::Blue => "Blue",
            ColorTheme::Rainbow => "Rainbow",
            ColorTheme::RainbowVertical => "Rainbow V",
            ColorTheme::GradientWarm => "Warm",
            ColorTheme::GradientCool => "Cool",
            ColorTheme::GradientOcean => "Ocean",
            ColorTheme::GradientNeon => "Neon",
            ColorTheme::GradientFire => "Fire",
            ColorTheme::GradientFrost => "Frost",
            ColorTheme::GradientAurora => "Aurora",
            ColorTheme::GradientWinter => "Winter",
        }
    }
}

/// Apply animation transformations to a color.
pub fn apply_animation(
    base_color: Color,
    animation_style: AnimationStyle,
    speed: AnimationSpeed,
    elapsed_ms: u64,
    x: usize,
    width: usize,
    flash_intensity: f32,
) -> Color {
    match animation_style {
        AnimationStyle::None => base_color,
        AnimationStyle::Shifting => apply_shifting(base_color, elapsed_ms, speed),
        AnimationStyle::Pulsing => apply_pulsing(base_color, elapsed_ms, speed),
        AnimationStyle::Wave => apply_wave(base_color, elapsed_ms, speed, x, width),
        AnimationStyle::Reactive => apply_reactive(base_color, flash_intensity),
    }
}

/// Shift hue over time.
fn apply_shifting(color: Color, elapsed_ms: u64, speed: AnimationSpeed) -> Color {
    let (r, g, b) = color_to_rgb(color);
    let (h, s, l) = rgb_to_hsl(r, g, b);

    let cycle_ms = speed.shift_cycle_ms();
    let hue_offset = ((elapsed_ms % cycle_ms) as f32 / cycle_ms as f32) * 360.0;
    let new_h = (h + hue_offset) % 360.0;

    let (nr, ng, nb) = hsl_to_rgb(new_h, s, l);
    Color::Rgb(nr, ng, nb)
}

/// Pulse brightness using sine wave.
fn apply_pulsing(color: Color, elapsed_ms: u64, speed: AnimationSpeed) -> Color {
    let (r, g, b) = color_to_rgb(color);

    let period_ms = speed.pulse_period_ms();
    let phase = (elapsed_ms % period_ms) as f32 / period_ms as f32;
    let brightness = 0.5 + 0.5 * (phase * 2.0 * std::f32::consts::PI).sin();

    // Apply brightness (minimum 30% to stay visible)
    let factor = 0.3 + 0.7 * brightness;
    Color::Rgb(
        (r as f32 * factor) as u8,
        (g as f32 * factor) as u8,
        (b as f32 * factor) as u8,
    )
}

/// Wave pattern flowing horizontally.
fn apply_wave(
    color: Color,
    elapsed_ms: u64,
    speed: AnimationSpeed,
    x: usize,
    width: usize,
) -> Color {
    let (r, g, b) = color_to_rgb(color);

    let period_ms = speed.wave_period_ms();
    let time_phase = (elapsed_ms % period_ms) as f32 / period_ms as f32;
    let x_phase = if width > 0 {
        x as f32 / width as f32
    } else {
        0.0
    };

    let wave = ((x_phase + time_phase) * 2.0 * std::f32::consts::PI).sin();
    let brightness = 0.6 + 0.4 * wave;

    Color::Rgb(
        (r as f32 * brightness) as u8,
        (g as f32 * brightness) as u8,
        (b as f32 * brightness) as u8,
    )
}

/// Apply flash intensity for reactive animation.
fn apply_reactive(color: Color, flash_intensity: f32) -> Color {
    let (r, g, b) = color_to_rgb(color);

    // Boost brightness based on flash intensity
    let factor = 1.0 + flash_intensity;
    Color::Rgb(
        (r as f32 * factor).min(255.0) as u8,
        (g as f32 * factor).min(255.0) as u8,
        (b as f32 * factor).min(255.0) as u8,
    )
}

/// Extract RGB values from a Color.
fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Yellow => (255, 255, 0),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::White => (255, 255, 255),
        _ => (128, 128, 128),
    }
}

/// Convert RGB to HSL.
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
    } else if max == g {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };

    (h, s, l)
}

/// Convert HSL to RGB.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let h = h / 360.0;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

/// Check if colon should be visible in the blink cycle.
/// Returns true during the "on" phase (first 500ms of each second).
pub fn is_colon_visible(elapsed_ms: u64) -> bool {
    let phase = (elapsed_ms % 1000) as f32 / 1000.0;
    phase < 0.5
}
