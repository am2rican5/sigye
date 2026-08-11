//! Life countdown display mode — counts down to (or up from) configured events.

use std::any::Any;

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use sigye_config::CountdownEvent;
use sigye_core::DisplayMode;

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Life countdown display mode — shows time until (or since) configured life events.
pub struct CountdownMode {
    /// Index of the currently shown event (wraps within `events.len()`).
    pub active_index: usize,
}

impl CountdownMode {
    pub fn new() -> Self {
        Self { active_index: 0 }
    }

    fn events<'a>(&self, ctx: &'a RenderContext) -> &'a [CountdownEvent] {
        &ctx.config.countdown_events
    }
}

impl Default for CountdownMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed countdown target resolved to a concrete local-time datetime.
struct Target {
    datetime: DateTime<Local>,
    /// Whether the source string had no time component (date-only).
    date_only: bool,
}

/// Returns `true` if `s` is a target date string the countdown mode can parse.
///
/// Used by the countdown event management dialog to validate user input.
pub(crate) fn validate_target(s: &str) -> bool {
    parse_target(s).is_some()
}

/// Parse an ISO 8601 date or datetime into a local-time `DateTime`.
///
/// Accepts:
/// - `YYYY-MM-DD` — interpreted as midnight local time
/// - `YYYY-MM-DDTHH:MM:SS` — interpreted as local time
/// - `YYYY-MM-DDTHH:MM:SS+09:00` (RFC 3339 with offset)
fn parse_target(s: &str) -> Option<Target> {
    let trimmed = s.trim();

    // Try full RFC 3339 with offset first.
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(Target {
            datetime: dt.with_timezone(&Local),
            date_only: false,
        });
    }

    // Try naive datetime (no offset) — interpret as local.
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Local
            .from_local_datetime(&naive)
            .single()
            .map(|datetime| Target {
                datetime,
                date_only: false,
            });
    }

    // Try naive date — interpret as midnight local.
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let naive = date.and_hms_opt(0, 0, 0)?;
        return Local
            .from_local_datetime(&naive)
            .single()
            .map(|datetime| Target {
                datetime,
                date_only: true,
            });
    }

    None
}

/// Rendered parts for a single countdown event.
struct EventView {
    /// Big text rendered as FIGlet art (e.g., `"42"`, `"07:14:22"`, `"0"`).
    big_text: String,
    /// Status label (e.g., `"days until Launch"`, `"today"`).
    status: String,
    /// Target date formatted for the footer.
    target_label: String,
}

fn compute_event_view(event: &CountdownEvent) -> EventView {
    let target = match parse_target(&event.target) {
        Some(t) => t,
        None => {
            return EventView {
                big_text: "?".to_string(),
                status: format!("invalid date for {}", event.name),
                target_label: event.target.clone(),
            };
        }
    };

    let now = Local::now();
    let target_dt = target.datetime;
    let date_label = if target.date_only {
        target_dt.format("%A, %B %-d, %Y").to_string()
    } else {
        target_dt.format("%A, %B %-d, %Y · %H:%M").to_string()
    };

    // For `since` events we reverse the frame: "now - target" positive after target, negative before.
    let (signed_secs, variant_since) = if event.since {
        ((now - target_dt).num_seconds(), true)
    } else {
        ((target_dt - now).num_seconds(), false)
    };

    // If target is the same calendar day, surface "today" for emotional clarity.
    if !variant_since && target.date_only && now.date_naive() == target_dt.date_naive() {
        return EventView {
            big_text: "TODAY".to_string(),
            status: event.name.clone(),
            target_label: date_label,
        };
    }

    if signed_secs >= 0 {
        // Future target in countdown mode, or elapsed time in since mode.
        if signed_secs >= 86_400 {
            let days = signed_secs / 86_400;
            let unit = if days == 1 { "day" } else { "days" };
            let status = if variant_since {
                format!("{unit} since {}", event.name)
            } else {
                format!("{unit} until {}", event.name)
            };
            EventView {
                big_text: days.to_string(),
                status,
                target_label: date_label,
            }
        } else {
            // Under 24h: show HH:MM:SS for tangible urgency.
            let h = signed_secs / 3600;
            let m = (signed_secs % 3600) / 60;
            let s = signed_secs % 60;
            let status = if variant_since {
                format!("since {}", event.name)
            } else {
                format!("until {}", event.name)
            };
            EventView {
                big_text: format!("{h:02}:{m:02}:{s:02}"),
                status,
                target_label: date_label,
            }
        }
    } else {
        // Countdown event already passed — flip the narrative.
        let elapsed = -signed_secs;
        let days = elapsed / 86_400;
        let unit = if days == 1 { "day" } else { "days" };
        let (big, status) = if variant_since {
            // Future date configured as "since" — treat as "starts in".
            if elapsed >= 86_400 {
                (days.to_string(), format!("{unit} until {}", event.name))
            } else {
                let h = elapsed / 3600;
                let m = (elapsed % 3600) / 60;
                let s = elapsed % 60;
                (
                    format!("{h:02}:{m:02}:{s:02}"),
                    format!("until {}", event.name),
                )
            }
        } else if elapsed >= 86_400 {
            (days.to_string(), format!("{unit} since {}", event.name))
        } else {
            let h = elapsed / 3600;
            let m = (elapsed % 3600) / 60;
            let s = elapsed % 60;
            (
                format!("{h:02}:{m:02}:{s:02}"),
                format!("since {}", event.name),
            )
        };
        EventView {
            big_text: big,
            status,
            target_label: date_label,
        }
    }
}

impl CountdownMode {
    /// Render the onboarding screen shown when no events are configured.
    ///
    /// Goal: make the first-run experience warm and self-explanatory — convey what
    /// the mode is for, show example use cases users can imagine themselves using,
    /// and surface the single key needed to get started.
    fn render_onboarding(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let actions = self.all_footer_actions();
        let footer_height = render::footer_height(area.width, &actions);
        let accent = ctx.color();
        let dim = ctx.dim_color();
        let muted = ctx.muted_color();

        let chunks = Layout::vertical([
            Constraint::Fill(1),               // [0] top spacer
            Constraint::Length(1),             // [1] decorative bar
            Constraint::Length(1),             // [2] title
            Constraint::Length(1),             // [3] decorative bar
            Constraint::Length(1),             // [4] gap
            Constraint::Length(1),             // [5] tagline
            Constraint::Length(2),             // [6] gap
            Constraint::Length(1),             // [7] examples header
            Constraint::Length(1),             // [8] gap
            Constraint::Length(4),             // [9] 4 example rows
            Constraint::Length(2),             // [10] gap
            Constraint::Length(1),             // [11] call-to-action
            Constraint::Fill(1),               // [12] bottom spacer
            Constraint::Length(footer_height), // [13] hints
        ])
        .split(area);

        let bar = "─".repeat(18);
        render::render_centered_text(frame, chunks[1], &bar, accent);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Life Countdown",
                Style::default().fg(accent).bold(),
            )))
            .alignment(Alignment::Center),
            chunks[2],
        );
        render::render_centered_text(frame, chunks[3], &bar, accent);

        render::render_centered_text(
            frame,
            chunks[5],
            "Count down to the moments that matter.",
            muted,
        );

        render::render_centered_text(frame, chunks[7], "── A few ideas ──", dim);

        let examples: [(&str, &str, &str); 4] = [
            ("Launch", "2026-06-01", "until"),
            ("Birthday", "2026-09-15", "until"),
            ("Trip to Tokyo", "2026-08-04", "until"),
            ("Sober", "2023-02-14", "since"),
        ];
        let example_lines: Vec<Line> = examples
            .iter()
            .map(|(name, target, kind)| {
                Line::from(vec![
                    Span::styled(format!("  {name:<18}"), Style::default().fg(muted)),
                    Span::styled(format!("{target:<14}"), Style::default().fg(muted)),
                    Span::styled(format!("({kind})"), Style::default().fg(dim)),
                ])
            })
            .collect();
        frame.render_widget(
            Paragraph::new(example_lines).alignment(Alignment::Center),
            chunks[9],
        );

        let cta = Line::from(vec![
            Span::styled("Press ", Style::default().fg(muted)),
            Span::styled("[e]", Style::default().fg(accent).bold()),
            Span::styled(" to add your first event", Style::default().fg(muted)),
        ]);
        frame.render_widget(Paragraph::new(cta).alignment(Alignment::Center), chunks[11]);

        render::render_footer(frame, chunks[13], ctx, &actions);
    }
}

impl Mode for CountdownMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Countdown
    }

    fn update(&mut self, ctx: &mut RenderContext) {
        ctx.decay_flash();

        // Keep active_index within bounds in case events list shrank.
        let n = self.events(ctx).len();
        if n == 0 {
            self.active_index = 0;
        } else if self.active_index >= n {
            self.active_index = n - 1;
        }
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let events = self.events(ctx);

        if events.is_empty() {
            self.render_onboarding(frame, ctx);
            return;
        }

        let event = &events[self.active_index.min(events.len() - 1)];
        let view = compute_event_view(event);
        let actions = self.all_footer_actions();
        let footer_height = render::footer_height(area.width, &actions);

        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        let chunks = Layout::vertical([
            Constraint::Fill(1),               // [0] top padding
            Constraint::Length(font_height),   // [1] big ASCII number
            Constraint::Length(1),             // [2] gap
            Constraint::Length(1),             // [3] status label
            Constraint::Length(1),             // [4] target date
            Constraint::Fill(1),               // [5] bottom padding
            Constraint::Length(1),             // [6] pager (n of N)
            Constraint::Length(footer_height), // [7] hints
        ])
        .split(area);

        let params = AsciiTextParams::from_ctx(ctx, ctx.color());

        render::render_ascii_text(frame, chunks[1], font, &view.big_text, &params);
        render::render_centered_text(frame, chunks[3], &view.status, ctx.color());
        render::render_centered_text(frame, chunks[4], &view.target_label, ctx.dim_color());

        if events.len() > 1 {
            let pager = format!("event {} of {}", self.active_index + 1, events.len());
            render::render_centered_text(frame, chunks[6], &pager, ctx.dim_color());
        }

        render::render_footer(frame, chunks[7], ctx, &actions);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut RenderContext) -> bool {
        let n = self.events(ctx).len();
        if n <= 1 {
            return false;
        }
        match key.code {
            KeyCode::Char('n') | KeyCode::Right => {
                self.active_index = (self.active_index + 1) % n;
                true
            }
            KeyCode::Char('p') | KeyCode::Left => {
                self.active_index = if self.active_index == 0 {
                    n - 1
                } else {
                    self.active_index - 1
                };
                true
            }
            _ => false,
        }
    }

    fn footer_actions(&self) -> Vec<render::FooterAction> {
        vec![
            render::FooterAction::new(KeyCode::Char('p'), "p", "previous"),
            render::FooterAction::new(KeyCode::Char('n'), "n", "next"),
            render::FooterAction::new(KeyCode::Char('e'), "e", "events"),
            render::FooterAction::new(KeyCode::Char('c'), "c", "color"),
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
    fn onboarding_hides_settings_hint_in_screensaver_mode() {
        let mut terminal = Terminal::new(TestBackend::new(100, 35)).unwrap();
        let mode = CountdownMode::new();
        let ctx = render_context(true);

        terminal.draw(|frame| mode.render(frame, &ctx)).unwrap();

        assert!(!buffer_contains_text(terminal.backend(), "[s] settings"));
    }

    #[test]
    fn parses_iso_date() {
        let t = parse_target("2026-06-01").expect("parses");
        assert!(t.date_only);
        assert_eq!(t.datetime.format("%Y-%m-%d").to_string(), "2026-06-01");
    }

    #[test]
    fn parses_iso_datetime_with_offset() {
        let t = parse_target("2026-06-01T09:00:00+09:00").expect("parses");
        assert!(!t.date_only);
    }

    #[test]
    fn parses_naive_datetime_as_local() {
        let t = parse_target("2026-06-01T09:00:00").expect("parses");
        assert!(!t.date_only);
    }

    #[test]
    fn rejects_invalid() {
        assert!(parse_target("not-a-date").is_none());
    }

    #[test]
    fn future_date_shows_days() {
        // A date ~365 days out; exact count depends on now, so just check format is digits.
        let far_future = Local::now()
            .date_naive()
            .checked_add_signed(chrono::Duration::days(100))
            .unwrap();
        let ev = CountdownEvent {
            name: "Launch".into(),
            target: far_future.format("%Y-%m-%d").to_string(),
            since: false,
        };
        let view = compute_event_view(&ev);
        assert!(view.big_text.chars().all(|c| c.is_ascii_digit()));
        assert!(view.status.contains("Launch"));
    }

    #[test]
    fn since_mode_counts_up() {
        let past = Local::now()
            .date_naive()
            .checked_sub_signed(chrono::Duration::days(42))
            .unwrap();
        let ev = CountdownEvent {
            name: "Sober".into(),
            target: past.format("%Y-%m-%d").to_string(),
            since: true,
        };
        let view = compute_event_view(&ev);
        // 42 or 41 depending on current time-of-day; both fine.
        assert!(view.status.contains("since Sober"));
    }

    #[test]
    fn today_is_surfaced() {
        let today = Local::now().date_naive();
        let ev = CountdownEvent {
            name: "Birthday".into(),
            target: today.format("%Y-%m-%d").to_string(),
            since: false,
        };
        let view = compute_event_view(&ev);
        assert_eq!(view.big_text, "TODAY");
        assert_eq!(view.status, "Birthday");
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

    fn text_matches_at(backend: &TestBackend, x: u16, y: u16, text: &str) -> bool {
        let buffer = backend.buffer();
        text.chars().enumerate().all(|(offset, ch)| {
            buffer
                .cell((x + offset as u16, y))
                .is_some_and(|cell| cell.symbol() == ch.to_string())
        })
    }
}
