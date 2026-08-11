//! Settings dialog widget for configuring the clock.

use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use sigye_core::{AnimationSpeed, AnimationStyle, BackgroundStyle, ColorTheme, TimeFormat};

use crate::dialog::{centered_rect, dialog_block};
use crate::render::{FooterAction, FooterButton, footer_action_at, footer_layout};

/// The settings field currently being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsField {
    #[default]
    Font,
    Color,
    TimeFormat,
    ShowSeconds,
    Animation,
    Speed,
    Background,
    ColonBlink,
    PomodoroWork,
    PomodoroBreak,
    PomodoroLongBreak,
    PomodoroSound,
    DesktopNotifications,
    TimerDuration,
}

/// Common Pomodoro work durations cycled through in the settings dialog (minutes).
const POMODORO_WORK_MINS: &[u32] = &[15, 20, 25, 30, 45, 50, 60];
/// Common Pomodoro break durations cycled through in the settings dialog (minutes).
const POMODORO_BREAK_MINS: &[u32] = &[3, 5, 10, 15];
/// Common Pomodoro long break durations cycled through in the settings dialog (minutes).
const POMODORO_LONG_BREAK_MINS: &[u32] = &[10, 15, 20, 30];

/// Step `current` forward (`delta = 1`) or backward (`delta = -1`) through `values`
/// with wraparound. If `current` isn't in `values`, fall back to `default`.
fn cycle_value(values: &[u32], current: u32, delta: i32, default: u32) -> u32 {
    match values.iter().position(|&v| v == current) {
        Some(idx) => {
            let len = values.len() as i32;
            let next = (idx as i32 + delta).rem_euclid(len) as usize;
            values[next]
        }
        None => default,
    }
}

/// A row in the settings dialog layout.
enum RowKind {
    Header(&'static str),
    Field(SettingsField),
    Spacer,
}

#[derive(Clone, Copy)]
struct DialogTextColors {
    dim: Color,
    muted: Color,
}

struct SettingsAreas {
    dialog_area: Rect,
    content_area: Rect,
    help_area: Rect,
}

fn settings_areas(area: Rect, total_content_rows: u16) -> SettingsAreas {
    let dialog_width = 50.min(area.width.saturating_sub(4));
    let dialog_height = (total_content_rows + 5).min(area.height.saturating_sub(2));
    let dialog_area = centered_rect(area, dialog_width, dialog_height);
    let inner_area = dialog_block(" Settings ", Color::Reset).inner(dialog_area);
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner_area);

    SettingsAreas {
        dialog_area,
        content_area: Rect::new(
            chunks[1].x + 2,
            chunks[1].y,
            chunks[1].width.saturating_sub(4),
            chunks[1].height,
        ),
        help_area: chunks[3],
    }
}

fn settings_footer_buttons(area: Rect) -> Vec<FooterButton> {
    footer_layout(
        area,
        &[
            FooterAction::new(KeyCode::Enter, "Enter", "save"),
            FooterAction::new(KeyCode::Esc, "Esc", "cancel"),
        ],
    )
}

/// Snapshot of settings values taken when the dialog opens, used to revert on cancel.
#[derive(Debug, Clone, Copy)]
pub struct SettingsSnapshot {
    pub font_index: usize,
    pub color_theme: ColorTheme,
    pub time_format: TimeFormat,
    pub animation_style: AnimationStyle,
    pub animation_speed: AnimationSpeed,
    pub background_style: BackgroundStyle,
    pub colon_blink: bool,
    pub show_seconds: bool,
    pub pomodoro_work_mins: u32,
    pub pomodoro_break_mins: u32,
    pub pomodoro_long_break_mins: u32,
    pub pomodoro_sound: bool,
    pub desktop_notifications: bool,
    pub timer_duration_mins: u32,
}

/// Settings dialog state.
#[derive(Debug)]
pub struct SettingsDialog {
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Currently selected field.
    pub selected_field: SettingsField,
    /// Scroll offset for vertical scrolling.
    scroll_offset: u16,
    /// Index into available fonts list.
    pub font_index: usize,
    /// List of available font names.
    pub available_fonts: Vec<String>,
    /// Current color theme selection.
    pub color_theme: ColorTheme,
    /// Current time format selection.
    pub time_format: TimeFormat,
    /// Current animation style selection.
    pub animation_style: AnimationStyle,
    /// Current animation speed selection.
    pub animation_speed: AnimationSpeed,
    /// Current background style selection.
    pub background_style: BackgroundStyle,
    /// Current colon blink setting.
    pub colon_blink: bool,
    /// Current show seconds setting.
    pub show_seconds: bool,
    /// Pomodoro work duration in minutes.
    pub pomodoro_work_mins: u32,
    /// Pomodoro break duration in minutes.
    pub pomodoro_break_mins: u32,
    /// Pomodoro long break duration in minutes.
    pub pomodoro_long_break_mins: u32,
    /// Pomodoro sound notification setting.
    pub pomodoro_sound: bool,
    /// Desktop notifications setting.
    pub desktop_notifications: bool,
    /// Timer countdown duration in minutes.
    pub timer_duration_mins: u32,
    /// Snapshot of values when the dialog opened (for cancel/revert).
    original: SettingsSnapshot,
}

impl SettingsDialog {
    /// Create a new settings dialog.
    pub fn new(available_fonts: Vec<String>) -> Self {
        Self {
            visible: false,
            selected_field: SettingsField::default(),
            scroll_offset: 0,
            font_index: 0,
            available_fonts,
            color_theme: ColorTheme::default(),
            time_format: TimeFormat::default(),
            animation_style: AnimationStyle::default(),
            animation_speed: AnimationSpeed::default(),
            background_style: BackgroundStyle::default(),
            colon_blink: false,
            show_seconds: true,
            pomodoro_work_mins: 25,
            pomodoro_break_mins: 5,
            pomodoro_long_break_mins: 15,
            pomodoro_sound: true,
            desktop_notifications: true,
            timer_duration_mins: 5,
            original: SettingsSnapshot {
                font_index: 0,
                color_theme: ColorTheme::default(),
                time_format: TimeFormat::default(),
                animation_style: AnimationStyle::default(),
                animation_speed: AnimationSpeed::default(),
                background_style: BackgroundStyle::default(),
                colon_blink: false,
                show_seconds: true,
                pomodoro_work_mins: 25,
                pomodoro_break_mins: 5,
                pomodoro_long_break_mins: 15,
                pomodoro_sound: true,
                desktop_notifications: true,
                timer_duration_mins: 5,
            },
        }
    }

    /// Open dialog with current settings.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &mut self,
        font_name: &str,
        color_theme: ColorTheme,
        time_format: TimeFormat,
        animation_style: AnimationStyle,
        animation_speed: AnimationSpeed,
        colon_blink: bool,
        show_seconds: bool,
        background_style: BackgroundStyle,
        pomodoro_work_mins: u32,
        pomodoro_break_mins: u32,
        pomodoro_long_break_mins: u32,
        pomodoro_sound: bool,
        desktop_notifications: bool,
        timer_duration_mins: u32,
    ) {
        self.visible = true;
        self.selected_field = SettingsField::default();
        self.scroll_offset = 0;
        self.color_theme = color_theme;
        self.time_format = time_format;
        self.animation_style = animation_style;
        self.animation_speed = animation_speed;
        self.background_style = background_style;
        self.colon_blink = colon_blink;
        self.show_seconds = show_seconds;
        self.pomodoro_work_mins = pomodoro_work_mins;
        self.pomodoro_break_mins = pomodoro_break_mins;
        self.pomodoro_long_break_mins = pomodoro_long_break_mins;
        self.pomodoro_sound = pomodoro_sound;
        self.desktop_notifications = desktop_notifications;
        self.timer_duration_mins = timer_duration_mins;

        // Find font index
        self.font_index = self
            .available_fonts
            .iter()
            .position(|f| f == font_name)
            .unwrap_or(0);

        // Store original values for cancel/revert
        self.original = SettingsSnapshot {
            font_index: self.font_index,
            color_theme,
            time_format,
            animation_style,
            animation_speed,
            background_style,
            colon_blink,
            show_seconds,
            pomodoro_work_mins,
            pomodoro_break_mins,
            pomodoro_long_break_mins,
            pomodoro_sound,
            desktop_notifications,
            timer_duration_mins,
        };
    }

    /// Close without saving.
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Get the snapshot of values taken when the dialog opened (for reverting on cancel).
    pub fn original(&self) -> &SettingsSnapshot {
        &self.original
    }

    /// Get original font name (for reverting on cancel).
    pub fn original_font(&self) -> &str {
        self.available_fonts
            .get(self.original.font_index)
            .map(String::as_str)
            .unwrap_or("Standard")
    }

    /// Get settings fields in their visible section layout order.
    fn field_order() -> Vec<SettingsField> {
        Self::section_layout()
            .into_iter()
            .filter_map(|row| match row {
                RowKind::Field(field) => Some(field),
                RowKind::Header(_) | RowKind::Spacer => None,
            })
            .collect()
    }

    /// Move to next field and ensure it's visible.
    pub fn next_field(&mut self) {
        let fields = Self::field_order();
        let idx = fields
            .iter()
            .position(|field| *field == self.selected_field)
            .unwrap_or(0);
        self.selected_field = fields[(idx + 1) % fields.len()];
    }

    /// Move to previous field and ensure it's visible.
    pub fn prev_field(&mut self) {
        let fields = Self::field_order();
        let idx = fields
            .iter()
            .position(|field| *field == self.selected_field)
            .unwrap_or(0);
        self.selected_field = fields[(idx + fields.len() - 1) % fields.len()];
    }

    /// Get the section layout: ordered list of rows (headers, fields, spacers).
    fn section_layout() -> Vec<RowKind> {
        vec![
            RowKind::Header("Clock"),
            RowKind::Field(SettingsField::Font),
            RowKind::Field(SettingsField::Color),
            RowKind::Field(SettingsField::TimeFormat),
            RowKind::Field(SettingsField::ShowSeconds),
            RowKind::Field(SettingsField::ColonBlink),
            RowKind::Spacer,
            RowKind::Header("Animation"),
            RowKind::Field(SettingsField::Animation),
            RowKind::Field(SettingsField::Speed),
            RowKind::Field(SettingsField::Background),
            RowKind::Spacer,
            RowKind::Header("Pomodoro"),
            RowKind::Field(SettingsField::PomodoroWork),
            RowKind::Field(SettingsField::PomodoroBreak),
            RowKind::Field(SettingsField::PomodoroLongBreak),
            RowKind::Field(SettingsField::PomodoroSound),
            RowKind::Field(SettingsField::DesktopNotifications),
            RowKind::Spacer,
            RowKind::Header("Timer"),
            RowKind::Field(SettingsField::TimerDuration),
        ]
    }

    /// Get the row index of the currently selected field in the section layout.
    fn selected_field_row_index(&self) -> usize {
        Self::section_layout()
            .iter()
            .position(|row| matches!(row, RowKind::Field(f) if *f == self.selected_field))
            .unwrap_or(0)
    }

    /// Adjust scroll_offset so the selected field is within the visible window.
    fn ensure_visible(&mut self, visible_rows: u16) {
        let row_idx = self.selected_field_row_index() as u16;
        // Scroll up if selected field is above the visible window
        if row_idx < self.scroll_offset {
            self.scroll_offset = row_idx;
        }
        // Scroll down if selected field is below the visible window
        if row_idx >= self.scroll_offset + visible_rows {
            self.scroll_offset = row_idx.saturating_sub(visible_rows - 1);
        }
    }

    fn move_selection_clamped(&mut self, delta: i32) {
        let fields = Self::field_order();
        let index = fields
            .iter()
            .position(|field| *field == self.selected_field)
            .unwrap_or(0) as i32;
        let next = (index + delta).clamp(0, fields.len() as i32 - 1) as usize;
        self.selected_field = fields[next];
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, area: Rect) -> Option<KeyCode> {
        let rows = Self::section_layout();
        let areas = settings_areas(area, rows.len() as u16);
        match mouse.kind {
            MouseEventKind::ScrollUp => self.move_selection_clamped(-1),
            MouseEventKind::ScrollDown => self.move_selection_clamped(1),
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(key) = footer_action_at(
                    &settings_footer_buttons(areas.help_area),
                    mouse.column,
                    mouse.row,
                ) {
                    return Some(key);
                }
                if areas
                    .content_area
                    .contains((mouse.column, mouse.row).into())
                {
                    let row_index =
                        self.scroll_offset + mouse.row.saturating_sub(areas.content_area.y);
                    if let Some(RowKind::Field(field)) = rows.get(row_index as usize) {
                        self.selected_field = *field;
                        let row_area =
                            Rect::new(areas.content_area.x, mouse.row, areas.content_area.width, 1);
                        let (_, _, enabled) = self.field_label_value(*field);
                        let (left, right) = self.field_arrow_areas(*field, row_area);
                        if enabled && left.contains((mouse.column, mouse.row).into()) {
                            return Some(KeyCode::Left);
                        }
                        if enabled && right.contains((mouse.column, mouse.row).into()) {
                            return Some(KeyCode::Right);
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// Select next value for current field.
    pub fn next_value(&mut self) {
        match self.selected_field {
            SettingsField::Font => {
                if !self.available_fonts.is_empty() {
                    self.font_index = (self.font_index + 1) % self.available_fonts.len();
                }
            }
            SettingsField::Color => {
                self.color_theme = self.color_theme.next();
            }
            SettingsField::TimeFormat => {
                self.time_format = self.time_format.toggle();
            }
            SettingsField::ShowSeconds => {
                self.show_seconds = !self.show_seconds;
            }
            SettingsField::Animation => {
                self.animation_style = self.animation_style.next();
            }
            SettingsField::Speed => {
                self.animation_speed = self.animation_speed.next();
            }
            SettingsField::Background => {
                self.background_style = self.background_style.next();
            }
            SettingsField::ColonBlink => {
                self.colon_blink = !self.colon_blink;
            }
            SettingsField::PomodoroWork => {
                // Cycle through common work durations: 15, 20, 25, 30, 45, 50, 60
                self.pomodoro_work_mins =
                    cycle_value(POMODORO_WORK_MINS, self.pomodoro_work_mins, 1, 15);
            }
            SettingsField::PomodoroBreak => {
                // Cycle through common break durations: 3, 5, 10, 15
                self.pomodoro_break_mins =
                    cycle_value(POMODORO_BREAK_MINS, self.pomodoro_break_mins, 1, 3);
            }
            SettingsField::PomodoroLongBreak => {
                // Cycle through common long break durations: 10, 15, 20, 30
                self.pomodoro_long_break_mins = cycle_value(
                    POMODORO_LONG_BREAK_MINS,
                    self.pomodoro_long_break_mins,
                    1,
                    10,
                );
            }
            SettingsField::PomodoroSound => {
                self.pomodoro_sound = !self.pomodoro_sound;
            }
            SettingsField::DesktopNotifications => {
                self.desktop_notifications = !self.desktop_notifications;
            }
            SettingsField::TimerDuration => {
                self.timer_duration_mins = (self.timer_duration_mins + 1).min(99);
            }
        }
    }

    /// Select previous value for current field.
    pub fn prev_value(&mut self) {
        match self.selected_field {
            SettingsField::Font => {
                if !self.available_fonts.is_empty() {
                    self.font_index = if self.font_index == 0 {
                        self.available_fonts.len() - 1
                    } else {
                        self.font_index - 1
                    };
                }
            }
            SettingsField::Color => {
                self.color_theme = self.color_theme.prev();
            }
            SettingsField::TimeFormat => {
                self.time_format = self.time_format.toggle();
            }
            SettingsField::ShowSeconds => {
                self.show_seconds = !self.show_seconds;
            }
            SettingsField::Animation => {
                self.animation_style = self.animation_style.prev();
            }
            SettingsField::Speed => {
                self.animation_speed = self.animation_speed.prev();
            }
            SettingsField::Background => {
                self.background_style = self.background_style.prev();
            }
            SettingsField::ColonBlink => {
                self.colon_blink = !self.colon_blink;
            }
            SettingsField::PomodoroWork => {
                // Cycle through common work durations (reverse): 60, 50, 45, 30, 25, 20, 15
                self.pomodoro_work_mins =
                    cycle_value(POMODORO_WORK_MINS, self.pomodoro_work_mins, -1, 25);
            }
            SettingsField::PomodoroBreak => {
                // Cycle through common break durations (reverse): 15, 10, 5, 3
                self.pomodoro_break_mins =
                    cycle_value(POMODORO_BREAK_MINS, self.pomodoro_break_mins, -1, 5);
            }
            SettingsField::PomodoroLongBreak => {
                // Cycle through common long break durations (reverse): 30, 20, 15, 10
                self.pomodoro_long_break_mins = cycle_value(
                    POMODORO_LONG_BREAK_MINS,
                    self.pomodoro_long_break_mins,
                    -1,
                    15,
                );
            }
            SettingsField::PomodoroSound => {
                self.pomodoro_sound = !self.pomodoro_sound;
            }
            SettingsField::DesktopNotifications => {
                self.desktop_notifications = !self.desktop_notifications;
            }
            SettingsField::TimerDuration => {
                self.timer_duration_mins = (self.timer_duration_mins.saturating_sub(1)).max(1);
            }
        }
    }

    /// Get currently selected font name.
    pub fn selected_font(&self) -> &str {
        self.available_fonts
            .get(self.font_index)
            .map(String::as_str)
            .unwrap_or("Standard")
    }

    /// Render the settings dialog.
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        accent_color: Color,
        dim: Color,
        muted: Color,
    ) {
        if !self.visible {
            return;
        }
        let text_colors = DialogTextColors { dim, muted };

        let layout = Self::section_layout();
        let total_content_rows = layout.len() as u16;
        let areas = settings_areas(area, total_content_rows);
        let dialog_area = areas.dialog_area;

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Create block with border and explicit colors for light theme support
        let block = dialog_block(" Settings ", accent_color);
        frame.render_widget(block, dialog_area);
        let content_area = areas.content_area;
        let visible_rows = content_area.height;

        // Ensure selected field is visible (adjust scroll)
        self.ensure_visible(visible_rows);

        // Determine if we need scroll indicators
        let can_scroll_up = self.scroll_offset > 0;
        let can_scroll_down = total_content_rows > self.scroll_offset + visible_rows;

        // Render content rows within the visible scroll window
        for (row_idx, row) in layout.iter().enumerate() {
            let row_idx = row_idx as u16;
            if row_idx < self.scroll_offset {
                continue;
            }
            let visible_y = row_idx - self.scroll_offset;
            if visible_y >= visible_rows {
                break;
            }

            let row_area = Rect::new(
                content_area.x,
                content_area.y + visible_y,
                content_area.width,
                1,
            );

            match row {
                RowKind::Header(name) => {
                    let header = self.render_section_header(name, accent_color);
                    frame.render_widget(
                        Paragraph::new(header).alignment(Alignment::Center),
                        row_area,
                    );
                }
                RowKind::Field(field) => {
                    let line = self.render_field_for(*field, accent_color, text_colors);
                    frame
                        .render_widget(Paragraph::new(line).alignment(Alignment::Center), row_area);
                }
                RowKind::Spacer => {} // Empty row
            }
        }

        // Render scroll indicators over first/last content rows
        if can_scroll_up {
            let indicator = Line::from(Span::styled("  ▲  ", Style::default().fg(accent_color)));
            let indicator_area = Rect::new(
                content_area.x + content_area.width.saturating_sub(6),
                content_area.y,
                6,
                1,
            );
            frame.render_widget(
                Paragraph::new(indicator).alignment(Alignment::Right),
                indicator_area,
            );
        }
        if can_scroll_down {
            let indicator = Line::from(Span::styled("  ▼  ", Style::default().fg(accent_color)));
            let indicator_area = Rect::new(
                content_area.x + content_area.width.saturating_sub(6),
                content_area.y + visible_rows.saturating_sub(1),
                6,
                1,
            );
            frame.render_widget(
                Paragraph::new(indicator).alignment(Alignment::Right),
                indicator_area,
            );
        }

        for button in settings_footer_buttons(areas.help_area) {
            let help = Line::from(vec![
                Span::styled(
                    format!("[{}]", button.action.key_label),
                    Style::default().fg(accent_color).bold(),
                ),
                Span::styled(
                    format!(" {}", button.action.label),
                    Style::default().fg(muted),
                ),
            ]);
            frame.render_widget(Paragraph::new(help), button.area);
        }
    }

    /// Render a section header line.
    fn render_section_header(&self, name: &str, accent_color: Color) -> Line<'static> {
        Line::from(Span::styled(
            format!("── {name} ──"),
            Style::default().fg(accent_color).bold(),
        ))
    }

    /// Render the appropriate field line for a given SettingsField.
    fn render_field_for(
        &self,
        field: SettingsField,
        accent_color: Color,
        text_colors: DialogTextColors,
    ) -> Line<'static> {
        let selected = self.selected_field == field;
        let (label, value, enabled) = self.field_label_value(field);
        self.render_field_with_style(label, &value, selected, accent_color, enabled, text_colors)
    }

    fn field_label_value(&self, field: SettingsField) -> (&'static str, String, bool) {
        match field {
            SettingsField::Font => ("Font", self.selected_font().to_string(), true),
            SettingsField::Color => ("Color", self.color_theme.display_name().to_string(), true),
            SettingsField::TimeFormat => (
                "Format",
                match self.time_format {
                    TimeFormat::TwentyFourHour => "24-hour",
                    TimeFormat::TwelveHour => "12-hour",
                }
                .to_string(),
                true,
            ),
            SettingsField::ShowSeconds => (
                "Seconds",
                if self.show_seconds { "On" } else { "Off" }.to_string(),
                true,
            ),
            SettingsField::ColonBlink => (
                "Colon Blink",
                if self.colon_blink { "On" } else { "Off" }.to_string(),
                true,
            ),
            SettingsField::Animation => (
                "Animation",
                self.animation_style.display_name().to_string(),
                true,
            ),
            SettingsField::Speed => (
                "Speed",
                self.animation_speed.display_name().to_string(),
                self.animation_style != AnimationStyle::None,
            ),
            SettingsField::Background => (
                "Background",
                self.background_style.display_name().to_string(),
                true,
            ),
            SettingsField::PomodoroWork => {
                ("Work", format!("{} min", self.pomodoro_work_mins), true)
            }
            SettingsField::PomodoroBreak => {
                ("Break", format!("{} min", self.pomodoro_break_mins), true)
            }
            SettingsField::PomodoroLongBreak => (
                "Long Break",
                format!("{} min", self.pomodoro_long_break_mins),
                true,
            ),
            SettingsField::PomodoroSound => (
                "Sound",
                if self.pomodoro_sound { "On" } else { "Off" }.to_string(),
                true,
            ),
            SettingsField::DesktopNotifications => (
                "Notifications",
                if self.desktop_notifications {
                    "On"
                } else {
                    "Off"
                }
                .to_string(),
                true,
            ),
            SettingsField::TimerDuration => (
                "Duration",
                format!("{} min", self.timer_duration_mins),
                true,
            ),
        }
    }

    fn field_arrow_areas(&self, field: SettingsField, row_area: Rect) -> (Rect, Rect) {
        let (label, value, _) = self.field_label_value(field);
        let text_width = (label.chars().count() + value.chars().count() + 8) as u16;
        let start_x = row_area.x + row_area.width.saturating_sub(text_width) / 2;
        (
            Rect::new(start_x + label.chars().count() as u16 + 4, row_area.y, 1, 1),
            Rect::new(start_x + text_width.saturating_sub(1), row_area.y, 1, 1),
        )
    }

    /// Render a single settings field line.
    fn render_field(
        &self,
        label: &str,
        value: &str,
        selected: bool,
        accent_color: Color,
        muted: Color,
    ) -> Line<'static> {
        if selected {
            let arrow_style = Style::default().fg(accent_color).bold();
            let value_style = Style::default().fg(accent_color).bold();
            let label_style = Style::default().fg(accent_color);
            Line::from(vec![
                Span::styled(String::from("► "), arrow_style),
                Span::styled(format!("{label}: "), label_style),
                Span::styled(String::from("◀ "), arrow_style),
                Span::styled(value.to_string(), value_style),
                Span::styled(String::from(" ▶"), arrow_style),
            ])
        } else {
            let label_style = Style::default().fg(muted);
            let value_style = Style::default().fg(Color::White);
            let arrow_style = Style::default().fg(muted);
            Line::from(vec![
                Span::styled(String::from("  "), Style::default()),
                Span::styled(format!("{label}: "), label_style),
                Span::styled(String::from("◀ "), arrow_style),
                Span::styled(value.to_string(), value_style),
                Span::styled(String::from(" ▶"), arrow_style),
            ])
        }
    }

    /// Render a single settings field line with enabled/disabled state.
    fn render_field_with_style(
        &self,
        label: &str,
        value: &str,
        selected: bool,
        accent_color: Color,
        enabled: bool,
        text_colors: DialogTextColors,
    ) -> Line<'static> {
        if !enabled {
            // Grayed out when disabled - no arrows
            let gray = Style::default().fg(text_colors.dim);
            return Line::from(vec![
                Span::styled(String::from("  "), Style::default()),
                Span::styled(format!("{label}: "), gray),
                Span::styled(value.to_string(), gray),
            ]);
        }

        self.render_field(label, value, selected, accent_color, text_colors.muted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::test_helpers::color_of_text;
    use ratatui::{Terminal, backend::TestBackend};

    fn click(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    fn scroll_down() -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn render_uses_dim_and_muted_for_secondary_text() {
        let mut terminal = Terminal::new(TestBackend::new(80, 30)).unwrap();
        let mut dialog = SettingsDialog::new(vec!["Standard".into()]);
        dialog.open(
            "Standard",
            ColorTheme::Cyan,
            TimeFormat::TwentyFourHour,
            AnimationStyle::None,
            AnimationSpeed::Medium,
            true,
            true,
            BackgroundStyle::None,
            25,
            5,
            15,
            true,
            true,
            5,
        );
        let accent = Color::Red;
        let dim = Color::Rgb(10, 20, 30);
        let muted = Color::Rgb(40, 50, 60);

        terminal
            .draw(|frame| dialog.render(frame, frame.area(), accent, dim, muted))
            .unwrap();

        let backend = terminal.backend();
        assert_eq!(color_of_text(backend, "Color: "), Some(muted));
        assert_eq!(color_of_text(backend, "Speed: "), Some(dim));
        assert_eq!(color_of_text(backend, " save"), Some(muted));
        assert_eq!(color_of_text(backend, "Cyan"), Some(Color::White));
    }

    fn dialog_at(field: SettingsField) -> SettingsDialog {
        let mut dialog = SettingsDialog::new(vec![String::from("Standard")]);
        dialog.selected_field = field;
        dialog
    }

    #[test]
    fn down_navigation_follows_visible_settings_rows() {
        let mut dialog = SettingsDialog::new(vec![String::from("Standard")]);

        let mut visited = vec![dialog.selected_field];
        for _ in 0..13 {
            dialog.next_field();
            visited.push(dialog.selected_field);
        }

        assert_eq!(
            visited,
            vec![
                SettingsField::Font,
                SettingsField::Color,
                SettingsField::TimeFormat,
                SettingsField::ShowSeconds,
                SettingsField::ColonBlink,
                SettingsField::Animation,
                SettingsField::Speed,
                SettingsField::Background,
                SettingsField::PomodoroWork,
                SettingsField::PomodoroBreak,
                SettingsField::PomodoroLongBreak,
                SettingsField::PomodoroSound,
                SettingsField::DesktopNotifications,
                SettingsField::TimerDuration,
            ]
        );
    }

    #[test]
    fn down_navigation_wraps_from_last_visible_row_to_first() {
        let mut dialog = dialog_at(SettingsField::TimerDuration);

        dialog.next_field();

        assert_eq!(dialog.selected_field, SettingsField::Font);
    }

    #[test]
    fn up_navigation_follows_reverse_visible_settings_rows() {
        let mut dialog = dialog_at(SettingsField::TimerDuration);

        let mut visited = vec![dialog.selected_field];
        for _ in 0..13 {
            dialog.prev_field();
            visited.push(dialog.selected_field);
        }

        assert_eq!(
            visited,
            vec![
                SettingsField::TimerDuration,
                SettingsField::DesktopNotifications,
                SettingsField::PomodoroSound,
                SettingsField::PomodoroLongBreak,
                SettingsField::PomodoroBreak,
                SettingsField::PomodoroWork,
                SettingsField::Background,
                SettingsField::Speed,
                SettingsField::Animation,
                SettingsField::ColonBlink,
                SettingsField::ShowSeconds,
                SettingsField::TimeFormat,
                SettingsField::Color,
                SettingsField::Font,
            ]
        );
    }

    #[test]
    fn up_navigation_wraps_from_first_visible_row_to_last() {
        let mut dialog = dialog_at(SettingsField::Font);

        dialog.prev_field();

        assert_eq!(dialog.selected_field, SettingsField::TimerDuration);
    }

    #[test]
    fn clicking_a_settings_arrow_selects_and_changes_that_field() {
        let mut dialog = dialog_at(SettingsField::Font);
        dialog.visible = true;
        let area = Rect::new(0, 0, 80, 30);
        let areas = settings_areas(area, SettingsDialog::section_layout().len() as u16);
        let row_index = SettingsDialog::section_layout()
            .iter()
            .position(|row| matches!(row, RowKind::Field(SettingsField::Color)))
            .unwrap() as u16;
        let row_area = Rect::new(
            areas.content_area.x,
            areas.content_area.y + row_index,
            areas.content_area.width,
            1,
        );
        let (_, right) = dialog.field_arrow_areas(SettingsField::Color, row_area);

        let key = dialog.handle_mouse(click(right.x, right.y), area);

        assert_eq!(dialog.selected_field, SettingsField::Color);
        assert_eq!(key, Some(KeyCode::Right));
    }

    #[test]
    fn clicking_save_returns_enter() {
        let mut dialog = dialog_at(SettingsField::Font);
        dialog.visible = true;
        let area = Rect::new(0, 0, 80, 30);
        let areas = settings_areas(area, SettingsDialog::section_layout().len() as u16);
        let buttons = settings_footer_buttons(areas.help_area);

        let key = dialog.handle_mouse(click(buttons[0].area.x, buttons[0].area.y), area);

        assert_eq!(key, Some(KeyCode::Enter));
    }

    #[test]
    fn settings_wheel_moves_selection_without_wrapping() {
        let mut dialog = dialog_at(SettingsField::Font);
        let area = Rect::new(0, 0, 80, 30);

        dialog.handle_mouse(scroll_down(), area);

        assert_eq!(dialog.selected_field, SettingsField::Color);
    }
}
