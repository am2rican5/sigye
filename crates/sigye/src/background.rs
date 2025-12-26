//! Background animation rendering for the sigye clock.

use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use sigye_core::{AnimationSpeed, BackgroundStyle};

/// Characters used for starfield background.
const STAR_CHARS: &[char] = &['.', '*', '+', '·', '✦', '✧'];

/// Characters used for matrix rain.
const MATRIX_CHARS: &[char] = &[
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ', 'タ',
    'チ', 'ツ', 'テ', 'ト', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// State for a single matrix rain column.
#[derive(Debug, Clone)]
struct MatrixColumn {
    /// Current y position of the raindrop head.
    y: f32,
    /// Speed multiplier for this column.
    speed: f32,
    /// Length of the trail.
    trail_length: usize,
    /// Seed for character generation.
    char_seed: usize,
}

/// Background animation state.
#[derive(Debug)]
pub struct BackgroundState {
    /// Matrix rain column states.
    matrix_columns: Vec<MatrixColumn>,
    /// Last known terminal width.
    last_width: u16,
    /// Last known terminal height.
    last_height: u16,
    /// Last update time in milliseconds.
    last_update_ms: u64,
}

impl Default for BackgroundState {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundState {
    /// Create a new background state.
    pub fn new() -> Self {
        Self {
            matrix_columns: Vec::new(),
            last_width: 0,
            last_height: 0,
            last_update_ms: 0,
        }
    }

    /// Initialize or reinitialize matrix columns for the given dimensions.
    fn init_matrix_columns(&mut self, width: u16, height: u16) {
        self.matrix_columns = (0..width)
            .map(|x| {
                let x = x as usize;
                let stagger = ((x * 7 + 3) % (height as usize * 2)) as f32;
                MatrixColumn {
                    // Stagger start positions so columns don't all start at top
                    y: -stagger,
                    // Vary speeds between columns
                    speed: 0.3 + ((x * 13) % 10) as f32 / 15.0,
                    // Vary trail lengths
                    trail_length: 4 + (x * 11) % 8,
                    // Seed for character selection
                    char_seed: x * 17,
                }
            })
            .collect();
        self.last_width = width;
        self.last_height = height;
    }

    /// Update matrix column positions.
    fn update_matrix(&mut self, elapsed_ms: u64, height: u16, speed: AnimationSpeed) {
        let delta_ms = elapsed_ms.saturating_sub(self.last_update_ms);
        self.last_update_ms = elapsed_ms;

        let fall_speed = speed.matrix_fall_speed();
        let delta_y = (delta_ms as f32 / 50.0) * fall_speed;

        for col in &mut self.matrix_columns {
            col.y += delta_y * col.speed;
            // Reset column when it goes off screen
            if col.y > (height as f32 + col.trail_length as f32) {
                col.y = -(col.trail_length as f32);
                col.char_seed = col.char_seed.wrapping_add(1);
            }
        }
    }

    /// Render the background to the frame.
    pub fn render(
        &mut self,
        frame: &mut Frame,
        style: BackgroundStyle,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) {
        if style == BackgroundStyle::None {
            return;
        }

        let area = frame.area();
        let width = area.width;
        let height = area.height;

        // Reinitialize if dimensions changed
        if style == BackgroundStyle::MatrixRain
            && (width != self.last_width || height != self.last_height)
        {
            self.init_matrix_columns(width, height);
        }

        // Update matrix state
        if style == BackgroundStyle::MatrixRain {
            self.update_matrix(elapsed_ms, height, speed);
        }

        let lines: Vec<Line> = (0..height)
            .map(|y| {
                let spans: Vec<Span> = (0..width)
                    .map(|x| self.render_char(x, y, width, height, style, elapsed_ms, speed))
                    .collect();
                Line::from(spans)
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), area);
    }

    /// Render a single background character at the given position.
    fn render_char(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        style: BackgroundStyle,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) -> Span<'static> {
        match style {
            BackgroundStyle::None => Span::raw(" "),
            BackgroundStyle::Starfield => self.render_starfield_char(x, y, elapsed_ms, speed),
            BackgroundStyle::MatrixRain => self.render_matrix_char(x, y, height),
            BackgroundStyle::GradientWave => {
                self.render_gradient_char(x, y, width, height, elapsed_ms, speed)
            }
        }
    }

    /// Render a starfield character using pseudo-random twinkling.
    fn render_starfield_char(
        &self,
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

    /// Render a matrix rain character.
    fn render_matrix_char(&self, x: u16, y: u16, _height: u16) -> Span<'static> {
        let x = x as usize;
        let y = y as f32;

        if x >= self.matrix_columns.len() {
            return Span::raw(" ");
        }

        let col = &self.matrix_columns[x];
        let head_y = col.y;
        let tail_y = head_y - col.trail_length as f32;

        // Check if this position is within the rain trail
        if y >= tail_y && y <= head_y {
            let distance_from_head = head_y - y;
            let intensity = 1.0 - (distance_from_head / col.trail_length as f32);

            // Select character based on position and seed
            let char_idx = (col.char_seed.wrapping_add(y as usize)) % MATRIX_CHARS.len();
            let ch = MATRIX_CHARS[char_idx];

            // Head is bright white-green, trail fades to dark green
            let color = if distance_from_head < 1.0 {
                Color::Rgb(200, 255, 200) // Bright head
            } else {
                let g = (80.0 + 120.0 * intensity) as u8;
                Color::Rgb(0, g, 0)
            };

            Span::styled(ch.to_string(), Style::new().fg(color))
        } else {
            Span::raw(" ")
        }
    }

    /// Render a gradient wave character.
    fn render_gradient_char(
        &self,
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
}

/// Convert HSL to RGB color.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return Color::Rgb(v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let h = h / 360.0;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    Color::Rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
