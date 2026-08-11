//! Timer display mode.

use std::any::Any;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Color,
    widgets::Paragraph,
};
use sigye_core::DisplayMode;

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Countdown timer display mode.
pub struct TimerMode {
    pub duration_secs: u32,
    pub remaining_secs: u32,
    pub running: bool,
    pub last_tick: Instant,
    pub completed: bool,
}

impl TimerMode {
    /// Create a new `TimerMode` with the given duration in minutes.
    pub fn new(duration_mins: u32) -> Self {
        let secs = duration_mins * 60;
        Self {
            duration_secs: secs,
            remaining_secs: secs,
            running: false,
            last_tick: Instant::now(),
            completed: false,
        }
    }

    /// Toggle running state. If completed, restart.
    pub fn toggle(&mut self) {
        if self.completed {
            self.reset();
            self.running = true;
            self.last_tick = Instant::now();
        } else {
            self.running = !self.running;
            if self.running {
                self.last_tick = Instant::now();
            }
        }
    }

    /// Reset the timer to the full duration.
    pub fn reset(&mut self) {
        self.remaining_secs = self.duration_secs;
        self.running = false;
        self.completed = false;
        self.last_tick = Instant::now();
    }

    /// Adjust the duration by `delta` minutes (clamped to 1–99). Saves to config.
    pub fn adjust_duration(&mut self, delta: i32, ctx: &mut RenderContext) {
        let mins = (self.duration_secs / 60) as i32 + delta;
        let mins = mins.clamp(1, 99) as u32;
        self.duration_secs = mins * 60;
        ctx.config.timer_duration_mins = mins;
        let _ = ctx.config.save();

        // If not running, also update remaining
        if !self.running && !self.completed {
            self.remaining_secs = self.duration_secs;
        }
    }

    /// Sync duration from a config change (e.g., settings dialog).
    pub fn sync_duration(&mut self, mins: u32) {
        self.duration_secs = mins * 60;
        if !self.running && !self.completed {
            self.remaining_secs = self.duration_secs;
        }
    }
}

impl Mode for TimerMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Timer
    }

    fn update(&mut self, ctx: &mut RenderContext) {
        ctx.decay_flash();

        if !self.running || self.completed {
            return;
        }

        let elapsed_secs = self.last_tick.elapsed().as_secs() as u32;
        if elapsed_secs >= 1 {
            self.last_tick = Instant::now();

            if self.remaining_secs > elapsed_secs {
                self.remaining_secs -= elapsed_secs;
            } else {
                self.remaining_secs = 0;
                self.running = false;
                self.completed = true;

                ctx.trigger_flash(1.0);
                ctx.ring_bell();
                ctx.send_notification("Timer", "Time's up!");
                ctx.run_on_complete();
            }
        }
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let actions = self.all_footer_actions();
        let footer_height = render::footer_height(area.width, &actions);
        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        // Layout: Fill(1), font height, Length(1) status, Length(1) progress, Fill(1), Length(1) hints
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(footer_height),
        ])
        .split(area);

        // Format remaining time
        let minutes = self.remaining_secs / 60;
        let seconds = self.remaining_secs % 60;
        let time_str = format!("{:02}:{:02}", minutes, seconds);

        let params = AsciiTextParams::from_ctx(
            ctx,
            if self.completed {
                Color::Red
            } else {
                ctx.color()
            },
        );

        render::render_ascii_text(frame, chunks[1], font, &time_str, &params);

        // Status label
        let (status_text, status_color) = if self.completed {
            ("TIME'S UP!", Color::Red)
        } else if !self.running {
            ("(PAUSED)", ctx.dim_color())
        } else {
            ("RUNNING", ctx.color())
        };

        render::render_centered_text(frame, chunks[2], status_text, status_color);

        // Progress bar
        let progress = if self.duration_secs > 0 {
            1.0 - (self.remaining_secs as f64 / self.duration_secs as f64)
        } else {
            1.0
        };
        let bar_color = if self.completed {
            Color::Red
        } else {
            ctx.color()
        };
        let bar = render::render_progress_bar(progress, area.width.saturating_sub(4), bar_color);
        let bar_widget = Paragraph::new(bar).alignment(Alignment::Center);
        frame.render_widget(bar_widget, chunks[3]);

        render::render_footer(frame, chunks[5], ctx, &actions);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut RenderContext) -> bool {
        match key.code {
            KeyCode::Char(' ') => {
                self.toggle();
                true
            }
            KeyCode::Char('r') => {
                self.reset();
                true
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_duration(1, ctx);
                true
            }
            KeyCode::Char('-') => {
                self.adjust_duration(-1, ctx);
                true
            }
            _ => false,
        }
    }

    fn footer_actions(&self) -> Vec<render::FooterAction> {
        vec![
            render::FooterAction::new(KeyCode::Char(' '), "Space", "start/pause"),
            render::FooterAction::new(KeyCode::Char('r'), "r", "reset"),
            render::FooterAction::new(KeyCode::Char('-'), "-", "less"),
            render::FooterAction::new(KeyCode::Char('+'), "+", "more"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
