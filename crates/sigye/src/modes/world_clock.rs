//! World Clock display mode.

use std::any::Any;
use std::str::FromStr;

use chrono::Utc;
use chrono_tz::Tz;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};
use sigye_core::{DisplayMode, TimeFormat};

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// World Clock display mode — shows multiple timezone clocks.
pub struct WorldClockMode {
    /// (label, timezone string) pairs.
    pub entries: Vec<(String, String)>,
}

impl WorldClockMode {
    /// Create a new `WorldClockMode` by parsing "Label=Timezone" entries.
    pub fn new(zones: &[String]) -> Self {
        Self {
            entries: parse_zones(zones),
        }
    }

    /// Re-parse entries when config changes.
    pub fn update_entries(&mut self, zones: &[String]) {
        self.entries = parse_zones(zones);
    }
}

/// Parse "Label=Timezone" strings into (label, tz) pairs. Invalid entries are skipped.
fn parse_zones(zones: &[String]) -> Vec<(String, String)> {
    zones
        .iter()
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.splitn(2, '=').collect();
            if parts.len() == 2 {
                let label = parts[0].trim().to_string();
                let tz_str = parts[1].trim().to_string();
                // Validate timezone
                if Tz::from_str(&tz_str).is_ok() {
                    Some((label, tz_str))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

impl Mode for WorldClockMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::WorldClock
    }

    fn update(&mut self, _ctx: &mut RenderContext) {
        // No-op: time is read fresh each frame
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let actions = self.all_footer_actions();
        let footer_height = render::footer_height(area.width, &actions);
        let root =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(footer_height)]).split(area);
        let content_area = root[0];

        if self.entries.is_empty() {
            render::render_centered_text(
                frame,
                content_area,
                "No world clock zones configured",
                ctx.dim_color(),
            );
            render::render_footer(frame, root[1], ctx, &actions);
            return;
        }

        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        // Each entry gets: Length(1) label + Length(font_height) time
        let entry_height = 1 + font_height;
        let n = self.entries.len() as u16;

        // Build constraints: fill top, then entries, fill bottom
        let mut constraints = vec![Constraint::Fill(1)];
        for _ in 0..n {
            constraints.push(Constraint::Length(entry_height));
        }
        constraints.push(Constraint::Fill(1));

        let chunks = Layout::vertical(constraints).split(content_area);

        let now_utc = Utc::now();

        let params = AsciiTextParams::from_ctx(ctx, ctx.color());

        for (i, (label, tz_str)) in self.entries.iter().enumerate() {
            let chunk_idx = 1 + i; // offset by the leading Fill(1)

            let entry_area = chunks[chunk_idx];

            // Split entry area into label row + clock rows
            let entry_chunks =
                Layout::vertical([Constraint::Length(1), Constraint::Length(font_height)])
                    .split(entry_area);

            // Render label
            render::render_centered_text(frame, entry_chunks[0], label, ctx.dim_color());

            // Render time in FIGlet font
            let time_str = if let Ok(tz) = Tz::from_str(tz_str) {
                let local_time = now_utc.with_timezone(&tz);
                match ctx.time_format {
                    TimeFormat::TwelveHour => local_time.format("%I:%M %p").to_string(),
                    TimeFormat::TwentyFourHour => local_time.format("%H:%M").to_string(),
                }
            } else {
                "??:??".to_string()
            };

            render::render_ascii_text(frame, entry_chunks[1], font, &time_str, &params);
        }

        render::render_footer(frame, root[1], ctx, &actions);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut RenderContext) -> bool {
        false
    }

    fn footer_actions(&self) -> Vec<render::FooterAction> {
        vec![
            render::FooterAction::new(KeyCode::Char('t'), "t", "12/24h"),
            render::FooterAction::new(KeyCode::Char('c'), "c", "color"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
