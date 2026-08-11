//! Shared rendering helpers for display modes.

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
};
use sigye_core::{AnimationSpeed, AnimationStyle, ColorTheme, apply_animation, is_colon_visible};
use sigye_fonts::Font;

use crate::context::RenderContext;

const FOOTER_GAP: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterAction {
    pub key_code: KeyCode,
    pub key_label: &'static str,
    pub label: &'static str,
}

impl FooterAction {
    pub const fn new(key_code: KeyCode, key_label: &'static str, label: &'static str) -> Self {
        Self {
            key_code,
            key_label,
            label,
        }
    }

    fn width(self) -> u16 {
        (self.key_label.chars().count() + self.label.chars().count() + 3) as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterButton {
    pub area: Rect,
    pub action: FooterAction,
}

fn footer_rows(width: u16, actions: &[FooterAction]) -> Vec<Vec<FooterAction>> {
    let mut rows: Vec<Vec<FooterAction>> = Vec::new();
    for &action in actions {
        let action_width = action.width().min(width);
        let fits = rows.last().is_some_and(|row| {
            let used = row.iter().map(|item| item.width()).sum::<u16>()
                + FOOTER_GAP * row.len().saturating_sub(1) as u16;
            used + FOOTER_GAP + action_width <= width
        });
        if fits {
            rows.last_mut().expect("row exists").push(action);
        } else {
            rows.push(vec![action]);
        }
    }
    rows
}

pub fn footer_height(width: u16, actions: &[FooterAction]) -> u16 {
    footer_rows(width, actions).len() as u16
}

pub fn footer_layout(area: Rect, actions: &[FooterAction]) -> Vec<FooterButton> {
    let rows = footer_rows(area.width, actions);
    let start_y = area.y + area.height.saturating_sub(rows.len() as u16);
    let mut buttons = Vec::with_capacity(actions.len());

    for (row_index, row) in rows.into_iter().enumerate() {
        let row_width = row.iter().map(|action| action.width()).sum::<u16>()
            + FOOTER_GAP * row.len().saturating_sub(1) as u16;
        let mut x = area.x + area.width.saturating_sub(row_width) / 2;
        for action in row {
            let width = action.width().min(area.right().saturating_sub(x));
            buttons.push(FooterButton {
                area: Rect::new(x, start_y + row_index as u16, width, 1),
                action,
            });
            x = x.saturating_add(width + FOOTER_GAP);
        }
    }
    buttons
}

pub fn footer_action_at(buttons: &[FooterButton], column: u16, row: u16) -> Option<KeyCode> {
    buttons
        .iter()
        .find(|button| button.area.contains((column, row).into()))
        .map(|button| button.action.key_code)
}

pub fn with_global_footer_actions(actions: &[FooterAction]) -> Vec<FooterAction> {
    const GLOBAL: [FooterAction; 4] = [
        FooterAction::new(KeyCode::Char('M'), "M", "mode"),
        FooterAction::new(KeyCode::Char('s'), "s", "settings"),
        FooterAction::new(KeyCode::Char('?'), "?", "help"),
        FooterAction::new(KeyCode::Char('q'), "q", "quit"),
    ];

    actions
        .iter()
        .copied()
        .filter(|action| {
            !GLOBAL
                .iter()
                .any(|global| global.key_code == action.key_code)
        })
        .chain(GLOBAL)
        .collect()
}

pub fn render_footer(frame: &mut Frame, area: Rect, ctx: &RenderContext, actions: &[FooterAction]) {
    if ctx.screensaver_mode {
        return;
    }

    for button in footer_layout(area, actions) {
        let text = format!("[{}] {}", button.action.key_label, button.action.label);
        render_centered_text(frame, button.area, &text, ctx.dim_color());
    }
}

/// Parameters for rendering ASCII art text to the frame buffer.
pub struct AsciiTextParams {
    pub color_theme: ColorTheme,
    pub static_color: Color,
    pub animation_style: AnimationStyle,
    pub animation_speed: AnimationSpeed,
    pub elapsed_ms: u64,
    pub flash_intensity: f32,
    pub colon_blink: bool,
}

impl AsciiTextParams {
    pub fn from_ctx(ctx: &RenderContext, static_color: Color) -> Self {
        Self {
            color_theme: ctx.color_theme,
            static_color,
            animation_style: ctx.animation_style,
            animation_speed: ctx.animation_speed,
            elapsed_ms: ctx.elapsed_ms(),
            flash_intensity: ctx.flash_intensity,
            colon_blink: ctx.colon_blink,
        }
    }
}

/// Render FIGlet ASCII art text centered in the given area.
/// Writes directly to the frame buffer, skipping spaces to preserve background transparency.
/// Returns (width, height) of the rendered text in characters.
pub fn render_ascii_text(
    frame: &mut Frame,
    area: Rect,
    font: &Font,
    text: &str,
    params: &AsciiTextParams,
) -> (usize, usize) {
    let time_lines = font.render_text(text);
    let height = time_lines.len();
    let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

    let text_width = width as u16;
    let start_x = area.x + (area.width.saturating_sub(text_width)) / 2;

    let colon_positions: Vec<bool> = if params.colon_blink {
        let mut mask = vec![false; width];
        let mut x_pos = 0;
        for ch in text.chars() {
            let char_width = font.char_width(ch);
            if ch == ':' {
                for i in 0..char_width {
                    if x_pos + i < mask.len() {
                        mask[x_pos + i] = true;
                    }
                }
            }
            x_pos += char_width;
        }
        mask
    } else {
        vec![]
    };

    let buf = frame.buffer_mut();
    for (line_idx, line) in time_lines.iter().enumerate() {
        let y_pos = area.y + line_idx as u16;
        if y_pos >= area.y + area.height {
            break;
        }

        for (char_idx, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue;
            }

            let x_pos = start_x + char_idx as u16;
            if x_pos >= area.x + area.width {
                continue;
            }

            if params.colon_blink {
                let is_colon = colon_positions.get(char_idx).copied().unwrap_or(false);
                if is_colon && !is_colon_visible(params.elapsed_ms) {
                    continue;
                }
            }

            let base_color = if params.color_theme.is_dynamic() {
                params
                    .color_theme
                    .color_at_position(char_idx, line_idx, width, height)
            } else {
                params.static_color
            };

            let animated_color = apply_animation(
                base_color,
                params.animation_style,
                params.animation_speed,
                params.elapsed_ms,
                char_idx,
                width,
                params.flash_intensity,
            );

            if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                cell.set_char(ch);
                cell.set_fg(animated_color);
            }
        }
    }

    (width, height)
}

/// Render a single line of text centered in the given area, directly to buffer.
/// Skips spaces to preserve background transparency.
pub fn render_centered_text(frame: &mut Frame, area: Rect, text: &str, color: Color) {
    // Use char count (not byte len) so multi-byte characters like `─` (U+2500)
    // center correctly. Each rendered char is assumed to occupy one terminal cell.
    let text_width = text.chars().count() as u16;
    let start_x = area.x + (area.width.saturating_sub(text_width)) / 2;
    let y = area.y;

    let buf = frame.buffer_mut();
    for (char_idx, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        let x_pos = start_x + char_idx as u16;
        if x_pos >= area.x + area.width {
            continue;
        }
        if let Some(cell) = buf.cell_mut(Position::new(x_pos, y)) {
            cell.set_char(ch);
            cell.set_fg(color);
        }
    }
}

/// Render a text-based progress bar.
pub fn render_progress_bar(progress: f64, width: u16, accent: Color) -> Line<'static> {
    let bar_width = width as usize;
    if bar_width == 0 {
        return Line::from("");
    }
    let filled = ((progress * bar_width as f64).round() as usize).min(bar_width);
    let empty = bar_width - filled;

    let filled_str: String = "\u{2501}".repeat(filled);
    let empty_str: String = "\u{2500}".repeat(empty);

    Line::from(vec![
        Span::styled(filled_str, Style::default().fg(accent)),
        Span::styled(empty_str, Style::default().dark_gray()),
    ])
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use ratatui::{Terminal, backend::TestBackend};
    use sigye_config::Config;
    use sigye_fonts::FontRegistry;

    use super::*;

    #[test]
    fn footer_layout_wraps_actions_and_preserves_click_targets() {
        let actions = [
            FooterAction::new(KeyCode::Char('m'), "m", "mode"),
            FooterAction::new(KeyCode::Char('s'), "s", "settings"),
        ];
        let area = Rect::new(0, 5, 18, 2);

        let buttons = footer_layout(area, &actions);

        assert_eq!(footer_height(area.width, &actions), 2);
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0].area, Rect::new(5, 5, 8, 1));
        assert_eq!(buttons[1].area, Rect::new(3, 6, 12, 1));
        assert_eq!(footer_action_at(&buttons, 5, 5), Some(KeyCode::Char('m')));
        assert_eq!(footer_action_at(&buttons, 14, 6), Some(KeyCode::Char('s')));
        assert_eq!(footer_action_at(&buttons, 0, 5), None);
    }

    #[test]
    fn global_footer_actions_are_appended_once() {
        let actions = with_global_footer_actions(&[
            FooterAction::new(KeyCode::Char('s'), "s", "settings"),
            FooterAction::new(KeyCode::Char('r'), "r", "reset"),
        ]);

        assert_eq!(
            actions
                .iter()
                .map(|action| action.key_code)
                .collect::<Vec<_>>(),
            vec![
                KeyCode::Char('r'),
                KeyCode::Char('M'),
                KeyCode::Char('s'),
                KeyCode::Char('?'),
                KeyCode::Char('q'),
            ]
        );
    }

    #[test]
    fn footer_renders_the_clickable_action_text() {
        let mut terminal = Terminal::new(TestBackend::new(30, 2)).unwrap();
        let config = Config::default();
        let ctx = RenderContext {
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
            animation_start: std::time::Instant::now(),
            flash_intensity: 0.0,
            flash_start: None,
            screensaver_mode: false,
            desktop_notifications: false,
            sunrise_sunset: None,
        };
        let actions = [FooterAction::new(KeyCode::Char('r'), "r", "reset")];

        terminal
            .draw(|frame| render_footer(frame, frame.area(), &ctx, &actions))
            .unwrap();

        let rendered = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("[r] reset"));
    }
}
