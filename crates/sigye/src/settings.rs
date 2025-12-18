//! Settings dialog widget for configuring the clock.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use sigye_core::{ColorTheme, TimeFormat};

/// The settings field currently being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsField {
    #[default]
    Font,
    Color,
    TimeFormat,
}

impl SettingsField {
    /// Move to the next field.
    pub fn next(self) -> Self {
        match self {
            Self::Font => Self::Color,
            Self::Color => Self::TimeFormat,
            Self::TimeFormat => Self::Font,
        }
    }

    /// Move to the previous field.
    pub fn prev(self) -> Self {
        match self {
            Self::Font => Self::TimeFormat,
            Self::Color => Self::Font,
            Self::TimeFormat => Self::Color,
        }
    }
}

/// Settings dialog state.
#[derive(Debug)]
pub struct SettingsDialog {
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Currently selected field.
    pub selected_field: SettingsField,
    /// Index into available fonts list.
    pub font_index: usize,
    /// List of available font names.
    pub available_fonts: Vec<String>,
    /// Current color theme selection.
    pub color_theme: ColorTheme,
    /// Current time format selection.
    pub time_format: TimeFormat,
    /// Original font index (for cancel/revert).
    original_font_index: usize,
    /// Original color theme (for cancel/revert).
    original_color_theme: ColorTheme,
    /// Original time format (for cancel/revert).
    original_time_format: TimeFormat,
}

impl SettingsDialog {
    /// Create a new settings dialog.
    pub fn new(available_fonts: Vec<String>) -> Self {
        Self {
            visible: false,
            selected_field: SettingsField::default(),
            font_index: 0,
            available_fonts,
            color_theme: ColorTheme::default(),
            time_format: TimeFormat::default(),
            original_font_index: 0,
            original_color_theme: ColorTheme::default(),
            original_time_format: TimeFormat::default(),
        }
    }

    /// Open dialog with current settings.
    pub fn open(&mut self, font_name: &str, color_theme: ColorTheme, time_format: TimeFormat) {
        self.visible = true;
        self.selected_field = SettingsField::default();
        self.color_theme = color_theme;
        self.time_format = time_format;

        // Find font index
        self.font_index = self
            .available_fonts
            .iter()
            .position(|f| f == font_name)
            .unwrap_or(0);

        // Store original values for cancel/revert
        self.original_font_index = self.font_index;
        self.original_color_theme = color_theme;
        self.original_time_format = time_format;
    }

    /// Close without saving.
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Get original font name (for reverting on cancel).
    pub fn original_font(&self) -> &str {
        self.available_fonts
            .get(self.original_font_index)
            .map(String::as_str)
            .unwrap_or("Standard")
    }

    /// Get original color theme (for reverting on cancel).
    pub fn original_color_theme(&self) -> ColorTheme {
        self.original_color_theme
    }

    /// Get original time format (for reverting on cancel).
    pub fn original_time_format(&self) -> TimeFormat {
        self.original_time_format
    }

    /// Move to next field.
    pub fn next_field(&mut self) {
        self.selected_field = self.selected_field.next();
    }

    /// Move to previous field.
    pub fn prev_field(&mut self) {
        self.selected_field = self.selected_field.prev();
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
    pub fn render(&self, frame: &mut Frame, area: Rect, accent_color: Color) {
        if !self.visible {
            return;
        }

        // Calculate centered dialog area
        let dialog_width = 40.min(area.width.saturating_sub(4));
        let dialog_height = 11.min(area.height.saturating_sub(2));

        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Create block with border
        let block = Block::default()
            .title(" Settings ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent_color));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout for settings fields
        let chunks = Layout::vertical([
            Constraint::Length(1), // Top padding
            Constraint::Length(1), // Font
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Color
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Time Format
            Constraint::Fill(1),   // Bottom space
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

        // Render font field
        let font_line = self.render_field(
            "Font",
            self.selected_font(),
            self.selected_field == SettingsField::Font,
            accent_color,
        );
        frame.render_widget(Paragraph::new(font_line).alignment(Alignment::Center), chunks[1]);

        // Render color field
        let color_name = format!("{:?}", self.color_theme);
        let color_line = self.render_field(
            "Color",
            &color_name,
            self.selected_field == SettingsField::Color,
            accent_color,
        );
        frame.render_widget(Paragraph::new(color_line).alignment(Alignment::Center), chunks[3]);

        // Render time format field
        let time_format_name = match self.time_format {
            TimeFormat::TwentyFourHour => "24-hour",
            TimeFormat::TwelveHour => "12-hour",
        };
        let time_line = self.render_field(
            "Format",
            time_format_name,
            self.selected_field == SettingsField::TimeFormat,
            accent_color,
        );
        frame.render_widget(Paragraph::new(time_line).alignment(Alignment::Center), chunks[5]);

        // Render help text
        let help = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(accent_color).bold()),
            Span::styled(" nav  ", Style::default().dark_gray()),
            Span::styled("←→", Style::default().fg(accent_color).bold()),
            Span::styled(" change  ", Style::default().dark_gray()),
            Span::styled("Enter", Style::default().fg(accent_color).bold()),
            Span::styled(" save  ", Style::default().dark_gray()),
            Span::styled("Esc", Style::default().fg(accent_color).bold()),
            Span::styled(" cancel", Style::default().dark_gray()),
        ]);
        frame.render_widget(Paragraph::new(help).alignment(Alignment::Center), chunks[7]);
    }

    /// Render a single settings field line.
    fn render_field(
        &self,
        label: &str,
        value: &str,
        selected: bool,
        accent_color: Color,
    ) -> Line<'static> {
        let arrow_style = if selected {
            Style::default().fg(accent_color).bold()
        } else {
            Style::default().dark_gray()
        };

        let value_style = if selected {
            Style::default().fg(accent_color).bold()
        } else {
            Style::default()
        };

        let label_style = Style::default().dark_gray();

        Line::from(vec![
            Span::styled(format!("{label}: "), label_style),
            Span::styled(String::from("◀ "), arrow_style),
            Span::styled(value.to_string(), value_style),
            Span::styled(String::from(" ▶"), arrow_style),
        ])
    }
}
