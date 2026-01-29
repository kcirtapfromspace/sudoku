use crossterm::style::Color;

/// Color theme for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color
    pub bg: Color,
    /// Default text color
    pub fg: Color,
    /// Grid border color
    pub border: Color,
    /// Box border color (thicker 3x3 separators)
    pub box_border: Color,
    /// Given (puzzle) cell color
    pub given: Color,
    /// User-entered value color
    pub filled: Color,
    /// Candidate (pencil mark) color
    pub candidate: Color,
    /// Selected cell background
    pub selected_bg: Color,
    /// Highlighted cells (same row/col/box)
    pub highlight_bg: Color,
    /// Error/conflict color
    pub error: Color,
    /// Success/complete color
    pub success: Color,
    /// Timer/info text color
    pub info: Color,
    /// Key binding text color
    pub key: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default) - improved contrast
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb { r: 20, g: 22, b: 30 },
            fg: Color::Rgb { r: 230, g: 230, b: 240 },
            border: Color::Rgb { r: 70, g: 75, b: 90 },
            box_border: Color::Rgb { r: 130, g: 140, b: 170 },
            given: Color::Rgb { r: 255, g: 255, b: 255 },
            filled: Color::Rgb { r: 80, g: 180, b: 255 },
            candidate: Color::Rgb { r: 140, g: 150, b: 180 },
            selected_bg: Color::Rgb { r: 70, g: 90, b: 140 },
            highlight_bg: Color::Rgb { r: 35, g: 40, b: 55 },
            error: Color::Rgb { r: 255, g: 90, b: 90 },
            success: Color::Rgb { r: 90, g: 255, b: 130 },
            info: Color::Rgb { r: 160, g: 165, b: 185 },
            key: Color::Rgb { r: 255, g: 210, b: 100 },
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            bg: Color::Rgb { r: 248, g: 248, b: 252 },
            fg: Color::Rgb { r: 30, g: 30, b: 40 },
            border: Color::Rgb { r: 180, g: 180, b: 195 },
            box_border: Color::Rgb { r: 60, g: 60, b: 80 },
            given: Color::Rgb { r: 0, g: 0, b: 0 },
            filled: Color::Rgb { r: 30, g: 100, b: 200 },
            candidate: Color::Rgb { r: 130, g: 130, b: 150 },
            selected_bg: Color::Rgb { r: 180, g: 200, b: 255 },
            highlight_bg: Color::Rgb { r: 230, g: 232, b: 242 },
            error: Color::Rgb { r: 220, g: 50, b: 50 },
            success: Color::Rgb { r: 40, g: 160, b: 60 },
            info: Color::Rgb { r: 90, g: 90, b: 110 },
            key: Color::Rgb { r: 200, g: 120, b: 20 },
        }
    }

    /// High contrast theme
    pub fn high_contrast() -> Self {
        Self {
            bg: Color::Black,
            fg: Color::White,
            border: Color::Grey,
            box_border: Color::White,
            given: Color::Yellow,
            filled: Color::Cyan,
            candidate: Color::Rgb { r: 150, g: 150, b: 150 },
            selected_bg: Color::Blue,
            highlight_bg: Color::Rgb { r: 30, g: 30, b: 30 },
            error: Color::Red,
            success: Color::Green,
            info: Color::Grey,
            key: Color::Yellow,
        }
    }
}
