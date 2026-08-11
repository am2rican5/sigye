//! Countdown event management dialog — list, add, edit, delete countdown events.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use sigye_config::CountdownEvent;

use crate::dialog::{centered_rect, dialog_block, inset};
use crate::modes::countdown::validate_target;
use crate::render::{FooterAction, FooterButton, footer_action_at, footer_layout};

/// Dialog outcome surfaced to the App's key handler.
pub enum CountdownAction {
    /// Stay open.
    Continue,
    /// User pressed Enter from the list view to commit edits and close.
    Commit(Vec<CountdownEvent>),
    /// User pressed Esc from the list view to discard edits and close.
    Cancel,
}

/// Active sub-view of the dialog.
#[derive(Debug, Default, PartialEq, Eq)]
enum View {
    #[default]
    List,
    Edit,
}

/// Field within the edit form.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FormField {
    Name,
    Target,
    Since,
}

/// Buffered edit form state.
struct EditForm {
    field: FormField,
    name: String,
    target: String,
    since: bool,
    /// `None` = adding a new event; `Some(idx)` = editing the event at that index.
    editing_index: Option<usize>,
}

const NAME_MAX: usize = 40;
const TARGET_MAX: usize = 32;

struct CountdownListAreas {
    dialog_area: Rect,
    list_area: Rect,
    footer_area: Rect,
}

fn countdown_list_areas(area: Rect, event_count: usize) -> CountdownListAreas {
    let dialog_width = 60u16.min(area.width.saturating_sub(4));
    let row_count = event_count.clamp(3, 10) as u16;
    let dialog_height = (row_count + 5).min(area.height.saturating_sub(2));
    let dialog_area = centered_rect(area, dialog_width, dialog_height);
    let inner = dialog_block(" Countdown Events ", Color::Reset).inner(dialog_area);
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .split(inner);

    CountdownListAreas {
        dialog_area,
        list_area: inset(chunks[1], 2),
        footer_area: chunks[2],
    }
}

fn countdown_list_buttons(area: Rect) -> Vec<FooterButton> {
    footer_layout(
        area,
        &[
            FooterAction::new(KeyCode::Char('a'), "a", "add"),
            FooterAction::new(KeyCode::Char('e'), "e", "edit"),
            FooterAction::new(KeyCode::Char('d'), "d", "delete"),
            FooterAction::new(KeyCode::Enter, "Enter", "save"),
            FooterAction::new(KeyCode::Esc, "Esc", "cancel"),
        ],
    )
}

struct CountdownEditAreas {
    dialog_area: Rect,
    name_area: Rect,
    target_area: Rect,
    hint_area: Rect,
    since_area: Rect,
    error_area: Rect,
    footer_area: Rect,
}

fn countdown_edit_areas(area: Rect) -> CountdownEditAreas {
    let dialog_width = 56u16.min(area.width.saturating_sub(4));
    let dialog_height = 13u16.min(area.height.saturating_sub(2));
    let dialog_area = centered_rect(area, dialog_width, dialog_height);
    let inner = dialog_block(" Edit Event ", Color::Reset).inner(dialog_area);
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .split(inner);

    CountdownEditAreas {
        dialog_area,
        name_area: inset(chunks[1], 2),
        target_area: inset(chunks[2], 2),
        hint_area: inset(chunks[3], 2),
        since_area: inset(chunks[4], 2),
        error_area: chunks[6],
        footer_area: chunks[8],
    }
}

fn countdown_edit_buttons(area: Rect) -> Vec<FooterButton> {
    footer_layout(
        area,
        &[
            FooterAction::new(KeyCode::Enter, "Enter", "commit"),
            FooterAction::new(KeyCode::Esc, "Esc", "back"),
        ],
    )
}

/// Countdown event management dialog state.
#[derive(Default)]
pub struct CountdownDialog {
    pub visible: bool,
    view: View,
    /// Working buffer of events; only flushed back to config on Commit.
    events: Vec<CountdownEvent>,
    selected: usize,
    scroll_offset: usize,
    visible_rows: usize,
    form: Option<EditForm>,
    /// Last validation error (cleared when the user types).
    error: Option<String>,
}

impl CountdownDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dialog with a snapshot of the current events.
    pub fn open(&mut self, events: Vec<CountdownEvent>) {
        self.visible = true;
        self.view = View::List;
        let n = events.len();
        self.events = events;
        self.selected = if n == 0 { 0 } else { self.selected.min(n - 1) };
        self.scroll_offset = 0;
        self.form = None;
        self.error = None;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.form = None;
        self.error = None;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> CountdownAction {
        match self.view {
            View::List => self.handle_list_key(key),
            View::Edit => self.handle_edit_key(key),
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> CountdownAction {
        if self.view == View::List && !self.events.is_empty() {
            let visible_rows = countdown_list_areas(area, self.events.len())
                .list_area
                .height;
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.selected = self.selected.saturating_sub(1);
                    self.ensure_selection_visible(visible_rows as usize);
                    return CountdownAction::Continue;
                }
                MouseEventKind::ScrollDown => {
                    self.selected = (self.selected + 1).min(self.events.len() - 1);
                    self.ensure_selection_visible(visible_rows as usize);
                    return CountdownAction::Continue;
                }
                _ => {}
            }
        }
        if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
            match self.view {
                View::List => {
                    let areas = countdown_list_areas(area, self.events.len());
                    if let Some(code) = footer_action_at(
                        &countdown_list_buttons(areas.footer_area),
                        mouse.column,
                        mouse.row,
                    ) {
                        return self.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
                    }
                    if areas.list_area.contains((mouse.column, mouse.row).into()) {
                        let index = self.scroll_offset + (mouse.row - areas.list_area.y) as usize;
                        if index < self.events.len() {
                            self.selected = index;
                        }
                    }
                }
                View::Edit => {
                    let areas = countdown_edit_areas(area);
                    if let Some(code) = footer_action_at(
                        &countdown_edit_buttons(areas.footer_area),
                        mouse.column,
                        mouse.row,
                    ) {
                        return self.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
                    }
                    if let Some(form) = self.form.as_mut() {
                        if areas.name_area.contains((mouse.column, mouse.row).into()) {
                            form.field = FormField::Name;
                        } else if areas.target_area.contains((mouse.column, mouse.row).into()) {
                            form.field = FormField::Target;
                        } else if areas.since_area.contains((mouse.column, mouse.row).into()) {
                            form.field = FormField::Since;
                            form.since = !form.since;
                        }
                    }
                }
            }
        }
        CountdownAction::Continue
    }

    fn ensure_selection_visible(&mut self, visible_rows: usize) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected + 1 - visible_rows;
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> CountdownAction {
        match key.code {
            KeyCode::Esc => {
                self.close();
                CountdownAction::Cancel
            }
            KeyCode::Enter => {
                let events = std::mem::take(&mut self.events);
                self.close();
                CountdownAction::Commit(events)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.events.is_empty() {
                    self.selected = if self.selected == 0 {
                        self.events.len() - 1
                    } else {
                        self.selected - 1
                    };
                    self.ensure_selection_visible(self.visible_rows.max(1));
                }
                CountdownAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.events.is_empty() {
                    self.selected = (self.selected + 1) % self.events.len();
                    self.ensure_selection_visible(self.visible_rows.max(1));
                }
                CountdownAction::Continue
            }
            KeyCode::Char('a') | KeyCode::Char('+') => {
                self.start_add();
                CountdownAction::Continue
            }
            KeyCode::Char('e') => {
                if !self.events.is_empty() {
                    self.start_edit(self.selected);
                }
                CountdownAction::Continue
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if !self.events.is_empty() {
                    self.events.remove(self.selected);
                    if self.events.is_empty() {
                        self.selected = 0;
                    } else if self.selected >= self.events.len() {
                        self.selected = self.events.len() - 1;
                    }
                    self.ensure_selection_visible(self.visible_rows.max(1));
                }
                CountdownAction::Continue
            }
            _ => CountdownAction::Continue,
        }
    }

    fn start_add(&mut self) {
        self.form = Some(EditForm {
            field: FormField::Name,
            name: String::new(),
            target: String::new(),
            since: false,
            editing_index: None,
        });
        self.view = View::Edit;
        self.error = None;
    }

    fn start_edit(&mut self, idx: usize) {
        let ev = &self.events[idx];
        self.form = Some(EditForm {
            field: FormField::Name,
            name: ev.name.clone(),
            target: ev.target.clone(),
            since: ev.since,
            editing_index: Some(idx),
        });
        self.view = View::Edit;
        self.error = None;
    }

    fn handle_edit_key(&mut self, key: KeyEvent) -> CountdownAction {
        // Take ownership briefly to avoid borrow gymnastics for self.error.
        let mut form = match self.form.take() {
            Some(f) => f,
            None => return CountdownAction::Continue,
        };

        let action = self.process_edit_key(&mut form, key);
        // The form is retained only if the dialog is still in Edit view.
        // Esc/Enter transitions to List view and leaves the form discarded.
        if self.view == View::Edit {
            self.form = Some(form);
        }
        action
    }

    /// Apply `key` to the edit form. Returns the dialog action.
    fn process_edit_key(&mut self, form: &mut EditForm, key: KeyEvent) -> CountdownAction {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
                self.error = None;
                return CountdownAction::Continue;
            }
            KeyCode::Tab | KeyCode::Down => {
                form.field = match form.field {
                    FormField::Name => FormField::Target,
                    FormField::Target => FormField::Since,
                    FormField::Since => FormField::Name,
                };
                return CountdownAction::Continue;
            }
            KeyCode::BackTab | KeyCode::Up => {
                form.field = match form.field {
                    FormField::Name => FormField::Since,
                    FormField::Target => FormField::Name,
                    FormField::Since => FormField::Target,
                };
                return CountdownAction::Continue;
            }
            KeyCode::Enter => {
                let name = form.name.trim();
                if name.is_empty() {
                    self.error = Some("Name cannot be empty.".into());
                    form.field = FormField::Name;
                    return CountdownAction::Continue;
                }
                let target = form.target.trim();
                if !validate_target(target) {
                    self.error =
                        Some("Target must be YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS, or RFC 3339.".into());
                    form.field = FormField::Target;
                    return CountdownAction::Continue;
                }
                let new_event = CountdownEvent {
                    name: name.to_string(),
                    target: target.to_string(),
                    since: form.since,
                };
                if let Some(idx) = form.editing_index {
                    if idx < self.events.len() {
                        self.events[idx] = new_event;
                    } else {
                        // The list shrank; treat as add.
                        self.events.push(new_event);
                        self.selected = self.events.len() - 1;
                    }
                } else {
                    self.events.push(new_event);
                    self.selected = self.events.len() - 1;
                }
                self.view = View::List;
                self.error = None;
                return CountdownAction::Continue;
            }
            _ => {}
        }

        // Field-specific input.
        match form.field {
            FormField::Name => match key.code {
                KeyCode::Backspace => {
                    form.name.pop();
                    self.error = None;
                }
                KeyCode::Char(c)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && form.name.chars().count() < NAME_MAX =>
                {
                    form.name.push(c);
                    self.error = None;
                }
                _ => {}
            },
            FormField::Target => match key.code {
                KeyCode::Backspace => {
                    form.target.pop();
                    self.error = None;
                }
                KeyCode::Char(c)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && form.target.chars().count() < TARGET_MAX =>
                {
                    form.target.push(c);
                    self.error = None;
                }
                _ => {}
            },
            FormField::Since => match key.code {
                KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right => {
                    form.since = !form.since;
                }
                _ => {}
            },
        }
        CountdownAction::Continue
    }

    /// Render the dialog (list or edit view depending on state).
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        accent: Color,
        dim: Color,
        muted: Color,
    ) {
        if !self.visible {
            return;
        }
        match self.view {
            View::List => self.render_list(frame, area, accent, dim, muted),
            View::Edit => self.render_edit(frame, area, accent, dim, muted),
        }
    }

    fn render_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        accent: Color,
        dim: Color,
        muted: Color,
    ) {
        let areas = countdown_list_areas(area, self.events.len());
        let dialog_area = areas.dialog_area;
        frame.render_widget(Clear, dialog_area);

        let block = dialog_block(" Countdown Events ", accent);
        frame.render_widget(block, dialog_area);

        let list_area = areas.list_area;
        self.visible_rows = list_area.height as usize;
        self.ensure_selection_visible(self.visible_rows.max(1));
        if self.events.is_empty() {
            let lines = vec![
                Line::from(""),
                Line::from(Span::styled("No events yet.", Style::default().fg(muted)))
                    .alignment(Alignment::Center),
                Line::from(Span::styled(
                    "Press [a] to add your first one.",
                    Style::default().fg(dim),
                ))
                .alignment(Alignment::Center),
            ];
            frame.render_widget(Paragraph::new(lines), list_area);
        } else {
            let mut lines: Vec<Line> = Vec::with_capacity(list_area.height as usize);
            for (i, ev) in self
                .events
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .take(list_area.height as usize)
            {
                let is_selected = i == self.selected;
                let marker = if is_selected { "►" } else { " " };
                let direction = if ev.since { "since" } else { "until" };
                let main = format!(" {marker} {:<20.20}  {:<22.22}", ev.name, ev.target);
                let style = if is_selected {
                    Style::default().fg(accent).bold()
                } else {
                    Style::default().fg(muted)
                };
                lines.push(Line::from(vec![
                    Span::styled(main, style),
                    Span::styled(format!("  {direction}"), Style::default().fg(dim)),
                ]));
            }
            frame.render_widget(Paragraph::new(lines), list_area);
        }

        for button in countdown_list_buttons(areas.footer_area) {
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

    fn render_edit(&self, frame: &mut Frame, area: Rect, accent: Color, dim: Color, muted: Color) {
        let form = match self.form.as_ref() {
            Some(f) => f,
            None => return,
        };

        let areas = countdown_edit_areas(area);
        let dialog_area = areas.dialog_area;
        frame.render_widget(Clear, dialog_area);

        let title = if form.editing_index.is_some() {
            " Edit Event "
        } else {
            " Add Event "
        };
        let block = dialog_block(title, accent);
        frame.render_widget(block, dialog_area);

        frame.render_widget(
            Paragraph::new(self.render_text_field(
                "Name   ",
                &form.name,
                form.field == FormField::Name,
                accent,
                muted,
            )),
            areas.name_area,
        );
        frame.render_widget(
            Paragraph::new(self.render_text_field(
                "Target ",
                &form.target,
                form.field == FormField::Target,
                accent,
                muted,
            )),
            areas.target_area,
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "         e.g. 2026-06-01  or  2026-06-01T09:00:00+09:00",
                Style::default().fg(dim),
            ))),
            areas.hint_area,
        );
        frame.render_widget(
            Paragraph::new(self.render_toggle_field(
                "Since  ",
                form.since,
                form.field == FormField::Since,
                accent,
                muted,
            )),
            areas.since_area,
        );

        if let Some(err) = &self.error {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    err.clone(),
                    Style::default().fg(Color::LightRed),
                )))
                .alignment(Alignment::Center),
                areas.error_area,
            );
        }

        for button in countdown_edit_buttons(areas.footer_area) {
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

    fn render_text_field(
        &self,
        label: &str,
        value: &str,
        focused: bool,
        accent: Color,
        muted: Color,
    ) -> Line<'static> {
        let label_style = if focused {
            Style::default().fg(accent).bold()
        } else {
            Style::default().fg(muted)
        };
        let field_style = if focused {
            Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 40))
        } else {
            Style::default().fg(Color::White)
        };
        let cursor = if focused { "▌" } else { " " };
        let display = format!(" {value}{cursor}");
        Line::from(vec![
            Span::styled(format!("{label}: "), label_style),
            Span::styled(display, field_style),
        ])
    }

    fn render_toggle_field(
        &self,
        label: &str,
        value: bool,
        focused: bool,
        accent: Color,
        muted: Color,
    ) -> Line<'static> {
        let label_style = if focused {
            Style::default().fg(accent).bold()
        } else {
            Style::default().fg(muted)
        };
        let toggle = if value {
            "[x] count up"
        } else {
            "[ ] count down"
        };
        let value_style = if focused {
            Style::default().fg(accent).bold()
        } else {
            Style::default().fg(Color::White)
        };
        Line::from(vec![
            Span::styled(format!("{label}: "), label_style),
            Span::styled(toggle.to_string(), value_style),
        ])
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

    fn scroll_down(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn ev(name: &str, target: &str) -> CountdownEvent {
        CountdownEvent {
            name: name.into(),
            target: target.into(),
            since: false,
        }
    }

    #[test]
    fn opens_with_seed_events() {
        let mut d = CountdownDialog::new();
        d.open(vec![ev("Launch", "2026-06-01")]);
        assert!(d.visible);
        assert_eq!(d.events.len(), 1);
    }

    #[test]
    fn clicking_a_countdown_row_selects_it_without_committing() {
        let mut d = CountdownDialog::new();
        d.open(vec![ev("A", "2026-01-01"), ev("B", "2026-02-01")]);
        let area = Rect::new(0, 0, 80, 20);
        let areas = countdown_list_areas(area, d.events.len());

        let action = d.handle_mouse(click(areas.list_area.x, areas.list_area.y + 1), area);

        assert!(matches!(action, CountdownAction::Continue));
        assert_eq!(d.selected, 1);
        assert!(d.visible);
    }

    #[test]
    fn clicking_add_opens_the_countdown_form() {
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        let area = Rect::new(0, 0, 80, 20);
        let areas = countdown_list_areas(area, 0);
        let buttons = countdown_list_buttons(areas.footer_area);

        let action = d.handle_mouse(click(buttons[0].area.x, buttons[0].area.y), area);

        assert!(matches!(action, CountdownAction::Continue));
        assert_eq!(d.view, View::Edit);
    }

    #[test]
    fn clicking_since_focuses_and_toggles_the_field() {
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        d.handle_key(key(KeyCode::Char('a')));
        let area = Rect::new(0, 0, 80, 20);
        let areas = countdown_edit_areas(area);

        d.handle_mouse(click(areas.since_area.x, areas.since_area.y), area);

        let form = d.form.as_ref().unwrap();
        assert_eq!(form.field, FormField::Since);
        assert!(form.since);
    }

    #[test]
    fn wheel_keeps_long_countdown_selection_visible() {
        let mut d = CountdownDialog::new();
        d.open(
            (0..15)
                .map(|index| ev(&format!("Event {index}"), "2026-01-01"))
                .collect(),
        );
        let area = Rect::new(0, 0, 80, 20);
        let list_area = countdown_list_areas(area, d.events.len()).list_area;

        for _ in 0..14 {
            d.handle_mouse(scroll_down(list_area.x, list_area.y), area);
        }

        assert_eq!(d.selected, 14);
        assert_eq!(d.scroll_offset, 5);
    }

    #[test]
    fn keyboard_keeps_selection_visible_in_short_terminals() {
        let mut d = CountdownDialog::new();
        d.open(
            (0..15)
                .map(|index| ev(&format!("Event {index}"), "2026-01-01"))
                .collect(),
        );
        let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
        terminal
            .draw(|frame| {
                d.render(
                    frame,
                    frame.area(),
                    Color::Red,
                    Color::DarkGray,
                    Color::Gray,
                )
            })
            .unwrap();

        for _ in 0..5 {
            d.handle_key(key(KeyCode::Down));
        }

        assert_eq!(d.selected, 5);
        assert_eq!(d.scroll_offset, 3);
    }

    #[test]
    fn enter_in_list_commits_buffered_events() {
        let mut d = CountdownDialog::new();
        d.open(vec![ev("Launch", "2026-06-01")]);
        let action = d.handle_key(key(KeyCode::Enter));
        match action {
            CountdownAction::Commit(events) => assert_eq!(events.len(), 1),
            _ => panic!("expected Commit"),
        }
        assert!(!d.visible);
    }

    #[test]
    fn esc_in_list_cancels_without_committing() {
        let mut d = CountdownDialog::new();
        d.open(vec![ev("Launch", "2026-06-01")]);
        let action = d.handle_key(key(KeyCode::Esc));
        assert!(matches!(action, CountdownAction::Cancel));
        assert!(!d.visible);
    }

    #[test]
    fn add_flow_inserts_valid_event() {
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        d.handle_key(key(KeyCode::Char('a')));
        assert_eq!(d.view, View::Edit);

        // Type name
        for c in "Birthday".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        // Tab to target
        d.handle_key(key(KeyCode::Tab));
        for c in "2026-09-15".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        // Commit
        d.handle_key(key(KeyCode::Enter));
        assert_eq!(d.view, View::List);
        assert_eq!(d.events.len(), 1);
        assert_eq!(d.events[0].name, "Birthday");
        assert_eq!(d.events[0].target, "2026-09-15");
        assert!(!d.events[0].since);
    }

    #[test]
    fn invalid_target_blocks_commit_with_error() {
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        d.handle_key(key(KeyCode::Char('a')));
        for c in "Bad".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        d.handle_key(key(KeyCode::Tab));
        for c in "not-a-date".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        d.handle_key(key(KeyCode::Enter));
        assert_eq!(d.view, View::Edit);
        assert!(d.error.is_some());
        assert_eq!(d.events.len(), 0);
    }

    #[test]
    fn empty_name_blocks_commit() {
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        d.handle_key(key(KeyCode::Char('a')));
        d.handle_key(key(KeyCode::Tab));
        for c in "2026-06-01".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        d.handle_key(key(KeyCode::Enter));
        assert_eq!(d.view, View::Edit);
        assert!(d.error.is_some());
    }

    #[test]
    fn delete_removes_selected() {
        let mut d = CountdownDialog::new();
        d.open(vec![ev("A", "2026-01-01"), ev("B", "2026-02-01")]);
        d.selected = 0;
        d.handle_key(key(KeyCode::Char('d')));
        assert_eq!(d.events.len(), 1);
        assert_eq!(d.events[0].name, "B");
    }

    #[test]
    fn edit_replaces_in_place() {
        let mut d = CountdownDialog::new();
        d.open(vec![ev("Old", "2026-01-01")]);
        d.handle_key(key(KeyCode::Char('e')));
        // Wipe name and retype
        for _ in 0..10 {
            d.handle_key(key(KeyCode::Backspace));
        }
        for c in "New".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        d.handle_key(key(KeyCode::Enter));
        assert_eq!(d.events.len(), 1);
        assert_eq!(d.events[0].name, "New");
        assert_eq!(d.events[0].target, "2026-01-01");
    }

    #[test]
    fn since_toggle_via_space() {
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        d.handle_key(key(KeyCode::Char('a')));
        for c in "Sober".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        d.handle_key(key(KeyCode::Tab));
        for c in "2023-02-14".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        d.handle_key(key(KeyCode::Tab));
        d.handle_key(key(KeyCode::Char(' ')));
        d.handle_key(key(KeyCode::Enter));
        assert!(d.events[0].since);
    }

    #[test]
    fn render_list_uses_dim_and_muted_for_secondary_text() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut d = CountdownDialog::new();
        d.open(vec![CountdownEvent {
            name: "Launch".into(),
            target: "2026-06-01".into(),
            since: false,
        }]);
        let accent = Color::Red;
        let dim = Color::Rgb(10, 20, 30);
        let muted = Color::Rgb(40, 50, 60);

        terminal
            .draw(|frame| d.render(frame, frame.area(), accent, dim, muted))
            .unwrap();

        let backend = terminal.backend();
        assert_eq!(color_of_text(backend, " until"), Some(dim));
        assert_eq!(color_of_text(backend, " add"), Some(muted));
    }

    #[test]
    fn render_edit_keeps_inputs_white_and_uses_dim_for_hint() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut d = CountdownDialog::new();
        d.open(vec![]);
        d.handle_key(key(KeyCode::Char('a')));
        for c in "Launch".chars() {
            d.handle_key(key(KeyCode::Char(c)));
        }
        let accent = Color::Red;
        let dim = Color::Rgb(10, 20, 30);
        let muted = Color::Rgb(40, 50, 60);

        terminal
            .draw(|frame| d.render(frame, frame.area(), accent, dim, muted))
            .unwrap();

        let backend = terminal.backend();
        assert_eq!(color_of_text(backend, "e.g. 2026-06-01"), Some(dim));
        assert_eq!(color_of_text(backend, "Launch"), Some(Color::White));
        assert_eq!(color_of_text(backend, " commit"), Some(muted));
    }
}
