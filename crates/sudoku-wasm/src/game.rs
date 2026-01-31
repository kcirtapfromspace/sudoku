//! Game state management for WASM Sudoku

use crate::animations::{LoseScreen, WinScreen};
use serde::{Deserialize, Serialize};
use sudoku_core::{BitSet, Difficulty, Generator, Grid, Hint, HintType, Position, Solver};

/// Maximum mistakes before game over
pub const MAX_MISTAKES: usize = 3;

/// Input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Normal,
    Candidate,
}

/// Screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenState {
    Playing,
    Paused,
    Win,
    Lose,
    Menu,
}

/// Serializable game state for save/load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableState {
    pub puzzle: String,
    pub current: String,
    pub solution: String,
    pub difficulty: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub mode: InputMode,
    pub screen: ScreenState,
    pub elapsed_secs: u32,
    pub mistakes: usize,
    pub hints_used: usize,
    pub message: Option<String>,
}

/// The game state
pub struct GameState {
    /// Current grid (player's progress)
    grid: Grid,
    /// Original puzzle (for reference)
    puzzle: Grid,
    /// Solution
    solution: Grid,
    /// Difficulty level
    difficulty: Difficulty,
    /// Cursor position
    cursor: Position,
    /// Input mode
    mode: InputMode,
    /// Screen state
    screen: ScreenState,
    /// Start timestamp (ms since epoch)
    start_time: f64,
    /// Elapsed time when paused
    paused_elapsed: f64,
    /// Number of mistakes
    mistakes: usize,
    /// Number of hints used
    hints_used: usize,
    /// Current message to display
    message: Option<String>,
    /// Message timer (ticks remaining)
    message_timer: u32,
    /// Current hint
    current_hint: Option<Hint>,
    /// Undo stack
    undo_stack: Vec<(Position, Option<u8>)>,
    /// Redo stack
    redo_stack: Vec<(Position, Option<u8>)>,
    /// Animation frame counter
    frame: u32,
    /// Win screen animation
    win_screen: Option<WinScreen>,
    /// Lose screen animation
    lose_screen: Option<LoseScreen>,
    /// Show ghost hints (valid candidates as faded numbers)
    show_ghost_hints: bool,
    /// Show valid cells (highlight cells with only one valid number)
    show_valid_cells: bool,
}

impl GameState {
    /// Create a new game
    pub fn new(difficulty: Difficulty) -> Self {
        let mut generator = Generator::new();
        let puzzle = generator.generate(difficulty);
        let mut grid = puzzle.deep_clone();
        grid.clear_all_candidates();

        let solver = Solver::new();
        let solution = solver.solve(&puzzle).expect("Puzzle should be solvable");

        Self {
            grid,
            puzzle,
            solution,
            difficulty,
            cursor: Position::new(4, 4),
            mode: InputMode::Normal,
            screen: ScreenState::Playing,
            start_time: Self::now(),
            paused_elapsed: 0.0,
            mistakes: 0,
            hints_used: 0,
            message: None,
            message_timer: 0,
            current_hint: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame: 0,
            win_screen: None,
            lose_screen: None,
            show_ghost_hints: false,
            show_valid_cells: false,
        }
    }

    /// Get current timestamp in milliseconds
    fn now() -> f64 {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> u32 {
        if self.screen == ScreenState::Paused
            || self.screen == ScreenState::Win
            || self.screen == ScreenState::Lose
        {
            (self.paused_elapsed / 1000.0) as u32
        } else {
            let elapsed = Self::now() - self.start_time + self.paused_elapsed;
            (elapsed / 1000.0) as u32
        }
    }

    /// Get formatted elapsed time
    pub fn elapsed_string(&self) -> String {
        let secs = self.elapsed_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Update game state (called each frame)
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);

        // Update message timer
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message = None;
            }
        }

        // Check win/lose conditions
        if self.screen == ScreenState::Playing {
            if self.is_complete() {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Win;
                // Create win screen animation
                let seed = (Self::now() * 1000.0) as u64;
                self.win_screen = Some(WinScreen::new(seed));
            } else if self.mistakes >= MAX_MISTAKES {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Lose;
                // Create lose screen animation
                let seed = (Self::now() * 1000.0) as u64;
                self.lose_screen = Some(LoseScreen::new(seed));
            }
        }

        // Update animation screens
        if let Some(ref mut win_screen) = self.win_screen {
            win_screen.update();
        }
        if let Some(ref mut lose_screen) = self.lose_screen {
            lose_screen.update();
        }
    }

    /// Handle keyboard input, returns true if game should continue
    pub fn handle_key(&mut self, key: &str, shift: bool, ctrl: bool) -> bool {
        // Clear hint on any key
        self.current_hint = None;

        match self.screen {
            ScreenState::Win | ScreenState::Lose => self.handle_endgame_key(key),
            ScreenState::Paused => self.handle_paused_key(key),
            ScreenState::Menu => self.handle_menu_key(key),
            ScreenState::Playing => self.handle_playing_key(key, shift, ctrl),
        }
    }

    fn handle_endgame_key(&mut self, key: &str) -> bool {
        match key {
            "q" | "Escape" => return false,
            "n" | "Enter" | " " => {
                *self = GameState::new(self.difficulty);
            }
            "1" => *self = GameState::new(Difficulty::Beginner),
            "2" => *self = GameState::new(Difficulty::Easy),
            "3" => *self = GameState::new(Difficulty::Medium),
            "4" => *self = GameState::new(Difficulty::Intermediate),
            "5" => *self = GameState::new(Difficulty::Hard),
            "6" => *self = GameState::new(Difficulty::Expert),
            _ => {}
        }
        true
    }

    fn handle_paused_key(&mut self, key: &str) -> bool {
        match key {
            "q" | "Escape" => return false,
            "p" | " " | "Enter" => {
                self.screen = ScreenState::Playing;
                self.start_time = Self::now();
            }
            _ => {}
        }
        true
    }

    fn handle_menu_key(&mut self, key: &str) -> bool {
        match key {
            "Escape" => self.screen = ScreenState::Playing,
            "1" => *self = GameState::new(Difficulty::Beginner),
            "2" => *self = GameState::new(Difficulty::Easy),
            "3" => *self = GameState::new(Difficulty::Medium),
            "4" => *self = GameState::new(Difficulty::Intermediate),
            "5" => *self = GameState::new(Difficulty::Hard),
            "6" => *self = GameState::new(Difficulty::Expert),
            _ => {}
        }
        true
    }

    fn handle_playing_key(&mut self, key: &str, shift: bool, ctrl: bool) -> bool {
        match key {
            // Quit
            "q" if !shift && !ctrl => return false,

            // Navigation
            "ArrowUp" | "k" => self.move_cursor(-1, 0),
            "ArrowDown" | "j" => self.move_cursor(1, 0),
            "ArrowLeft" | "h" => self.move_cursor(0, -1),
            "ArrowRight" | "l" => self.move_cursor(0, 1),

            // Box navigation
            "w" => self.jump_box(-1, 0),
            "s" if !shift => self.jump_box(1, 0),
            "a" => self.jump_box(0, -1),
            "d" => self.jump_box(0, 1),

            // Number input
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let value = key.parse::<u8>().unwrap();
                if shift || self.mode == InputMode::Candidate {
                    self.toggle_candidate(value);
                } else {
                    self.set_value(value);
                }
            }

            // Clear cell
            "0" | "Delete" | "Backspace" => {
                if self.mode == InputMode::Candidate {
                    self.clear_candidates();
                } else {
                    self.clear_cell();
                }
            }

            // Clear notes (x = current cell, X = all cells)
            "x" if !shift => self.clear_candidates(),
            "X" | "x" if shift => self.clear_all_candidates(),

            // Fill candidates (f = current cell, F = all cells)
            "f" if !shift => self.fill_candidates(),
            "F" | "f" if shift => self.fill_all_candidates(),

            // Mode toggle
            "c" => {
                self.mode = match self.mode {
                    InputMode::Normal => InputMode::Candidate,
                    InputMode::Candidate => InputMode::Normal,
                };
                let mode_name = match self.mode {
                    InputMode::Normal => "Normal",
                    InputMode::Candidate => "Candidate",
                };
                self.show_message(&format!("{} mode", mode_name));
            }

            // Undo/Redo
            "u" => {
                if self.undo() {
                    self.show_message("Undo");
                }
            }
            "r" if ctrl => {
                if self.redo() {
                    self.show_message("Redo");
                }
            }

            // Hint
            "?" => {
                if let Some(hint) = self.get_hint() {
                    self.current_hint = Some(hint);
                    self.hints_used += 1;
                } else {
                    self.show_message("No hint available");
                }
            }

            // Apply hint
            "!" => {
                if let Some(pos) = self.apply_hint() {
                    self.cursor = pos;
                    self.show_message("Hint applied");
                }
            }

            // New game
            "n" => self.screen = ScreenState::Menu,

            // Pause
            "p" => {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Paused;
            }

            // Ghost hints toggle
            "g" => {
                self.show_ghost_hints = !self.show_ghost_hints;
                let status = if self.show_ghost_hints { "ON" } else { "OFF" };
                self.show_message(&format!("Ghost hints: {}", status));
            }

            // Valid cells toggle
            "v" => {
                self.show_valid_cells = !self.show_valid_cells;
                let status = if self.show_valid_cells { "ON" } else { "OFF" };
                self.show_message(&format!("Valid cells: {}", status));
            }

            _ => {}
        }
        true
    }

    fn move_cursor(&mut self, row_delta: i32, col_delta: i32) {
        let new_row = (self.cursor.row as i32 + row_delta).clamp(0, 8) as usize;
        let new_col = (self.cursor.col as i32 + col_delta).clamp(0, 8) as usize;
        self.cursor = Position::new(new_row, new_col);
    }

    fn jump_box(&mut self, row_delta: i32, col_delta: i32) {
        let box_row = (self.cursor.row / 3) as i32;
        let box_col = (self.cursor.col / 3) as i32;

        let new_box_row = (box_row + row_delta).clamp(0, 2) as usize;
        let new_box_col = (box_col + col_delta).clamp(0, 2) as usize;

        self.cursor = Position::new(new_box_row * 3 + 1, new_box_col * 3 + 1);
    }

    fn set_value(&mut self, value: u8) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() {
            return;
        }

        // Check if correct
        let is_correct = self.solution.get(self.cursor) == Some(value);
        if !is_correct {
            self.mistakes += 1;
            let remaining = MAX_MISTAKES.saturating_sub(self.mistakes);
            if remaining > 0 {
                self.show_message(&format!(
                    "Incorrect! {} {} left",
                    remaining,
                    if remaining == 1 { "chance" } else { "chances" }
                ));
            }
        }

        // Save for undo
        let old_value = self.grid.get(self.cursor);
        self.undo_stack.push((self.cursor, old_value));
        self.redo_stack.clear();

        // Set the value
        self.grid.set_cell_unchecked(self.cursor, Some(value));
        self.grid.recalculate_candidates();
    }

    fn clear_cell(&mut self) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.value().is_none() {
            return;
        }

        let old_value = self.grid.get(self.cursor);
        self.undo_stack.push((self.cursor, old_value));
        self.redo_stack.clear();

        self.grid.set_cell_unchecked(self.cursor, None);
        self.grid.recalculate_candidates();
    }

    fn toggle_candidate(&mut self, value: u8) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.is_filled() {
            return;
        }
        self.grid.cell_mut(self.cursor).toggle_candidate(value);
    }

    fn clear_candidates(&mut self) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.is_filled() {
            return;
        }
        self.grid
            .cell_mut(self.cursor)
            .set_candidates(BitSet::empty());
        self.show_message("Cleared notes");
    }

    fn fill_candidates(&mut self) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.is_filled() {
            return;
        }
        let valid = self.grid.get_candidates(self.cursor);
        self.grid.cell_mut(self.cursor).set_candidates(valid);
        self.show_message("Filled valid notes");
    }

    fn fill_all_candidates(&mut self) {
        self.grid.recalculate_candidates();
        self.show_message("Filled all notes");
    }

    fn clear_all_candidates(&mut self) {
        self.grid.clear_all_candidates();
        self.show_message("Cleared all notes");
    }

    fn undo(&mut self) -> bool {
        if let Some((pos, old_value)) = self.undo_stack.pop() {
            let current_value = self.grid.get(pos);
            self.redo_stack.push((pos, current_value));
            self.grid.set_cell_unchecked(pos, old_value);
            self.grid.recalculate_candidates();
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        if let Some((pos, value)) = self.redo_stack.pop() {
            let current_value = self.grid.get(pos);
            self.undo_stack.push((pos, current_value));
            self.grid.set_cell_unchecked(pos, value);
            self.grid.recalculate_candidates();
            true
        } else {
            false
        }
    }

    fn get_hint(&self) -> Option<Hint> {
        let solver = Solver::new();
        solver.get_hint(&self.grid)
    }

    fn apply_hint(&mut self) -> Option<Position> {
        let hint = self.get_hint()?;
        self.hints_used += 1;

        match hint.hint_type {
            HintType::SetValue { pos, value } => {
                self.cursor = pos;
                self.set_value(value);
                Some(pos)
            }
            HintType::EliminateCandidates { pos, values } => {
                self.cursor = pos;
                for value in values {
                    if self.grid.cell(pos).has_candidate(value) {
                        self.grid.cell_mut(pos).remove_candidate(value);
                    }
                }
                Some(pos)
            }
        }
    }

    fn show_message(&mut self, msg: &str) {
        self.message = Some(msg.to_string());
        self.message_timer = 90; // ~3 seconds at 30fps
    }

    // Getters
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn puzzle(&self) -> &Grid {
        &self.puzzle
    }
    pub fn solution(&self) -> &Grid {
        &self.solution
    }
    pub fn cursor(&self) -> Position {
        self.cursor
    }
    pub fn mode(&self) -> InputMode {
        self.mode
    }
    pub fn screen(&self) -> ScreenState {
        self.screen
    }
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }
    pub fn mistakes(&self) -> usize {
        self.mistakes
    }
    pub fn hints_used(&self) -> usize {
        self.hints_used
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    pub fn current_hint(&self) -> Option<&Hint> {
        self.current_hint.as_ref()
    }
    pub fn frame(&self) -> u32 {
        self.frame
    }
    pub fn win_screen(&self) -> Option<&WinScreen> {
        self.win_screen.as_ref()
    }
    pub fn lose_screen(&self) -> Option<&LoseScreen> {
        self.lose_screen.as_ref()
    }
    pub fn show_ghost_hints(&self) -> bool {
        self.show_ghost_hints
    }
    pub fn show_valid_cells(&self) -> bool {
        self.show_valid_cells
    }

    /// Get ghost candidates for a cell (valid candidates not yet noted)
    pub fn get_ghost_candidates(&self, pos: Position) -> Vec<u8> {
        if self.grid.cell(pos).is_filled() || self.grid.cell(pos).is_given() {
            return Vec::new();
        }
        self.grid.get_candidates(pos).iter().collect()
    }

    /// Check if a cell has only one valid candidate (naked single)
    pub fn is_naked_single(&self, pos: Position) -> bool {
        if self.grid.cell(pos).is_filled() || self.grid.cell(pos).is_given() {
            return false;
        }
        self.grid.get_candidates(pos).count() == 1
    }

    pub fn is_complete(&self) -> bool {
        self.grid.is_complete() && self.grid.validate().is_valid
    }

    pub fn is_game_over(&self) -> bool {
        self.mistakes >= MAX_MISTAKES
    }

    pub fn is_paused(&self) -> bool {
        self.screen == ScreenState::Paused
    }

    pub fn toggle_pause(&mut self) {
        match self.screen {
            ScreenState::Playing => {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Paused;
            }
            ScreenState::Paused => {
                self.start_time = Self::now();
                self.screen = ScreenState::Playing;
            }
            _ => {}
        }
    }

    /// Check if a cell has a conflict
    #[allow(clippy::needless_range_loop)]
    pub fn has_conflict(&self, pos: Position) -> bool {
        if let Some(value) = self.grid.get(pos) {
            let values = self.grid.values();

            // Row
            for col in 0..9 {
                if col != pos.col && values[pos.row][col] == Some(value) {
                    return true;
                }
            }

            // Column
            for row in 0..9 {
                if row != pos.row && values[row][pos.col] == Some(value) {
                    return true;
                }
            }

            // Box
            let box_row = (pos.row / 3) * 3;
            let box_col = (pos.col / 3) * 3;
            for row in box_row..box_row + 3 {
                for col in box_col..box_col + 3 {
                    if (row != pos.row || col != pos.col) && values[row][col] == Some(value) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if position is highlighted (same row/col/box as cursor)
    pub fn is_highlighted(&self, pos: Position) -> bool {
        pos.row == self.cursor.row
            || pos.col == self.cursor.col
            || pos.box_index() == self.cursor.box_index()
    }

    /// Check if position has same value as cursor
    pub fn has_same_value(&self, pos: Position) -> bool {
        if let Some(cursor_value) = self.grid.get(self.cursor) {
            self.grid.get(pos) == Some(cursor_value)
        } else {
            false
        }
    }

    /// Get completed numbers (all 9 placed)
    pub fn completed_numbers(&self) -> [bool; 9] {
        let mut counts = [0u8; 9];
        let values = self.grid.values();

        for row in &values {
            for v in row.iter().flatten() {
                if (1..=9).contains(v) {
                    counts[(v - 1) as usize] += 1;
                }
            }
        }

        std::array::from_fn(|i| counts[i] >= 9)
    }

    /// Convert to serializable format
    pub fn to_serializable(&self) -> SerializableState {
        SerializableState {
            puzzle: self.puzzle.to_string_compact(),
            current: self.grid.to_string_compact(),
            solution: self.solution.to_string_compact(),
            difficulty: format!("{:?}", self.difficulty),
            cursor_row: self.cursor.row,
            cursor_col: self.cursor.col,
            mode: self.mode,
            screen: self.screen,
            elapsed_secs: self.elapsed_secs(),
            mistakes: self.mistakes,
            hints_used: self.hints_used,
            message: self.message.clone(),
        }
    }

    /// Create from serializable format
    pub fn from_serializable(state: SerializableState) -> Self {
        let puzzle = Grid::from_string(&state.puzzle).unwrap_or_else(|| {
            let mut gen = Generator::new();
            gen.generate(Difficulty::Medium)
        });
        let grid = Grid::from_string(&state.current).unwrap_or_else(|| puzzle.deep_clone());
        let solution = Grid::from_string(&state.solution).unwrap_or_else(|| {
            let solver = Solver::new();
            solver.solve(&puzzle).unwrap_or_else(|| puzzle.deep_clone())
        });

        let difficulty = match state.difficulty.as_str() {
            "Beginner" => Difficulty::Beginner,
            "Easy" => Difficulty::Easy,
            "Medium" => Difficulty::Medium,
            "Intermediate" => Difficulty::Intermediate,
            "Hard" => Difficulty::Hard,
            "Expert" => Difficulty::Expert,
            "Master" => Difficulty::Master,
            "Extreme" => Difficulty::Extreme,
            _ => Difficulty::Medium,
        };

        Self {
            grid,
            puzzle,
            solution,
            difficulty,
            cursor: Position::new(state.cursor_row.min(8), state.cursor_col.min(8)),
            mode: state.mode,
            screen: state.screen,
            start_time: Self::now() - (state.elapsed_secs as f64 * 1000.0),
            paused_elapsed: 0.0,
            mistakes: state.mistakes,
            hints_used: state.hints_used,
            message: state.message,
            message_timer: 0,
            current_hint: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame: 0,
            win_screen: None,
            lose_screen: None,
            show_ghost_hints: false,
            show_valid_cells: false,
        }
    }
}
