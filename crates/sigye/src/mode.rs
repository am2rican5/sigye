//! Display mode trait for extensible mode system.

use std::any::Any;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use sigye_core::DisplayMode;

use crate::context::RenderContext;
use crate::render::{self, FooterAction};

/// Trait implemented by each display mode (Clock, Pomodoro, Timer, etc.).
///
/// To add a new display mode:
/// 1. Add a variant to `DisplayMode` in `sigye-core`
/// 2. Create a new file in `modes/` implementing this trait
/// 3. Register it in `modes/mod.rs`
pub trait Mode {
    /// The display mode enum variant this mode corresponds to.
    fn display_mode(&self) -> DisplayMode;

    /// Update mode state (timers, counters, etc.). Called every frame.
    fn update(&mut self, ctx: &mut RenderContext);

    /// Render the mode's UI. Called every frame after update.
    fn render(&self, frame: &mut Frame, ctx: &RenderContext);

    /// Handle a key event. Return `true` if the key was consumed.
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut RenderContext) -> bool;

    /// Return actions shown in the mode footer.
    fn footer_actions(&self) -> Vec<FooterAction>;

    /// Return mode actions followed by the shared application actions.
    fn all_footer_actions(&self) -> Vec<FooterAction> {
        render::with_global_footer_actions(&self.footer_actions())
    }

    /// Downcast support for mode-specific operations.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
