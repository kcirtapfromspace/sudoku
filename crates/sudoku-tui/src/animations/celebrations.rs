use crossterm::style::Color;

/// Type of completion celebration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CelebrationType {
    Row(usize),    // Row index 0-8
    Column(usize), // Column index 0-8
    Box(usize),    // Box index 0-8
}

/// A single celebration event with animation state
#[derive(Debug, Clone)]
pub struct Celebration {
    pub celebration_type: CelebrationType,
    pub start_frame: u32,
    pub duration_frames: u32,
}

impl Celebration {
    pub fn new(celebration_type: CelebrationType, current_frame: u32) -> Self {
        Self {
            celebration_type,
            start_frame: current_frame,
            duration_frames: 90, // ~3 seconds at 30 FPS
        }
    }

    pub fn is_active(&self, current_frame: u32) -> bool {
        current_frame < self.start_frame + self.duration_frames
    }

    /// Get the current intensity (0.0 to 1.0) based on animation phase
    pub fn intensity(&self, current_frame: u32) -> f32 {
        let elapsed = current_frame.saturating_sub(self.start_frame);
        let progress = elapsed as f32 / self.duration_frames as f32;

        // Initial bright flash for the first ~10 frames
        if elapsed < 10 {
            return 1.0;
        }

        // Pulse frequency: fast at start, then slows down
        let pulse_freq = if progress < 0.3 {
            6.0 // Fast pulsing
        } else if progress < 0.6 {
            4.0 // Medium pulsing
        } else {
            2.0 // Slow fade
        };

        // Amplitude fades out towards the end
        let amplitude = if progress > 0.6 {
            1.0 - (progress - 0.6) / 0.4
        } else {
            1.0
        };

        // Base intensity stays high during the celebration
        let base = 0.4;
        let pulse = (elapsed as f32 * 0.4 * pulse_freq).sin() * 0.5 + 0.5;
        (base + pulse * 0.6) * amplitude
    }
}

/// Manages all active celebrations
#[derive(Default)]
pub struct CelebrationManager {
    celebrations: Vec<Celebration>,
    frame_count: u32,
    // Track what was previously completed to detect new completions
    prev_completed_rows: [bool; 9],
    prev_completed_cols: [bool; 9],
    prev_completed_boxes: [bool; 9],
}

impl CelebrationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update celebrations based on current game state
    pub fn update(
        &mut self,
        completed_rows: [bool; 9],
        completed_cols: [bool; 9],
        completed_boxes: [bool; 9],
    ) {
        self.frame_count += 1;

        // Check for newly completed rows
        for i in 0..9 {
            if completed_rows[i] && !self.prev_completed_rows[i] {
                self.celebrations
                    .push(Celebration::new(CelebrationType::Row(i), self.frame_count));
            }
        }

        // Check for newly completed columns
        for i in 0..9 {
            if completed_cols[i] && !self.prev_completed_cols[i] {
                self.celebrations
                    .push(Celebration::new(CelebrationType::Column(i), self.frame_count));
            }
        }

        // Check for newly completed boxes
        for i in 0..9 {
            if completed_boxes[i] && !self.prev_completed_boxes[i] {
                self.celebrations
                    .push(Celebration::new(CelebrationType::Box(i), self.frame_count));
            }
        }

        // Update previous state
        self.prev_completed_rows = completed_rows;
        self.prev_completed_cols = completed_cols;
        self.prev_completed_boxes = completed_boxes;

        // Remove expired celebrations
        self.celebrations
            .retain(|c| c.is_active(self.frame_count));
    }

    /// Reset all celebrations (for new game)
    pub fn reset(&mut self) {
        self.celebrations.clear();
        self.frame_count = 0;
        self.prev_completed_rows = [false; 9];
        self.prev_completed_cols = [false; 9];
        self.prev_completed_boxes = [false; 9];
    }

    /// Get the throbbing intensity for a specific row (0.0 if not active)
    pub fn row_intensity(&self, row: usize) -> f32 {
        for c in &self.celebrations {
            if let CelebrationType::Row(r) = c.celebration_type {
                if r == row {
                    return c.intensity(self.frame_count);
                }
            }
        }
        0.0
    }

    /// Get the throbbing intensity for a specific column (0.0 if not active)
    pub fn column_intensity(&self, col: usize) -> f32 {
        for c in &self.celebrations {
            if let CelebrationType::Column(c_idx) = c.celebration_type {
                if c_idx == col {
                    return c.intensity(self.frame_count);
                }
            }
        }
        0.0
    }

    /// Get the throbbing intensity for a specific box (0.0 if not active)
    pub fn box_intensity(&self, box_idx: usize) -> f32 {
        for c in &self.celebrations {
            if let CelebrationType::Box(b) = c.celebration_type {
                if b == box_idx {
                    return c.intensity(self.frame_count);
                }
            }
        }
        0.0
    }

    /// Check if there are any active celebrations
    pub fn has_active_celebrations(&self) -> bool {
        !self.celebrations.is_empty()
    }

    /// Generate a throbbing color based on intensity
    pub fn throb_color(base_color: Color, intensity: f32) -> Color {
        if intensity <= 0.0 {
            return base_color;
        }

        // Cycle through celebration colors: gold -> orange -> yellow -> white
        let cycle = (intensity * 3.0) % 1.0;
        let (target_r, target_g, target_b) = if cycle < 0.25 {
            // Gold
            (255.0, 215.0, 0.0)
        } else if cycle < 0.5 {
            // Orange
            (255.0, 165.0, 0.0)
        } else if cycle < 0.75 {
            // Bright yellow
            (255.0, 255.0, 100.0)
        } else {
            // White flash
            (255.0, 255.0, 220.0)
        };

        match base_color {
            Color::Rgb { r, g, b } => {
                // Strong blend for high visibility
                let blend = (intensity * 0.85).min(0.95);
                Color::Rgb {
                    r: (r as f32 + (target_r - r as f32) * blend) as u8,
                    g: (g as f32 + (target_g - g as f32) * blend) as u8,
                    b: (b as f32 + (target_b - b as f32) * blend) as u8,
                }
            }
            Color::White | Color::Grey | Color::DarkGrey => {
                Color::Rgb {
                    r: (target_r * intensity) as u8,
                    g: (target_g * intensity) as u8,
                    b: (target_b * intensity) as u8,
                }
            }
            _ => {
                // For other colors, return celebration color if active
                if intensity > 0.3 {
                    Color::Rgb {
                        r: target_r as u8,
                        g: target_g as u8,
                        b: target_b as u8,
                    }
                } else {
                    base_color
                }
            }
        }
    }

    /// Get a celebration message for recent completions
    pub fn get_celebration_message(&self) -> Option<&'static str> {
        if self.celebrations.is_empty() {
            return None;
        }

        // Get the most recent celebration
        let recent = self.celebrations.last()?;
        if self.frame_count - recent.start_frame > 60 {
            return None; // Only show message for ~2 seconds
        }

        let messages = match recent.celebration_type {
            CelebrationType::Row(_) => &["Row Complete!", "Nice Row!", "Row Cleared!"][..],
            CelebrationType::Column(_) => &["Column Complete!", "Nice Column!", "Column Cleared!"][..],
            CelebrationType::Box(_) => &["Box Complete!", "Nice Box!", "Section Cleared!"][..],
        };

        let idx = (self.frame_count / 20) as usize % messages.len();
        Some(messages[idx])
    }
}
