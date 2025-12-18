//! Core types for the sigye clock application.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

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
            ColorTheme::Rainbow
            | ColorTheme::RainbowVertical
            | ColorTheme::GradientNeon => Color::Magenta,
            ColorTheme::GradientWarm | ColorTheme::GradientFire => Color::Red,
            ColorTheme::GradientCool | ColorTheme::GradientOcean => Color::Cyan,
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
        }
    }
}
