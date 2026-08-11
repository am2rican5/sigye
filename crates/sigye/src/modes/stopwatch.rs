//! Stopwatch display mode.

use std::any::Any;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
};
use sigye_core::DisplayMode;

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Stopwatch display mode.
pub struct StopwatchMode {
    /// Accumulated elapsed milliseconds (not counting current running interval).
    pub elapsed_ms: u64,
    pub running: bool,
    /// Timestamp when the current running interval started.
    pub last_tick: Instant,
    pub laps: Vec<u64>,
}

impl StopwatchMode {
    /// Create a new `StopwatchMode` at zero.
    pub fn new() -> Self {
        Self {
            elapsed_ms: 0,
            running: false,
            last_tick: Instant::now(),
            laps: Vec::new(),
        }
    }

    /// Returns the total elapsed milliseconds (accumulated + running delta).
    pub fn get_elapsed(&self) -> u64 {
        if self.running {
            self.elapsed_ms + self.last_tick.elapsed().as_millis() as u64
        } else {
            self.elapsed_ms
        }
    }

    /// Toggle running state, accumulating elapsed time.
    pub fn toggle(&mut self) {
        if self.running {
            // Accumulate elapsed
            self.elapsed_ms += self.last_tick.elapsed().as_millis() as u64;
            self.running = false;
        } else {
            self.last_tick = Instant::now();
            self.running = true;
        }
    }

    /// Reset all state to zero.
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
        self.running = false;
        self.last_tick = Instant::now();
        self.laps.clear();
    }

    /// Record a lap time (only if running).
    pub fn lap(&mut self) {
        if self.running {
            self.laps.push(self.get_elapsed());
        }
    }
}

impl Default for StopwatchMode {
    fn default() -> Self {
        Self::new()
    }
}

impl Mode for StopwatchMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Stopwatch
    }

    fn update(&mut self, _ctx: &mut RenderContext) {
        // Elapsed is computed on read; no state tick needed
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let actions = self.all_footer_actions();
        let footer_height = render::footer_height(area.width, &actions);
        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        // How many laps to show (last 5)
        let lap_count = self.laps.len().min(5) as u16;

        // Layout: Fill(1), font height, Length(1) centiseconds, Length(1) status, lap rows, Fill(1), Length(1) hints
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(1),                // centiseconds
            Constraint::Length(1),                // status
            Constraint::Length(lap_count.max(1)), // laps area
            Constraint::Fill(1),
            Constraint::Length(footer_height), // hints
        ])
        .split(area);

        let elapsed = self.get_elapsed();
        let total_secs = elapsed / 1000;
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        let time_str = format!("{:02}:{:02}", minutes, seconds);

        let params = AsciiTextParams {
            color_theme: ctx.color_theme,
            static_color: ctx.color(),
            animation_style: ctx.animation_style,
            animation_speed: ctx.animation_speed,
            elapsed_ms: ctx.elapsed_ms(),
            flash_intensity: ctx.flash_intensity,
            colon_blink: false,
        };

        // Render big ASCII MM:SS
        render::render_ascii_text(frame, chunks[1], font, &time_str, &params);

        // Centiseconds below (dimmed)
        let centis = (elapsed % 1000) / 10;
        let centis_str = format!(".{:02}", centis);
        render::render_centered_text(frame, chunks[2], &centis_str, ctx.dim_color());

        // Status
        let (status_text, status_color) = if self.running {
            ("RUNNING", Color::Green)
        } else if elapsed > 0 {
            ("PAUSED", Color::Yellow)
        } else {
            ("STOPPED", ctx.dim_color())
        };
        render::render_centered_text(frame, chunks[3], status_text, status_color);

        // Laps (last 5, with deltas)
        let lap_area = chunks[4];
        if !self.laps.is_empty() {
            let start_lap = self.laps.len().saturating_sub(5);
            let displayed_laps: Vec<u64> = self.laps[start_lap..].to_vec();

            for (i, &lap_ms) in displayed_laps.iter().enumerate() {
                let lap_secs = lap_ms / 1000;
                let lap_m = lap_secs / 60;
                let lap_s = lap_secs % 60;
                let lap_cs = (lap_ms % 1000) / 10;

                let delta_ms = if start_lap + i == 0 {
                    lap_ms
                } else {
                    lap_ms - self.laps[start_lap + i - 1]
                };
                let delta_secs = delta_ms / 1000;
                let delta_m = delta_secs / 60;
                let delta_s = delta_secs % 60;
                let delta_cs = (delta_ms % 1000) / 10;

                let lap_num = start_lap + i + 1;
                let lap_str = format!(
                    "Lap {:2}: {:02}:{:02}.{:02}  (+{:02}:{:02}.{:02})",
                    lap_num, lap_m, lap_s, lap_cs, delta_m, delta_s, delta_cs
                );

                let row_area = ratatui::layout::Rect {
                    x: lap_area.x,
                    y: lap_area.y + i as u16,
                    width: lap_area.width,
                    height: 1,
                };
                render::render_centered_text(frame, row_area, &lap_str, ctx.dim_color());
            }
        }

        render::render_footer(frame, chunks[6], ctx, &actions);
    }

    fn handle_key(&mut self, key: KeyEvent, _ctx: &mut RenderContext) -> bool {
        match key.code {
            KeyCode::Char(' ') => {
                self.toggle();
                true
            }
            KeyCode::Char('r') => {
                self.reset();
                true
            }
            KeyCode::Char('l') => {
                self.lap();
                true
            }
            _ => false,
        }
    }

    fn footer_actions(&self) -> Vec<render::FooterAction> {
        vec![
            render::FooterAction::new(KeyCode::Char(' '), "Space", "start/pause"),
            render::FooterAction::new(KeyCode::Char('r'), "r", "reset"),
            render::FooterAction::new(KeyCode::Char('l'), "l", "lap"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
