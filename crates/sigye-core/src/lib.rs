//! Core types for the sigye clock application.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

fn cycle_next<T: PartialEq + Copy>(all: &[T], cur: T) -> T {
    let idx = all.iter().position(|x| x == &cur).unwrap_or(0);
    all[(idx + 1) % all.len()]
}

fn cycle_prev<T: PartialEq + Copy>(all: &[T], cur: T) -> T {
    let idx = all.iter().position(|x| x == &cur).unwrap_or(0);
    all[if idx == 0 { all.len() - 1 } else { idx - 1 }]
}

/// Display mode for the application.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    #[default]
    Clock,
    Pomodoro,
    Timer,
    Stopwatch,
    WorldClock,
    Countdown,
}

/// All display modes for cycling.
const ALL_DISPLAY_MODES: &[DisplayMode] = &[
    DisplayMode::Clock,
    DisplayMode::Pomodoro,
    DisplayMode::Timer,
    DisplayMode::Stopwatch,
    DisplayMode::WorldClock,
    DisplayMode::Countdown,
];

impl DisplayMode {
    /// Cycle to the next display mode.
    pub fn next(&self) -> Self {
        cycle_next(ALL_DISPLAY_MODES, *self)
    }

    /// Get display name for the mode.
    pub fn display_name(self) -> &'static str {
        match self {
            DisplayMode::Clock => "Clock",
            DisplayMode::Pomodoro => "Pomodoro",
            DisplayMode::Timer => "Timer",
            DisplayMode::Stopwatch => "Stopwatch",
            DisplayMode::WorldClock => "World Clock",
            DisplayMode::Countdown => "Countdown",
        }
    }
}

/// Display format for the clock mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClockDisplayFormat {
    #[default]
    HumanReadable,
    UnixTimestamp,
    Iso8601,
    HexTime,
}

const ALL_CLOCK_DISPLAY_FORMATS: &[ClockDisplayFormat] = &[
    ClockDisplayFormat::HumanReadable,
    ClockDisplayFormat::UnixTimestamp,
    ClockDisplayFormat::Iso8601,
    ClockDisplayFormat::HexTime,
];

impl ClockDisplayFormat {
    pub fn next(&self) -> Self {
        cycle_next(ALL_CLOCK_DISPLAY_FORMATS, *self)
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ClockDisplayFormat::HumanReadable => "Clock",
            ClockDisplayFormat::UnixTimestamp => "Unix Timestamp",
            ClockDisplayFormat::Iso8601 => "ISO 8601",
            ClockDisplayFormat::HexTime => "Hex Time",
        }
    }
}

/// Pomodoro timer phase.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroPhase {
    #[default]
    Work,
    ShortBreak,
    LongBreak,
}

impl PomodoroPhase {
    /// Get display name for the phase.
    pub fn display_name(self) -> &'static str {
        match self {
            PomodoroPhase::Work => "WORK",
            PomodoroPhase::ShortBreak => "BREAK",
            PomodoroPhase::LongBreak => "LONG BREAK",
        }
    }

    /// Check if this is a break phase.
    pub fn is_break(self) -> bool {
        matches!(self, PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak)
    }
}

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
        cycle_next(ALL_ANIMATION_STYLES, *self)
    }

    /// Cycle to the previous animation style.
    pub fn prev(&self) -> Self {
        cycle_prev(ALL_ANIMATION_STYLES, *self)
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
    // Spring theme backgrounds
    CherryBlossom,
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
    BackgroundStyle::CherryBlossom,
    BackgroundStyle::SystemPulse,
    BackgroundStyle::ResourceWave,
    BackgroundStyle::DataFlow,
    BackgroundStyle::HeatMap,
];

impl BackgroundStyle {
    /// Cycle to the next background style.
    pub fn next(&self) -> Self {
        cycle_next(ALL_BACKGROUND_STYLES, *self)
    }

    /// Cycle to the previous background style.
    pub fn prev(&self) -> Self {
        cycle_prev(ALL_BACKGROUND_STYLES, *self)
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
            BackgroundStyle::CherryBlossom => "Sakura",
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
        cycle_next(ALL_ANIMATION_SPEEDS, *self)
    }

    /// Cycle to the previous speed.
    pub fn prev(&self) -> Self {
        cycle_prev(ALL_ANIMATION_SPEEDS, *self)
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

    /// Get the cherry blossom petal fall speed multiplier.
    pub fn petal_fall_speed(self) -> f32 {
        match self {
            AnimationSpeed::Slow => 0.3,
            AnimationSpeed::Medium => 0.6,
            AnimationSpeed::Fast => 1.0,
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
    // Spring color themes
    GradientSakura,
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
    ColorTheme::GradientSakura,
];

impl ColorTheme {
    /// Cycle to the next color theme.
    pub fn next(&self) -> Self {
        cycle_next(ALL_THEMES, *self)
    }

    /// Cycle to the previous color theme.
    pub fn prev(&self) -> Self {
        cycle_prev(ALL_THEMES, *self)
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
            ColorTheme::GradientSakura => Color::Rgb(255, 183, 197), // Sakura pink
        }
    }

    /// Get a dimmed version of the current theme color for secondary text.
    pub fn secondary_color(self) -> Color {
        ensure_min_contrast_on_black(blend_toward_gray(self.color(), 110, 0.55), 3.0)
    }

    /// Get a brighter dimmed version of the current theme color for muted text.
    pub fn muted_color(self) -> Color {
        ensure_min_contrast_on_black(blend_toward_gray(self.color(), 150, 0.50), 4.5)
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
                | ColorTheme::GradientSakura
        )
    }

    /// Get color at a specific position for dynamic themes.
    /// `x` is the horizontal position (column), `y` is the vertical position (row).
    /// `width` and `height` are the total dimensions for normalization.
    pub fn color_at_position(self, x: usize, y: usize, width: usize, height: usize) -> Color {
        const RAINBOW: [Color; 7] = [
            Color::Red,
            Color::Rgb(255, 127, 0), // Orange
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Magenta,
        ];
        // Index into RAINBOW for `pos` out of `len` (0 when `len` is 0).
        let band = |pos: usize, len: usize| (pos * RAINBOW.len() / len.max(1)) % RAINBOW.len();
        // Horizontal position normalized to 0.0..1.0.
        let progress = if width > 0 {
            x as f32 / width as f32
        } else {
            0.0
        };

        match self {
            ColorTheme::Rainbow => RAINBOW[band(x, width)],
            ColorTheme::RainbowVertical => RAINBOW[band(y, height)],
            ColorTheme::GradientWarm => {
                // Red -> Orange -> Yellow
                if progress < 0.5 {
                    Color::Rgb(255, (127.0 * (progress * 2.0)) as u8, 0)
                } else {
                    let g = 127 + (128.0 * ((progress - 0.5) * 2.0)) as u8;
                    Color::Rgb(255, g, 0)
                }
            }
            ColorTheme::GradientCool => {
                // Blue -> Cyan -> Green
                if progress < 0.5 {
                    Color::Rgb(0, (255.0 * (progress * 2.0)) as u8, 255)
                } else {
                    let b = 255 - (255.0 * ((progress - 0.5) * 2.0)) as u8;
                    Color::Rgb(0, 255, b)
                }
            }
            ColorTheme::GradientOcean => {
                // Dark blue -> Cyan -> Teal
                if progress < 0.5 {
                    let t = progress * 2.0;
                    Color::Rgb((100.0 * t) as u8, (150.0 + 105.0 * t) as u8, 255)
                } else {
                    let t = (progress - 0.5) * 2.0;
                    Color::Rgb(100, 255, 255 - (127.0 * t) as u8)
                }
            }
            ColorTheme::GradientNeon => {
                // Magenta -> Cyan (synthwave style)
                let g = (255.0 * progress) as u8;
                Color::Rgb(255 - g, g, 255)
            }
            ColorTheme::GradientFire => {
                // Dark red -> Red -> Orange -> Yellow (fire effect)
                if progress < 0.33 {
                    Color::Rgb(128 + (127.0 * (progress * 3.0)) as u8, 0, 0)
                } else if progress < 0.66 {
                    Color::Rgb(255, (165.0 * ((progress - 0.33) * 3.0)) as u8, 0)
                } else {
                    // The last segment spans 0.34, not 1/3, so the add must saturate:
                    // it reaches 256 at the right edge of text wider than ~340 columns.
                    let g = 165u8.saturating_add((90.0 * ((progress - 0.66) * 3.0)) as u8);
                    Color::Rgb(255, g, 0)
                }
            }
            ColorTheme::GradientFrost => {
                // White -> Ice Blue -> Steel Blue
                if progress < 0.5 {
                    let t = progress * 2.0;
                    Color::Rgb(
                        255 - (79.0 * t) as u8,
                        255 - (31.0 * t) as u8,
                        255 - (25.0 * t) as u8,
                    )
                } else {
                    let t = (progress - 0.5) * 2.0;
                    Color::Rgb(
                        176 - (106.0 * t) as u8,
                        224 - (94.0 * t) as u8,
                        230 - (50.0 * t) as u8,
                    )
                }
            }
            ColorTheme::GradientAurora => {
                // Green -> Cyan -> Blue -> Purple (aurora colors)
                if progress < 0.33 {
                    let t = progress * 3.0;
                    Color::Rgb(0, (255.0 - 128.0 * t) as u8, (127.0 + 128.0 * t) as u8)
                } else if progress < 0.66 {
                    let t = (progress - 0.33) * 3.0;
                    Color::Rgb(
                        (65.0 * t) as u8,
                        (127.0 - 22.0 * t) as u8,
                        (255.0 - 30.0 * t) as u8,
                    )
                } else {
                    let t = (progress - 0.66) * 3.0;
                    Color::Rgb(
                        (65.0 + 73.0 * t) as u8,
                        (105.0 - 62.0 * t) as u8,
                        (225.0 + t) as u8,
                    )
                }
            }
            ColorTheme::GradientWinter => {
                // Deep Blue -> Royal Blue -> Ice Blue
                if progress < 0.5 {
                    let t = progress * 2.0;
                    Color::Rgb(
                        (25.0 + 40.0 * t) as u8,
                        (25.0 + 80.0 * t) as u8,
                        (112.0 + 113.0 * t) as u8,
                    )
                } else {
                    let t = (progress - 0.5) * 2.0;
                    Color::Rgb(
                        (65.0 + 70.0 * t) as u8,
                        (105.0 + 101.0 * t) as u8,
                        (225.0 + 25.0 * t) as u8,
                    )
                }
            }
            ColorTheme::GradientSakura => {
                // Sakura pink #FFB7C5 -> lavender blush #FFF0F5
                Color::Rgb(
                    255,
                    (183.0 + 57.0 * progress) as u8,
                    (197.0 + 48.0 * progress) as u8,
                )
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
            ColorTheme::GradientSakura => "Sakura",
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

/// Extract RGB values from a color.
pub fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Yellow => (255, 255, 0),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::Gray => (128, 128, 128),
        Color::DarkGray => (80, 80, 80),
        Color::LightRed => (255, 85, 85),
        Color::LightGreen => (85, 255, 85),
        Color::LightYellow => (255, 255, 85),
        Color::LightBlue => (85, 85, 255),
        Color::LightMagenta => (255, 85, 255),
        Color::LightCyan => (85, 255, 255),
        Color::White => (255, 255, 255),
        _ => (128, 128, 128),
    }
}

/// Scale a color's RGB channels by a clamped factor.
pub fn dim_color(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    let (r, g, b) = color_to_rgb(color);
    Color::Rgb(
        (r as f32 * factor) as u8,
        (g as f32 * factor) as u8,
        (b as f32 * factor) as u8,
    )
}

/// Blend a color toward neutral gray while retaining some theme tint.
pub fn blend_toward_gray(color: Color, baseline: u8, tint: f32) -> Color {
    let tint = tint.clamp(0.0, 1.0);
    let (r, g, b) = color_to_rgb(color);
    let mix = |channel: u8| (baseline as f32 * (1.0 - tint) + channel as f32 * tint).round() as u8;
    Color::Rgb(mix(r), mix(g), mix(b))
}

/// Convert one 8-bit sRGB channel to linear light.
fn srgb_channel_to_linear(channel: u8) -> f32 {
    let channel = channel as f32 / 255.0;
    if channel <= 0.03928 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of a color.
pub fn relative_luminance(color: Color) -> f32 {
    let (r, g, b) = color_to_rgb(color);
    0.2126 * srgb_channel_to_linear(r)
        + 0.7152 * srgb_channel_to_linear(g)
        + 0.0722 * srgb_channel_to_linear(b)
}

/// WCAG contrast ratio of a color against pure black.
pub fn contrast_ratio_on_black(color: Color) -> f32 {
    (relative_luminance(color) + 0.05) / 0.05
}

/// Lighten a color toward white until it meets a minimum contrast ratio on black.
pub fn ensure_min_contrast_on_black(color: Color, min_ratio: f32) -> Color {
    let (r, g, b) = color_to_rgb(color);
    let color = Color::Rgb(r, g, b);
    if contrast_ratio_on_black(color) >= min_ratio {
        return color;
    }

    let lerp = |channel: u8, t: f32| (channel as f32 + (255.0 - channel as f32) * t).round() as u8;
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..16 {
        let mid = (lo + hi) / 2.0;
        let candidate = Color::Rgb(lerp(r, mid), lerp(g, mid), lerp(b, mid));
        if contrast_ratio_on_black(candidate) >= min_ratio {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    Color::Rgb(lerp(r, hi), lerp(g, hi), lerp(b, hi))
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
    elapsed_ms % 1000 < 500
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_to_rgb_maps_named_colors_used_by_app() {
        let cases = [
            (Color::Cyan, (0, 255, 255)),
            (Color::Green, (0, 255, 0)),
            (Color::White, (255, 255, 255)),
            (Color::Magenta, (255, 0, 255)),
            (Color::Yellow, (255, 255, 0)),
            (Color::Red, (255, 0, 0)),
            (Color::Blue, (0, 0, 255)),
            (Color::Gray, (128, 128, 128)),
            (Color::DarkGray, (80, 80, 80)),
            (Color::Black, (0, 0, 0)),
            (Color::LightRed, (255, 85, 85)),
            (Color::LightGreen, (85, 255, 85)),
            (Color::LightYellow, (255, 255, 85)),
            (Color::LightBlue, (85, 85, 255)),
            (Color::LightMagenta, (255, 85, 255)),
            (Color::LightCyan, (85, 255, 255)),
        ];

        for (color, expected) in cases {
            assert_eq!(color_to_rgb(color), expected);
        }
    }

    #[test]
    fn color_to_rgb_passes_through_rgb_and_defaults_indexed_reset_to_gray() {
        assert_eq!(color_to_rgb(Color::Rgb(12, 34, 56)), (12, 34, 56));
        assert_eq!(color_to_rgb(Color::Indexed(7)), (128, 128, 128));
        assert_eq!(color_to_rgb(Color::Reset), (128, 128, 128));
    }

    #[test]
    fn dim_color_scales_rgb_channels_and_clamps_factor() {
        assert_eq!(
            dim_color(Color::Rgb(200, 100, 50), 0.5),
            Color::Rgb(100, 50, 25)
        );
        assert_eq!(dim_color(Color::Red, 2.0), Color::Rgb(255, 0, 0));
        assert_eq!(dim_color(Color::Blue, -1.0), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn blend_toward_gray_mixes_theme_with_baseline_and_clamps_tint() {
        assert_eq!(
            blend_toward_gray(Color::Blue, 110, 0.55),
            Color::Rgb(50, 50, 190)
        );
        assert_eq!(
            blend_toward_gray(Color::Rgb(10, 20, 30), 110, -1.0),
            Color::Rgb(110, 110, 110)
        );
        assert_eq!(
            blend_toward_gray(Color::Rgb(10, 20, 30), 110, 2.0),
            Color::Rgb(10, 20, 30)
        );
    }

    #[test]
    fn ensure_min_contrast_keeps_bright_colors_and_lightens_dark_colors() {
        assert_eq!(
            ensure_min_contrast_on_black(Color::White, 4.5),
            Color::Rgb(255, 255, 255)
        );

        let dark = Color::Rgb(0, 0, 60);
        let adjusted = ensure_min_contrast_on_black(dark, 4.5);

        assert!(contrast_ratio_on_black(adjusted) >= 4.5);
        assert_ne!(adjusted, dark);
    }

    #[test]
    fn color_at_position_never_overflows_on_very_wide_text() {
        for theme in ALL_THEMES {
            for x in 0..2000 {
                theme.color_at_position(x, 0, 2000, 1);
            }
        }
        assert_eq!(
            ColorTheme::GradientFire.color_at_position(1999, 0, 2000, 1),
            Color::Rgb(255, 255, 0)
        );
    }

    #[test]
    fn secondary_and_muted_colors_meet_wcag_contrast_floor_for_every_theme() {
        for theme in ALL_THEMES {
            let secondary_contrast = contrast_ratio_on_black(theme.secondary_color());
            let muted_contrast = contrast_ratio_on_black(theme.muted_color());

            assert!(
                secondary_contrast >= 3.0,
                "{} secondary contrast {secondary_contrast}",
                theme.display_name()
            );
            assert!(
                muted_contrast >= 4.5,
                "{} muted contrast {muted_contrast}",
                theme.display_name()
            );
            assert!(
                muted_contrast >= secondary_contrast,
                "{} muted contrast {muted_contrast} < secondary {secondary_contrast}",
                theme.display_name()
            );
        }
    }
}
