//! Font struct and rendering functionality.

use std::collections::HashMap;

/// A FIGlet font containing character definitions.
#[derive(Debug, Clone)]
pub struct Font {
    /// Font name.
    pub name: String,
    /// Height in lines.
    pub height: usize,
    /// Character definitions.
    pub chars: HashMap<char, Vec<String>>,
}

impl Font {
    /// Render text using this font.
    ///
    /// Returns a vector of strings, one for each line of the output.
    pub fn render_text(&self, text: &str) -> Vec<String> {
        let mut lines: Vec<String> = vec![String::new(); self.height];

        for ch in text.chars() {
            if let Some(char_lines) = self.chars.get(&ch) {
                for (i, char_line) in char_lines.iter().enumerate() {
                    if i < lines.len() {
                        lines[i].push_str(char_line);
                    }
                }
            } else if let Some(space_lines) = self.chars.get(&' ') {
                // Use space for unknown characters
                for (i, space_line) in space_lines.iter().enumerate() {
                    if i < lines.len() {
                        lines[i].push_str(space_line);
                    }
                }
            }
        }

        lines
    }

    /// Get the width of a character.
    pub fn char_width(&self, ch: char) -> usize {
        self.chars
            .get(&ch)
            .and_then(|lines| lines.first())
            .map(|line| line.chars().count())
            .unwrap_or(0)
    }
}
