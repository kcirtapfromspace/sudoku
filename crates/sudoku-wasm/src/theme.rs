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
    // Hint visualization colors
    /// Generic involved cell background
    pub hint_involved_bg: Color,
    /// Target cell (placement/elimination) background
    pub hint_target_bg: Color,
    /// AIC ON-polarity cell
    pub hint_chain_on: Color,
    /// AIC OFF-polarity cell
    pub hint_chain_off: Color,
    /// Fish base sector cell
    pub hint_fish_base: Color,
    /// Fish cover sector cell
    pub hint_fish_cover: Color,
    /// Fish fin cells
    pub hint_fish_fin: Color,
    /// UR floor (bivalue cells)
    pub hint_ur_floor: Color,
    /// UR roof (extra candidate cells)
    pub hint_ur_roof: Color,
    /// ALS group cell
    pub hint_als_group: Color,
    /// Hint panel background
    pub hint_panel_bg: Color,
    /// Technique name text color
    pub hint_technique_text: Color,
    /// Explanation body text color
    pub hint_explain_text: Color,
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
            hint_involved_bg: Color::new(60, 70, 50),
            hint_target_bg: Color::new(100, 60, 30),
            hint_chain_on: Color::new(50, 120, 80),
            hint_chain_off: Color::new(130, 50, 50),
            hint_fish_base: Color::new(40, 80, 120),
            hint_fish_cover: Color::new(120, 80, 40),
            hint_fish_fin: Color::new(120, 100, 40),
            hint_ur_floor: Color::new(50, 80, 120),
            hint_ur_roof: Color::new(120, 50, 90),
            hint_als_group: Color::new(80, 60, 120),
            hint_panel_bg: Color::new(20, 30, 20),
            hint_technique_text: Color::new(100, 220, 140),
            hint_explain_text: Color::new(200, 200, 200),
        }
    }

    /// Light theme — warm paper palette matching the ukodus.now site
    pub fn light() -> Self {
        Self {
            background: Color::new(246, 241, 231), // --paper #f6f1e7
            grid_lines: Color::new(190, 182, 168),
            box_border: Color::new(100, 95, 82),
            cell_bg: Color::new(250, 247, 240),
            highlight_bg: Color::new(237, 228, 210),
            cursor_bg: Color::new(200, 185, 155),
            same_value_bg: Color::new(220, 210, 188),
            given_text: Color::new(20, 20, 20),    // --ink
            player_text: Color::new(10, 132, 255), // --accent2
            candidate_text: Color::new(150, 144, 130),
            error_text: Color::new(255, 59, 48), // --accent
            completed_bg: Color::new(210, 228, 200),
            info_text: Color::new(99, 99, 93), // --muted
            message_text: Color::new(180, 120, 0),
            win_color: Color::new(50, 180, 80),
            lose_color: Color::new(255, 59, 48),
            hint_involved_bg: Color::new(220, 230, 200),
            hint_target_bg: Color::new(255, 220, 180),
            hint_chain_on: Color::new(200, 240, 210),
            hint_chain_off: Color::new(255, 210, 210),
            hint_fish_base: Color::new(200, 220, 245),
            hint_fish_cover: Color::new(255, 230, 200),
            hint_fish_fin: Color::new(255, 240, 200),
            hint_ur_floor: Color::new(200, 220, 250),
            hint_ur_roof: Color::new(245, 210, 230),
            hint_als_group: Color::new(225, 215, 245),
            hint_panel_bg: Color::new(240, 235, 225),
            hint_technique_text: Color::new(20, 120, 60),
            hint_explain_text: Color::new(40, 40, 40),
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
            hint_involved_bg: Color::new(60, 80, 40),
            hint_target_bg: Color::new(140, 80, 0),
            hint_chain_on: Color::new(0, 160, 80),
            hint_chain_off: Color::new(180, 0, 0),
            hint_fish_base: Color::new(0, 80, 200),
            hint_fish_cover: Color::new(200, 120, 0),
            hint_fish_fin: Color::new(200, 160, 0),
            hint_ur_floor: Color::new(0, 100, 200),
            hint_ur_roof: Color::new(200, 0, 120),
            hint_als_group: Color::new(120, 60, 200),
            hint_panel_bg: Color::new(0, 20, 0),
            hint_technique_text: Color::new(0, 255, 100),
            hint_explain_text: Color::new(255, 255, 255),
        }
    }
}
