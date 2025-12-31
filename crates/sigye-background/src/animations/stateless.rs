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
/// Fresh morning light with cool indigo-to-gold gradient and horizontal light rays.
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

    // Cool gradient: indigo (top) → lavender → soft pink → bright gold (horizon)
    let (r, g, b) = if y_norm < 0.25 {
        // Top sky - deep indigo to soft blue
        let t = y_norm / 0.25;
        (
            (30.0 + 50.0 * t) as u8,
            (40.0 + 50.0 * t) as u8,
            (100.0 + 50.0 * t) as u8,
        )
    } else if y_norm < 0.5 {
        // Upper-mid - lavender to soft pink
        let t = (y_norm - 0.25) / 0.25;
        (
            (80.0 + 100.0 * t) as u8,
            (90.0 + 50.0 * t) as u8,
            (150.0 + 30.0 * t) as u8,
        )
    } else if y_norm < 0.75 {
        // Lower-mid - soft pink to peach/warm gold
        let t = (y_norm - 0.5) / 0.25;
        (
            (180.0 + 75.0 * t).min(255.0) as u8,
            (140.0 + 50.0 * t) as u8,
            (180.0 - 60.0 * t) as u8,
        )
    } else {
        // Horizon - bright golden yellow
        let t = (y_norm - 0.75) / 0.25;
        (
            255,
            (190.0 + 50.0 * t).min(255.0) as u8,
            (120.0 + 30.0 * t) as u8,
        )
    };

    // Subtle shimmer effect
    let shimmer_period = speed.aurora_wave_period_ms();
    let shimmer =
        ((elapsed_ms % shimmer_period) as f32 / shimmer_period as f32 * 2.0 * std::f32::consts::PI)
            .sin()
            * 0.08
            + 0.92;

    // Pseudo-random seed for patterns
    let seed = (x as usize)
        .wrapping_mul(31)
        .wrapping_add((y as usize).wrapping_mul(17));

    // Dawn-specific patterns: fading stars at top, clouds in mid, horizontal rays at horizon
    let ch = if y_norm < 0.2 {
        // Fading stars at top of sky
        if seed % 100 < 3 {
            '·'
        } else {
            return Span::raw(" ");
        }
    } else if y_norm < 0.6 {
        // Soft wispy clouds in mid-sky
        let cloud_wave =
            ((x_norm * 4.0 + elapsed_ms as f32 / 15000.0) * std::f32::consts::PI).sin();
        let cloud_threshold = 3 + (cloud_wave * 2.0).abs() as usize;
        if seed % 100 < cloud_threshold {
            '░'
        } else {
            return Span::raw(" ");
        }
    } else {
        // Horizontal light rays emanating from horizon
        let ray_wave = ((x_norm * 8.0 + elapsed_ms as f32 / 8000.0) * std::f32::consts::PI).sin();
        let ray_intensity = (1.0 - (y_norm - 0.6) / 0.4) * 0.12; // Stronger near horizon
        let ray_threshold = (ray_intensity * 100.0 + ray_wave.abs() * 4.0) as usize;
        if seed % 100 < ray_threshold {
            '─'
        } else {
            return Span::raw(" ");
        }
    };

    let r = (r as f32 * shimmer) as u8;
    let g = (g as f32 * shimmer) as u8;
    let b = (b as f32 * shimmer) as u8;

    Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
}

/// Render a twilight dusk background character (sunset).
/// Warm sunset glow with deep purple-to-red gradient and vertical color bands.
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

    // Warm gradient: deep purple (top) → magenta → rich orange → deep red (horizon)
    let (r, g, b) = if y_norm < 0.25 {
        // Top sky - deep purple
        let t = y_norm / 0.25;
        (
            (50.0 + 40.0 * t) as u8,
            (30.0 + 10.0 * t) as u8,
            (90.0 + 20.0 * t) as u8,
        )
    } else if y_norm < 0.5 {
        // Upper-mid - magenta to crimson
        let t = (y_norm - 0.25) / 0.25;
        (
            (90.0 + 90.0 * t) as u8,
            (40.0 + 30.0 * t) as u8,
            (110.0 - 30.0 * t) as u8,
        )
    } else if y_norm < 0.75 {
        // Lower-mid - rich orange
        let t = (y_norm - 0.5) / 0.25;
        (
            (180.0 + 70.0 * t).min(255.0) as u8,
            (70.0 + 60.0 * t) as u8,
            (80.0 - 40.0 * t) as u8,
        )
    } else {
        // Horizon - deep red-orange, fiery glow
        let t = (y_norm - 0.75) / 0.25;
        (255, (130.0 - 50.0 * t) as u8, (40.0 - 10.0 * t) as u8)
    };

    // Subtle shimmer effect
    let shimmer_period = speed.aurora_wave_period_ms();
    let shimmer =
        ((elapsed_ms % shimmer_period) as f32 / shimmer_period as f32 * 2.0 * std::f32::consts::PI)
            .sin()
            * 0.06
            + 0.94;

    // Pseudo-random seed for patterns (different primes from dawn)
    let seed = (x as usize)
        .wrapping_mul(37)
        .wrapping_add((y as usize).wrapping_mul(19));

    // Dusk-specific patterns: emerging stars at top, vertical striations, color bands at horizon
    let ch = if y_norm < 0.2 {
        // Emerging stars at top of sky
        if seed % 100 < 5 {
            if seed.is_multiple_of(3) { '✦' } else { '·' }
        } else {
            return Span::raw(" ");
        }
    } else if y_norm < 0.5 {
        // Vertical striations for atmospheric depth
        let striation_wave =
            ((y_norm * 12.0 + elapsed_ms as f32 / 10000.0) * std::f32::consts::PI).sin();
        let striation_threshold = 2 + (striation_wave.abs() * 2.0) as usize;
        if seed % 100 < striation_threshold {
            '│'
        } else {
            return Span::raw(" ");
        }
    } else {
        // Color bands near horizon - layered sunset effect
        let band_wave = ((x_norm * 6.0 - elapsed_ms as f32 / 12000.0) * std::f32::consts::PI).sin();
        let band_intensity = (y_norm - 0.5) / 0.5 * 0.08; // Stronger near bottom
        let band_threshold = (band_intensity * 100.0 + band_wave.abs() * 4.0) as usize;
        if seed % 100 < band_threshold {
            if seed.is_multiple_of(3) { '▒' } else { '░' }
        } else {
            return Span::raw(" ");
        }
    };

    let r = (r as f32 * shimmer) as u8;
    let g = (g as f32 * shimmer) as u8;
    let b = (b as f32 * shimmer) as u8;

    Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
}
