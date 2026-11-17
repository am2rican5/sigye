//! Pomodoro display mode.

use std::any::Any;
use std::time::{Duration, Instant};

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
    pub last_tick: Instant,
    pub running: bool,
    /// Whether this phase's timer has been started since the phase began or was
    /// reset. `on_start` fires on a work phase's first start, and settings changes
    /// resize only a phase that has not started.
    pub started: bool,
}

impl PomodoroMode {
    /// Create a new `PomodoroMode` initialized in the Work phase.
    pub fn new(work_mins: u32, sessions: u32, focus_mins: u32) -> Self {
        Self {
            phase: PomodoroPhase::Work,
            remaining_secs: work_mins * 60,
            sessions_completed: sessions,
            total_focus_secs: focus_mins as u64 * 60,
            last_tick: Instant::now(),
            running: false,
            started: false,
        }
    }

    /// Start or pause the timer. The first start of a work phase runs `on_start`.
    pub fn toggle(&mut self, ctx: &RenderContext) {
        self.running = !self.running;
        if self.running {
            self.last_tick = Instant::now();
            if !self.started {
                self.started = true;
                if self.phase == PomodoroPhase::Work {
                    ctx.run_command(&ctx.config.on_start);
                }
            }
        }
    }

    /// Reset the timer to the current phase's duration from config.
    pub fn reset(&mut self, ctx: &RenderContext) {
        self.running = false;
        self.started = false;
        self.remaining_secs = phase_duration_secs(self.phase, ctx);
        self.last_tick = Instant::now();
    }

    /// Resize a phase that has not started yet to the configured duration, so
    /// settings changes show up immediately like they do for the timer mode.
    pub fn sync_duration(&mut self, ctx: &RenderContext) {
        if !self.started {
            self.remaining_secs = phase_duration_secs(self.phase, ctx);
        }
    }

    /// Advance to the next phase, handling session tracking and notifications.
    /// `completed` is false when the user skips a phase; a skipped work phase does
    /// not count as a session.
    pub fn transition_phase(&mut self, ctx: &mut RenderContext, completed: bool) {
        let finished_work = self.phase == PomodoroPhase::Work && completed;
        if finished_work {
            self.sessions_completed += 1;
        }

        // Determine next phase
        self.phase = if self.phase != PomodoroPhase::Work {
            PomodoroPhase::Work
        } else if finished_work
            && self
                .sessions_completed
                .is_multiple_of(ctx.config.pomodoro_sessions_until_long)
        {
            PomodoroPhase::LongBreak
        } else {
            PomodoroPhase::ShortBreak
        };

        // Set up new phase duration
        self.remaining_secs = phase_duration_secs(self.phase, ctx);
        self.running = false;
        self.started = false;
        self.last_tick = Instant::now();

        // Persist session data
        ctx.config.pomodoro_sessions_completed = self.sessions_completed;
        ctx.config.pomodoro_total_focus_mins = (self.total_focus_secs / 60) as u32;
        let _ = ctx.save_config();

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

        // `on_start` waits for the work phase's first start; see `toggle`.
        if self.phase.is_break() {
            ctx.run_command(&ctx.config.on_break);
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

    fn tick(&mut self, ctx: &mut RenderContext) {
        if !self.running {
            return;
        }

        let elapsed_secs = self.last_tick.elapsed().as_secs() as u32;

        if elapsed_secs >= 1 {
            // Advance by the whole seconds consumed, keeping the fraction for the next
            // tick. Resetting to `now` would drop it and run the timer slow.
            self.last_tick += Duration::from_secs(elapsed_secs.into());

            // Focus time counts only seconds the work timer actually ran.
            let consumed = elapsed_secs.min(self.remaining_secs);
            if self.phase == PomodoroPhase::Work {
                self.total_focus_secs += u64::from(consumed);
            }
            self.remaining_secs -= consumed;
            if self.remaining_secs == 0 {
                self.transition_phase(ctx, true);
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
            "{}{}",
            self.phase.display_name(),
            if self.running { "" } else { " (PAUSED)" }
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
                self.toggle(ctx);
                true
            }
            KeyCode::Char('r') => {
                self.reset(ctx);
                true
            }
            KeyCode::Char('n') => {
                self.transition_phase(ctx, false);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn running_pomodoro(ctx: &RenderContext) -> PomodoroMode {
        let mut pomodoro = PomodoroMode::new(ctx.config.pomodoro_work_mins, 0, 0);
        pomodoro.toggle(ctx);
        pomodoro
    }

    // Tests back-date `last_tick` by seconds only: on Windows `Instant` counts from
    // boot, and a freshly started CI runner may not have minutes to subtract.

    #[test]
    fn focus_time_counts_only_seconds_the_work_timer_ran() {
        let mut ctx = RenderContext::for_test();
        let mut pomodoro = running_pomodoro(&ctx);
        pomodoro.last_tick = Instant::now() - Duration::from_millis(2500);
        pomodoro.tick(&mut ctx);

        // Time spent paused adds nothing; only ticks while running count.
        pomodoro.toggle(&ctx);
        pomodoro.last_tick = Instant::now() - Duration::from_millis(2500);
        pomodoro.tick(&mut ctx);

        assert_eq!(pomodoro.total_focus_secs, 2);
        assert_eq!(pomodoro.remaining_secs, 25 * 60 - 2);
    }

    #[test]
    fn a_completed_work_phase_counts_as_a_session_and_a_skipped_one_does_not() {
        let mut ctx = RenderContext::for_test();
        ctx.config.pomodoro_sessions_until_long = 1;

        let mut skipped = running_pomodoro(&ctx);
        skipped.transition_phase(&mut ctx, false);
        assert_eq!(skipped.sessions_completed, 0);
        assert_eq!(skipped.phase, PomodoroPhase::ShortBreak);

        let mut finished = running_pomodoro(&ctx);
        finished.remaining_secs = 2;
        finished.last_tick = Instant::now() - Duration::from_millis(2500);
        finished.tick(&mut ctx);
        assert_eq!(finished.sessions_completed, 1);
        assert_eq!(finished.total_focus_secs, 2);
        assert_eq!(finished.phase, PomodoroPhase::LongBreak);
        assert!(!finished.started);
    }

    #[test]
    fn settings_resize_a_phase_only_until_it_starts() {
        let mut ctx = RenderContext::for_test();
        let mut pomodoro = PomodoroMode::new(25, 0, 0);
        ctx.config.pomodoro_work_mins = 50;
        pomodoro.sync_duration(&ctx);
        assert_eq!(pomodoro.remaining_secs, 50 * 60);

        pomodoro.toggle(&ctx);
        pomodoro.toggle(&ctx);
        ctx.config.pomodoro_work_mins = 15;
        pomodoro.sync_duration(&ctx);
        assert_eq!(pomodoro.remaining_secs, 50 * 60);
    }

    #[cfg(unix)]
    #[test]
    fn on_start_runs_once_per_work_phase_not_on_every_resume() {
        let log = std::env::temp_dir().join(format!("sigye-on-start-{}", std::process::id()));
        let _ = std::fs::remove_file(&log);
        let mut ctx = RenderContext::for_test();
        ctx.config.on_start = Some(format!("echo started >> '{}'", log.display()));

        let mut pomodoro = running_pomodoro(&ctx);
        pomodoro.toggle(&ctx);
        pomodoro.toggle(&ctx);

        let lines = || std::fs::read_to_string(&log).map_or(0, |text| text.lines().count());
        let deadline = Instant::now() + Duration::from_secs(5);
        while lines() == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(lines(), 1);
        std::fs::remove_file(&log).unwrap();
    }
}
