//! Color themes for the WASM Sudoku UI

use serde::{Deserialize, Serialize};

/// RGB color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn as_css(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    pub fn as_css_alpha(&self, alpha: f64) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, alpha)
    }
}

/// Color theme for the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Background color
    pub background: Color,
    /// Grid lines color
    pub grid_lines: Color,
    /// Box border color (thicker lines)
    pub box_border: Color,
    /// Cell background
    pub cell_bg: Color,
    /// Highlighted cell background
    pub highlight_bg: Color,
    /// Cursor cell background
    pub cursor_bg: Color,
    /// Same value highlight
    pub same_value_bg: Color,
    /// Given number color
    pub given_text: Color,
    /// Player-entered number color
    pub player_text: Color,
    /// Candidate (note) color
    pub candidate_text: Color,
    /// Error/conflict color
    pub error_text: Color,
    /// Completed number indicator
    pub completed_bg: Color,
    /// Info panel text
    pub info_text: Color,
    /// Message text
    pub message_text: Color,
    /// Win screen color
    pub win_color: Color,
    /// Lose screen color
    pub lose_color: Color,
}

impl Theme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            background: Color::new(24, 24, 32),
            grid_lines: Color::new(60, 60, 80),
            box_border: Color::new(100, 100, 140),
            cell_bg: Color::new(32, 32, 44),
            highlight_bg: Color::new(48, 48, 64),
            cursor_bg: Color::new(70, 100, 150),
            same_value_bg: Color::new(60, 80, 100),
            given_text: Color::new(200, 200, 220),
            player_text: Color::new(100, 180, 255),
            candidate_text: Color::new(120, 120, 140),
            error_text: Color::new(255, 100, 100),
            completed_bg: Color::new(40, 80, 40),
            info_text: Color::new(160, 160, 180),
            message_text: Color::new(255, 220, 100),
            win_color: Color::new(100, 255, 150),
            lose_color: Color::new(255, 100, 100),
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            background: Color::new(245, 245, 250),
            grid_lines: Color::new(180, 180, 200),
            box_border: Color::new(80, 80, 100),
            cell_bg: Color::new(255, 255, 255),
            highlight_bg: Color::new(230, 240, 255),
            cursor_bg: Color::new(180, 210, 255),
            same_value_bg: Color::new(200, 220, 255),
            given_text: Color::new(20, 20, 40),
            player_text: Color::new(30, 100, 200),
            candidate_text: Color::new(140, 140, 160),
            error_text: Color::new(220, 50, 50),
            completed_bg: Color::new(200, 240, 200),
            info_text: Color::new(60, 60, 80),
            message_text: Color::new(180, 120, 0),
            win_color: Color::new(50, 180, 80),
            lose_color: Color::new(220, 50, 50),
        }
    }

    /// High contrast theme
    pub fn high_contrast() -> Self {
        Self {
            background: Color::new(0, 0, 0),
            grid_lines: Color::new(100, 100, 100),
            box_border: Color::new(255, 255, 255),
            cell_bg: Color::new(0, 0, 0),
            highlight_bg: Color::new(40, 40, 60),
            cursor_bg: Color::new(0, 80, 160),
            same_value_bg: Color::new(60, 60, 0),
            given_text: Color::new(255, 255, 255),
            player_text: Color::new(0, 255, 255),
            candidate_text: Color::new(150, 150, 150),
            error_text: Color::new(255, 0, 0),
            completed_bg: Color::new(0, 100, 0),
            info_text: Color::new(200, 200, 200),
            message_text: Color::new(255, 255, 0),
            win_color: Color::new(0, 255, 0),
            lose_color: Color::new(255, 0, 0),
        }
    }
}
