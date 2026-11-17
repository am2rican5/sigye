//! Character constants for background animations.

/// Characters used for starfield background.
pub const STAR_CHARS: &[char] = &['.', '*', '+', '·', '✦', '✧'];

/// Characters used for matrix rain. The katakana are the half-width forms: the
/// full-width ones take two cells each and knock the rain columns out of line.
pub const MATRIX_CHARS: &[char] = &[
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ',
    'ﾄ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// Characters used for snowfall background.
pub const SNOW_CHARS: &[char] = &['*', '·', '•', '❄', '❅', '❆', '✦', '✧', '°'];

/// Characters used for frost crystals.
pub const FROST_CHARS: &[char] = &['·', '•', '*', '×', '✕', '✱', '░'];

// Weather character constants

/// Characters used for rain drops - vertical streaks.
pub const RAIN_CHARS: &[char] = &['│', '|', '¦', '┃', '╏', '┊', '┆'];

/// Characters used for heavy storm rain - more intense.
pub const STORM_RAIN_CHARS: &[char] = &['┃', '║', '│', '|', '/', '\\'];

/// Characters used for sun rays and shimmer.
pub const SUN_CHARS: &[char] = &['·', '•', '*', '✦', '✧', '○', '◌'];

/// Characters used for wind streaks - horizontal motion.
pub const WIND_CHARS: &[char] = &['─', '-', '~', '∼', '≈', '━', '╌', '╍'];

/// Characters used for cloud puffs and density.
pub const CLOUD_CHARS: &[char] = &['░', '▒', '▓', '·', '•', '○', '◌', '◦'];

/// Characters used for fog/mist - soft wisps and dots.
pub const FOG_CHARS: &[char] = &['·', '.', '\'', ':', '°', '∙', ','];

/// Characters used for cherry blossom petals.
pub const PETAL_CHARS: &[char] = &['✿', '❀', '✾', '·', '°'];

#[cfg(test)]
mod tests {
    use super::*;

    /// Backgrounds place one glyph per terminal cell, so a wide glyph would shift
    /// everything to its right and push the end of the row off screen.
    #[test]
    fn every_background_glyph_is_one_cell_wide() {
        let sets = [
            STAR_CHARS,
            MATRIX_CHARS,
            SNOW_CHARS,
            FROST_CHARS,
            RAIN_CHARS,
            STORM_RAIN_CHARS,
            SUN_CHARS,
            WIND_CHARS,
            CLOUD_CHARS,
            FOG_CHARS,
            PETAL_CHARS,
        ];
        for &ch in sets.into_iter().flatten() {
            let width = ratatui::text::Span::raw(ch.to_string()).width();
            assert_eq!(width, 1, "{ch:?} is {width} cells wide");
        }
    }
}
