//! Weather animation effects (stateful and stateless).

use ratatui::{
    style::{Color, Style},
    text::Span,
};
use sigye_core::AnimationSpeed;

use crate::chars::{
    CLOUD_CHARS, FOG_CHARS, RAIN_CHARS, SNOW_CHARS, STORM_RAIN_CHARS, SUN_CHARS, WIND_CHARS,
};

// ========== RAIN STATE (Stateful) ==========

/// State for a single rain column.
#[derive(Debug, Clone)]
pub struct RainColumn {
    /// Current y position of the raindrop.
    pub y: f32,
    /// Speed multiplier for this column.
    pub speed: f32,
    /// Character seed for variety.
    pub char_seed: usize,
    /// Intensity (0=light, 1=medium, 2=heavy).
    pub intensity: u8,
}

/// Initialize rain columns.
pub fn init_rain_columns(width: u16, height: u16, init_seed: u64) -> Vec<RainColumn> {
    (0..width)
        .map(|x| {
            let x = x as usize;
            let mixed = x.wrapping_mul(29).wrapping_add(init_seed as usize);
            let stagger = ((mixed.wrapping_mul(13)) % (height as usize * 2)) as f32;
            RainColumn {
                y: -stagger,
                speed: 0.8 + ((mixed.wrapping_mul(17)) % 10) as f32 / 25.0,
                char_seed: mixed.wrapping_mul(23),
                intensity: ((mixed.wrapping_mul(7)) % 3) as u8,
            }
        })
        .collect()
}

/// Update rain column positions.
pub fn update_rain(columns: &mut [RainColumn], delta_ms: u64, height: u16, speed: AnimationSpeed) {
    let fall_speed = speed.rain_fall_speed();
    let delta_y = (delta_ms as f32 / 40.0) * fall_speed;

    for col in columns {
        col.y += delta_y * col.speed;
        if col.y > height as f32 + 1.0 {
            col.y = -1.0;
            col.char_seed = col.char_seed.wrapping_add(1);
        }
    }
}

/// Render a rain character.
pub fn render_rain_char(columns: &[RainColumn], x: u16, y: u16) -> Span<'static> {
    let x_idx = x as usize;
    let y_f = y as f32;

    if x_idx >= columns.len() {
        return Span::raw(" ");
    }

    let col = &columns[x_idx];
    let distance = (y_f - col.y).abs();

    if distance < 0.6 {
        let char_idx = col.char_seed % RAIN_CHARS.len();
        let ch = RAIN_CHARS[char_idx];

        // Blue-gray rain colors
        let color = match col.intensity {
            0 => Color::Rgb(100, 120, 150), // Light rain
            1 => Color::Rgb(80, 100, 140),  // Medium rain
            _ => Color::Rgb(60, 80, 120),   // Heavy rain
        };

        Span::styled(ch.to_string(), Style::new().fg(color))
    } else {
        Span::raw(" ")
    }
}

// ========== SNOW STATE (Stateful) ==========

/// State for a single snowfall column.
#[derive(Debug, Clone)]
pub struct SnowColumn {
    /// Current y position of the snowflake.
    pub y: f32,
    /// Speed multiplier for this column.
    pub speed: f32,
    /// Horizontal drift phase offset.
    pub drift_phase: f32,
    /// Size category (0=small, 1=medium, 2=large).
    pub size: u8,
    /// Seed for character generation.
    pub char_seed: usize,
}

/// Initialize snowfall columns for the given dimensions.
pub fn init_snow_columns(width: u16, height: u16, init_seed: u64) -> Vec<SnowColumn> {
    (0..width)
        .map(|x| {
            let x = x as usize;
            // Mix column index with time-based seed for better randomness
            let mixed = x.wrapping_mul(31).wrapping_add(init_seed as usize);
            let stagger = ((mixed.wrapping_mul(11).wrapping_add(7)) % (height as usize * 3)) as f32;
            SnowColumn {
                y: -stagger,
                speed: 0.2 + ((mixed.wrapping_mul(17)) % 10) as f32 / 20.0,
                drift_phase: ((mixed.wrapping_mul(23)) % 100) as f32 / 100.0,
                size: ((mixed.wrapping_mul(13)) % 3) as u8,
                char_seed: mixed.wrapping_mul(19),
            }
        })
        .collect()
}

/// Update snowfall column positions.
pub fn update_snow(columns: &mut [SnowColumn], delta_ms: u64, height: u16, speed: AnimationSpeed) {
    let fall_speed = speed.snow_fall_speed();
    let delta_y = (delta_ms as f32 / 80.0) * fall_speed;

    for col in columns {
        col.y += delta_y * col.speed;
        if col.y > height as f32 + 2.0 {
            col.y = -2.0;
            col.char_seed = col.char_seed.wrapping_add(1);
        }
    }
}

/// Render a snowfall character.
pub fn render_snow_char(columns: &[SnowColumn], x: u16, y: u16, elapsed_ms: u64) -> Span<'static> {
    let x_idx = x as usize;
    let y_f = y as f32;

    if x_idx >= columns.len() {
        return Span::raw(" ");
    }

    let col = &columns[x_idx];

    // Calculate horizontal drift for visual effect
    let drift_period = 3000.0;
    let drift = ((elapsed_ms as f32 / drift_period + col.drift_phase) * 2.0 * std::f32::consts::PI)
        .sin()
        * 1.5;

    // Check if snowflake is at this position (applying drift effect)
    let flake_y = col.y + drift * 0.1;
    let distance = (y_f - flake_y).abs();

    if distance < 0.8 {
        // Select character based on size
        let char_idx = match col.size {
            0 => col.char_seed % 3,
            1 => 3 + col.char_seed % 3,
            _ => 6 + col.char_seed % 3,
        };
        let ch = SNOW_CHARS[char_idx % SNOW_CHARS.len()];

        // Color based on size - using deeper blues visible on both light and dark themes
        let color = match col.size {
            0 => Color::Rgb(70, 100, 160), // Small - dark steel blue
            1 => Color::Rgb(65, 105, 225), // Medium - royal blue
            _ => Color::Rgb(30, 144, 255), // Large - dodger blue
        };

        Span::styled(ch.to_string(), Style::new().fg(color))
    } else {
        Span::raw(" ")
    }
}

// ========== STORM STATE (Stateful - extends Rain) ==========

/// State for storm lightning.
#[derive(Debug, Clone)]
pub struct StormState {
    /// Rain columns (reuses rain logic).
    pub rain_columns: Vec<RainColumn>,
    /// Time of last lightning flash.
    pub last_lightning_ms: u64,
    /// Duration of current lightning flash (0 = no flash).
    pub lightning_duration_ms: u64,
    /// Next lightning interval.
    pub next_lightning_interval: u64,
    /// Lightning flash intensity (0.0-1.0).
    pub flash_intensity: f32,
    /// Seed for lightning randomness.
    pub lightning_seed: u64,
}

/// Initialize storm state.
pub fn init_storm(width: u16, height: u16, init_seed: u64) -> StormState {
    StormState {
        rain_columns: init_rain_columns(width, height, init_seed),
        last_lightning_ms: 0,
        lightning_duration_ms: 0,
        next_lightning_interval: 2000 + (init_seed % 3000),
        flash_intensity: 0.0,
        lightning_seed: init_seed,
    }
}

/// Update storm state.
pub fn update_storm(
    state: &mut StormState,
    elapsed_ms: u64,
    delta_ms: u64,
    height: u16,
    speed: AnimationSpeed,
) {
    // Update rain
    update_rain(&mut state.rain_columns, delta_ms, height, speed);

    // Handle lightning
    let time_since_flash = elapsed_ms.saturating_sub(state.last_lightning_ms);

    if state.lightning_duration_ms > 0 {
        // Flash in progress
        if time_since_flash > state.lightning_duration_ms {
            state.lightning_duration_ms = 0;
            state.flash_intensity = 0.0;
            // Schedule next lightning
            let (min_interval, max_interval) = speed.lightning_interval_ms();
            state.next_lightning_interval = min_interval
                + (state.lightning_seed.wrapping_mul(17) % (max_interval - min_interval));
            state.lightning_seed = state.lightning_seed.wrapping_add(elapsed_ms);
        } else {
            // Decay flash intensity
            let progress = time_since_flash as f32 / state.lightning_duration_ms as f32;
            state.flash_intensity = (1.0 - progress).max(0.0);
        }
    } else if time_since_flash > state.next_lightning_interval {
        // Trigger new lightning
        state.last_lightning_ms = elapsed_ms;
        state.lightning_duration_ms = 100 + (state.lightning_seed % 151); // 100-250ms
        state.flash_intensity = 1.0;
    }
}

/// Render a storm character.
pub fn render_storm_char(state: &StormState, x: u16, y: u16, _elapsed_ms: u64) -> Span<'static> {
    let x_idx = x as usize;
    let y_f = y as f32;

    if x_idx >= state.rain_columns.len() {
        return Span::raw(" ");
    }

    let col = &state.rain_columns[x_idx];
    let distance = (y_f - col.y).abs();

    // Rain with trail effect (2-3 char vertical streak)
    let trail_length = 2.5;
    let in_trail = distance < trail_length && y_f >= col.y;

    if in_trail {
        // Use storm rain characters
        let char_idx = col.char_seed % STORM_RAIN_CHARS.len();
        let ch = STORM_RAIN_CHARS[char_idx];

        // Calculate trail fade (brighter at head)
        let trail_pos = distance / trail_length;
        let trail_fade = 1.0 - trail_pos * 0.6;

        // Purple-gray storm colors (more dramatic)
        let base_r = 70u8;
        let base_g = 65u8;
        let base_b = 100u8;

        // Lightning illuminates the rain
        let color = if state.flash_intensity > 0.0 {
            // Dramatic white-blue flash on rain
            let boost = (state.flash_intensity * 150.0 * trail_fade) as u8;
            Color::Rgb(
                base_r.saturating_add(boost),
                base_g.saturating_add(boost + 10),
                base_b.saturating_add(boost + 20),
            )
        } else {
            // Normal storm rain - purple-gray tones
            let fade = trail_fade * 0.7 + 0.3;
            Color::Rgb(
                (base_r as f32 * fade) as u8,
                (base_g as f32 * fade) as u8,
                (base_b as f32 * fade) as u8,
            )
        };

        Span::styled(ch.to_string(), Style::new().fg(color))
    } else if state.flash_intensity > 0.3 {
        // Lightning ambient glow - sparse flicker effect
        let seed = (x as usize).wrapping_mul(17).wrapping_add(y as usize * 31);
        if seed % 8 < 3 {
            let brightness = (state.flash_intensity * 80.0) as u8;
            Span::styled(
                "·".to_string(),
                Style::new().fg(Color::Rgb(
                    brightness + 40,
                    brightness + 50,
                    brightness + 80,
                )),
            )
        } else {
            Span::raw(" ")
        }
    } else {
        Span::raw(" ")
    }
}

// ========== WIND STATE (Stateful) ==========

/// State for a single wind streak.
#[derive(Debug, Clone)]
pub struct WindStreak {
    /// Current x position.
    pub x: f32,
    /// Y position (row).
    pub y: u16,
    /// Speed multiplier.
    pub speed: f32,
    /// Length of streak.
    pub length: u8,
    /// Character seed.
    pub char_seed: usize,
}

/// Initialize wind streaks.
pub fn init_wind_streaks(width: u16, height: u16, init_seed: u64) -> Vec<WindStreak> {
    let num_streaks = ((width as usize * height as usize) / 40).clamp(10, 200);
    (0..num_streaks)
        .map(|i| {
            let mixed = i.wrapping_mul(37).wrapping_add(init_seed as usize);
            let start_offset = ((mixed.wrapping_mul(19)) % (width as usize * 2)) as f32;
            WindStreak {
                x: -start_offset,
                y: ((mixed.wrapping_mul(23)) % height as usize) as u16,
                speed: 0.5 + ((mixed.wrapping_mul(13)) % 10) as f32 / 10.0,
                length: 3 + ((mixed.wrapping_mul(7)) % 6) as u8,
                char_seed: mixed.wrapping_mul(31),
            }
        })
        .collect()
}

/// Update wind streak positions.
pub fn update_wind(
    streaks: &mut [WindStreak],
    delta_ms: u64,
    width: u16,
    height: u16,
    speed: AnimationSpeed,
) {
    let wind_speed = speed.wind_streak_speed();
    let delta_x = (delta_ms as f32 / 30.0) * wind_speed;

    for streak in streaks {
        streak.x += delta_x * streak.speed;
        if streak.x > width as f32 + streak.length as f32 {
            streak.x = -(streak.length as f32);
            // Move to new random row
            streak.y = ((streak.char_seed.wrapping_mul(17)) % height as usize) as u16;
            streak.char_seed = streak.char_seed.wrapping_add(1);
        }
    }
}

/// Render wind at position.
pub fn render_wind_char(streaks: &[WindStreak], x: u16, y: u16, elapsed_ms: u64) -> Span<'static> {
    let x_f = x as f32;

    for streak in streaks {
        if streak.y != y {
            continue;
        }

        let head_x = streak.x;
        let tail_x = head_x - streak.length as f32;

        if x_f >= tail_x && x_f <= head_x {
            let distance_from_head = head_x - x_f;
            let intensity = 1.0 - (distance_from_head / streak.length as f32);

            let char_idx = (streak.char_seed.wrapping_add(x as usize)) % WIND_CHARS.len();
            let ch = WIND_CHARS[char_idx];

            // Grayish wind colors
            let base = 80 + (intensity * 60.0) as u8;

            // Add shimmer
            let shimmer = ((elapsed_ms as f32 / 200.0 + x_f * 0.5).sin() * 20.0) as i16;
            let r = (base as i16 + shimmer).clamp(40, 180) as u8;

            return Span::styled(
                ch.to_string(),
                Style::new().fg(Color::Rgb(r, base + 10, base + 20)),
            );
        }
    }

    Span::raw(" ")
}

// ========== SUNNY (Stateless) ==========

/// Render a sunny background character.
pub fn render_sunny_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let x_f = x as f32;
    let y_f = y as f32;
    let w_f = width.max(1) as f32;
    let h_f = height.max(1) as f32;

    // Sun position in top-right corner
    let sun_x = w_f * 0.85;
    let sun_y = h_f * 0.15;

    // Distance from sun center
    let dx = x_f - sun_x;
    let dy = (y_f - sun_y) * 2.0; // Aspect ratio correction
    let distance = (dx * dx + dy * dy).sqrt();

    // Sun core (small bright center)
    let sun_radius = w_f.min(h_f) * 0.08;
    if distance < sun_radius {
        let core_intensity = 1.0 - (distance / sun_radius);
        let brightness = (200.0 + core_intensity * 55.0) as u8;
        return Span::styled(
            "●".to_string(),
            Style::new().fg(Color::Rgb(255, brightness, 100)),
        );
    }

    // Rays emanating from sun
    let angle = dy.atan2(dx);
    let shimmer_period = speed.sun_shimmer_period_ms();
    let shimmer_phase = (elapsed_ms % shimmer_period) as f32 / shimmer_period as f32;

    // Create ray pattern
    let num_rays = 12.0;
    let ray_angle = ((angle + shimmer_phase * 2.0 * std::f32::consts::PI) * num_rays).sin();
    let ray_intensity = ray_angle.abs();

    // Rays fade with distance
    let max_ray_distance = w_f.max(h_f) * 0.6;
    let distance_factor = 1.0 - (distance / max_ray_distance).min(1.0);

    if distance_factor > 0.05 && ray_intensity > 0.6 {
        let combined_intensity = ray_intensity * distance_factor;

        if combined_intensity > 0.3 {
            // Character selection based on intensity
            let char_idx = ((combined_intensity * 5.0) as usize).min(SUN_CHARS.len() - 1);
            let ch = SUN_CHARS[char_idx];

            // Warm yellow/orange gradient
            let r = 255;
            let g = (180.0 + combined_intensity * 50.0) as u8;
            let b = (50.0 + combined_intensity * 30.0) as u8;

            return Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)));
        }
    }

    // Background warmth - subtle warm tint
    let seed = (x as usize).wrapping_mul(31).wrapping_add(y as usize * 17);
    if seed % 150 < 2 {
        let ch = SUN_CHARS[seed % 3]; // Small sparkle
        return Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(200, 180, 80)));
    }

    Span::raw(" ")
}

// ========== CLOUDY (Stateless) ==========

/// Helper function for cloud density calculation.
fn cloud_density(x: f32, y: f32, frequency: f32, phase: f32) -> f32 {
    let wave1 = ((x * frequency + phase) * 2.0 * std::f32::consts::PI).sin();
    let wave2 = ((y * frequency * 0.5 + phase + 1.0) * 2.0 * std::f32::consts::PI).cos();
    let wave3 = (((x + y) * frequency * 0.3 + phase) * 2.0 * std::f32::consts::PI).sin();

    ((wave1 + wave2 + wave3) / 3.0 + 1.0) / 2.0
}

/// Render a cloudy background character.
pub fn render_cloudy_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let x_norm = x as f32 / width.max(1) as f32;
    let y_norm = y as f32 / height.max(1) as f32;

    let drift_period = speed.cloud_drift_period_ms();
    let drift_phase = (elapsed_ms % drift_period) as f32 / drift_period as f32;

    // Multiple cloud layers with different speeds
    let cloud1 = cloud_density(x_norm + drift_phase * 0.5, y_norm, 3.0, 0.0);
    let cloud2 = cloud_density(x_norm - drift_phase * 0.3, y_norm, 5.0, 0.3);
    let cloud3 = cloud_density(x_norm + drift_phase * 0.2, y_norm, 2.0, 0.7);

    // Combine layers
    let density = (cloud1 * 0.5 + cloud2 * 0.3 + cloud3 * 0.2).min(1.0);

    // Clouds are denser at top
    let vertical_factor = 1.0 - y_norm.powf(0.7);
    let final_density = density * vertical_factor;

    if final_density < 0.2 {
        return Span::raw(" ");
    }

    // Character based on density
    let ch = if final_density > 0.7 {
        CLOUD_CHARS[2] // '▓'
    } else if final_density > 0.5 {
        CLOUD_CHARS[1] // '▒'
    } else if final_density > 0.3 {
        CLOUD_CHARS[0] // '░'
    } else {
        CLOUD_CHARS[4] // '•'
    };

    // Gray cloud colors
    let gray = (120.0 + final_density * 60.0) as u8;
    let color = Color::Rgb(gray, gray + 5, gray + 10);

    Span::styled(ch.to_string(), Style::new().fg(color))
}

// ========== FOGGY (Stateless) ==========

/// Helper for organic fog noise using layered sine waves.
fn fog_noise(x: f32, y: f32, time: f32) -> f32 {
    // Multiple overlapping waves at different frequencies for organic feel
    let wave1 = ((x * 0.8 + time * 0.3) * std::f32::consts::PI).sin();
    let wave2 = ((y * 0.6 - time * 0.2) * std::f32::consts::PI).cos();
    let wave3 = (((x + y) * 0.4 + time * 0.15) * std::f32::consts::PI).sin();
    let wave4 = ((x * 1.2 - y * 0.5 + time * 0.25) * std::f32::consts::PI).cos();

    // Combine waves with different weights
    (wave1 * 0.35 + wave2 * 0.25 + wave3 * 0.25 + wave4 * 0.15 + 1.0) / 2.0
}

/// Render a foggy background character.
pub fn render_foggy_char(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    elapsed_ms: u64,
    speed: AnimationSpeed,
) -> Span<'static> {
    let w_f = width.max(1) as f32;
    let h_f = height.max(1) as f32;

    // Normalize coordinates
    let x_norm = x as f32 / w_f;
    let y_norm = y as f32 / h_f;

    // Time-based drift for movement
    let drift_period = speed.fog_pulse_period_ms();
    let time = (elapsed_ms % (drift_period * 4)) as f32 / drift_period as f32;

    // Generate organic fog density using layered noise
    let noise = fog_noise(x_norm * 4.0, y_norm * 3.0, time);

    // Ground-hugging effect - denser at bottom
    let vertical_factor = y_norm.powf(0.4) * 0.6 + 0.4;

    // Create patchy fog with threshold
    let base_density = noise * vertical_factor;

    // Add secondary drift layer for patches
    let patch_noise = fog_noise(x_norm * 2.0 + 1.5, y_norm * 2.0, time * 0.7);
    let patch_factor = if patch_noise > 0.55 { 1.2 } else { 0.7 };

    let final_density = (base_density * patch_factor).min(1.0);

    // Sparse fog - only show when density is high enough
    if final_density < 0.35 {
        return Span::raw(" ");
    }

    // Further sparsity based on position hash for natural gaps
    let seed = (x as usize)
        .wrapping_mul(31)
        .wrapping_add((y as usize).wrapping_mul(17));
    let threshold = 0.35 + ((seed % 30) as f32 / 100.0);
    if final_density < threshold {
        return Span::raw(" ");
    }

    // Character selection - use softer chars for lighter fog
    let char_idx = if final_density > 0.7 {
        seed % 2 // '·' or '.'
    } else if final_density > 0.5 {
        2 + (seed % 2) // '\'' or ':'
    } else {
        4 + (seed % 3) // '°', '∙', or ','
    };
    let ch = FOG_CHARS[char_idx % FOG_CHARS.len()];

    // Soft gray-blue fog colors
    let intensity = ((final_density - 0.35) / 0.65).min(1.0);
    let gray = (100.0 + intensity * 55.0) as u8;
    let color = Color::Rgb(gray, gray + 8, gray + 20);

    Span::styled(ch.to_string(), Style::new().fg(color))
}
