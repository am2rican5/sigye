//! Stateless background animations (computed from position and time only).

use ratatui::{
    style::{Color, Style},
    text::Span,
};
use sigye_core::AnimationSpeed;

use crate::chars::{FROST_CHARS, STAR_CHARS};
use crate::color::hsl_to_rgb;

/// Render a starfield character using pseudo-random twinkling.
pub fn render_starfield_char(
    x: u16,
    y: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let x = x as usize;
    let y = y as usize;
    let period = speed.star_twinkle_period_ms();
    let frame_num = elapsed_ms / period;

    // Use deterministic "random" based on position and time
    let seed = (x.wrapping_mul(31))
        .wrapping_add(y.wrapping_mul(17))
        .wrapping_add(frame_num as usize);

    // Only show stars at ~3% of positions
    if seed % 100 < 3 {
        let char_idx = seed % STAR_CHARS.len();
        let ch = STAR_CHARS[char_idx];

        // Vary brightness based on position
        let brightness = (seed % 3) as u8;
        let color = match brightness {
            0 => Color::Rgb(60, 60, 80),    // Dim
            1 => Color::Rgb(100, 100, 140), // Medium
            _ => Color::Rgb(150, 150, 200), // Bright
        };

        Span::styled(ch.to_string(), Style::new().fg(color))
    } else {
        Span::raw(" ")
    }
}

/// Render a gradient wave character.
pub fn render_gradient_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let period = speed.gradient_scroll_period_ms();
    let time_phase = (elapsed_ms % period) as f32 / period as f32;

    let x_norm = x as f32 / width.max(1) as f32;
    let y_norm = y as f32 / height.max(1) as f32;

    // Create a diagonal wave pattern
    let wave = ((x_norm + y_norm * 0.5 + time_phase) * 2.0 * std::f32::consts::PI).sin();
    let intensity = (wave + 1.0) / 2.0; // Normalize to 0..1

    // Use block characters with varying density
    let ch = if intensity < 0.25 {
        ' '
    } else if intensity < 0.5 {
        '░'
    } else if intensity < 0.75 {
        '▒'
    } else {
        '▓'
    };

    // Color gradient from deep blue to cyan to purple
    let hue_offset = time_phase * 360.0;
    let base_hue = (x_norm * 60.0 + hue_offset) % 360.0;

    let color = hsl_to_rgb(base_hue, 0.7, 0.15 + intensity * 0.2);

    if ch == ' ' {
        Span::raw(" ")
    } else {
        Span::styled(ch.to_string(), Style::new().fg(color))
    }
}

/// Render a frost crystal character.
pub fn render_frost_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let x_f = x as f32;
    let y_f = y as f32;
    let w_f = width as f32;
    let h_f = height as f32;

    // Calculate distance from nearest edge
    let edge_dist_x = x_f.min(w_f - 1.0 - x_f);
    let edge_dist_y = y_f.min(h_f - 1.0 - y_f);
    let edge_dist = edge_dist_x.min(edge_dist_y * 2.0);

    // Frost growth from edges - controlled by time
    let growth_period = speed.frost_growth_period_ms();
    let growth_phase =
        ((elapsed_ms % growth_period) as f32 / growth_period as f32) * std::f32::consts::PI;
    let growth_factor = growth_phase.sin() * 0.3 + 0.7;

    let max_frost_depth = (w_f.min(h_f) / 3.0) * growth_factor;

    if edge_dist > max_frost_depth {
        return Span::raw(" ");
    }

    // Crystal pattern using pseudo-random based on position
    let seed = (x as usize)
        .wrapping_mul(31)
        .wrapping_add((y as usize).wrapping_mul(17));

    // Density decreases toward center
    let density_threshold = ((edge_dist / max_frost_depth) * 85.0) as usize;
    if seed % 100 > (100 - density_threshold).max(15) {
        return Span::raw(" ");
    }

    // Character selection
    let char_idx = seed % FROST_CHARS.len();
    let ch = FROST_CHARS[char_idx];

    // Color based on distance from edge (darker toward center)
    let depth_ratio = edge_dist / max_frost_depth;
    let base_color = if depth_ratio < 0.3 {
        (200u8, 230u8, 255u8)
    } else if depth_ratio < 0.6 {
        (135, 206, 235)
    } else {
        (70, 130, 180)
    };

    // Add shimmer effect
    let shimmer = (elapsed_ms as f32 / 500.0 + seed as f32 * 0.1).sin() * 0.15 + 0.85;
    let r = (base_color.0 as f32 * shimmer) as u8;
    let g = (base_color.1 as f32 * shimmer) as u8;
    let b = (base_color.2 as f32 * shimmer) as u8;

    Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
}

/// Render an aurora borealis character.
pub fn render_aurora_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let x_norm = x as f32 / width.max(1) as f32;
    let y_norm = y as f32 / height.max(1) as f32;

    let period = speed.aurora_wave_period_ms();
    let time_phase = (elapsed_ms % period) as f32 / period as f32;

    // Multiple overlapping waves for aurora curtain effect
    let wave1 = ((x_norm * 3.0 + time_phase * 2.0 * std::f32::consts::PI).sin() + 1.0) / 2.0;
    let wave2 = ((x_norm * 5.0 - time_phase * 1.5 * std::f32::consts::PI + 1.0).sin() + 1.0) / 2.0;
    let wave3 = ((x_norm * 2.0 + time_phase * std::f32::consts::PI + 2.0).sin() + 1.0) / 2.0;

    // Combine waves
    let combined_wave = wave1 * 0.5 + wave2 * 0.3 + wave3 * 0.2;

    // Vertical falloff (aurora is brighter at top)
    let vertical_factor = 1.0 - y_norm.powf(0.5);

    // Final intensity
    let intensity = combined_wave * vertical_factor;

    if intensity < 0.15 {
        return Span::raw(" ");
    }

    // Select character based on intensity
    let ch = if intensity > 0.7 {
        '▓'
    } else if intensity > 0.5 {
        '▒'
    } else if intensity > 0.3 {
        '░'
    } else {
        return Span::raw(" ");
    };

    // Aurora colors - cycle through greens, blues, purples
    let color_phase = (elapsed_ms as f32 / 10000.0 + x_norm * 0.5) % 1.0;

    let (r, g, b) = if color_phase < 0.4 {
        // Green phase
        let t = color_phase / 0.4;
        (50, (127.0 + 128.0 * t) as u8, (80.0 + 50.0 * t) as u8)
    } else if color_phase < 0.7 {
        // Blue phase
        let t = (color_phase - 0.4) / 0.3;
        (
            (50.0 * (1.0 - t)) as u8,
            (255.0 - 100.0 * t) as u8,
            (150.0 + 105.0 * t) as u8,
        )
    } else {
        // Purple/pink phase
        let t = (color_phase - 0.7) / 0.3;
        (
            (80.0 + 80.0 * t) as u8,
            (155.0 - 50.0 * t) as u8,
            (255.0 - 30.0 * t) as u8,
        )
    };

    // Apply vertical dimming
    let dimming = 0.3 + vertical_factor * 0.7;
    let r = (r as f32 * dimming) as u8;
    let g = (g as f32 * dimming) as u8;
    let b = (b as f32 * dimming) as u8;

    Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
}

/// Render a twilight dawn background character (golden hour - sunrise).
pub fn render_twilight_dawn_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let y_norm = y as f32 / height.max(1) as f32;
    let x_norm = x as f32 / width.max(1) as f32;

    // Gradient from dark blue (top) to orange/pink (horizon at bottom)
    let (r, g, b) = if y_norm < 0.3 {
        // Upper sky - deep blue transitioning to lighter
        let t = y_norm / 0.3;
        (
            (15.0 + 40.0 * t) as u8,
            (20.0 + 50.0 * t) as u8,
            (60.0 + 60.0 * t) as u8,
        )
    } else if y_norm < 0.6 {
        // Middle sky - blue to pink/orange transition
        let t = (y_norm - 0.3) / 0.3;
        (
            (55.0 + 150.0 * t) as u8,
            (70.0 + 60.0 * t) as u8,
            (120.0 - 40.0 * t) as u8,
        )
    } else {
        // Horizon - golden orange to warm yellow
        let t = (y_norm - 0.6) / 0.4;
        (
            (205.0 + 50.0 * t).min(255.0) as u8,
            (130.0 + 70.0 * t) as u8,
            (80.0 + 40.0 * t) as u8,
        )
    };

    // Subtle shimmer effect
    let shimmer_period = speed.aurora_wave_period_ms();
    let shimmer =
        ((elapsed_ms % shimmer_period) as f32 / shimmer_period as f32 * 2.0 * std::f32::consts::PI)
            .sin()
            * 0.05
            + 0.95;

    // Sparse clouds/wisps pattern
    let seed = (x as usize)
        .wrapping_mul(31)
        .wrapping_add((y as usize).wrapping_mul(17));
    let cloud_wave = ((x_norm * 4.0 + elapsed_ms as f32 / 15000.0) * std::f32::consts::PI).sin();
    let cloud_threshold = 3 + (cloud_wave * 2.0).abs() as usize;

    let ch = if seed % 100 < cloud_threshold && y_norm > 0.2 && y_norm < 0.7 {
        '░'
    } else if seed % 200 < 2 && y_norm < 0.4 {
        '·'
    } else {
        return Span::raw(" ");
    };

    let r = (r as f32 * shimmer) as u8;
    let g = (g as f32 * shimmer) as u8;
    let b = (b as f32 * shimmer) as u8;

    Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
}

/// Render a twilight dusk background character (sunset).
pub fn render_twilight_dusk_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let y_norm = y as f32 / height.max(1) as f32;
    let x_norm = x as f32 / width.max(1) as f32;

    // Dusk: deeper purple at top, rich orange/red at horizon
    let (r, g, b) = if y_norm < 0.3 {
        // Upper sky - deep purple/blue
        let t = y_norm / 0.3;
        (
            (20.0 + 40.0 * t) as u8,
            (10.0 + 20.0 * t) as u8,
            (50.0 + 50.0 * t) as u8,
        )
    } else if y_norm < 0.6 {
        // Middle sky - purple to deep orange
        let t = (y_norm - 0.3) / 0.3;
        (
            (60.0 + 140.0 * t) as u8,
            (30.0 + 40.0 * t) as u8,
            (100.0 - 60.0 * t) as u8,
        )
    } else {
        // Horizon - deep orange to red
        let t = (y_norm - 0.6) / 0.4;
        (
            (200.0 + 55.0 * t).min(255.0) as u8,
            (70.0 + 50.0 * t) as u8,
            (40.0 + 20.0 * t) as u8,
        )
    };

    // Subtle shimmer effect
    let shimmer_period = speed.aurora_wave_period_ms();
    let shimmer =
        ((elapsed_ms % shimmer_period) as f32 / shimmer_period as f32 * 2.0 * std::f32::consts::PI)
            .sin()
            * 0.05
            + 0.95;

    // Sparse clouds/wisps pattern - slightly different from dawn
    let seed = (x as usize)
        .wrapping_mul(37)
        .wrapping_add((y as usize).wrapping_mul(19));
    let cloud_wave = ((x_norm * 3.0 - elapsed_ms as f32 / 12000.0) * std::f32::consts::PI).sin();
    let cloud_threshold = 4 + (cloud_wave * 2.5).abs() as usize;

    let ch = if seed % 100 < cloud_threshold && y_norm > 0.15 && y_norm < 0.65 {
        '░'
    } else if seed % 180 < 2 && y_norm < 0.35 {
        '·'
    } else {
        return Span::raw(" ");
    };

    let r = (r as f32 * shimmer) as u8;
    let g = (g as f32 * shimmer) as u8;
    let b = (b as f32 * shimmer) as u8;

    Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
}
