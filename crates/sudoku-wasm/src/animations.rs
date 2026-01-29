//! Animations for WASM Sudoku (win/lose screens with particles)

use crate::theme::Color;

/// Simple PRNG for animations
struct AnimRng {
    state: u64,
}

impl AnimRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.state >> 33) ^ self.state) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    fn gen_range_usize(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_u32() as usize % (max - min))
    }

    fn gen_bool(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}

/// A single particle
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub char: char,
    pub color: Color,
    pub lifetime: f32,
    pub size: f32,
}

impl Particle {
    pub fn is_visible(&self, width: f32, height: f32) -> bool {
        self.x >= -10.0
            && self.x < width + 10.0
            && self.y >= -10.0
            && self.y < height + 10.0
            && self.lifetime > 0.0
    }
}

/// Effect types for win screen
#[derive(Clone, Copy, PartialEq)]
pub enum EffectType {
    Confetti,
    Fireworks,
    Sparkles,
    Rainbow,
}

impl EffectType {
    fn random(rng: &mut AnimRng) -> Self {
        match rng.gen_range_usize(0, 4) {
            0 => EffectType::Confetti,
            1 => EffectType::Fireworks,
            2 => EffectType::Sparkles,
            _ => EffectType::Rainbow,
        }
    }
}

/// Background pattern types
#[derive(Clone, Copy)]
pub enum BgPattern {
    Gradient,
    Waves,
    Plasma,
    Spiral,
}

impl BgPattern {
    fn random(rng: &mut AnimRng) -> Self {
        match rng.gen_range_usize(0, 4) {
            0 => BgPattern::Gradient,
            1 => BgPattern::Waves,
            2 => BgPattern::Plasma,
            _ => BgPattern::Spiral,
        }
    }
}

/// Confetti/sparkle characters
const CONFETTI_CHARS: &[char] = &['*', '✦', '✧', '◆', '◇', '○', '●', '■', '□', '▲', '▽'];
const SPARKLE_CHARS: &[char] = &['✨', '⭐', '✦', '★', '☆', '✫', '✬'];

/// Win messages
const WIN_MESSAGES: &[&str] = &[
    "SUDOKU SOLVED!",
    "BRILLIANT!",
    "AMAZING!",
    "CHAMPION!",
    "PERFECT!",
    "EXCELLENT!",
    "CONGRATULATIONS!",
    "WELL DONE!",
    "INCREDIBLE!",
    "SUPERSTAR!",
    "LEGENDARY!",
    "FLAWLESS!",
    "MAGNIFICENT!",
];

/// ASCII art banners
pub const ASCII_BANNERS: &[&str] = &[
    r#"
██╗    ██╗██╗███╗   ██╗███╗   ██╗███████╗██████╗
██║    ██║██║████╗  ██║████╗  ██║██╔════╝██╔══██╗
██║ █╗ ██║██║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝
██║███╗██║██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗
╚███╔███╔╝██║██║ ╚████║██║ ╚████║███████╗██║  ██║
 ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝
"#,
    r#"
██╗   ██╗██╗ ██████╗████████╗ ██████╗ ██████╗ ██╗   ██╗
██║   ██║██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝
██║   ██║██║██║        ██║   ██║   ██║██████╔╝ ╚████╔╝
╚██╗ ██╔╝██║██║        ██║   ██║   ██║██╔══██╗  ╚██╔╝
 ╚████╔╝ ██║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║
  ╚═══╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝
"#,
    r#"
 ____   ___  _ __     _______ ____
/ ___| / _ \| |\ \   / / ____|  _ \
\___ \| | | | | \ \ / /|  _| | | | |
 ___) | |_| | |__\ V / | |___| |_| |
|____/ \___/|_____\_/  |_____|____/
"#,
];

/// Configuration for animated background
#[derive(Clone)]
pub struct WinBackground {
    pub pattern: BgPattern,
    pub hue_offset: f32,
    pub hue_range: f32,
    pub speed: f32,
    pub wave_freq_x: f32,
    pub wave_freq_y: f32,
    pub dim_factor: f32,
}

impl WinBackground {
    fn random_new(rng: &mut AnimRng) -> Self {
        Self {
            pattern: BgPattern::random(rng),
            hue_offset: rng.next_f32(),
            hue_range: rng.gen_range_f32(0.5, 1.0),
            speed: rng.gen_range_f32(0.5, 1.5),
            wave_freq_x: rng.gen_range_f32(0.05, 0.3),
            wave_freq_y: rng.gen_range_f32(0.05, 0.25),
            dim_factor: rng.gen_range_f32(0.3, 0.5),
        }
    }

    /// Get color at position for background effect
    pub fn color_at(&self, x: f32, y: f32, width: f32, height: f32, time: f32) -> Color {
        let t = time * self.speed * 0.05;
        let w = width.max(1.0);
        let h = height.max(1.0);

        let hue = match self.pattern {
            BgPattern::Gradient => {
                (x / w + y / h * 0.5 + t * 0.1) % 1.0
            }
            BgPattern::Waves => {
                let wave = ((x * self.wave_freq_x + t).sin() * 0.5
                    + 0.5
                    + (y * self.wave_freq_y + t * 0.7).cos() * 0.5)
                    % 1.0;
                wave
            }
            BgPattern::Plasma => {
                let cx = w / 2.0;
                let cy = h / 2.0;
                let dx = x - cx;
                let dy = y - cy;

                let v1 = (dx * 0.02 + t).sin();
                let v2 = (dy * 0.02 + t * 0.5).sin();
                let v3 = ((dx + dy) * 0.01 + t * 0.3).sin();
                let v4 = ((dx * dx + dy * dy).sqrt() * 0.01 - t).sin();

                (v1 + v2 + v3 + v4 + 4.0) / 8.0
            }
            BgPattern::Spiral => {
                let cx = w / 2.0;
                let cy = h / 2.0;
                let dx = x - cx;
                let dy = y - cy;

                let angle = dy.atan2(dx);
                let dist = (dx * dx + dy * dy).sqrt();
                let spiral = (angle + dist * 0.02 - t * 0.5) % (std::f32::consts::PI * 2.0);
                (spiral / (std::f32::consts::PI * 2.0) + 1.0) % 1.0
            }
        };

        let adjusted_hue = (self.hue_offset + hue * self.hue_range) % 1.0;
        hue_to_color(adjusted_hue, self.dim_factor)
    }
}

/// Convert hue to RGB color
fn hue_to_color(hue: f32, brightness: f32) -> Color {
    let h = hue * 6.0;
    let c = brightness;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());

    let (r, g, b) = match h as i32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::new(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

/// Random bright color
fn random_bright_color(rng: &mut AnimRng) -> Color {
    let hue = rng.next_f32();
    hue_to_color(hue, 1.0)
}

/// The animated win screen
pub struct WinScreen {
    particles: Vec<Particle>,
    effect_type: EffectType,
    frame_count: u32,
    rainbow_offset: f32,
    message_index: usize,
    banner_index: usize,
    firework_cooldown: u32,
    pub background: WinBackground,
    pub width: f32,
    pub height: f32,
    rng: AnimRng,
}

impl WinScreen {
    pub fn new(seed: u64) -> Self {
        let mut rng = AnimRng::new(seed);
        let message_index = rng.gen_range_usize(0, WIN_MESSAGES.len());
        let banner_index = rng.gen_range_usize(0, ASCII_BANNERS.len());
        let background = WinBackground::random_new(&mut rng);
        let effect_type = EffectType::random(&mut rng);

        Self {
            particles: Vec::new(),
            effect_type,
            frame_count: 0,
            rainbow_offset: 0.0,
            message_index,
            banner_index,
            firework_cooldown: 0,
            background,
            width: 800.0,
            height: 600.0,
            rng,
        }
    }

    pub fn reset(&mut self) {
        self.particles.clear();
        self.frame_count = 0;
        self.rainbow_offset = 0.0;
        self.effect_type = EffectType::random(&mut self.rng);
        self.message_index = self.rng.gen_range_usize(0, WIN_MESSAGES.len());
        self.banner_index = self.rng.gen_range_usize(0, ASCII_BANNERS.len());
        self.background = WinBackground::random_new(&mut self.rng);
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        self.rainbow_offset += 0.02;

        // Switch effects periodically
        if self.frame_count % 300 == 0 {
            self.effect_type = EffectType::random(&mut self.rng);
            self.message_index = self.rng.gen_range_usize(0, WIN_MESSAGES.len());
        }

        // Update particles
        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.1; // Gravity
            p.lifetime -= 0.016;
            p.is_visible(self.width, self.height)
        });

        // Spawn new particles based on effect type
        match self.effect_type {
            EffectType::Confetti => self.spawn_confetti(),
            EffectType::Fireworks => self.spawn_fireworks(),
            EffectType::Sparkles => self.spawn_sparkles(),
            EffectType::Rainbow => self.spawn_rainbow(),
        }
    }

    fn spawn_confetti(&mut self) {
        for _ in 0..2 {
            let char_idx = self.rng.gen_range_usize(0, CONFETTI_CHARS.len());
            self.particles.push(Particle {
                x: self.rng.gen_range_f32(0.0, self.width),
                y: -10.0,
                vx: self.rng.gen_range_f32(-1.0, 1.0),
                vy: self.rng.gen_range_f32(1.0, 3.0),
                char: CONFETTI_CHARS[char_idx],
                color: random_bright_color(&mut self.rng),
                lifetime: self.rng.gen_range_f32(4.0, 8.0),
                size: self.rng.gen_range_f32(12.0, 24.0),
            });
        }
    }

    fn spawn_fireworks(&mut self) {
        if self.firework_cooldown > 0 {
            self.firework_cooldown -= 1;
            return;
        }

        if self.rng.gen_bool(0.05) {
            let x = self.rng.gen_range_f32(100.0, self.width - 100.0);
            let y = self.rng.gen_range_f32(100.0, self.height / 2.0);
            let color = random_bright_color(&mut self.rng);

            for _ in 0..20 {
                let angle = self.rng.gen_range_f32(0.0, std::f32::consts::TAU);
                let speed = self.rng.gen_range_f32(2.0, 6.0);
                self.particles.push(Particle {
                    x,
                    y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    char: '●',
                    color,
                    lifetime: self.rng.gen_range_f32(1.0, 2.5),
                    size: self.rng.gen_range_f32(6.0, 12.0),
                });
            }
            self.firework_cooldown = 20;
        }
    }

    fn spawn_sparkles(&mut self) {
        for _ in 0..3 {
            let char_idx = self.rng.gen_range_usize(0, SPARKLE_CHARS.len());
            self.particles.push(Particle {
                x: self.rng.gen_range_f32(0.0, self.width),
                y: self.rng.gen_range_f32(0.0, self.height),
                vx: self.rng.gen_range_f32(-0.5, 0.5),
                vy: self.rng.gen_range_f32(-0.5, 0.5),
                char: SPARKLE_CHARS[char_idx],
                color: Color::new(255, 255, self.rng.gen_range_usize(200, 256) as u8),
                lifetime: self.rng.gen_range_f32(0.5, 1.5),
                size: self.rng.gen_range_f32(16.0, 28.0),
            });
        }
    }

    fn spawn_rainbow(&mut self) {
        for _ in 0..2 {
            let hue = (self.rainbow_offset + self.rng.next_f32()) % 1.0;
            self.particles.push(Particle {
                x: self.rng.gen_range_f32(0.0, self.width),
                y: -5.0,
                vx: self.rng.gen_range_f32(-0.5, 0.5),
                vy: self.rng.gen_range_f32(2.0, 4.0),
                char: '█',
                color: hue_to_color(hue, 1.0),
                lifetime: self.rng.gen_range_f32(5.0, 8.0),
                size: self.rng.gen_range_f32(10.0, 20.0),
            });
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn current_message(&self) -> &'static str {
        WIN_MESSAGES[self.message_index]
    }

    pub fn current_banner(&self) -> &'static str {
        ASCII_BANNERS[self.banner_index]
    }

    pub fn rainbow_offset(&self) -> f32 {
        self.rainbow_offset
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn effect_type(&self) -> EffectType {
        self.effect_type
    }
}

/// Lose screen with sad effects
pub struct LoseScreen {
    particles: Vec<Particle>,
    frame_count: u32,
    pub width: f32,
    pub height: f32,
    rng: AnimRng,
}

const RAIN_CHARS: &[char] = &['│', '╎', '┊', '┆', '|', '.'];
const DEBRIS_CHARS: &[char] = &['×', '✕', '✖', '▼', '▾', '◾'];

impl LoseScreen {
    pub fn new(seed: u64) -> Self {
        Self {
            particles: Vec::new(),
            frame_count: 0,
            width: 800.0,
            height: 600.0,
            rng: AnimRng::new(seed),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self) {
        self.frame_count += 1;

        // Update particles
        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;
            p.lifetime -= 0.016;
            p.is_visible(self.width, self.height)
        });

        // Spawn rain/debris
        self.spawn_rain();
    }

    fn spawn_rain(&mut self) {
        for _ in 0..3 {
            let use_rain = self.rng.gen_bool(0.7);
            let chars = if use_rain { RAIN_CHARS } else { DEBRIS_CHARS };
            let char_idx = self.rng.gen_range_usize(0, chars.len());

            let gray = self.rng.gen_range_usize(60, 120) as u8;
            let red_tint = self.rng.gen_range_usize(0, 40) as u8;

            self.particles.push(Particle {
                x: self.rng.gen_range_f32(0.0, self.width),
                y: -5.0,
                vx: self.rng.gen_range_f32(-0.3, 0.3),
                vy: self.rng.gen_range_f32(3.0, 6.0),
                char: chars[char_idx],
                color: Color::new(gray + red_tint, gray, gray),
                lifetime: self.rng.gen_range_f32(3.0, 5.0),
                size: self.rng.gen_range_f32(10.0, 16.0),
            });
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}
