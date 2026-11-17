//! Life countdown display mode — counts down to (or up from) configured events.

use std::any::Any;

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};
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

    // If target is the same calendar day, surface "today" for emotional clarity.
    if !event.since && target.date_only && now.date_naive() == target_dt.date_naive() {
        return EventView {
            big_text: "TODAY".to_string(),
            status: event.name.clone(),
            target_label: date_label,
        };
    }

    // The direction shown follows the clock, not the `since` flag: a target still
    // ahead reads "until" and one already behind reads "since", whichever was set.
    let remaining = (target_dt - now).num_seconds();
    let direction = if remaining > 0 || (remaining == 0 && !event.since) {
        "until"
    } else {
        "since"
    };
    let secs = remaining.abs();

    let (big_text, status) = if secs >= 86_400 {
        let days = secs / 86_400;
        let unit = if days == 1 { "day" } else { "days" };
        (
            days.to_string(),
            format!("{unit} {direction} {}", event.name),
        )
    } else {
        // Under 24h: show HH:MM:SS for tangible urgency.
        let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
        (
            format!("{h:02}:{m:02}:{s:02}"),
            format!("{direction} {}", event.name),
        )
    };
    EventView {
        big_text,
        status,
        target_label: date_label,
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

        // Dated next year so the "until" examples never read as already past.
        let next_year = Local::now().year() + 1;
        let examples = [
            ("Launch", format!("{next_year}-06-01"), "until"),
            ("Birthday", format!("{next_year}-09-15"), "until"),
            ("Trip to Tokyo", format!("{next_year}-08-04"), "until"),
            ("Sober", "2023-02-14".to_string(), "since"),
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
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::dialog::test_helpers::contains_text;

    #[test]
    fn onboarding_hides_settings_hint_in_screensaver_mode() {
        let mut terminal = Terminal::new(TestBackend::new(100, 35)).unwrap();
        let mode = CountdownMode::new();
        let mut ctx = RenderContext::for_test();
        ctx.screensaver_mode = true;

        terminal.draw(|frame| mode.render(frame, &ctx)).unwrap();

        assert!(!contains_text(terminal.backend(), "[s] settings"));
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
        // A date 100 days out; exact count depends on now, so just check format is digits.
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
    fn direction_follows_the_clock_whichever_way_the_event_was_configured() {
        // RFC 3339 targets are exact instants, so DST changes cannot shift the result,
        // and every offset sits well clear of an hour or day boundary.
        let view = |offset: chrono::Duration, since: bool| {
            compute_event_view(&CountdownEvent {
                name: "X".into(),
                target: (Local::now() + offset).to_rfc3339(),
                since,
            })
        };
        let hours = chrono::Duration::minutes(5 * 60 + 30);
        let days = chrono::Duration::hours(3 * 24 + 12);

        for since in [false, true] {
            // Under a day: HH:MM:SS toward or away from the target.
            assert!(view(hours, since).big_text.starts_with("05:"));
            assert_eq!(view(hours, since).status, "until X");
            assert!(view(-hours, since).big_text.starts_with("05:"));
            assert_eq!(view(-hours, since).status, "since X");
            // A day or more: whole days.
            assert_eq!(view(days, since).big_text, "3");
            assert_eq!(view(days, since).status, "days until X");
            assert_eq!(view(-days, since).status, "days since X");
        }
        assert_eq!(
            view(-chrono::Duration::hours(30), false).status,
            "day since X"
        );
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
}
