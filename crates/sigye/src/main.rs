//! sigye - A terminal clock application with configurable fonts.

mod settings;
mod system_metrics;
mod weather;

use std::time::{Duration, Instant};

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Position},
    style::Stylize,
    text::Line,
};
use sigye_config::Config;
use sigye_core::{
    AnimationSpeed, AnimationStyle, BackgroundStyle, ColorTheme, TimeFormat, apply_animation,
    is_colon_visible,
};
use sigye_fonts::FontRegistry;

use settings::SettingsDialog;
use sigye_background::BackgroundState;
use system_metrics::SystemMonitor;
use weather::WeatherMonitor;

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
    /// Current animation style.
    animation_style: AnimationStyle,
    /// Current animation speed.
    animation_speed: AnimationSpeed,
    /// Whether colon blinks.
    colon_blink: bool,
    /// Current background style.
    background_style: BackgroundStyle,
    /// Current font name.
    current_font: String,
    /// Font registry containing all available fonts.
    font_registry: FontRegistry,
    /// Settings dialog state.
    settings_dialog: SettingsDialog,
    /// Configuration for persistence.
    config: Config,
    /// Animation start time.
    animation_start: Instant,
    /// Last recorded second (for reactive animation).
    last_second: u32,
    /// Last recorded minute (for reactive animation).
    last_minute: u32,
    /// Last recorded hour (for reactive animation).
    last_hour: u32,
    /// Current flash intensity (0.0 to 1.0).
    flash_intensity: f32,
    /// When the last flash started (for decay calculation).
    flash_start: Option<Instant>,
    /// Background animation state.
    background_state: BackgroundState,
    /// System monitor for reactive backgrounds (lazy initialized).
    system_monitor: Option<SystemMonitor>,
    /// Weather monitor for dynamic weather background (lazy initialized).
    weather_monitor: Option<WeatherMonitor>,
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

        // Get current time for initial state
        let now = chrono::Local::now();

        // Initialize system monitor if reactive background is selected
        let system_monitor = if config.background_style.is_reactive() {
            let monitor = SystemMonitor::new();
            monitor.start();
            Some(monitor)
        } else {
            None
        };

        // Initialize weather monitor if weather background is selected
        let weather_monitor = if config.background_style.requires_weather() {
            let monitor = WeatherMonitor::new(config.weather_location.clone());
            monitor.start();
            Some(monitor)
        } else {
            None
        };

        Self {
            running: false,
            time_format: config.time_format,
            color_theme: config.color_theme,
            animation_style: config.animation_style,
            animation_speed: config.animation_speed,
            colon_blink: config.colon_blink,
            background_style: config.background_style,
            current_font: config.font_name.clone(),
            font_registry,
            settings_dialog,
            config,
            animation_start: Instant::now(),
            last_second: now.format("%S").to_string().parse().unwrap_or(0),
            last_minute: now.format("%M").to_string().parse().unwrap_or(0),
            last_hour: now.format("%H").to_string().parse().unwrap_or(0),
            flash_intensity: 0.0,
            flash_start: None,
            background_state: BackgroundState::new(),
            system_monitor,
            weather_monitor,
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

        // Calculate animation elapsed time
        let elapsed_ms = self.animation_start.elapsed().as_millis() as u64;

        // Get metrics for reactive backgrounds
        let metrics = self.system_monitor.as_ref().map(|m| m.get_metrics());

        // Resolve weather background to actual style
        let effective_background = if self.background_style == BackgroundStyle::Weather {
            self.weather_monitor
                .as_ref()
                .map(|m| m.get_background())
                .unwrap_or(BackgroundStyle::Starfield)
        } else {
            self.background_style
        };

        // Render background first (behind everything else)
        self.background_state.render(
            frame,
            effective_background,
            elapsed_ms,
            self.animation_speed,
            metrics.as_ref(),
        );

        // Update flash intensity for reactive animation
        self.update_flash(&now);

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
        let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

        // Build colon position mask for blink effect
        // Maps x-positions in rendered ASCII art back to colon characters in time_str
        let colon_positions: Vec<bool> = if self.colon_blink {
            let mut mask = vec![false; width];
            let mut x_pos = 0;
            for ch in time_str.chars() {
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

        // Render time directly to buffer, skipping spaces to preserve background
        let chunk = chunks[1];
        let text_width = width as u16;
        let start_x = chunk.x + (chunk.width.saturating_sub(text_width)) / 2;

        let buf = frame.buffer_mut();
        for (line_idx, line) in time_lines.iter().enumerate() {
            let y_pos = chunk.y + line_idx as u16;
            if y_pos >= chunk.y + chunk.height {
                break;
            }

            for (char_idx, ch) in line.chars().enumerate() {
                // Skip spaces to preserve background transparency
                if ch == ' ' {
                    continue;
                }

                let x_pos = start_x + char_idx as u16;
                if x_pos >= chunk.x + chunk.width {
                    continue;
                }

                // Apply colon blink by skipping colon characters during "off" phase
                let is_colon = colon_positions.get(char_idx).copied().unwrap_or(false);
                let should_hide = self.colon_blink && is_colon && !is_colon_visible(elapsed_ms);
                if should_hide {
                    continue;
                }

                // Get base color
                let base_color = if self.color_theme.is_dynamic() {
                    self.color_theme
                        .color_at_position(char_idx, line_idx, width, height)
                } else {
                    color
                };

                // Apply animation
                let animated_color = apply_animation(
                    base_color,
                    self.animation_style,
                    self.animation_speed,
                    elapsed_ms,
                    char_idx,
                    width,
                    self.flash_intensity,
                );

                // Write directly to buffer
                if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                    cell.set_char(ch);
                    cell.set_fg(animated_color);
                }
            }
        }

        // Render date directly to buffer, skipping spaces to preserve background
        let date_chunk = chunks[3];
        let date_width = date_str.len() as u16;
        let date_start_x = date_chunk.x + (date_chunk.width.saturating_sub(date_width)) / 2;
        let date_y = date_chunk.y;

        let buf = frame.buffer_mut();
        for (char_idx, ch) in date_str.chars().enumerate() {
            // Skip spaces to preserve background transparency
            if ch == ' ' {
                continue;
            }

            let x_pos = date_start_x + char_idx as u16;
            if x_pos >= date_chunk.x + date_chunk.width {
                continue;
            }

            // Get base color
            let base_color = if self.color_theme.is_dynamic() {
                self.color_theme
                    .color_at_position(char_idx, 0, date_str.len(), 1)
            } else {
                color
            };

            // Apply animation
            let animated_color = apply_animation(
                base_color,
                self.animation_style,
                self.animation_speed,
                elapsed_ms,
                char_idx,
                date_str.len(),
                self.flash_intensity,
            );

            // Write directly to buffer
            if let Some(cell) = buf.cell_mut(Position::new(x_pos, date_y)) {
                cell.set_char(ch);
                cell.set_fg(animated_color);
            }
        }

        // Render help text
        let help = Line::from(vec![
            "q".bold().fg(color),
            " quit  ".dark_gray(),
            "t".bold().fg(color),
            " 12/24h  ".dark_gray(),
            "c".bold().fg(color),
            " color  ".dark_gray(),
            "a".bold().fg(color),
            " anim  ".dark_gray(),
            "b".bold().fg(color),
            " bg  ".dark_gray(),
            "s".bold().fg(color),
            " settings".dark_gray(),
        ])
        .centered();
        frame.render_widget(help, chunks[5]);

        // Render settings dialog if visible
        self.settings_dialog.render(frame, area, color);
    }

    /// Update flash intensity for reactive animation.
    fn update_flash(&mut self, now: &chrono::DateTime<chrono::Local>) {
        let second: u32 = now.format("%S").to_string().parse().unwrap_or(0);
        let minute: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let hour: u32 = now.format("%H").to_string().parse().unwrap_or(0);

        // Check for time changes and trigger flash
        if hour != self.last_hour {
            self.flash_intensity = 1.0; // Full flash for hour change
            self.flash_start = Some(Instant::now());
            self.last_hour = hour;
            self.last_minute = minute;
            self.last_second = second;
        } else if minute != self.last_minute {
            self.flash_intensity = 0.7; // Strong flash for minute change
            self.flash_start = Some(Instant::now());
            self.last_minute = minute;
            self.last_second = second;
        } else if second != self.last_second {
            self.flash_intensity = 0.3; // Subtle flash for second change
            self.flash_start = Some(Instant::now());
            self.last_second = second;
        }

        // Decay flash over time
        if let Some(flash_start) = self.flash_start {
            let decay_ms = self.animation_speed.flash_decay_ms();
            let flash_elapsed = flash_start.elapsed().as_millis() as f32;
            let decay_progress = (flash_elapsed / decay_ms as f32).min(1.0);
            self.flash_intensity *= 1.0 - decay_progress;

            if self.flash_intensity < 0.01 {
                self.flash_intensity = 0.0;
                self.flash_start = None;
            }
        }
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
            (_, KeyCode::Char('a')) => self.cycle_animation(),
            (_, KeyCode::Char('b')) => self.cycle_background(),
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
        self.animation_style = self.settings_dialog.animation_style;
        self.animation_speed = self.settings_dialog.animation_speed;
        self.colon_blink = self.settings_dialog.colon_blink;
        self.background_style = self.settings_dialog.background_style;
        self.update_background_monitors();
    }

    /// Open settings dialog with current settings.
    fn open_settings(&mut self) {
        self.settings_dialog.open(
            &self.current_font,
            self.color_theme,
            self.time_format,
            self.animation_style,
            self.animation_speed,
            self.colon_blink,
            self.background_style,
        );
    }

    /// Save current settings to config file and close dialog.
    fn save_settings(&mut self) {
        // Update and save config (values already applied via preview)
        self.config.font_name = self.current_font.clone();
        self.config.color_theme = self.color_theme;
        self.config.time_format = self.time_format;
        self.config.animation_style = self.animation_style;
        self.config.animation_speed = self.animation_speed;
        self.config.colon_blink = self.colon_blink;
        self.config.background_style = self.background_style;

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
        self.animation_style = self.settings_dialog.original_animation_style();
        self.animation_speed = self.settings_dialog.original_animation_speed();
        self.colon_blink = self.settings_dialog.original_colon_blink();
        self.background_style = self.settings_dialog.original_background_style();
        self.update_background_monitors();

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

    /// Cycle through animation styles.
    fn cycle_animation(&mut self) {
        self.animation_style = self.animation_style.next();
    }

    /// Cycle through background styles.
    fn cycle_background(&mut self) {
        self.background_style = self.background_style.next();
        self.update_background_monitors();
    }

    /// Start or stop background monitors based on current background style.
    fn update_background_monitors(&mut self) {
        // System monitor for reactive backgrounds
        if self.background_style.is_reactive() && self.system_monitor.is_none() {
            let monitor = SystemMonitor::new();
            monitor.start();
            self.system_monitor = Some(monitor);
        } else if !self.background_style.is_reactive() && self.system_monitor.is_some() {
            self.system_monitor = None;
        }

        // Weather monitor for weather background
        if self.background_style.requires_weather() && self.weather_monitor.is_none() {
            let monitor = WeatherMonitor::new(self.config.weather_location.clone());
            monitor.start();
            self.weather_monitor = Some(monitor);
        } else if !self.background_style.requires_weather() && self.weather_monitor.is_some() {
            self.weather_monitor = None;
        }
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
