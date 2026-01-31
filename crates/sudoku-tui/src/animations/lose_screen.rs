use crossterm::style::Color;
use rand::prelude::SliceRandom;
use rand::Rng;

use super::particles::{random_dark_color, LoseEffectType, Particle, DEBRIS_CHARS, RAIN_CHARS};

const LOSE_MESSAGES: [&str; 12] = [
    "GAME OVER",
    "TOO MANY MISTAKES",
    "BETTER LUCK NEXT TIME",
    "PUZZLE FAILED",
    "TRY AGAIN",
    "DON'T GIVE UP!",
    "PRACTICE MAKES PERFECT",
    "KEEP TRYING",
    "ALMOST THERE",
    "LEARN FROM MISTAKES",
    "YOU'LL GET IT NEXT TIME",
    "STAY DETERMINED",
];

const ASCII_BANNERS: [&str; 3] = [
    r#"
  ▄████  ▄▄▄       ███▄ ▄███▓▓█████     ▒█████   ██▒   █▓▓█████  ██▀███
 ██▒ ▀█▒▒████▄    ▓██▒▀█▀ ██▒▓█   ▀    ▒██▒  ██▒▓██░   █▒▓█   ▀ ▓██ ▒ ██▒
▒██░▄▄▄░▒██  ▀█▄  ▓██    ▓██░▒███      ▒██░  ██▒ ▓██  █▒░▒███   ▓██ ░▄█ ▒
░▓█  ██▓░██▄▄▄▄██ ▒██    ▒██ ▒▓█  ▄    ▒██   ██░  ▒██ █░░▒▓█  ▄ ▒██▀▀█▄
░▒▓███▀▒ ▓█   ▓██▒▒██▒   ░██▒░▒████▒   ░ ████▓▒░   ▒▀█░  ░▒████▒░██▓ ▒██▒
 ░▒   ▒  ▒▒   ▓▒█░░ ▒░   ░  ░░░ ▒░ ░   ░ ▒░▒░▒░    ░ ▐░  ░░ ▒░ ░░ ▒▓ ░▒▓░
"#,
    r#"
 ██████╗  █████╗ ███╗   ███╗███████╗     ██████╗ ██╗   ██╗███████╗██████╗
██╔════╝ ██╔══██╗████╗ ████║██╔════╝    ██╔═══██╗██║   ██║██╔════╝██╔══██╗
██║  ███╗███████║██╔████╔██║█████╗      ██║   ██║██║   ██║█████╗  ██████╔╝
██║   ██║██╔══██║██║╚██╔╝██║██╔══╝      ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗    ╚██████╔╝ ╚████╔╝ ███████╗██║  ██║
 ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝     ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝
"#,
    r#"
  _____          __  __ ______    ______      ________ _____
 / ____|   /\   |  \/  |  ____|  / __ \ \    / /  ____|  __ \
| |  __   /  \  | \  / | |__    | |  | \ \  / /| |__  | |__) |
| | |_ | / /\ \ | |\/| |  __|   | |  | |\ \/ / |  __| |  _  /
| |__| |/ ____ \| |  | | |____  | |__| | \  /  | |____| | \ \
 \_____/_/    \_\_|  |_|______|  \____/   \/   |______|_|  \_\
"#,
];

/// Background pattern for lose screen (darker, sadder)
#[derive(Clone, Copy)]
pub enum LoseBgPattern {
    StaticNoise,
    DarkWaves,
    Cracks,
    Fade,
}

impl LoseBgPattern {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => LoseBgPattern::StaticNoise,
            1 => LoseBgPattern::DarkWaves,
            2 => LoseBgPattern::Cracks,
            _ => LoseBgPattern::Fade,
        }
    }
}

const DARK_BG_CHARS: &[char] = &['░', '▒', '▓', '█', '·', ':', '.'];

/// Configuration for lose screen background
#[derive(Clone)]
pub struct LoseBackground {
    pub pattern: LoseBgPattern,
    pub speed: f32,
}

impl LoseBackground {
    pub fn random_new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pattern: LoseBgPattern::random(),
            speed: rng.gen_range(0.3..0.8),
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
        let t = frame * self.speed * 0.03;
        let w = width.max(1) as f32;
        let h = height.max(1) as f32;
        let xf = x as f32;
        let yf = y as f32;

        let mut rng = rand::thread_rng();

        // Base dark color to avoid black spots
        let base_color = Color::Rgb {
            r: 15,
            g: 10,
            b: 12,
        };

        match self.pattern {
            LoseBgPattern::StaticNoise => {
                // Random static with occasional flicker
                if rng.gen_bool(0.15) {
                    let char_idx = rng.gen_range(0..DARK_BG_CHARS.len());
                    let brightness = rng.gen_range(20..60);
                    (
                        DARK_BG_CHARS[char_idx],
                        Color::Rgb {
                            r: brightness,
                            g: brightness,
                            b: brightness + 10,
                        },
                    )
                } else {
                    ('░', base_color)
                }
            }
            LoseBgPattern::DarkWaves => {
                let wave = ((xf * 0.1 + t).sin() * 0.3 + (yf * 0.15 - t * 0.5).cos() * 0.3 + 0.6)
                    .clamp(0.0, 1.0);
                let brightness = (wave * 40.0).max(10.0) as u8;
                let char_idx = ((wave * DARK_BG_CHARS.len() as f32) as usize) % DARK_BG_CHARS.len();
                (
                    DARK_BG_CHARS[char_idx],
                    Color::Rgb {
                        r: brightness,
                        g: brightness / 2,
                        b: brightness / 2,
                    },
                )
            }
            LoseBgPattern::Cracks => {
                // Diagonal crack patterns
                let diag1 = ((xf - yf + t * 10.0) % 20.0).abs() < 1.0;
                let diag2 = ((xf + yf - t * 8.0) % 25.0).abs() < 1.0;
                if diag1 || diag2 {
                    (
                        '╱',
                        Color::Rgb {
                            r: 80,
                            g: 30,
                            b: 30,
                        },
                    )
                } else if rng.gen_bool(0.02) {
                    ('·', Color::DarkGrey)
                } else {
                    ('░', base_color)
                }
            }
            LoseBgPattern::Fade => {
                // Fading from center
                let cx = w / 2.0;
                let cy = h / 2.0;
                let dist = ((xf - cx).powi(2) + (yf - cy).powi(2)).sqrt();
                let max_dist = (cx.powi(2) + cy.powi(2)).sqrt();
                let fade = (dist / max_dist).clamp(0.0, 1.0);
                let brightness = ((1.0 - fade) * 30.0 + (t * 2.0).sin() * 10.0).max(10.0) as u8;

                (
                    '░',
                    Color::Rgb {
                        r: brightness,
                        g: brightness / 3,
                        b: brightness / 2,
                    },
                )
            }
        }
    }
}

/// The animated lose screen
pub struct LoseScreen {
    particles: Vec<Particle>,
    effect_type: LoseEffectType,
    frame_count: u32,
    message_index: usize,
    banner_index: usize,
    pub background: LoseBackground,
    pub width: u16,
    pub height: u16,
}

impl LoseScreen {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            particles: Vec::new(),
            effect_type: LoseEffectType::random(),
            frame_count: 0,
            message_index: rng.gen_range(0..LOSE_MESSAGES.len()),
            banner_index: rng.gen_range(0..ASCII_BANNERS.len()),
            background: LoseBackground::random_new(),
            width: 80,
            height: 24,
        }
    }

    pub fn reset(&mut self) {
        let mut rng = rand::thread_rng();
        self.particles.clear();
        self.frame_count = 0;
        self.effect_type = LoseEffectType::random();
        self.message_index = rng.gen_range(0..LOSE_MESSAGES.len());
        self.banner_index = rng.gen_range(0..ASCII_BANNERS.len());
        self.background = LoseBackground::random_new();
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self) {
        self.frame_count += 1;

        // Switch effects periodically
        if self.frame_count.is_multiple_of(400) {
            self.effect_type = LoseEffectType::random();
            let mut rng = rand::thread_rng();
            self.message_index = rng.gen_range(0..LOSE_MESSAGES.len());
        }

        // Update particles
        self.particles.retain_mut(|p| {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.08; // Slower gravity
            p.lifetime -= 0.016;
            p.lifetime > 0.0 && p.y < self.height as f32 + 5.0
        });

        // Spawn new particles
        match self.effect_type {
            LoseEffectType::Rain => self.spawn_rain(),
            LoseEffectType::Falling => self.spawn_falling(),
            LoseEffectType::Glitch => self.spawn_glitch(),
        }
    }

    fn spawn_rain(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..2 {
            self.particles.push(Particle {
                x: rng.gen_range(0.0..self.width as f32),
                y: -1.0,
                vx: rng.gen_range(-0.1..0.1),
                vy: rng.gen_range(0.8..1.5),
                char: *RAIN_CHARS.choose(&mut rng).unwrap(),
                color: Color::Rgb {
                    r: 60,
                    g: 60,
                    b: 100,
                },
                lifetime: rng.gen_range(2.0..4.0),
            });
        }
    }

    fn spawn_falling(&mut self) {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.1) {
            self.particles.push(Particle {
                x: rng.gen_range(0.0..self.width as f32),
                y: -1.0,
                vx: rng.gen_range(-0.3..0.3),
                vy: rng.gen_range(0.2..0.5),
                char: *DEBRIS_CHARS.choose(&mut rng).unwrap(),
                color: random_dark_color(),
                lifetime: rng.gen_range(3.0..6.0),
            });
        }
    }

    fn spawn_glitch(&mut self) {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.2) {
            // Random glitch block
            let x = rng.gen_range(0.0..self.width as f32);
            let y = rng.gen_range(0.0..self.height as f32);
            self.particles.push(Particle {
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                char: '█',
                color: Color::Rgb {
                    r: rng.gen_range(80..150),
                    g: rng.gen_range(0..50),
                    b: rng.gen_range(0..50),
                },
                lifetime: rng.gen_range(0.1..0.3),
            });
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn current_message(&self) -> &str {
        LOSE_MESSAGES[self.message_index]
    }

    pub fn current_banner(&self) -> &str {
        ASCII_BANNERS[self.banner_index]
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}

impl Default for LoseScreen {
    fn default() -> Self {
        Self::new()
    }
}
