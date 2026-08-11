//! Pomodoro display mode.

use std::any::Any;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Color,
    widgets::Paragraph,
};
use sigye_core::{DisplayMode, PomodoroPhase};

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Pomodoro timer display mode.
pub struct PomodoroMode {
    pub phase: PomodoroPhase,
    pub remaining_secs: u32,
    pub sessions_completed: u32,
    pub total_focus_secs: u64,
    pub work_start: Option<Instant>,
    pub last_tick: Instant,
    pub running: bool,
}

impl PomodoroMode {
    /// Create a new `PomodoroMode` initialized in the Work phase.
    pub fn new(work_mins: u32, sessions: u32, focus_mins: u32) -> Self {
        Self {
            phase: PomodoroPhase::Work,
            remaining_secs: work_mins * 60,
            sessions_completed: sessions,
            total_focus_secs: focus_mins as u64 * 60,
            work_start: None,
            last_tick: Instant::now(),
            running: false,
        }
    }

    /// Toggle the timer running state.
    pub fn toggle(&mut self) {
        self.running = !self.running;
        if self.running {
            self.last_tick = Instant::now();
            if self.phase == PomodoroPhase::Work && self.work_start.is_none() {
                self.work_start = Some(Instant::now());
            }
        }
    }

    /// Reset the timer to the current phase's duration from config.
    pub fn reset(&mut self, ctx: &RenderContext) {
        self.running = false;
        self.work_start = None;
        self.remaining_secs = phase_duration_secs(self.phase, ctx);
        self.last_tick = Instant::now();
    }

    /// Advance to the next phase, handling session tracking and notifications.
    pub fn transition_phase(&mut self, ctx: &mut RenderContext) {
        // Track focus time if we were in a work phase
        if self.phase == PomodoroPhase::Work {
            if let Some(start) = self.work_start.take() {
                self.total_focus_secs += start.elapsed().as_secs();
            }
            self.sessions_completed += 1;
        }

        // Determine next phase
        self.phase = if self.phase == PomodoroPhase::Work {
            let long_break_threshold = ctx.config.pomodoro_sessions_until_long;
            if self.sessions_completed.is_multiple_of(long_break_threshold) {
                PomodoroPhase::LongBreak
            } else {
                PomodoroPhase::ShortBreak
            }
        } else {
            PomodoroPhase::Work
        };

        // Set up new phase duration
        self.remaining_secs = phase_duration_secs(self.phase, ctx);
        self.running = false;
        self.last_tick = Instant::now();

        // Persist session data
        ctx.config.pomodoro_sessions_completed = self.sessions_completed;
        ctx.config.pomodoro_total_focus_mins = (self.total_focus_secs / 60) as u32;
        let _ = ctx.config.save();

        // Trigger effects
        ctx.trigger_flash(1.0);
        ctx.ring_bell();

        let (title, body) = match self.phase {
            PomodoroPhase::Work => ("Pomodoro", "Time to focus!"),
            PomodoroPhase::ShortBreak => ("Pomodoro", "Short break time!"),
            PomodoroPhase::LongBreak => ("Pomodoro", "Long break time!"),
        };
        ctx.send_notification(title, body);
        ctx.run_on_complete();

        // Lifecycle hooks
        if self.phase == PomodoroPhase::Work {
            ctx.run_command(&ctx.config.on_start.clone());
        } else {
            ctx.run_command(&ctx.config.on_break.clone());
        }

        if self.phase == PomodoroPhase::Work {
            self.work_start = None;
        }
    }
}

/// Get the duration in seconds for a given phase from config.
fn phase_duration_secs(phase: PomodoroPhase, ctx: &RenderContext) -> u32 {
    match phase {
        PomodoroPhase::Work => ctx.config.pomodoro_work_mins * 60,
        PomodoroPhase::ShortBreak => ctx.config.pomodoro_break_mins * 60,
        PomodoroPhase::LongBreak => ctx.config.pomodoro_long_break_mins * 60,
    }
}

impl Mode for PomodoroMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Pomodoro
    }

    fn update(&mut self, ctx: &mut RenderContext) {
        ctx.decay_flash();

        if !self.running {
            return;
        }

        let elapsed = self.last_tick.elapsed();
        let elapsed_secs = elapsed.as_secs() as u32;

        if elapsed_secs >= 1 {
            self.last_tick = Instant::now();

            if self.remaining_secs > elapsed_secs {
                self.remaining_secs -= elapsed_secs;
            } else {
                self.remaining_secs = 0;
                self.transition_phase(ctx);
            }
        }
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let actions = self.all_footer_actions();
        let footer_height = render::footer_height(area.width, &actions);
        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        // Layout: Fill(1), font height, Length(1) phase, Length(1) sessions, Length(1) progress, Fill(1), Length(1) hints
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(footer_height),
        ])
        .split(area);

        // Format remaining time as MM:SS
        let minutes = self.remaining_secs / 60;
        let seconds = self.remaining_secs % 60;
        let time_str = format!("{:02}:{:02}", minutes, seconds);

        let params = AsciiTextParams::from_ctx(ctx, ctx.color());

        // Render big ASCII timer
        render::render_ascii_text(frame, chunks[1], font, &time_str, &params);

        // Phase indicator — green for breaks
        let phase_color = if self.phase.is_break() {
            Color::Green
        } else {
            ctx.color()
        };
        let phase_label = format!(
            "{} {}",
            self.phase.display_name(),
            if self.running { "" } else { "(PAUSED)" }
        );
        render::render_centered_text(frame, chunks[2], &phase_label, phase_color);

        // Session counter
        let session_str = format!(
            "Sessions: {}  |  Focus: {}m",
            self.sessions_completed,
            self.total_focus_secs / 60
        );
        render::render_centered_text(frame, chunks[3], &session_str, ctx.dim_color());

        // Progress bar
        let total_secs = phase_duration_secs(self.phase, ctx);
        let progress = if total_secs > 0 {
            1.0 - (self.remaining_secs as f64 / total_secs as f64)
        } else {
            1.0
        };
        let bar = render::render_progress_bar(progress, area.width.saturating_sub(4), phase_color);
        let bar_widget = Paragraph::new(bar).alignment(Alignment::Center);
        frame.render_widget(bar_widget, chunks[4]);

        render::render_footer(frame, chunks[6], ctx, &actions);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut RenderContext) -> bool {
        match key.code {
            KeyCode::Char(' ') => {
                let was_running = self.running;
                self.toggle();
                if !was_running && self.running && self.phase == PomodoroPhase::Work {
                    ctx.run_command(&ctx.config.on_start.clone());
                }
                true
            }
            KeyCode::Char('r') => {
                self.reset(ctx);
                true
            }
            KeyCode::Char('n') => {
                self.transition_phase(ctx);
                true
            }
            _ => false,
        }
    }

    fn footer_actions(&self) -> Vec<render::FooterAction> {
        vec![
            render::FooterAction::new(KeyCode::Char(' '), "Space", "start/pause"),
            render::FooterAction::new(KeyCode::Char('r'), "r", "reset"),
            render::FooterAction::new(KeyCode::Char('n'), "n", "skip"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
