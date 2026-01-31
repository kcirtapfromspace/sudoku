use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use sudoku_core::{Difficulty, Generator, Grid, Hint, Position, Solver};

/// A single move in the game (for undo/redo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMove {
    SetValue {
        pos: Position,
        old_value: Option<u8>,
        new_value: Option<u8>,
    },
    ToggleCandidate {
        pos: Position,
        value: u8,
    },
    AddCandidate {
        pos: Position,
        value: u8,
    },
    RemoveCandidate {
        pos: Position,
        value: u8,
    },
    SetCandidates {
        pos: Position,
        old_candidates: u16,
        new_candidates: u16,
    },
}

/// The game state
#[derive(Clone)]
pub struct Game {
    /// The current grid
    grid: Grid,
    /// The solution (for checking)
    solution: Grid,
    /// The original puzzle string (for replay/stats)
    original_puzzle: String,
    /// Difficulty level
    difficulty: Difficulty,
    /// Undo stack
    undo_stack: Vec<GameMove>,
    /// Redo stack
    redo_stack: Vec<GameMove>,
    /// Start time
    start_time: Instant,
    /// Elapsed time (for pause/resume)
    elapsed: Duration,
    /// Whether the game is paused
    paused: bool,
    /// Whether the game is completed
    completed: bool,
    /// Number of hints used
    hints_used: usize,
    /// Number of mistakes made
    mistakes: usize,
    /// Time of last move (for anti-bot tracking)
    last_move_time: Instant,
    /// All move times in milliseconds (for anti-bot)
    move_times_ms: Vec<u64>,
    /// Whether notes (candidates) were used during this game
    notes_used: bool,
}

impl Game {
    /// Create a new game with the specified difficulty
    pub fn new(difficulty: Difficulty) -> Self {
        let mut generator = Generator::new();
        let mut grid = generator.generate(difficulty);

        // Save original puzzle before solving
        let original_puzzle = grid.to_string_compact();

        let solver = Solver::new();
        let solution = solver
            .solve(&grid)
            .expect("Generated puzzle should be solvable");

        // Clear all candidates - players add their own notes manually
        grid.clear_all_candidates();

        let now = Instant::now();
        Self {
            grid,
            solution,
            original_puzzle,
            difficulty,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            start_time: now,
            elapsed: Duration::ZERO,
            paused: false,
            completed: false,
            hints_used: 0,
            mistakes: 0,
            last_move_time: now,
            move_times_ms: Vec::new(),
            notes_used: false,
        }
    }

    /// Create a game from a puzzle string
    pub fn from_string(puzzle: &str) -> Option<Self> {
        let grid = Grid::from_string(puzzle)?;
        let solver = Solver::new();
        let solution = solver.solve(&grid)?;
        let difficulty = solver.rate_difficulty(&grid);

        let now = Instant::now();
        Some(Self {
            grid,
            solution,
            original_puzzle: puzzle.to_string(),
            difficulty,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            start_time: now,
            elapsed: Duration::ZERO,
            paused: false,
            completed: false,
            hints_used: 0,
            mistakes: 0,
            last_move_time: now,
            move_times_ms: Vec::new(),
            notes_used: false,
        })
    }

    /// Get the current grid
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Get the difficulty
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// Get the elapsed time
    pub fn elapsed(&self) -> Duration {
        if self.paused || self.completed {
            self.elapsed
        } else {
            self.elapsed + self.start_time.elapsed()
        }
    }

    /// Format the elapsed time as MM:SS
    pub fn elapsed_string(&self) -> String {
        let secs = self.elapsed().as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Check if the game is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Check if the game is completed
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Get hints used count
    pub fn hints_used(&self) -> usize {
        self.hints_used
    }

    /// Get mistakes count
    pub fn mistakes(&self) -> usize {
        self.mistakes
    }

    /// Get the original puzzle string
    pub fn original_puzzle(&self) -> &str {
        &self.original_puzzle
    }

    /// Get move times for anti-bot analysis
    pub fn move_times_ms(&self) -> &[u64] {
        &self.move_times_ms
    }

    /// Get total moves made
    pub fn moves_count(&self) -> usize {
        self.move_times_ms.len()
    }

    /// Check if notes (candidates) were used during this game
    pub fn notes_used(&self) -> bool {
        self.notes_used
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.completed {
            return;
        }

        if self.paused {
            // Resume: reset start time, keeping elapsed
            self.start_time = Instant::now();
        } else {
            // Pause: save current elapsed
            self.elapsed += self.start_time.elapsed();
        }
        self.paused = !self.paused;
    }

    /// Set a value at a position
    pub fn set_value(&mut self, pos: Position, value: u8) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() {
            return false;
        }

        // Track move timing for anti-bot
        let now = Instant::now();
        let move_time = now.duration_since(self.last_move_time).as_millis() as u64;
        self.move_times_ms.push(move_time);
        self.last_move_time = now;

        let old_value = cell.value();

        // Check if this is correct
        let is_correct = self.solution.get(pos) == Some(value);

        if !is_correct {
            self.mistakes += 1;
        }

        // Make the move
        let game_move = GameMove::SetValue {
            pos,
            old_value,
            new_value: Some(value),
        };

        self.grid.set_cell_unchecked(pos, Some(value));
        self.grid.recalculate_candidates();

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        // Check for completion
        if self.grid.is_complete() && self.grid.validate().is_valid {
            self.completed = true;
            self.elapsed += self.start_time.elapsed();
        }

        is_correct
    }

    /// Clear a cell
    pub fn clear_cell(&mut self, pos: Position) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() {
            return false;
        }

        let old_value = cell.value();
        if old_value.is_none() {
            return false;
        }

        let game_move = GameMove::SetValue {
            pos,
            old_value,
            new_value: None,
        };

        self.grid.set_cell_unchecked(pos, None);
        self.grid.recalculate_candidates();

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        true
    }

    /// Toggle a candidate
    pub fn toggle_candidate(&mut self, pos: Position, value: u8) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }

        let game_move = GameMove::ToggleCandidate { pos, value };

        self.grid.cell_mut(pos).toggle_candidate(value);

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        // Mark that notes were used
        self.notes_used = true;

        true
    }

    /// Add a candidate (Create)
    #[allow(dead_code)]
    pub fn add_candidate(&mut self, pos: Position, value: u8) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }

        // Only add if not already present
        if cell.has_candidate(value) {
            return false;
        }

        let game_move = GameMove::AddCandidate { pos, value };

        self.grid.cell_mut(pos).add_candidate(value);

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        // Mark that notes were used
        self.notes_used = true;

        true
    }

    /// Remove a candidate (Delete single)
    #[allow(dead_code)]
    pub fn remove_candidate(&mut self, pos: Position, value: u8) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }

        // Only remove if present
        if !cell.has_candidate(value) {
            return false;
        }

        let game_move = GameMove::RemoveCandidate { pos, value };

        self.grid.cell_mut(pos).remove_candidate(value);

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        true
    }

    /// Clear all candidates from a cell (Delete all)
    pub fn clear_candidates(&mut self, pos: Position) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }

        let old_candidates = cell.candidates().as_raw();
        if old_candidates == 0 {
            return false; // Already empty
        }

        let game_move = GameMove::SetCandidates {
            pos,
            old_candidates,
            new_candidates: 0,
        };

        self.grid
            .cell_mut(pos)
            .set_candidates(sudoku_core::BitSet::empty());

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        true
    }

    /// Set all valid candidates for a cell (based on constraints)
    pub fn fill_candidates(&mut self, pos: Position) -> bool {
        if self.completed || self.paused {
            return false;
        }

        let cell = self.grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }

        let old_candidates = cell.candidates().as_raw();

        // Calculate valid candidates based on current grid state
        let valid = self.grid.get_candidates(pos);
        let new_candidates = valid.as_raw();

        if old_candidates == new_candidates {
            return false; // No change needed
        }

        let game_move = GameMove::SetCandidates {
            pos,
            old_candidates,
            new_candidates,
        };

        self.grid.cell_mut(pos).set_candidates(valid);

        self.undo_stack.push(game_move);
        self.redo_stack.clear();

        // Mark that notes were used
        self.notes_used = true;

        true
    }

    /// Fill all empty cells with valid candidates
    pub fn fill_all_candidates(&mut self) -> bool {
        if self.completed || self.paused {
            return false;
        }

        // Recalculate all candidates based on constraints
        self.grid.recalculate_candidates();
        // Note: This is not undoable as a single action (would need complex undo)
        self.redo_stack.clear();

        // Mark that notes were used
        self.notes_used = true;

        true
    }

    /// Clear all candidates from all cells
    pub fn clear_all_candidates(&mut self) -> bool {
        if self.completed || self.paused {
            return false;
        }

        self.grid.clear_all_candidates();
        self.redo_stack.clear();

        true
    }

    /// Get candidates for a cell (Read)
    #[allow(dead_code)]
    pub fn get_cell_candidates(&self, pos: Position) -> Vec<u8> {
        self.grid.cell(pos).candidates().iter().collect()
    }

    /// Check if a cell has a specific candidate
    #[allow(dead_code)]
    pub fn has_candidate(&self, pos: Position, value: u8) -> bool {
        self.grid.cell(pos).has_candidate(value)
    }

    /// Undo the last move
    pub fn undo(&mut self) -> bool {
        if self.completed || self.paused {
            return false;
        }

        if let Some(game_move) = self.undo_stack.pop() {
            match &game_move {
                GameMove::SetValue { pos, old_value, .. } => {
                    self.grid.set_cell_unchecked(*pos, *old_value);
                    self.grid.recalculate_candidates();
                }
                GameMove::ToggleCandidate { pos, value } => {
                    self.grid.cell_mut(*pos).toggle_candidate(*value);
                }
                GameMove::AddCandidate { pos, value } => {
                    // Undo add = remove
                    self.grid.cell_mut(*pos).remove_candidate(*value);
                }
                GameMove::RemoveCandidate { pos, value } => {
                    // Undo remove = add
                    self.grid.cell_mut(*pos).add_candidate(*value);
                }
                GameMove::SetCandidates {
                    pos,
                    old_candidates,
                    ..
                } => {
                    self.grid
                        .cell_mut(*pos)
                        .set_candidates(sudoku_core::BitSet::from_raw(*old_candidates));
                }
            }
            self.redo_stack.push(game_move);
            true
        } else {
            false
        }
    }

    /// Redo the last undone move
    pub fn redo(&mut self) -> bool {
        if self.completed || self.paused {
            return false;
        }

        if let Some(game_move) = self.redo_stack.pop() {
            match &game_move {
                GameMove::SetValue { pos, new_value, .. } => {
                    self.grid.set_cell_unchecked(*pos, *new_value);
                    self.grid.recalculate_candidates();
                }
                GameMove::ToggleCandidate { pos, value } => {
                    self.grid.cell_mut(*pos).toggle_candidate(*value);
                }
                GameMove::AddCandidate { pos, value } => {
                    self.grid.cell_mut(*pos).add_candidate(*value);
                }
                GameMove::RemoveCandidate { pos, value } => {
                    self.grid.cell_mut(*pos).remove_candidate(*value);
                }
                GameMove::SetCandidates {
                    pos,
                    new_candidates,
                    ..
                } => {
                    self.grid
                        .cell_mut(*pos)
                        .set_candidates(sudoku_core::BitSet::from_raw(*new_candidates));
                }
            }
            self.undo_stack.push(game_move);
            true
        } else {
            false
        }
    }

    /// Get a hint
    pub fn get_hint(&mut self) -> Option<Hint> {
        if self.completed || self.paused {
            return None;
        }

        let solver = Solver::new();
        let hint = solver.get_hint(&self.grid);

        if hint.is_some() {
            self.hints_used += 1;
        }

        hint
    }

    /// Apply a hint directly
    pub fn apply_hint(&mut self) -> Option<Position> {
        let hint = self.get_hint()?;

        match hint.hint_type {
            sudoku_core::HintType::SetValue { pos, value } => {
                self.set_value(pos, value);
                Some(pos)
            }
            sudoku_core::HintType::EliminateCandidates { pos, values } => {
                for value in values {
                    if self.grid.get_candidates(pos).contains(value) {
                        self.toggle_candidate(pos, value);
                    }
                }
                Some(pos)
            }
        }
    }

    /// Get which numbers (1-9) are fully placed on the board (all 9 instances)
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

    /// Get which rows are completely and correctly filled
    pub fn completed_rows(&self) -> [bool; 9] {
        let values = self.grid.values();
        let solution_values = self.solution.values();
        let mut result = [false; 9];

        for row in 0..9 {
            let mut all_filled = true;
            let mut all_correct = true;

            for col in 0..9 {
                match values[row][col] {
                    Some(v) => {
                        if solution_values[row][col] != Some(v) {
                            all_correct = false;
                        }
                    }
                    None => {
                        all_filled = false;
                    }
                }
            }

            result[row] = all_filled && all_correct;
        }
        result
    }

    /// Get which columns are completely and correctly filled
    pub fn completed_columns(&self) -> [bool; 9] {
        let values = self.grid.values();
        let solution_values = self.solution.values();
        let mut result = [false; 9];

        for col in 0..9 {
            let mut all_filled = true;
            let mut all_correct = true;

            for row in 0..9 {
                match values[row][col] {
                    Some(v) => {
                        if solution_values[row][col] != Some(v) {
                            all_correct = false;
                        }
                    }
                    None => {
                        all_filled = false;
                    }
                }
            }

            result[col] = all_filled && all_correct;
        }
        result
    }

    /// Get which 3x3 boxes are completely and correctly filled
    #[allow(clippy::needless_range_loop)]
    pub fn completed_boxes(&self) -> [bool; 9] {
        let values = self.grid.values();
        let solution_values = self.solution.values();
        let mut result = [false; 9];

        for box_idx in 0..9 {
            let box_row = (box_idx / 3) * 3;
            let box_col = (box_idx % 3) * 3;

            let mut all_filled = true;
            let mut all_correct = true;

            for dr in 0..3 {
                for dc in 0..3 {
                    let row = box_row + dr;
                    let col = box_col + dc;

                    match values[row][col] {
                        Some(v) => {
                            if solution_values[row][col] != Some(v) {
                                all_correct = false;
                            }
                        }
                        None => {
                            all_filled = false;
                        }
                    }
                }
            }

            result[box_idx] = all_filled && all_correct;
        }
        result
    }

    /// Check if a position has a conflict
    #[allow(clippy::needless_range_loop)]
    pub fn has_conflict(&self, pos: Position) -> bool {
        if let Some(value) = self.grid.get(pos) {
            // Check if this value appears elsewhere in same row/col/box
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

    /// Serialize the game state for saving
    pub fn serialize(&self) -> String {
        let state = SaveState {
            puzzle: self.grid.to_string_compact(),
            solution: self.solution.to_string_compact(),
            difficulty: self.difficulty,
            elapsed_secs: self.elapsed().as_secs(),
            hints_used: self.hints_used,
            mistakes: self.mistakes,
        };
        serde_json::to_string(&state).unwrap_or_default()
    }

    /// Deserialize a saved game state
    pub fn deserialize(json: &str) -> Option<Self> {
        let state: SaveState = serde_json::from_str(json).ok()?;

        let grid = Grid::from_string(&state.puzzle)?;
        let solution = Grid::from_string(&state.solution)?;

        let now = Instant::now();
        Some(Self {
            grid,
            solution,
            original_puzzle: state.puzzle.clone(),
            difficulty: state.difficulty,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            start_time: now,
            elapsed: Duration::from_secs(state.elapsed_secs),
            paused: true, // Start paused when loading
            completed: false,
            hints_used: state.hints_used,
            mistakes: state.mistakes,
            last_move_time: now,
            move_times_ms: Vec::new(), // Can't restore move times from save
            notes_used: false,         // Reset for loaded game
        })
    }
}

#[derive(Serialize, Deserialize)]
struct SaveState {
    puzzle: String,
    solution: String,
    difficulty: Difficulty,
    elapsed_secs: u64,
    hints_used: usize,
    mistakes: usize,
}
