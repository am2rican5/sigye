//! Font registry for managing available fonts.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::bundled::BUNDLED_FONTS;
use crate::font::Font;
use crate::parser::parse_flf;

/// Registry of available fonts.
#[derive(Debug)]
pub struct FontRegistry {
    fonts: HashMap<String, Font>,
}

impl FontRegistry {
    /// Create a new registry with bundled fonts loaded.
    pub fn new() -> Self {
        let mut registry = Self {
            fonts: HashMap::new(),
        };

        // Load all bundled fonts
        for (name, content) in BUNDLED_FONTS {
            match parse_flf(name, content) {
                Ok(font) => {
                    registry.fonts.insert(name.to_string(), font);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load bundled font '{name}': {e}");
                }
            }
        }

        registry
    }

    /// Load custom fonts from a directory.
    pub fn load_custom_fonts(&mut self, fonts_dir: &Path) {
        if !fonts_dir.exists() {
            return;
        }

        let entries = match fs::read_dir(fonts_dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Warning: Failed to read fonts directory: {e}");
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "flf")
                && let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();

                    // Skip if already loaded (bundled fonts take precedence)
                    if self.fonts.contains_key(&name) {
                        continue;
                    }

                    match fs::read_to_string(&path) {
                        Ok(content) => match parse_flf(&name, &content) {
                            Ok(font) => {
                                self.fonts.insert(name, font);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to parse font '{}': {e}", path.display());
                            }
                        },
                        Err(e) => {
                            eprintln!("Warning: Failed to read font '{}': {e}", path.display());
                        }
                    }
                }
        }
    }

    /// Get a font by name.
    pub fn get(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name)
    }

    /// Get a font by name, or the default font if not found.
    pub fn get_or_default(&self, name: &str) -> &Font {
        self.fonts
            .get(name)
            .or_else(|| self.fonts.get("Standard"))
            .expect("Standard font should always be available")
    }

    /// List all available font names.
    pub fn list_fonts(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.fonts.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Check if a font exists.
    pub fn has_font(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }

    /// Get the number of loaded fonts.
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}
