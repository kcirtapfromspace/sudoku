//! WebAssembly Sudoku game with terminal-like UI
//!
//! This crate provides a browser-based Sudoku game that looks and feels
//! like the terminal UI version.

use sudoku_core::Difficulty;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, KeyboardEvent};

mod theme;
mod render;
mod game;
mod animations;

// WASM tests require wasm-pack test to run
#[cfg(all(test, target_arch = "wasm32"))]
mod tests;

pub use theme::Theme;
pub use game::GameState;

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// The main WASM game controller
#[wasm_bindgen]
pub struct SudokuGame {
    state: GameState,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    theme: Theme,
    cell_size: f64,
    font_size: f64,
    width: u32,
    height: u32,
    dpr: f64, // Device pixel ratio for crisp rendering
}

#[wasm_bindgen]
impl SudokuGame {
    /// Create a new game attached to a canvas element
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<SudokuGame, JsValue> {
        let document = web_sys::window()
            .ok_or("No window")?
            .document()
            .ok_or("No document")?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        let ctx = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Get device pixel ratio for crisp rendering on high-DPI displays
        let dpr = web_sys::window()
            .map(|w| w.device_pixel_ratio())
            .unwrap_or(1.0);

        // Set canvas size for crisp rendering
        let width = 1000;
        let height = 700;

        // Set actual canvas resolution (scaled by dpr)
        canvas.set_width((width as f64 * dpr) as u32);
        canvas.set_height((height as f64 * dpr) as u32);

        // Set CSS display size (logical pixels)
        let html_element: &HtmlElement = canvas.as_ref();
        let style = html_element.style();
        let _ = style.set_property("width", &format!("{}px", width));
        let _ = style.set_property("height", &format!("{}px", height));

        // Scale context to account for dpr
        let _ = ctx.scale(dpr, dpr);

        let cell_size = 56.0;
        let font_size = 30.0;

        let game = SudokuGame {
            state: GameState::new(Difficulty::Medium),
            canvas,
            ctx,
            theme: Theme::dark(),
            cell_size,
            font_size,
            width,
            height,
            dpr,
        };

        game.render();
        Ok(game)
    }

    /// Handle keyboard input
    #[wasm_bindgen]
    pub fn handle_key(&mut self, event: &KeyboardEvent) -> bool {
        let key = event.key();
        let shift = event.shift_key();
        let ctrl = event.ctrl_key();

        let action = self.state.handle_key(&key, shift, ctrl);

        self.render();
        action
    }

    /// Update game state (call from requestAnimationFrame)
    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.state.tick();
        self.render();
    }

    /// Start a new game with specified difficulty
    #[wasm_bindgen]
    pub fn new_game(&mut self, difficulty: &str) {
        let diff = match difficulty {
            "beginner" => Difficulty::Beginner,
            "easy" => Difficulty::Easy,
            "medium" => Difficulty::Medium,
            "intermediate" => Difficulty::Intermediate,
            "hard" => Difficulty::Hard,
            "expert" => Difficulty::Expert,
            "master" => Difficulty::Master,
            "extreme" => Difficulty::Extreme,
            _ => Difficulty::Medium,
        };
        self.state = GameState::new(diff);
        self.render();
    }

    /// Set the color theme
    #[wasm_bindgen]
    pub fn set_theme(&mut self, theme_name: &str) {
        self.theme = match theme_name {
            "light" => Theme::light(),
            "high_contrast" => Theme::high_contrast(),
            _ => Theme::dark(),
        };
        self.render();
    }

    /// Get current game state as JSON
    #[wasm_bindgen]
    pub fn get_state_json(&self) -> String {
        serde_json::to_string(&self.state.to_serializable()).unwrap_or_default()
    }

    /// Load game state from JSON
    #[wasm_bindgen]
    pub fn load_state_json(&mut self, json: &str) -> bool {
        if let Ok(state) = serde_json::from_str(json) {
            self.state = GameState::from_serializable(state);
            self.render();
            true
        } else {
            false
        }
    }

    /// Check if game is complete
    #[wasm_bindgen]
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Check if game is over (too many mistakes)
    #[wasm_bindgen]
    pub fn is_game_over(&self) -> bool {
        self.state.is_game_over()
    }

    /// Get elapsed time in seconds
    #[wasm_bindgen]
    pub fn elapsed_secs(&self) -> u32 {
        self.state.elapsed_secs()
    }

    /// Get formatted elapsed time
    #[wasm_bindgen]
    pub fn elapsed_string(&self) -> String {
        self.state.elapsed_string()
    }

    /// Get current difficulty
    #[wasm_bindgen]
    pub fn difficulty(&self) -> String {
        format!("{}", self.state.difficulty())
    }

    /// Get number of mistakes
    #[wasm_bindgen]
    pub fn mistakes(&self) -> usize {
        self.state.mistakes()
    }

    /// Get number of hints used
    #[wasm_bindgen]
    pub fn hints_used(&self) -> usize {
        self.state.hints_used()
    }

    /// Toggle pause
    #[wasm_bindgen]
    pub fn toggle_pause(&mut self) {
        self.state.toggle_pause();
        self.render();
    }

    /// Check if paused
    #[wasm_bindgen]
    pub fn is_paused(&self) -> bool {
        self.state.is_paused()
    }

    /// Resize the game canvas
    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        // Minimum sizes
        let width = width.max(600);
        let height = height.max(500);

        self.width = width;
        self.height = height;

        // Update dpr in case it changed (e.g., moving to different monitor)
        self.dpr = web_sys::window()
            .map(|w| w.device_pixel_ratio())
            .unwrap_or(1.0);

        // Set actual canvas resolution (scaled by dpr for crisp rendering)
        self.canvas.set_width((width as f64 * self.dpr) as u32);
        self.canvas.set_height((height as f64 * self.dpr) as u32);

        // Set CSS display size (logical pixels)
        let html_element: &HtmlElement = self.canvas.as_ref();
        let style = html_element.style();
        let _ = style.set_property("width", &format!("{}px", width));
        let _ = style.set_property("height", &format!("{}px", height));

        // Reset and scale context to account for dpr
        let _ = self.ctx.reset_transform();
        let _ = self.ctx.scale(self.dpr, self.dpr);

        // Calculate cell size based on available height (grid should fit vertically)
        // Grid needs 9 cells + some padding
        let max_grid_height = (height as f64 - 80.0).max(300.0);
        let max_grid_width = (width as f64 * 0.6).max(300.0); // Leave room for info panel

        // Cell size is limited by both dimensions
        let cell_by_height = max_grid_height / 9.0;
        let cell_by_width = max_grid_width / 9.0;
        self.cell_size = cell_by_height.min(cell_by_width).min(70.0).max(35.0);

        // Font size scales with cell size
        self.font_size = (self.cell_size * 0.55).max(16.0).min(36.0);

        self.render();
    }

    /// Get current width
    #[wasm_bindgen]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Get current height
    #[wasm_bindgen]
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Render the game to canvas
    fn render(&self) {
        render::render_game(&self.ctx, &self.state, &self.theme, self.width, self.height, self.cell_size, self.font_size);
    }
}
