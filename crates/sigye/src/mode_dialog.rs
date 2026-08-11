//! Mode selection dialog — pick a display mode from a list.

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use sigye_core::DisplayMode;

use crate::dialog::{centered_rect, dialog_block};
use crate::render::{FooterAction, FooterButton, footer_action_at, footer_layout};

/// All modes shown in the picker, in display order.
const MODES: &[DisplayMode] = &[
    DisplayMode::Clock,
    DisplayMode::Pomodoro,
    DisplayMode::Timer,
    DisplayMode::Stopwatch,
    DisplayMode::WorldClock,
    DisplayMode::Countdown,
];

struct ModeDialogLayout {
    dialog_area: Rect,
    list_area: Rect,
    footer_area: Rect,
}

fn dialog_layout(area: Rect) -> ModeDialogLayout {
    let dialog_width = 36u16.min(area.width.saturating_sub(4));
    let dialog_height = (MODES.len() as u16 + 6).min(area.height.saturating_sub(2));
    let dialog_area = centered_rect(area, dialog_width, dialog_height);
    let inner = dialog_block(" Mode ", Color::Reset).inner(dialog_area);
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    ModeDialogLayout {
        dialog_area,
        list_area: Rect::new(
            chunks[1].x + 2,
            chunks[1].y,
            chunks[1].width.saturating_sub(4),
            chunks[1].height,
        ),
        footer_area: chunks[3],
    }
}

fn mode_dialog_buttons(area: Rect) -> Vec<FooterButton> {
    footer_layout(area, &[FooterAction::new(KeyCode::Esc, "Esc", "cancel")])
}

/// Outcome of a key event in the dialog.
pub enum ModeAction {
    /// Stay open.
    Continue,
    /// User picked a mode (Enter).
    Select(DisplayMode),
    /// User dismissed (Esc).
    Cancel,
}

/// Mode selection dialog state.
#[derive(Debug, Default)]
pub struct ModeDialog {
    pub visible: bool,
    /// Index into `MODES` of the highlighted entry.
    selected: usize,
}

impl ModeDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dialog with the cursor on `current`.
    pub fn open(&mut self, current: DisplayMode) {
        self.visible = true;
        self.selected = MODES.iter().position(|m| *m == current).unwrap_or(0);
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Process a key event. Caller acts on the returned action.
    pub fn handle_key(&mut self, key: KeyEvent) -> ModeAction {
        match key.code {
            KeyCode::Esc => {
                self.close();
                ModeAction::Cancel
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let picked = MODES[self.selected];
                self.close();
                ModeAction::Select(picked)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = if self.selected == 0 {
                    MODES.len() - 1
                } else {
                    self.selected - 1
                };
                ModeAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % MODES.len();
                ModeAction::Continue
            }
            // Number-key shortcuts: 1-6 jumps directly.
            KeyCode::Char(c @ '1'..='9') => {
                let idx = (c as usize) - ('1' as usize);
                if idx < MODES.len() {
                    let picked = MODES[idx];
                    self.close();
                    return ModeAction::Select(picked);
                }
                ModeAction::Continue
            }
            _ => ModeAction::Continue,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> ModeAction {
        let layout = dialog_layout(area);
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            MouseEventKind::ScrollDown => {
                self.selected = (self.selected + 1).min(MODES.len() - 1);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if layout.list_area.contains((mouse.column, mouse.row).into()) {
                    let index = (mouse.row - layout.list_area.y) as usize;
                    if let Some(&mode) = MODES.get(index) {
                        self.close();
                        return ModeAction::Select(mode);
                    }
                } else if footer_action_at(
                    &mode_dialog_buttons(layout.footer_area),
                    mouse.column,
                    mouse.row,
                ) == Some(KeyCode::Esc)
                {
                    self.close();
                    return ModeAction::Cancel;
                }
            }
            _ => {}
        }
        ModeAction::Continue
    }

    /// Render the dialog. `current` marks the active mode with a dot.
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        accent: Color,
        _dim: Color,
        muted: Color,
        current: DisplayMode,
    ) {
        if !self.visible {
            return;
        }

        let layout = dialog_layout(area);
        let dialog_area = layout.dialog_area;

        frame.render_widget(Clear, dialog_area);

        let block = dialog_block(" Mode ", accent);
        frame.render_widget(block, dialog_area);

        // Build list lines
        let mut lines: Vec<Line> = Vec::with_capacity(MODES.len());
        for (i, mode) in MODES.iter().enumerate() {
            let is_selected = i == self.selected;
            let is_current = *mode == current;
            let marker = if is_selected { "►" } else { " " };
            let active = if is_current { " ●" } else { "  " };
            let number = format!(" {} ", i + 1);
            let label = format!("{}{}{}", marker, number, mode.display_name());

            let style = if is_selected {
                Style::default().fg(accent).bold()
            } else if is_current {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(muted)
            };

            lines.push(Line::from(vec![
                Span::styled(label, style),
                Span::styled(active.to_string(), Style::default().fg(accent)),
            ]));
        }

        frame.render_widget(Paragraph::new(lines), layout.list_area);

        for button in mode_dialog_buttons(layout.footer_area) {
            let text = Line::from(vec![
                Span::styled(
                    format!("[{}]", button.action.key_label),
                    Style::default().fg(accent).bold(),
                ),
                Span::styled(
                    format!(" {}", button.action.label),
                    Style::default().fg(muted),
                ),
            ]);
            frame.render_widget(Paragraph::new(text), button.area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::test_helpers::color_of_text;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn clicking_a_mode_row_selects_it() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let layout = dialog_layout(Rect::new(0, 0, 50, 16));

        let action = d.handle_mouse(
            click(layout.list_area.x, layout.list_area.y + 1),
            Rect::new(0, 0, 50, 16),
        );

        assert!(matches!(action, ModeAction::Select(DisplayMode::Pomodoro)));
        assert!(!d.visible);
    }

    #[test]
    fn only_the_visible_mode_cancel_button_cancels() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let area = Rect::new(0, 0, 50, 16);
        let layout = dialog_layout(area);
        let button = mode_dialog_buttons(layout.footer_area)[0];

        let blank = d.handle_mouse(click(layout.footer_area.x, layout.footer_area.y), area);
        assert!(matches!(blank, ModeAction::Continue));
        assert!(d.visible);

        let cancel = d.handle_mouse(click(button.area.x, button.area.y), area);
        assert!(matches!(cancel, ModeAction::Cancel));
        assert!(!d.visible);
    }

    #[test]
    fn opens_with_cursor_on_current() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Timer);
        assert!(d.visible);
        assert_eq!(MODES[d.selected], DisplayMode::Timer);
    }

    #[test]
    fn enter_selects_and_closes() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let action = d.handle_key(key(KeyCode::Down));
        assert!(matches!(action, ModeAction::Continue));
        let action = d.handle_key(key(KeyCode::Enter));
        match action {
            ModeAction::Select(m) => assert_eq!(m, DisplayMode::Pomodoro),
            _ => panic!("expected Select"),
        }
        assert!(!d.visible);
    }

    #[test]
    fn esc_cancels() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let action = d.handle_key(key(KeyCode::Esc));
        assert!(matches!(action, ModeAction::Cancel));
        assert!(!d.visible);
    }

    #[test]
    fn number_shortcut_jumps() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let action = d.handle_key(key(KeyCode::Char('6')));
        match action {
            ModeAction::Select(m) => assert_eq!(m, DisplayMode::Countdown),
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn up_wraps() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        d.handle_key(key(KeyCode::Up));
        assert_eq!(MODES[d.selected], DisplayMode::Countdown);
    }

    #[test]
    fn render_uses_muted_for_secondary_help_text() {
        let mut terminal = Terminal::new(TestBackend::new(50, 16)).unwrap();
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let accent = Color::Red;
        let dim = Color::Rgb(10, 20, 30);
        let muted = Color::Rgb(40, 50, 60);

        terminal
            .draw(|frame| d.render(frame, frame.area(), accent, dim, muted, DisplayMode::Clock))
            .unwrap();

        assert_eq!(color_of_text(terminal.backend(), " cancel"), Some(muted));
    }
}
