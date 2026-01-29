use crossterm::style::Color;
use rand::Rng;

/// A single particle in the celebration
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub char: char,
    pub color: Color,
    pub lifetime: f32,
}

impl Particle {
    pub fn is_visible(&self, width: u16, height: u16) -> bool {
        self.x >= 0.0
            && self.x < width as f32
            && self.y >= 0.0
            && self.y < height as f32
            && self.lifetime > 0.0
    }
}

/// Effect types for win screen
#[derive(Clone, Copy)]
pub enum EffectType {
    Confetti,
    Fireworks,
    Sparkles,
    Rainbow,
}

impl EffectType {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => EffectType::Confetti,
            1 => EffectType::Fireworks,
            2 => EffectType::Sparkles,
            _ => EffectType::Rainbow,
        }
    }
}

/// Effect types for lose screen
#[derive(Clone, Copy)]
pub enum LoseEffectType {
    Rain,
    Falling,
    Glitch,
}

impl LoseEffectType {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => LoseEffectType::Rain,
            1 => LoseEffectType::Falling,
            _ => LoseEffectType::Glitch,
        }
    }
}

/// Generate a random bright color
pub fn random_bright_color() -> Color {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..7) {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Yellow,
        3 => Color::Blue,
        4 => Color::Magenta,
        5 => Color::Cyan,
        _ => Color::White,
    }
}

/// Generate a random dark/sad color for lose screen
pub fn random_dark_color() -> Color {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..4) {
        0 => Color::DarkRed,
        1 => Color::DarkGrey,
        2 => Color::Rgb { r: 100, g: 50, b: 50 },
        _ => Color::Rgb { r: 80, g: 80, b: 100 },
    }
}

/// Convert hue (0.0-1.0) to RGB color
pub fn hue_to_rgb(hue: f32) -> Color {
    let h = hue * 6.0;
    let x = (1.0 - (h % 2.0 - 1.0).abs()) * 255.0;

    let (r, g, b) = match h as i32 % 6 {
        0 => (255, x as u8, 0),
        1 => (x as u8, 255, 0),
        2 => (0, 255, x as u8),
        3 => (0, x as u8, 255),
        4 => (x as u8, 0, 255),
        _ => (255, 0, x as u8),
    };

    Color::Rgb { r, g, b }
}

/// Confetti characters
pub const CONFETTI_CHARS: &[char] = &['*', '✦', '✧', '◆', '◇', '○', '●', '■', '□', '▲', '▽'];

/// Sparkle characters
pub const SPARKLE_CHARS: &[char] = &['✨', '⭐', '✦', '★', '☆', '✫', '✬'];

/// Rain characters (for lose screen)
pub const RAIN_CHARS: &[char] = &['│', '╎', '┊', '┆', '|', '.'];

/// Falling debris characters
pub const DEBRIS_CHARS: &[char] = &['×', '✕', '✖', '▼', '▾', '◾'];
