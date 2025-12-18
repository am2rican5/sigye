//! ASCII art fonts for the sigye clock application.
//!
//! This crate provides FIGlet font parsing and rendering for the terminal clock.

mod bundled;
mod font;
mod parser;
mod registry;

pub use font::Font;
pub use parser::{ParseError, parse_flf};
pub use registry::FontRegistry;

// Re-export bundled font constants for direct access
pub use bundled::BUNDLED_FONTS;
