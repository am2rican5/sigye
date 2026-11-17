//! Shared scaffolding for centered, bordered overlay dialogs.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
};

/// Center a `width` x `height` rect within `area`.
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Shrink `area` horizontally by `padding` on each side (vertical extent unchanged).
pub fn inset(area: Rect, padding: u16) -> Rect {
    Rect::new(
        area.x + padding,
        area.y,
        area.width.saturating_sub(padding * 2),
        area.height,
    )
}

/// The standard bordered dialog block: centered title, all borders in `accent`,
/// white-on-black content style for light-theme support.
pub fn dialog_block(title: &'static str, accent: Color) -> Block<'static> {
    Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .style(Style::default().fg(Color::White).bg(Color::Black))
}

/// Shared assertion helpers for dialog render tests.
#[cfg(test)]
pub mod test_helpers {
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;

    /// Find the first cell where `text` starts and return its foreground color.
    pub fn color_of_text(backend: &TestBackend, text: &str) -> Option<Color> {
        let buffer = backend.buffer();
        let area = buffer.area;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if text_matches_at(backend, x, y, text) {
                    return buffer.cell((x, y)).map(|cell| cell.fg);
                }
            }
        }
        None
    }

    /// Whether `text` is rendered anywhere in the buffer.
    pub fn contains_text(backend: &TestBackend, text: &str) -> bool {
        color_of_text(backend, text).is_some()
    }

    /// Whether `text` is rendered starting at cell `(x, y)`.
    pub fn text_matches_at(backend: &TestBackend, x: u16, y: u16, text: &str) -> bool {
        let buffer = backend.buffer();
        text.chars().enumerate().all(|(offset, ch)| {
            buffer
                .cell((x + offset as u16, y))
                .is_some_and(|cell| cell.symbol() == ch.to_string())
        })
    }
}
