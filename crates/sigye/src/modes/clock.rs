//! Clock display mode.

use std::any::Any;
use std::time::Instant;

use chrono::{Datelike, Local, Timelike};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::Color,
};
use sigye_core::{ClockDisplayFormat, DisplayMode, TimeFormat, color_to_rgb};

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Duration in milliseconds for toast visibility.
const TOAST_DURATION_MS: u128 = 2000;

/// Clock display mode — shows the current time as big ASCII art.
pub struct ClockMode {
    pub last_second: u32,
    pub last_minute: u32,
    pub last_hour: u32,
    pub display_format: ClockDisplayFormat,
    /// Toast message to display temporarily.
    toast_message: Option<String>,
    /// When the toast was triggered.
    toast_start: Option<Instant>,
}

impl ClockMode {
    /// Create a new `ClockMode` initialized from the current local time.
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            last_second: now.format("%S").to_string().parse().unwrap_or(0),
            last_minute: now.format("%M").to_string().parse().unwrap_or(0),
            last_hour: now.format("%H").to_string().parse().unwrap_or(0),
            display_format: ClockDisplayFormat::HumanReadable,
            toast_message: None,
            toast_start: None,
        }
    }

    /// Detect time changes and trigger flash effects with appropriate intensity.
    fn update_flash(&mut self, ctx: &mut RenderContext) {
        let now = Local::now();
        let second: u32 = now.format("%S").to_string().parse().unwrap_or(0);
        let minute: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let hour: u32 = now.format("%H").to_string().parse().unwrap_or(0);

        if hour != self.last_hour {
            ctx.trigger_flash(1.0);
            self.last_hour = hour;
        } else if minute != self.last_minute {
            ctx.trigger_flash(0.7);
            self.last_minute = minute;
        } else if second != self.last_second {
            ctx.trigger_flash(0.3);
            self.last_second = second;
        }

        ctx.decay_flash();
    }

    /// Show a toast message for a brief duration.
    fn show_toast(&mut self, message: String) {
        self.toast_message = Some(message);
        self.toast_start = Some(Instant::now());
    }

    /// Returns the active toast text if still within display duration.
    fn active_toast(&self) -> Option<&str> {
        if let (Some(msg), Some(start)) = (&self.toast_message, self.toast_start)
            && start.elapsed().as_millis() < TOAST_DURATION_MS
        {
            return Some(msg.as_str());
        }
        None
    }
}

impl Default for ClockMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a compact, polished progress bar directly to the buffer.
/// Format: `label ▮▮▮▮▮▮▯▯▯▯▯▯ pct%`
#[allow(clippy::too_many_arguments)]
fn render_mini_bar(
    buf: &mut ratatui::buffer::Buffer,
    x: u16,
    y: u16,
    max_x: u16,
    label: &str,
    progress: f64,
    accent: Color,
    dim: Color,
) -> u16 {
    let pct = (progress * 100.0) as u32;
    let pct_str = format!("{pct:>3}%");
    let bar_width: usize = 12;
    let filled = (progress * bar_width as f64).round() as usize;
    let empty = bar_width - filled;

    // Write label
    let mut cx = x;
    for ch in label.chars() {
        if cx >= max_x {
            break;
        }
        if let Some(cell) = buf.cell_mut(Position::new(cx, y)) {
            cell.set_char(ch);
            cell.set_fg(dim);
        }
        cx += 1;
    }

    // Space
    cx += 1;

    // Filled portion
    for _ in 0..filled {
        if cx >= max_x {
            break;
        }
        if let Some(cell) = buf.cell_mut(Position::new(cx, y)) {
            cell.set_char('▮');
            cell.set_fg(accent);
        }
        cx += 1;
    }

    // Empty portion
    for _ in 0..empty {
        if cx >= max_x {
            break;
        }
        if let Some(cell) = buf.cell_mut(Position::new(cx, y)) {
            cell.set_char('▯');
            cell.set_fg(Color::Rgb(60, 60, 60));
        }
        cx += 1;
    }

    // Space + percentage
    cx += 1;
    for ch in pct_str.chars() {
        if cx >= max_x {
            break;
        }
        if let Some(cell) = buf.cell_mut(Position::new(cx, y)) {
            cell.set_char(ch);
            cell.set_fg(dim);
        }
        cx += 1;
    }

    cx
}

/// Render a toast message centered in the given area with accent color.
fn render_toast(frame: &mut Frame, area: Rect, text: &str, accent: Color, elapsed_ms: u128) {
    // Fade out over the last 500ms
    let opacity = if elapsed_ms > TOAST_DURATION_MS.saturating_sub(500) {
        let fade_progress = (TOAST_DURATION_MS.saturating_sub(elapsed_ms)) as f32 / 500.0;
        fade_progress.clamp(0.0, 1.0)
    } else {
        1.0
    };

    let (r, g, b) = color_to_rgb(accent);

    let faded = Color::Rgb(
        (r as f32 * opacity) as u8,
        (g as f32 * opacity) as u8,
        (b as f32 * opacity) as u8,
    );

    // Render: ✓ Copied: <value>
    let display = format!("✓ {text}");
    let text_width = display.chars().count() as u16;
    let start_x = area.x + (area.width.saturating_sub(text_width)) / 2;
    let y = area.y;

    let buf = frame.buffer_mut();
    for (i, ch) in display.chars().enumerate() {
        let x_pos = start_x + i as u16;
        if x_pos >= area.x + area.width {
            break;
        }
        if ch == ' ' {
            continue;
        }
        if let Some(cell) = buf.cell_mut(Position::new(x_pos, y)) {
            cell.set_char(ch);
            cell.set_fg(faded);
        }
    }
}

impl Mode for ClockMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Clock
    }

    fn update(&mut self, ctx: &mut RenderContext) {
        self.update_flash(ctx);
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();

        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        let chunks = if ctx.config.minimal_mode {
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(font_height),
                Constraint::Fill(1),
            ])
            .split(area)
        } else {
            let sun_height = if ctx.sunrise_sunset.is_some() { 1 } else { 0 };
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(font_height),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(sun_height),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area)
        };

        let now = Local::now();

        // Build the text to display as big ASCII art based on display_format
        let big_text = match self.display_format {
            ClockDisplayFormat::HumanReadable => match ctx.time_format {
                TimeFormat::TwelveHour => {
                    if ctx.show_seconds {
                        now.format("%I:%M:%S %p").to_string()
                    } else {
                        now.format("%I:%M %p").to_string()
                    }
                }
                TimeFormat::TwentyFourHour => {
                    if ctx.show_seconds {
                        now.format("%H:%M:%S").to_string()
                    } else {
                        now.format("%H:%M").to_string()
                    }
                }
            },
            ClockDisplayFormat::UnixTimestamp => now.timestamp().to_string(),
            ClockDisplayFormat::Iso8601 => {
                // Big text is still the normal time; ISO string goes in the info line
                match ctx.time_format {
                    TimeFormat::TwelveHour => {
                        if ctx.show_seconds {
                            now.format("%I:%M:%S %p").to_string()
                        } else {
                            now.format("%I:%M %p").to_string()
                        }
                    }
                    TimeFormat::TwentyFourHour => {
                        if ctx.show_seconds {
                            now.format("%H:%M:%S").to_string()
                        } else {
                            now.format("%H:%M").to_string()
                        }
                    }
                }
            }
            ClockDisplayFormat::HexTime => {
                let h = now.hour();
                let m = now.minute();
                let s = now.second();
                format!("{h:02X}:{m:02X}:{s:02X}")
            }
        };

        let params = AsciiTextParams::from_ctx(ctx, ctx.color());

        // Render big ASCII time
        render::render_ascii_text(frame, chunks[1], font, &big_text, &params);

        if ctx.config.minimal_mode {
            return;
        }

        // Render date string
        let date_str = now.format("%A, %B %-d, %Y").to_string();
        render::render_centered_text(frame, chunks[3], &date_str, ctx.dim_color());

        // Render sunrise/sunset info if available
        if let Some((ref sunrise, ref sunset)) = ctx.sunrise_sunset {
            let sun_str = format!("Sunrise {}  Sunset {}", sunrise, sunset);
            render::render_centered_text(frame, chunks[4], &sun_str, ctx.dim_color());
        }

        // Render format info line
        match self.display_format {
            ClockDisplayFormat::HumanReadable => {}
            ClockDisplayFormat::UnixTimestamp => {
                render::render_centered_text(frame, chunks[5], "Unix Timestamp", ctx.dim_color());
            }
            ClockDisplayFormat::Iso8601 => {
                let iso = now.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
                render::render_centered_text(frame, chunks[5], &iso, ctx.muted_color());
            }
            ClockDisplayFormat::HexTime => {
                render::render_centered_text(frame, chunks[5], "Hex Time", ctx.dim_color());
            }
        }

        // Render toast OR progress bars in the bottom info row
        let info_area = chunks[7];
        if let Some(toast_text) = self.active_toast() {
            let elapsed = self
                .toast_start
                .map(|s| s.elapsed().as_millis())
                .unwrap_or(0);
            render_toast(frame, info_area, toast_text, ctx.color(), elapsed);
        } else {
            // Render day/year progress bars
            let day_progress = {
                let h = now.hour();
                let m = now.minute();
                let s = now.second();
                (h * 3600 + m * 60 + s) as f64 / 86400.0
            };
            let year_progress = {
                let day_of_year = now.ordinal() as f64;
                let year = now.year();
                let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    366.0
                } else {
                    365.0
                };
                day_of_year / days_in_year
            };

            // Center the two bars with a gap between them
            // Each bar: "Day " (4) + bar(12) + " " (1) + "xxx%" (4) = 21 chars
            // Gap: "   " (3)
            // Total: 21 + 3 + 22 = 46 chars
            let total_width: u16 = 46;
            let start_x = info_area.x + (info_area.width.saturating_sub(total_width)) / 2;
            let max_x = info_area.x + info_area.width;
            let y = info_area.y;
            let dim = ctx.dim_color();

            let buf = frame.buffer_mut();
            let after_day = render_mini_bar(
                buf,
                start_x,
                y,
                max_x,
                "Day",
                day_progress,
                ctx.color(),
                dim,
            );

            // 3-char gap
            render_mini_bar(
                buf,
                after_day + 3,
                y,
                max_x,
                "Year",
                year_progress,
                ctx.color(),
                dim,
            );
        }

        render::render_key_hints(frame, chunks[8], ctx, &self.key_hints());
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut RenderContext) -> bool {
        match key.code {
            KeyCode::Char('z') => {
                ctx.config.minimal_mode = !ctx.config.minimal_mode;
                if let Err(e) = ctx.config.save() {
                    eprintln!("Warning: Failed to save config: {e}");
                }
                true
            }
            KeyCode::Char('f') => {
                self.display_format = self.display_format.next();
                true
            }
            KeyCode::Char('u') => {
                let ts = Local::now().timestamp().to_string();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(ts.clone());
                    self.show_toast(format!("Copied: {ts}"));
                }
                ctx.trigger_flash(0.5);
                true
            }
            KeyCode::Char('i') => {
                let iso = Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(iso.clone());
                    self.show_toast(format!("Copied: {iso}"));
                }
                ctx.trigger_flash(0.5);
                true
            }
            _ => false,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("z", "minimal"),
            ("f", "format"),
            ("u", "copy unix"),
            ("i", "copy iso"),
            ("t", "12/24h"),
            ("c", "color"),
            ("s", "settings"),
            ("?", "help"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use ratatui::{Terminal, backend::TestBackend};
    use sigye_config::Config;
    use sigye_fonts::FontRegistry;

    use super::*;

    fn render_context(screensaver_mode: bool) -> RenderContext {
        let config = Config::default();
        RenderContext {
            time_format: config.time_format,
            color_theme: config.color_theme,
            animation_style: config.animation_style,
            animation_speed: config.animation_speed,
            colon_blink: config.colon_blink,
            show_seconds: config.show_seconds,
            background_style: config.background_style,
            current_font: config.font_name.clone(),
            font_registry: FontRegistry::new(),
            on_complete_command: config.on_complete.clone(),
            config,
            animation_start: Instant::now(),
            flash_intensity: 0.0,
            flash_start: None,
            screensaver_mode,
            desktop_notifications: false,
            sunrise_sunset: None,
        }
    }

    #[test]
    fn render_shows_settings_hint_by_default() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mode = ClockMode::new();
        let ctx = render_context(false);

        terminal.draw(|frame| mode.render(frame, &ctx)).unwrap();

        assert!(buffer_contains_text(terminal.backend(), "[s] settings"));
    }

    #[test]
    fn render_hides_settings_hint_in_screensaver_mode() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mode = ClockMode::new();
        let ctx = render_context(true);

        terminal.draw(|frame| mode.render(frame, &ctx)).unwrap();

        assert!(!buffer_contains_text(terminal.backend(), "[s] settings"));
    }

    #[test]
    fn render_minimal_view_shows_only_centered_clock() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mode = ClockMode::new();
        let mut ctx = render_context(false);
        ctx.config.minimal_mode = true;
        ctx.sunrise_sunset = Some(("06:00".into(), "18:00".into()));
        let date = Local::now().format("%A, %B %-d, %Y").to_string();

        terminal.draw(|frame| mode.render(frame, &ctx)).unwrap();

        let backend = terminal.backend();
        assert!(!buffer_contains_text(backend, &date));
        assert!(!buffer_contains_text(backend, "Sunrise"));
        assert!(!buffer_contains_text(backend, "Day"));
        assert!(!buffer_contains_text(backend, "Year"));
        assert!(!buffer_contains_text(backend, "[s] settings"));

        let rows = occupied_rows(backend);
        assert!(!rows.is_empty());
        let top_padding = rows[0];
        let bottom_padding = 39 - rows[rows.len() - 1];
        assert!(top_padding.abs_diff(bottom_padding) <= 1);
    }

    fn buffer_contains_text(backend: &TestBackend, text: &str) -> bool {
        let buffer = backend.buffer();
        let area = buffer.area;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if text_matches_at(backend, x, y, text) {
                    return true;
                }
            }
        }
        false
    }

    fn occupied_rows(backend: &TestBackend) -> Vec<u16> {
        let buffer = backend.buffer();
        let area = buffer.area;
        (area.top()..area.bottom())
            .filter(|&y| {
                (area.left()..area.right())
                    .any(|x| buffer.cell((x, y)).is_some_and(|cell| cell.symbol() != " "))
            })
            .collect()
    }

    fn text_matches_at(backend: &TestBackend, x: u16, y: u16, text: &str) -> bool {
        let buffer = backend.buffer();
        text.chars().enumerate().all(|(offset, ch)| {
            buffer
                .cell((x + offset as u16, y))
                .is_some_and(|cell| cell.symbol() == ch.to_string())
        })
    }
}
