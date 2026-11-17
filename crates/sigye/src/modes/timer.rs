//! Timer display mode.

use std::any::Any;
use std::time::{Duration, Instant};

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

/// Shortest timer duration allowed, in minutes.
pub const MIN_MINS: u32 = 1;
/// Longest timer duration allowed, in minutes.
pub const MAX_MINS: u32 = 99;

/// Clamp a timer duration to the allowed range. The settings dialog, `+`/`-`, the
/// config file and `--timer` all go through this, so they agree on the limits.
pub fn clamp_mins(mins: u32) -> u32 {
    mins.clamp(MIN_MINS, MAX_MINS)
}

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
        let secs = clamp_mins(duration_mins) * 60;
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

    /// Adjust the duration by `delta` minutes, within the allowed range. Saves to config.
    pub fn adjust_duration(&mut self, delta: i32, ctx: &mut RenderContext) {
        let mins = clamp_mins((self.duration_secs / 60).saturating_add_signed(delta));
        self.duration_secs = mins * 60;
        ctx.config.timer_duration_mins = mins;
        let _ = ctx.save_config();

        // If not running, also update remaining
        if !self.running && !self.completed {
            self.remaining_secs = self.duration_secs;
        }
    }

    /// Sync duration from a config change (e.g., settings dialog).
    pub fn sync_duration(&mut self, mins: u32) {
        self.duration_secs = clamp_mins(mins) * 60;
        if !self.running && !self.completed {
            self.remaining_secs = self.duration_secs;
        }
    }
}

impl Mode for TimerMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Timer
    }

    fn tick(&mut self, ctx: &mut RenderContext) {
        if !self.running || self.completed {
            return;
        }

        let elapsed_secs = self.last_tick.elapsed().as_secs() as u32;
        if elapsed_secs >= 1 {
            // Advance by the whole seconds consumed, keeping the fraction for the next
            // tick. Resetting to `now` would drop it and run the timer slow.
            self.last_tick += Duration::from_secs(elapsed_secs.into());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_stay_within_the_allowed_range() {
        assert_eq!(TimerMode::new(0).duration_secs, 60);
        assert_eq!(TimerMode::new(500).duration_secs, 99 * 60);

        let mut ctx = RenderContext::for_test();
        let mut timer = TimerMode::new(1);
        timer.adjust_duration(-1, &mut ctx);
        assert_eq!(timer.duration_secs, 60);
        timer.sync_duration(120);
        timer.adjust_duration(1, &mut ctx);
        assert_eq!(timer.duration_secs, 99 * 60);
        assert_eq!(ctx.config.timer_duration_mins, 99);
    }

    #[test]
    fn tick_keeps_the_fraction_of_a_second_it_did_not_consume() {
        let mut ctx = RenderContext::for_test();
        let mut timer = TimerMode::new(1);
        timer.toggle();
        timer.last_tick = Instant::now() - Duration::from_millis(1900);

        timer.tick(&mut ctx);

        assert_eq!(timer.remaining_secs, 59);
        assert!(timer.last_tick.elapsed() >= Duration::from_millis(900));
    }
}
