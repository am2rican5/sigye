//! sigye - A terminal clock application with configurable fonts.

mod context;
mod countdown_dialog;
mod dialog;
mod mode;
mod mode_dialog;
mod modes;
mod render;
mod settings;
mod system_metrics;
mod weather;

use std::time::{Duration, Instant};

use clap::Parser;
#[cfg(not(unix))]
use crossterm::event::EnableMouseCapture;
use crossterm::event::{
    self, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind,
};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};
use sigye_config::Config;
use sigye_core::{BackgroundStyle, ColorTheme, DisplayMode};
use sigye_fonts::FontRegistry;

use context::RenderContext;
use countdown_dialog::{CountdownAction, CountdownDialog};
use mode::Mode;
use mode_dialog::{ModeAction, ModeDialog};
use modes::clock::ClockMode;
use modes::countdown::CountdownMode;
use modes::pomodoro::PomodoroMode;
use modes::stopwatch::StopwatchMode;
use modes::timer::TimerMode;
use modes::world_clock::WorldClockMode;
use settings::SettingsDialog;
use sigye_background::BackgroundState;
use system_metrics::SystemMonitor;
use weather::WeatherMonitor;

#[cfg(unix)]
fn enable_mouse_capture() -> std::io::Result<()> {
    // Crossterm's command also enables motion tracking, which floods the input queue.
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::Print("\x1b[?1000h\x1b[?1006h")
    )
}

#[cfg(not(unix))]
fn enable_mouse_capture() -> std::io::Result<()> {
    crossterm::execute!(std::io::stdout(), EnableMouseCapture)
}

fn footer_key_at(
    mode: &dyn Mode,
    area: ratatui::layout::Rect,
    column: u16,
    row: u16,
) -> Option<KeyCode> {
    let actions = mode.all_footer_actions();
    let height = render::footer_height(area.width, &actions);
    let footer_area = ratatui::layout::Rect::new(
        area.x,
        area.bottom().saturating_sub(height),
        area.width,
        height,
    );
    render::footer_action_at(&render::footer_layout(footer_area, &actions), column, row)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootMouseAction {
    None,
    Key(KeyCode),
    Quit,
}

fn root_mouse_action(
    mode: &dyn Mode,
    screensaver: bool,
    minimal_clock: bool,
    area: ratatui::layout::Rect,
    mouse: MouseEvent,
) -> RootMouseAction {
    if mouse.kind != MouseEventKind::Up(MouseButton::Left) {
        return RootMouseAction::None;
    }
    if screensaver {
        return RootMouseAction::Quit;
    }
    if minimal_clock && mode.display_mode() == DisplayMode::Clock {
        return RootMouseAction::Key(KeyCode::Char('z'));
    }
    footer_key_at(mode, area, mouse.column, mouse.row)
        .map(RootMouseAction::Key)
        .unwrap_or(RootMouseAction::None)
}

/// A beautiful terminal clock with ASCII art fonts, animations, and backgrounds.
#[derive(Parser)]
#[command(name = "sigye", version, about)]
struct Cli {
    /// Start in screensaver mode (fullscreen, no UI chrome)
    #[arg(long)]
    screensaver: bool,

    /// Start in demo mode (auto-cycle themes and backgrounds)
    #[arg(long)]
    demo: bool,

    /// Set the font name
    #[arg(long)]
    font: Option<String>,

    /// Set the color theme
    #[arg(long)]
    theme: Option<String>,

    /// Set the background style
    #[arg(long = "bg", alias = "background")]
    background: Option<String>,

    /// Set the display mode (clock, pomodoro, timer, stopwatch, worldclock, countdown)
    #[arg(long)]
    mode: Option<String>,

    /// Add a world clock timezone (e.g., "Tokyo=Asia/Tokyo"). Can be repeated.
    #[arg(long = "tz", value_name = "LABEL=TIMEZONE")]
    timezones: Vec<String>,

    /// Shell command to execute on timer/pomodoro completion
    #[arg(long)]
    on_complete: Option<String>,

    /// Auto-start the active timer mode (pomodoro, timer, or stopwatch)
    #[arg(long)]
    start: bool,

    /// Start in pomodoro mode and auto-start
    #[arg(long)]
    pomo: bool,

    /// Start in stopwatch mode and auto-start
    #[arg(long)]
    sw: bool,

    /// Set timer duration in minutes and auto-start timer mode
    #[arg(long = "timer", value_name = "MINS")]
    timer_mins: Option<u32>,

    /// Print time once and exit (no TUI)
    #[arg(long)]
    once: bool,

    /// Output format for --once mode (human, unix, iso, hex)
    #[arg(long, default_value = "human")]
    format: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    // Handle --once mode: print and exit without TUI
    if cli.once {
        let now = chrono::Local::now();
        let output = match cli.format.as_str() {
            "unix" | "timestamp" | "epoch" => now.timestamp().to_string(),
            "iso" | "iso8601" => now.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
            "hex" => {
                let h = now.format("%H").to_string().parse::<u32>().unwrap_or(0);
                let m = now.format("%M").to_string().parse::<u32>().unwrap_or(0);
                let s = now.format("%S").to_string().parse::<u32>().unwrap_or(0);
                format!("{h:02X}:{m:02X}:{s:02X}")
            }
            _ => {
                // Default human-readable
                now.format("%Y-%m-%d %H:%M:%S").to_string()
            }
        };
        println!("{output}");
        return Ok(());
    }

    let terminal = ratatui::init();
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
        previous_hook(panic_info);
    }));
    if let Err(error) = enable_mouse_capture() {
        ratatui::restore();
        return Err(error.into());
    }

    let result = App::new_with_cli(cli).run(terminal);
    let mouse_restore = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result?;
    mouse_restore?;
    Ok(())
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    /// Shared rendering context for all modes.
    ctx: RenderContext,
    /// All display mode implementations.
    modes: Vec<Box<dyn Mode>>,
    /// Index of the currently active mode.
    active_mode_index: usize,
    /// Settings dialog state.
    settings_dialog: SettingsDialog,
    /// Mode picker dialog state.
    mode_dialog: ModeDialog,
    /// Countdown event management dialog state.
    countdown_dialog: CountdownDialog,
    /// Background animation state.
    background_state: BackgroundState,
    /// System monitor for reactive backgrounds (lazy initialized).
    system_monitor: Option<SystemMonitor>,
    /// Weather monitor for dynamic weather background (lazy initialized).
    weather_monitor: Option<WeatherMonitor>,
    /// Whether help overlay is visible.
    show_help: bool,
    /// Whether demo mode is active (auto-cycle).
    demo_mode: bool,
    /// Timer for demo mode theme cycling.
    demo_theme_cycle: Instant,
    /// Timer for demo mode background cycling.
    demo_bg_cycle: Instant,
    /// Timer for demo mode font cycling.
    demo_font_cycle: Instant,
    /// Terminal area from the latest frame, used to resolve mouse hit targets.
    frame_area: ratatui::layout::Rect,
}

impl App {
    /// Construct a new instance of [`App`].
    // A `Default` impl would only forward to `new()` and is never used; the entry point
    // always builds via `new_with_cli`.
    #[allow(clippy::new_without_default)]
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

        let on_complete_command = config.on_complete.clone();

        // Build RenderContext from config
        let ctx = RenderContext {
            time_format: config.time_format,
            color_theme: config.color_theme,
            animation_style: config.animation_style,
            animation_speed: config.animation_speed,
            colon_blink: config.colon_blink,
            show_seconds: config.show_seconds,
            background_style: config.background_style,
            current_font: config.font_name.clone(),
            font_registry,
            config,
            animation_start: Instant::now(),
            flash_intensity: 0.0,
            flash_start: None,
            screensaver_mode: false,
            on_complete_command,
            desktop_notifications: false,
            sunrise_sunset: None,
        };

        // Create all mode structs
        let modes: Vec<Box<dyn Mode>> = vec![
            Box::new(ClockMode::new()),
            Box::new(PomodoroMode::new(
                ctx.config.pomodoro_work_mins,
                ctx.config.pomodoro_sessions_completed,
                ctx.config.pomodoro_total_focus_mins,
            )),
            Box::new(TimerMode::new(ctx.config.timer_duration_mins)),
            Box::new(StopwatchMode::new()),
            Box::new(WorldClockMode::new(&ctx.config.world_clock_zones)),
            Box::new(CountdownMode::new()),
        ];

        Self {
            running: false,
            ctx,
            modes,
            active_mode_index: 0,
            settings_dialog,
            mode_dialog: ModeDialog::new(),
            countdown_dialog: CountdownDialog::new(),
            background_state: BackgroundState::new(),
            system_monitor,
            weather_monitor,
            show_help: false,
            demo_mode: false,
            demo_theme_cycle: Instant::now(),
            demo_bg_cycle: Instant::now(),
            demo_font_cycle: Instant::now(),
            frame_area: ratatui::layout::Rect::default(),
        }
    }

    /// Construct a new instance with CLI overrides applied.
    fn new_with_cli(cli: Cli) -> Self {
        let mut app = Self::new();

        if let Some(font_name) = cli.font {
            let fonts = app.ctx.font_registry.list_fonts();
            if let Some(matched) = fonts.iter().find(|f| f.eq_ignore_ascii_case(&font_name)) {
                app.ctx.current_font = matched.to_string();
            } else {
                app.ctx.current_font = font_name;
            }
        }

        if let Some(theme_str) = cli.theme
            && let Some(theme) = parse_color_theme(&theme_str)
        {
            app.ctx.color_theme = theme;
        }

        if let Some(bg_str) = cli.background
            && let Some(bg) = parse_background_style(&bg_str)
        {
            app.ctx.background_style = bg;
            app.update_background_monitors();
        }

        if let Some(mode_str) = cli.mode
            && let Some(mode) = parse_display_mode(&mode_str)
        {
            // Find the matching mode index
            if let Some(idx) = app.modes.iter().position(|m| m.display_mode() == mode) {
                app.active_mode_index = idx;
            }
        }

        if !cli.timezones.is_empty() {
            app.ctx.config.world_clock_zones = cli.timezones;
            // Update the WorldClockMode entries via downcast
            for m in app.modes.iter_mut() {
                if let Some(wc) = m.as_any_mut().downcast_mut::<WorldClockMode>() {
                    wc.update_entries(&app.ctx.config.world_clock_zones);
                    break;
                }
            }
        }

        if cli.on_complete.is_some() {
            app.ctx.on_complete_command = cli.on_complete;
        }

        app.ctx.screensaver_mode = cli.screensaver;
        app.ctx.desktop_notifications = app.ctx.config.desktop_notifications;
        app.demo_mode = cli.demo;

        // In screensaver mode with no background, default to Aurora
        if app.ctx.screensaver_mode && app.ctx.background_style == BackgroundStyle::None {
            app.ctx.background_style = BackgroundStyle::Aurora;
            app.update_background_monitors();
        }

        // Quick-start shortcuts
        if cli.pomo
            && let Some(idx) = app
                .modes
                .iter()
                .position(|m| m.display_mode() == DisplayMode::Pomodoro)
        {
            app.active_mode_index = idx;
        }
        if cli.sw
            && let Some(idx) = app
                .modes
                .iter()
                .position(|m| m.display_mode() == DisplayMode::Stopwatch)
        {
            app.active_mode_index = idx;
        }
        if let Some(mins) = cli.timer_mins
            && let Some(idx) = app
                .modes
                .iter()
                .position(|m| m.display_mode() == DisplayMode::Timer)
        {
            app.active_mode_index = idx;
            if let Some(tm) = app.modes[idx].as_any_mut().downcast_mut::<TimerMode>() {
                tm.sync_duration(mins);
            }
        }

        // Auto-start if any quick-start flag is set
        let auto_start = cli.start || cli.pomo || cli.sw || cli.timer_mins.is_some();
        if auto_start {
            let idx = app.active_mode_index;
            let mode = &mut app.modes[idx];
            if let Some(pm) = mode.as_any_mut().downcast_mut::<PomodoroMode>() {
                pm.toggle();
            } else if let Some(tm) = mode.as_any_mut().downcast_mut::<TimerMode>() {
                tm.toggle();
            } else if let Some(sw) = mode.as_any_mut().downcast_mut::<StopwatchMode>() {
                sw.toggle();
            }
        }

        app
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
        self.frame_area = frame.area();
        // Demo mode auto-cycling
        if self.demo_mode {
            if self.demo_theme_cycle.elapsed() >= Duration::from_secs(5) {
                self.ctx.color_theme = self.ctx.color_theme.next();
                self.demo_theme_cycle = Instant::now();
            }
            if self.demo_bg_cycle.elapsed() >= Duration::from_secs(15) {
                self.ctx.background_style = self.ctx.background_style.next();
                self.update_background_monitors();
                self.demo_bg_cycle = Instant::now();
            }
            if self.demo_font_cycle.elapsed() >= Duration::from_secs(30) {
                let fonts = self.ctx.font_registry.list_fonts();
                let current_idx = fonts
                    .iter()
                    .position(|f| f == &self.ctx.current_font)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % fonts.len();
                self.ctx.current_font = fonts[next_idx].to_string();
                self.demo_font_cycle = Instant::now();
            }
        }

        // Get metrics for reactive backgrounds
        let elapsed_ms = self.ctx.elapsed_ms();
        let metrics = self.system_monitor.as_ref().map(|m| m.get_metrics());

        // Resolve weather background to actual style
        let effective_background = if self.ctx.background_style == BackgroundStyle::Weather {
            self.weather_monitor
                .as_ref()
                .map(|m| m.get_background())
                .unwrap_or(BackgroundStyle::Starfield)
        } else {
            self.ctx.background_style
        };

        // Render background first (behind everything else)
        self.background_state.render(
            frame,
            effective_background,
            elapsed_ms,
            self.ctx.animation_speed,
            metrics.as_ref(),
        );

        // Update sunrise/sunset from weather monitor
        self.ctx.sunrise_sunset = self
            .weather_monitor
            .as_ref()
            .and_then(|m| m.get_sunrise_sunset());

        // Dispatch to active mode: update then render
        let mode = &mut self.modes[self.active_mode_index];
        mode.update(&mut self.ctx);
        mode.render(frame, &self.ctx);

        // Render dialogs (last visible wins) — settings, then mode picker, then countdown editor.
        let color = self.ctx.color();
        let dim = self.ctx.dim_color();
        let muted = self.ctx.muted_color();
        let area = frame.area();
        self.settings_dialog.render(frame, area, color, dim, muted);
        self.mode_dialog.render(
            frame,
            area,
            color,
            dim,
            muted,
            self.modes[self.active_mode_index].display_mode(),
        );
        self.countdown_dialog.render(frame, area, color, dim, muted);

        // Render help overlay
        self.render_help_overlay(frame);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    /// Uses polling with timeout for real-time clock updates.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        // Poll for events with 100ms timeout for smooth clock updates
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(mouse) => self.on_mouse_event(mouse),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        // Dialogs are modal: route to whichever is visible.
        if self.settings_dialog.visible {
            self.handle_settings_key(key);
            return;
        }
        if self.countdown_dialog.visible {
            self.handle_countdown_dialog_key(key);
            return;
        }
        if self.mode_dialog.visible {
            self.handle_mode_dialog_key(key);
            return;
        }

        // Toggle help overlay
        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return;
        }
        // Dismiss help on any key if visible
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Let the active mode handle the key first
        if self.modes[self.active_mode_index].handle_key(key, &mut self.ctx) {
            return;
        }

        // Mode-conditional global keys.
        if let KeyCode::Char('e') = key.code
            && self.modes[self.active_mode_index].display_mode() == DisplayMode::Countdown
        {
            self.open_countdown_dialog();
            return;
        }

        // Global keybindings
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('m')) => self.toggle_display_mode(),
            (_, KeyCode::Char('M')) => self.open_mode_dialog(),
            (_, KeyCode::Char('t')) => {
                self.ctx.time_format = self.ctx.time_format.toggle();
            }
            (_, KeyCode::Char('c')) => {
                self.ctx.color_theme = self.ctx.color_theme.next();
            }
            (_, KeyCode::Char('a')) => {
                self.ctx.animation_style = self.ctx.animation_style.next();
            }
            (_, KeyCode::Char('b')) => {
                self.ctx.background_style = self.ctx.background_style.next();
                self.update_background_monitors();
            }
            (_, KeyCode::Char('s')) => self.open_settings(),
            _ => {}
        }
    }

    fn on_mouse_event(&mut self, mouse: MouseEvent) {
        if self.show_help {
            if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
                self.show_help = false;
            }
            return;
        }
        if self.countdown_dialog.visible {
            let action = self.countdown_dialog.handle_mouse(mouse, self.frame_area);
            self.apply_countdown_action(action);
            return;
        }
        if self.mode_dialog.visible {
            let action = self.mode_dialog.handle_mouse(mouse, self.frame_area);
            self.apply_mode_action(action);
            return;
        }
        if self.settings_dialog.visible {
            if let Some(code) = self.settings_dialog.handle_mouse(mouse, self.frame_area) {
                self.handle_settings_key(KeyEvent::new(code, KeyModifiers::NONE));
            }
            return;
        }

        let mode = &*self.modes[self.active_mode_index];
        match root_mouse_action(
            mode,
            self.ctx.screensaver_mode,
            self.ctx.config.minimal_mode,
            self.frame_area,
            mouse,
        ) {
            RootMouseAction::None => {}
            RootMouseAction::Key(code) => {
                self.on_key_event(KeyEvent::new(code, KeyModifiers::NONE));
            }
            RootMouseAction::Quit => self.quit(),
        }
    }

    /// Open the mode picker dialog with the cursor on the active mode.
    fn open_mode_dialog(&mut self) {
        let current = self.modes[self.active_mode_index].display_mode();
        self.mode_dialog.open(current);
    }

    /// Route key events to the mode picker and act on the result.
    fn handle_mode_dialog_key(&mut self, key: KeyEvent) {
        let action = self.mode_dialog.handle_key(key);
        self.apply_mode_action(action);
    }

    fn apply_mode_action(&mut self, action: ModeAction) {
        match action {
            ModeAction::Continue => {}
            ModeAction::Cancel => {}
            ModeAction::Select(mode) => {
                if let Some(idx) = self.modes.iter().position(|m| m.display_mode() == mode) {
                    self.active_mode_index = idx;
                }
            }
        }
    }

    /// Open the countdown event management dialog with a snapshot of current events.
    fn open_countdown_dialog(&mut self) {
        let events = self.ctx.config.countdown_events.clone();
        self.countdown_dialog.open(events);
    }

    /// Route key events to the countdown editor and persist on commit.
    fn handle_countdown_dialog_key(&mut self, key: KeyEvent) {
        let action = self.countdown_dialog.handle_key(key);
        self.apply_countdown_action(action);
    }

    fn apply_countdown_action(&mut self, action: CountdownAction) {
        match action {
            CountdownAction::Continue => {}
            CountdownAction::Cancel => {}
            CountdownAction::Commit(events) => {
                self.ctx.config.countdown_events = events;
                if let Err(e) = self.ctx.config.save() {
                    eprintln!("Warning: Failed to save config: {e}");
                }
            }
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
        self.ctx.current_font = self.settings_dialog.selected_font().to_string();
        self.ctx.color_theme = self.settings_dialog.color_theme;
        self.ctx.time_format = self.settings_dialog.time_format;
        self.ctx.animation_style = self.settings_dialog.animation_style;
        self.ctx.animation_speed = self.settings_dialog.animation_speed;
        self.ctx.colon_blink = self.settings_dialog.colon_blink;
        self.ctx.show_seconds = self.settings_dialog.show_seconds;
        self.ctx.background_style = self.settings_dialog.background_style;
        // Update pomodoro config (will take effect on next timer reset)
        self.ctx.config.pomodoro_work_mins = self.settings_dialog.pomodoro_work_mins;
        self.ctx.config.pomodoro_break_mins = self.settings_dialog.pomodoro_break_mins;
        self.ctx.config.pomodoro_long_break_mins = self.settings_dialog.pomodoro_long_break_mins;
        self.ctx.config.pomodoro_sound = self.settings_dialog.pomodoro_sound;
        self.ctx.config.desktop_notifications = self.settings_dialog.desktop_notifications;
        // Update timer duration via downcast
        let new_timer_mins = self.settings_dialog.timer_duration_mins;
        self.ctx.config.timer_duration_mins = new_timer_mins;
        for m in self.modes.iter_mut() {
            if let Some(tm) = m.as_any_mut().downcast_mut::<TimerMode>() {
                tm.sync_duration(new_timer_mins);
                break;
            }
        }
        self.update_background_monitors();
    }

    /// Open settings dialog with current settings.
    fn open_settings(&mut self) {
        self.settings_dialog.open(
            &self.ctx.current_font,
            self.ctx.color_theme,
            self.ctx.time_format,
            self.ctx.animation_style,
            self.ctx.animation_speed,
            self.ctx.colon_blink,
            self.ctx.show_seconds,
            self.ctx.background_style,
            self.ctx.config.pomodoro_work_mins,
            self.ctx.config.pomodoro_break_mins,
            self.ctx.config.pomodoro_long_break_mins,
            self.ctx.config.pomodoro_sound,
            self.ctx.config.desktop_notifications,
            self.ctx.config.timer_duration_mins,
        );
    }

    /// Save current settings to config file and close dialog.
    fn save_settings(&mut self) {
        // Update and save config (values already applied via preview)
        self.ctx.config.font_name = self.ctx.current_font.clone();
        self.ctx.config.color_theme = self.ctx.color_theme;
        self.ctx.config.time_format = self.ctx.time_format;
        self.ctx.config.animation_style = self.ctx.animation_style;
        self.ctx.config.animation_speed = self.ctx.animation_speed;
        self.ctx.config.colon_blink = self.ctx.colon_blink;
        self.ctx.config.show_seconds = self.ctx.show_seconds;
        self.ctx.config.background_style = self.ctx.background_style;
        self.ctx.config.desktop_notifications = self.settings_dialog.desktop_notifications;
        self.ctx.config.timer_duration_mins = self.settings_dialog.timer_duration_mins;

        if let Err(e) = self.ctx.config.save() {
            eprintln!("Warning: Failed to save config: {e}");
        }

        self.settings_dialog.close();
    }

    /// Cancel settings and revert to original values.
    fn cancel_settings(&mut self) {
        // Revert to original values
        self.ctx.current_font = self.settings_dialog.original_font().to_string();
        let original = *self.settings_dialog.original();
        self.ctx.color_theme = original.color_theme;
        self.ctx.time_format = original.time_format;
        self.ctx.animation_style = original.animation_style;
        self.ctx.animation_speed = original.animation_speed;
        self.ctx.colon_blink = original.colon_blink;
        self.ctx.show_seconds = original.show_seconds;
        self.ctx.background_style = original.background_style;
        self.ctx.config.pomodoro_work_mins = original.pomodoro_work_mins;
        self.ctx.config.pomodoro_break_mins = original.pomodoro_break_mins;
        self.ctx.config.pomodoro_long_break_mins = original.pomodoro_long_break_mins;
        self.ctx.config.pomodoro_sound = original.pomodoro_sound;
        self.ctx.config.desktop_notifications = original.desktop_notifications;
        let orig_timer_mins = original.timer_duration_mins;
        self.ctx.config.timer_duration_mins = orig_timer_mins;
        // Sync timer mode via downcast
        for m in self.modes.iter_mut() {
            if let Some(tm) = m.as_any_mut().downcast_mut::<TimerMode>() {
                tm.sync_duration(orig_timer_mins);
                break;
            }
        }
        self.update_background_monitors();

        self.settings_dialog.close();
    }

    /// Start or stop background monitors based on current background style.
    fn update_background_monitors(&mut self) {
        // System monitor for reactive backgrounds
        if self.ctx.background_style.is_reactive() && self.system_monitor.is_none() {
            let monitor = SystemMonitor::new();
            monitor.start();
            self.system_monitor = Some(monitor);
        } else if !self.ctx.background_style.is_reactive() && self.system_monitor.is_some() {
            self.system_monitor = None;
        }

        // Weather monitor for weather background
        if self.ctx.background_style.requires_weather() && self.weather_monitor.is_none() {
            let monitor = WeatherMonitor::new(self.ctx.config.weather_location.clone());
            monitor.start();
            self.weather_monitor = Some(monitor);
        } else if !self.ctx.background_style.requires_weather() && self.weather_monitor.is_some() {
            self.weather_monitor = None;
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    /// Cycle through display modes by advancing active_mode_index.
    fn toggle_display_mode(&mut self) {
        self.active_mode_index = (self.active_mode_index + 1) % self.modes.len();
    }

    /// Render the help overlay showing all keybindings.
    fn render_help_overlay(&self, frame: &mut Frame) {
        if !self.show_help {
            return;
        }

        let area = frame.area();
        let width = 56u16.min(area.width.saturating_sub(4));
        let height = 35u16.min(area.height.saturating_sub(2));
        let overlay_area = dialog::centered_rect(area, width, height);

        frame.render_widget(Clear, overlay_area);

        let accent = self.ctx.color();
        let fg = Color::White;
        let dim = self.ctx.muted_color();

        let key_line = |key: &'static str, desc: &'static str| -> Line<'static> {
            Line::from(vec![
                Span::styled(key, Style::default().fg(fg).bold()),
                Span::styled(desc, Style::default().fg(dim)),
            ])
        };

        let help_lines = vec![
            Line::from(Span::styled("Controls", Style::default().fg(accent).bold())).centered(),
            Line::from(""),
            Line::from(Span::styled("  Global", Style::default().fg(accent).bold())),
            key_line("    q / Esc     ", "Quit"),
            key_line("    m           ", "Cycle mode"),
            key_line("    M           ", "Pick mode (dialog)"),
            key_line("    t           ", "Toggle 12/24 hour"),
            key_line("    c           ", "Cycle color theme"),
            key_line("    a           ", "Cycle animation style"),
            key_line("    b           ", "Cycle background"),
            key_line("    s           ", "Open settings"),
            key_line("    ?           ", "Toggle this help"),
            Line::from(""),
            Line::from(Span::styled(
                "  Pomodoro",
                Style::default().fg(accent).bold(),
            )),
            key_line("    Space       ", "Start / Pause"),
            key_line("    r           ", "Reset current phase"),
            key_line("    n           ", "Skip to next phase"),
            Line::from(""),
            Line::from(Span::styled("  Timer", Style::default().fg(accent).bold())),
            key_line("    Space       ", "Start / Pause"),
            key_line("    r           ", "Reset"),
            key_line("    + / -       ", "Adjust duration"),
            Line::from(""),
            Line::from(Span::styled(
                "  Stopwatch",
                Style::default().fg(accent).bold(),
            )),
            key_line("    Space       ", "Start / Pause"),
            key_line("    r           ", "Reset"),
            key_line("    l           ", "Lap"),
            Line::from(""),
            Line::from(Span::styled(
                "  Countdown",
                Style::default().fg(accent).bold(),
            )),
            key_line("    n / →       ", "Next event"),
            key_line("    p / ←       ", "Previous event"),
            key_line("    e           ", "Manage events (dialog)"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key or click to close",
                Style::default().fg(dim),
            ))
            .centered(),
        ];

        let help_widget = Paragraph::new(help_lines).block(dialog::dialog_block(" Help ", accent));

        frame.render_widget(help_widget, overlay_area);
    }
}

/// Parse a string into a ColorTheme (case-insensitive).
fn parse_color_theme(s: &str) -> Option<ColorTheme> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "cyan" => Some(ColorTheme::Cyan),
        "green" => Some(ColorTheme::Green),
        "white" => Some(ColorTheme::White),
        "magenta" => Some(ColorTheme::Magenta),
        "yellow" => Some(ColorTheme::Yellow),
        "red" => Some(ColorTheme::Red),
        "blue" => Some(ColorTheme::Blue),
        "rainbow" => Some(ColorTheme::Rainbow),
        "rainbowvertical" | "rainbow-vertical" => Some(ColorTheme::RainbowVertical),
        "gradientwarm" | "gradient-warm" | "warm" => Some(ColorTheme::GradientWarm),
        "gradientcool" | "gradient-cool" | "cool" => Some(ColorTheme::GradientCool),
        "gradientocean" | "gradient-ocean" | "ocean" => Some(ColorTheme::GradientOcean),
        "gradientneon" | "gradient-neon" | "neon" => Some(ColorTheme::GradientNeon),
        "gradientfire" | "gradient-fire" | "fire" => Some(ColorTheme::GradientFire),
        "gradientfrost" | "gradient-frost" | "frost" => Some(ColorTheme::GradientFrost),
        "gradientaurora" | "gradient-aurora" | "aurora" => Some(ColorTheme::GradientAurora),
        "gradientwinter" | "gradient-winter" | "winter" => Some(ColorTheme::GradientWinter),
        "gradientsakura" | "gradient-sakura" | "sakura" => Some(ColorTheme::GradientSakura),
        _ => None,
    }
}

/// Parse a string into a BackgroundStyle (case-insensitive).
fn parse_background_style(s: &str) -> Option<BackgroundStyle> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "none" => Some(BackgroundStyle::None),
        "starfield" | "stars" => Some(BackgroundStyle::Starfield),
        "matrix" | "matrixrain" | "matrix-rain" => Some(BackgroundStyle::MatrixRain),
        "gradient" | "gradientwave" | "gradient-wave" => Some(BackgroundStyle::GradientWave),
        "snowfall" | "snow" => Some(BackgroundStyle::Snowfall),
        "frost" => Some(BackgroundStyle::Frost),
        "aurora" => Some(BackgroundStyle::Aurora),
        "sunny" | "sun" => Some(BackgroundStyle::Sunny),
        "rainy" | "rain" => Some(BackgroundStyle::Rainy),
        "stormy" | "storm" => Some(BackgroundStyle::Stormy),
        "windy" | "wind" => Some(BackgroundStyle::Windy),
        "cloudy" | "clouds" => Some(BackgroundStyle::Cloudy),
        "foggy" | "fog" => Some(BackgroundStyle::Foggy),
        "weather" => Some(BackgroundStyle::Weather),
        "dawn" | "twilightdawn" | "twilight-dawn" => Some(BackgroundStyle::TwilightDawn),
        "dusk" | "twilightdusk" | "twilight-dusk" => Some(BackgroundStyle::TwilightDusk),
        "cherryblossom" | "cherry-blossom" | "sakura" => Some(BackgroundStyle::CherryBlossom),
        "systempulse" | "system-pulse" | "pulse" => Some(BackgroundStyle::SystemPulse),
        "resourcewave" | "resource-wave" | "resource" => Some(BackgroundStyle::ResourceWave),
        "dataflow" | "data-flow" => Some(BackgroundStyle::DataFlow),
        "heatmap" | "heat-map" | "heat" => Some(BackgroundStyle::HeatMap),
        _ => None,
    }
}

/// Parse a string into a DisplayMode (case-insensitive).
fn parse_display_mode(s: &str) -> Option<DisplayMode> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "clock" => Some(DisplayMode::Clock),
        "pomodoro" => Some(DisplayMode::Pomodoro),
        "timer" => Some(DisplayMode::Timer),
        "stopwatch" => Some(DisplayMode::Stopwatch),
        "worldclock" | "world-clock" | "world" => Some(DisplayMode::WorldClock),
        "countdown" | "life" => Some(DisplayMode::Countdown),
        _ => None,
    }
}

#[cfg(test)]
mod mouse_tests {
    use super::*;

    fn click(button: MouseButton) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(button),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn release(button: MouseButton) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Up(button),
            ..click(button)
        }
    }

    #[test]
    fn footer_click_resolves_to_the_existing_key_action() {
        let mode = TimerMode::new(5);
        let area = ratatui::layout::Rect::new(0, 0, 80, 20);

        assert_eq!(footer_key_at(&mode, area, 41, 19), Some(KeyCode::Char('q')));
        assert_eq!(footer_key_at(&mode, area, 0, 19), None);
    }

    #[test]
    fn root_mouse_behavior_handles_chrome_free_views() {
        let mode = ClockMode::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 20);

        assert_eq!(
            root_mouse_action(&mode, true, false, area, release(MouseButton::Left)),
            RootMouseAction::Quit
        );
        assert_eq!(
            root_mouse_action(&mode, false, true, area, release(MouseButton::Left)),
            RootMouseAction::Key(KeyCode::Char('z'))
        );
        assert_eq!(
            root_mouse_action(&mode, true, false, area, release(MouseButton::Right)),
            RootMouseAction::None
        );
    }

    #[test]
    fn root_clicks_trigger_on_button_release() {
        let mode = ClockMode::new();
        let area = ratatui::layout::Rect::new(0, 0, 80, 20);
        let mut mouse = click(MouseButton::Left);

        assert_eq!(
            root_mouse_action(&mode, true, false, area, mouse),
            RootMouseAction::None
        );

        mouse.kind = MouseEventKind::Up(MouseButton::Left);
        assert_eq!(
            root_mouse_action(&mode, true, false, area, mouse),
            RootMouseAction::Quit
        );
    }
}
