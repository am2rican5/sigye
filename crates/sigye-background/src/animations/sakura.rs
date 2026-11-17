//! Cherry blossom (sakura) petal animation.

use ratatui::{
    style::{Color, Style},
    text::Span,
};
use sigye_core::AnimationSpeed;

use crate::chars::PETAL_CHARS;

/// Sakura pink color palette for petals.
pub const SAKURA_COLORS: &[Color] = &[
    Color::Rgb(255, 183, 197), // #FFB7C5 - Sakura pink
    Color::Rgb(255, 192, 203), // #FFC0CB - Light pink
    Color::Rgb(255, 209, 220), // #FFD1DC - Pale pink
    Color::Rgb(255, 240, 245), // #FFF0F5 - Lavender blush
];

/// A single cherry blossom petal.
#[derive(Debug, Clone)]
pub struct Petal {
    /// Horizontal position (can be fractional for smooth movement).
    pub x: f32,
    /// Vertical position (can be fractional for smooth movement).
    pub y: f32,
    /// Original x position for sway calculation.
    pub base_x: f32,
    /// Phase offset for sinusoidal sway (0.0 - 2π).
    pub sway_phase: f32,
    /// Index into PETAL_CHARS.
    pub char_idx: usize,
    /// Index into SAKURA_COLORS.
    pub color_idx: usize,
}

/// Initialize petals for the given screen dimensions.
pub fn init_petals(width: u16, height: u16, seed: u64) -> Vec<Petal> {
    let area = (width as usize) * (height as usize);
    // Target ~4% coverage, but each petal is one character
    let num_petals = (area * 4 / 100).clamp(10, 500);

    let mut petals = Vec::with_capacity(num_petals);
    let mut rng_state = seed;

    for i in 0..num_petals {
        // Simple pseudo-random number generation (LCG)
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand1 = ((rng_state >> 32) as u32) as f32 / (u32::MAX as f32);

        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand2 = ((rng_state >> 32) as u32) as f32 / (u32::MAX as f32);

        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand3 = ((rng_state >> 32) as u32) as f32 / (u32::MAX as f32);

        let x = rand1 * width as f32;
        let y = rand2 * height as f32;
        let sway_phase = rand3 * std::f32::consts::TAU;

        // Larger characters (✿❀✾) appear less frequently
        let char_idx = if (i + seed as usize) % 5 < 2 {
            (i + seed as usize) % 3 // Large petals (indices 0-2)
        } else {
            3 + ((i + seed as usize) % 2) // Small petals (indices 3-4)
        };

        let color_idx = (i + seed as usize) % SAKURA_COLORS.len();

        petals.push(Petal {
            x,
            y,
            base_x: x,
            sway_phase,
            char_idx,
            color_idx,
        });
    }

    petals
}

/// Update petal positions based on elapsed time.
pub fn update_petals(
    petals: &mut [Petal],
    delta_ms: u64,
    width: u16,
    height: u16,
    speed: AnimationSpeed,
) {
    let fall_speed = speed.petal_fall_speed();
    let delta_sec = delta_ms as f32 / 1000.0;

    // Sway parameters
    let sway_amplitude = 2.0; // How far left/right petals sway
    let sway_period_ms = 3000.0; // Time for one complete sway cycle

    for petal in petals.iter_mut() {
        // Update vertical position (falling)
        // Larger characters fall slower (more air resistance)
        let size_factor = if petal.char_idx < 3 { 0.7 } else { 1.0 };
        petal.y += fall_speed * size_factor * delta_sec * 8.0;

        // Update horizontal position (swaying)
        // Calculate sway based on current time embedded in y position
        let sway_time = petal.y * 100.0 + petal.sway_phase * sway_period_ms;
        let sway_offset =
            (sway_time / sway_period_ms * std::f32::consts::TAU).sin() * sway_amplitude;
        petal.x = petal.base_x + sway_offset;

        // Respawn at top if fallen below screen
        if petal.y >= height as f32 {
            petal.y = -1.0;
            // Randomize new x position using current position as seed
            let seed_val = (petal.base_x * 17.0 + petal.sway_phase * 31.0).abs() as u64;
            let new_x = (seed_val % width as u64) as f32;
            petal.base_x = new_x;
            petal.x = petal.base_x;
        }
    }
}

/// Render a character at the given position, returning a petal if one exists there.
pub fn render_petal_char(petals: &[Petal], x: u16, y: u16) -> Span<'static> {
    // Check if any petal occupies this position
    for petal in petals {
        let px = petal.x.round() as i32;
        let py = petal.y.round() as i32;

        if px == x as i32 && py == y as i32 && py >= 0 {
            let ch = PETAL_CHARS[petal.char_idx % PETAL_CHARS.len()];
            let color = SAKURA_COLORS[petal.color_idx % SAKURA_COLORS.len()];
            return Span::styled(ch.to_string(), Style::new().fg(color));
        }
    }

    Span::raw(" ")
}
