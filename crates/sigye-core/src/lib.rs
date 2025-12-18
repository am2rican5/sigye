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
}

impl ColorTheme {
    /// Cycle to the next color theme.
    pub fn next(&self) -> Self {
        match self {
            ColorTheme::Cyan => ColorTheme::Green,
            ColorTheme::Green => ColorTheme::Magenta,
            ColorTheme::Magenta => ColorTheme::Yellow,
            ColorTheme::Yellow => ColorTheme::Red,
            ColorTheme::Red => ColorTheme::Blue,
            ColorTheme::Blue => ColorTheme::White,
            ColorTheme::White => ColorTheme::Cyan,
        }
    }

    /// Cycle to the previous color theme.
    pub fn prev(&self) -> Self {
        match self {
            ColorTheme::Cyan => ColorTheme::White,
            ColorTheme::Green => ColorTheme::Cyan,
            ColorTheme::Magenta => ColorTheme::Green,
            ColorTheme::Yellow => ColorTheme::Magenta,
            ColorTheme::Red => ColorTheme::Yellow,
            ColorTheme::Blue => ColorTheme::Red,
            ColorTheme::White => ColorTheme::Blue,
        }
    }

    /// Convert theme to Ratatui Color.
    pub fn color(self) -> Color {
        match self {
            ColorTheme::Cyan => Color::Cyan,
            ColorTheme::Green => Color::Green,
            ColorTheme::White => Color::White,
            ColorTheme::Magenta => Color::Magenta,
            ColorTheme::Yellow => Color::Yellow,
            ColorTheme::Red => Color::Red,
            ColorTheme::Blue => Color::Blue,
        }
    }
}
