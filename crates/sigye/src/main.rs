//! sigye - A terminal clock application with configurable fonts.

mod settings;

use std::time::Duration;

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};
use sigye_config::Config;
use sigye_core::{ColorTheme, TimeFormat};
use sigye_fonts::FontRegistry;

use settings::SettingsDialog;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    /// Current time format (12h or 24h).
    time_format: TimeFormat,
    /// Current color theme.
    color_theme: ColorTheme,
    /// Current font name.
    current_font: String,
    /// Font registry containing all available fonts.
    font_registry: FontRegistry,
    /// Settings dialog state.
    settings_dialog: SettingsDialog,
    /// Configuration for persistence.
    config: Config,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Load configuration
        let config = Config::load();

        // Initialize font registry with bundled fonts
        let mut font_registry = FontRegistry::new();

        // Load custom fonts from config directory
        font_registry.load_custom_fonts(&Config::fonts_dir());

        // Get list of available fonts for settings dialog
        let available_fonts: Vec<String> = font_registry
            .list_fonts()
            .into_iter()
            .map(String::from)
            .collect();

        // Create settings dialog
        let settings_dialog = SettingsDialog::new(available_fonts);

        Self {
            running: false,
            time_format: config.time_format,
            color_theme: config.color_theme,
            current_font: config.font_name.clone(),
            font_registry,
            settings_dialog,
            config,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let now = Local::now();

        // Get time components
        let (hours, is_pm) = match self.time_format {
            TimeFormat::TwentyFourHour => {
                (now.format("%H").to_string().parse().unwrap_or(0), false)
            }
            TimeFormat::TwelveHour => {
                let h: u32 = now.format("%I").to_string().parse().unwrap_or(12);
                let pm = now.format("%p").to_string() == "PM";
                (h, pm)
            }
        };
        let minutes: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let seconds: u32 = now.format("%S").to_string().parse().unwrap_or(0);

        // Format date
        let date_str = now.format("%A, %B %d, %Y").to_string();

        let color = self.color_theme.color();
        let area = frame.area();

        // Build time string
        let time_str = match self.time_format {
            TimeFormat::TwentyFourHour => {
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
            TimeFormat::TwelveHour => {
                let ampm = if is_pm { "PM" } else { "AM" };
                format!("{hours:2}:{minutes:02}:{seconds:02} {ampm}")
            }
        };

        // Get current font and render
        let font = self.font_registry.get_or_default(&self.current_font);
        let time_lines = font.render_text(&time_str);
        let font_height = font.height as u16;

        // Create vertical layout for centering
        let chunks = Layout::vertical([
            Constraint::Fill(1),             // Top padding
            Constraint::Length(font_height), // Big digits (dynamic height)
            Constraint::Length(2),           // Spacing
            Constraint::Length(1),           // Date
            Constraint::Fill(1),             // Bottom padding
            Constraint::Length(1),           // Help text
        ])
        .split(area);

        // Render big time
        let height = time_lines.len();
        let width = time_lines.first().map(|s| s.len()).unwrap_or(0);

        let time_text: Vec<Line> = if self.color_theme.is_dynamic() {
            // Apply per-character coloring for dynamic themes
            time_lines
                .into_iter()
                .enumerate()
                .map(|(y, line)| {
                    let spans: Vec<Span> = line
                        .chars()
                        .enumerate()
                        .map(|(x, ch)| {
                            let char_color =
                                self.color_theme.color_at_position(x, y, width, height);
                            Span::styled(ch.to_string(), Style::new().fg(char_color))
                        })
                        .collect();
                    Line::from(spans)
                })
                .collect()
        } else {
            // Static color for the whole text
            time_lines
                .into_iter()
                .map(|s| Line::from(s).style(Style::new().fg(color)))
                .collect()
        };

        let time_widget = Paragraph::new(time_text).alignment(Alignment::Center);
        frame.render_widget(time_widget, chunks[1]);

        // Render date (also with dynamic colors if applicable)
        let date_widget = if self.color_theme.is_dynamic() {
            let date_spans: Vec<Span> = date_str
                .chars()
                .enumerate()
                .map(|(x, ch)| {
                    let char_color = self.color_theme.color_at_position(x, 0, date_str.len(), 1);
                    Span::styled(ch.to_string(), Style::new().fg(char_color))
                })
                .collect();
            Paragraph::new(Line::from(date_spans)).alignment(Alignment::Center)
        } else {
            Paragraph::new(date_str)
                .style(Style::new().fg(color))
                .alignment(Alignment::Center)
        };
        frame.render_widget(date_widget, chunks[3]);

        // Render help text
        let help = Line::from(vec![
            "q".bold().fg(color),
            " quit  ".dark_gray(),
            "t".bold().fg(color),
            " toggle 12/24h  ".dark_gray(),
            "c".bold().fg(color),
            " cycle color  ".dark_gray(),
            "s".bold().fg(color),
            " settings".dark_gray(),
        ])
        .centered();
        frame.render_widget(help, chunks[5]);

        // Render settings dialog if visible
        self.settings_dialog.render(frame, area, color);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    /// Uses polling with timeout for real-time clock updates.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        // Poll for events with 100ms timeout for smooth clock updates
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        // If settings dialog is visible, handle dialog keys
        if self.settings_dialog.visible {
            self.handle_settings_key(key);
            return;
        }

        // Main app keybindings
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('t')) => self.toggle_time_format(),
            (_, KeyCode::Char('c')) => self.cycle_color_theme(),
            (_, KeyCode::Char('s')) => self.open_settings(),
            _ => {}
        }
    }

    /// Handle key events when settings dialog is open.
    fn handle_settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.cancel_settings();
            }
            KeyCode::Enter => {
                self.save_settings();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_dialog.prev_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_dialog.next_field();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.settings_dialog.prev_value();
                self.apply_preview();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.settings_dialog.next_value();
                self.apply_preview();
            }
            _ => {}
        }
    }

    /// Apply current dialog values as live preview.
    fn apply_preview(&mut self) {
        self.current_font = self.settings_dialog.selected_font().to_string();
        self.color_theme = self.settings_dialog.color_theme;
        self.time_format = self.settings_dialog.time_format;
    }

    /// Open settings dialog with current settings.
    fn open_settings(&mut self) {
        self.settings_dialog
            .open(&self.current_font, self.color_theme, self.time_format);
    }

    /// Save current settings to config file and close dialog.
    fn save_settings(&mut self) {
        // Update and save config (values already applied via preview)
        self.config.font_name = self.current_font.clone();
        self.config.color_theme = self.color_theme;
        self.config.time_format = self.time_format;

        if let Err(e) = self.config.save() {
            eprintln!("Warning: Failed to save config: {e}");
        }

        self.settings_dialog.close();
    }

    /// Cancel settings and revert to original values.
    fn cancel_settings(&mut self) {
        // Revert to original values
        self.current_font = self.settings_dialog.original_font().to_string();
        self.color_theme = self.settings_dialog.original_color_theme();
        self.time_format = self.settings_dialog.original_time_format();

        self.settings_dialog.close();
    }

    /// Toggle between 12-hour and 24-hour time format.
    fn toggle_time_format(&mut self) {
        self.time_format = self.time_format.toggle();
    }

    /// Cycle through available color themes.
    fn cycle_color_theme(&mut self) {
        self.color_theme = self.color_theme.next();
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
