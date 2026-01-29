use crossterm::style::Color;
use rand::prelude::SliceRandom;
use rand::Rng;

use super::particles::{
    hue_to_rgb, random_bright_color, EffectType, Particle, CONFETTI_CHARS, SPARKLE_CHARS,
};

/// Background pattern types
#[derive(Clone, Copy)]
pub enum BgPattern {
    Gradient,
    Waves,
    Checkerboard,
    Plasma,
    Spiral,
}

impl BgPattern {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..5) {
            0 => BgPattern::Gradient,
            1 => BgPattern::Waves,
            2 => BgPattern::Checkerboard,
            3 => BgPattern::Plasma,
            _ => BgPattern::Spiral,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            BgPattern::Gradient => "Gradient",
            BgPattern::Waves => "Waves",
            BgPattern::Checkerboard => "Checker",
            BgPattern::Plasma => "Plasma",
            BgPattern::Spiral => "Spiral",
        }
    }
}

const BG_CHARS: &[char] = &['█', '▓', '▒', '░', '◆', '◇', '○', '●'];

/// Configuration for a unique win background
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
    pub fn random_new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pattern: BgPattern::random(),
            hue_offset: rng.gen_range(0.0..1.0),
            hue_range: rng.gen_range(0.5..1.0),
            speed: rng.gen_range(0.5..1.5),
            wave_freq_x: rng.gen_range(0.05..0.3),
            wave_freq_y: rng.gen_range(0.05..0.25),
            dim_factor: rng.gen_range(0.5..0.8),
        }
    }

    pub fn render_at(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        frame: f32,
    ) -> (char, Color) {
        let t = frame * self.speed * 0.05;
        let w = width.max(1) as f32;
        let h = height.max(1) as f32;
        let xf = x as f32;
        let yf = y as f32;

        let (char_val, hue) = match self.pattern {
            BgPattern::Gradient => {
                let hue = (xf / w + yf / h * 0.5 + t * 0.1) % 1.0;
                let char_idx = ((xf + yf) as usize) % BG_CHARS.len();
                (BG_CHARS[char_idx], hue)
            }
            BgPattern::Waves => {
                let wave = ((xf * self.wave_freq_x + t).sin() * 0.5
                    + 0.5
                    + (yf * self.wave_freq_y + t * 0.7).cos() * 0.5)
                    % 1.0;
                let char_idx = ((wave * BG_CHARS.len() as f32) as usize) % BG_CHARS.len();
                (BG_CHARS[char_idx], wave)
            }
            BgPattern::Checkerboard => {
                let check = ((x + y + (t * 5.0) as usize) % 2) == 0;
                let hue = if check { 0.0 } else { 0.5 };
                let char_idx = if check { 0 } else { BG_CHARS.len() / 2 };
                (BG_CHARS[char_idx % BG_CHARS.len()], (hue + t * 0.1) % 1.0)
            }
            BgPattern::Plasma => {
                let cx = w / 2.0;
                let cy = h / 2.0;
                let dx = xf - cx;
                let dy = yf - cy;

                let v1 = (dx * 0.1 + t).sin();
                let v2 = (dy * 0.1 + t * 0.5).sin();
                let v3 = ((dx + dy) * 0.05 + t * 0.3).sin();
                let v4 = ((dx * dx + dy * dy).sqrt() * 0.05 - t).sin();

                let value = (v1 + v2 + v3 + v4 + 4.0) / 8.0;
                let char_idx = ((value * BG_CHARS.len() as f32) as usize) % BG_CHARS.len();
                (BG_CHARS[char_idx], value)
            }
            BgPattern::Spiral => {
                let cx = w / 2.0;
                let cy = h / 2.0;
                let dx = xf - cx;
                let dy = (yf - cy) * 2.0;

                let angle = dy.atan2(dx);
                let dist = (dx * dx + dy * dy).sqrt();
                let spiral = (angle + dist * 0.1 - t * 0.5) % (std::f32::consts::PI * 2.0);
                let normalized = (spiral / (std::f32::consts::PI * 2.0) + 1.0) % 1.0;

                let char_idx = ((normalized * BG_CHARS.len() as f32) as usize) % BG_CHARS.len();
                (BG_CHARS[char_idx], normalized)
            }
        };

        let adjusted_hue = (self.hue_offset + hue * self.hue_range) % 1.0;
        let color = self.hue_to_rgb_dimmed(adjusted_hue);

        (char_val, color)
    }

    fn hue_to_rgb_dimmed(&self, hue: f32) -> Color {
        let h = hue * 6.0;
        let c = self.dim_factor;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());

        let (r, g, b) = match h as i32 % 6 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::Rgb {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }
}

const WIN_MESSAGES: [&str; 15] = [
    "SUDOKU SOLVED!",
    "BRILLIANT!",
    "AMAZING!",
    "CHAMPION!",
    "PERFECT!",
    "EXCELLENT!",
    "CONGRATULATIONS!",
    "WELL DONE!",
    "ON FIRE!",
    "INCREDIBLE!",
    "SUPERSTAR!",
    "BULLSEYE!",
    "LEGENDARY!",
    "FLAWLESS!",
    "MAGNIFICENT!",
];

const ASCII_BANNERS: [&str; 3] = [
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
  ____  _   _  ____   ___  _  ___   _
 / ___|| | | ||  _ \ / _ \| |/ / | | |
 \___ \| | | || | | | | | | ' /| | | |
  ___) | |_| || |_| | |_| | . \| |_| |
 |____/ \___/ |____/ \___/|_|\_\\___/
"#,
];

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
    pub width: u16,
    pub height: u16,
}

impl WinScreen {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            particles: Vec::new(),
            effect_type: EffectType::random(),
            frame_count: 0,
            rainbow_offset: 0.0,
            message_index: rng.gen_range(0..WIN_MESSAGES.len()),
            banner_index: rng.gen_range(0..ASCII_BANNERS.len()),
            firework_cooldown: 0,
            background: WinBackground::random_new(),
            width: 80,
            height: 24,
        }
    }

    pub fn reset(&mut self) {
        let mut rng = rand::thread_rng();
        self.particles.clear();
        self.frame_count = 0;
        self.rainbow_offset = 0.0;
        self.effect_type = EffectType::random();
        self.message_index = rng.gen_range(0..WIN_MESSAGES.len());
        self.banner_index = rng.gen_range(0..ASCII_BANNERS.len());
        self.background = WinBackground::random_new();
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        self.rainbow_offset += 0.05;

        // Switch effects periodically
        if self.frame_count % 300 == 0 {
            self.effect_type = EffectType::random();
            let mut rng = rand::thread_rng();
            self.message_index = rng.gen_range(0..WIN_MESSAGES.len());
        }

        // Update particles
        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.15; // Gravity
            p.lifetime -= 0.016;
            p.lifetime > 0.0 && p.y < self.height as f32 + 5.0
        });

        // Spawn new particles
        match self.effect_type {
            EffectType::Confetti => self.spawn_confetti(),
            EffectType::Fireworks => self.spawn_fireworks(),
            EffectType::Sparkles => self.spawn_sparkles(),
            EffectType::Rainbow => self.spawn_rainbow(),
        }
    }

    fn spawn_confetti(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..3 {
            self.particles.push(Particle {
                x: rng.gen_range(0.0..self.width as f32),
                y: -2.0,
                vx: rng.gen_range(-0.5..0.5),
                vy: rng.gen_range(0.3..1.0),
                char: *CONFETTI_CHARS.choose(&mut rng).unwrap(),
                color: random_bright_color(),
                lifetime: rng.gen_range(3.0..6.0),
            });
        }
    }

    fn spawn_fireworks(&mut self) {
        if self.firework_cooldown > 0 {
            self.firework_cooldown -= 1;
            return;
        }

        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.08) {
            let x = rng.gen_range(10.0..(self.width as f32 - 10.0));
            let y = rng.gen_range(5.0..(self.height as f32 / 2.0));
            let color = random_bright_color();

            for _ in 0..25 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = rng.gen_range(0.5..2.0);
                self.particles.push(Particle {
                    x,
                    y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    char: '●',
                    color,
                    lifetime: rng.gen_range(1.0..2.5),
                });
            }
            self.firework_cooldown = 15;
        }
    }

    fn spawn_sparkles(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..4 {
            self.particles.push(Particle {
                x: rng.gen_range(0.0..self.width as f32),
                y: rng.gen_range(0.0..self.height as f32),
                vx: rng.gen_range(-0.2..0.2),
                vy: rng.gen_range(-0.2..0.2),
                char: *SPARKLE_CHARS.choose(&mut rng).unwrap(),
                color: Color::Rgb {
                    r: 255,
                    g: 255,
                    b: rng.gen_range(150..255),
                },
                lifetime: rng.gen_range(0.5..1.5),
            });
        }
    }

    fn spawn_rainbow(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..2 {
            let hue = (self.rainbow_offset + rng.gen_range(0.0..1.0)) % 1.0;
            self.particles.push(Particle {
                x: rng.gen_range(0.0..self.width as f32),
                y: -1.0,
                vx: rng.gen_range(-0.3..0.3),
                vy: rng.gen_range(0.5..1.5),
                char: '█',
                color: hue_to_rgb(hue),
                lifetime: rng.gen_range(4.0..7.0),
            });
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn current_message(&self) -> &str {
        WIN_MESSAGES[self.message_index]
    }

    pub fn current_banner(&self) -> &str {
        ASCII_BANNERS[self.banner_index]
    }

    pub fn rainbow_offset(&self) -> f32 {
        self.rainbow_offset
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}

impl Default for WinScreen {
    fn default() -> Self {
        Self::new()
    }
}
