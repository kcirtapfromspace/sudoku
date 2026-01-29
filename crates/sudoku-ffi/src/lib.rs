use sudoku_core::{Difficulty, Generator, Grid, Hint, HintType, Position, Solver};
use std::sync::{Arc, Mutex};

uniffi::setup_scaffolding!();

/// A move result returned from making a move
#[derive(Debug, Clone, uniffi::Enum)]
pub enum MoveResult {
    /// Move was successful
    Success,
    /// Move was successful and the puzzle is now complete
    Complete,
    /// The move conflicts with existing values
    Conflict,
    /// Cannot modify a given cell
    CannotModifyGiven,
    /// Invalid value (not 1-9)
    InvalidValue,
}

/// Difficulty level for puzzle generation
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum GameDifficulty {
    Beginner,
    Easy,
    Medium,
    Intermediate,
    Hard,
    Expert,
    Master,
    Extreme,
}

impl From<GameDifficulty> for Difficulty {
    fn from(d: GameDifficulty) -> Self {
        match d {
            GameDifficulty::Beginner => Difficulty::Beginner,
            GameDifficulty::Easy => Difficulty::Easy,
            GameDifficulty::Medium => Difficulty::Medium,
            GameDifficulty::Intermediate => Difficulty::Intermediate,
            GameDifficulty::Hard => Difficulty::Hard,
            GameDifficulty::Expert => Difficulty::Expert,
            GameDifficulty::Master => Difficulty::Master,
            GameDifficulty::Extreme => Difficulty::Extreme,
        }
    }
}

impl From<Difficulty> for GameDifficulty {
    fn from(d: Difficulty) -> Self {
        match d {
            Difficulty::Beginner => GameDifficulty::Beginner,
            Difficulty::Easy => GameDifficulty::Easy,
            Difficulty::Medium => GameDifficulty::Medium,
            Difficulty::Intermediate => GameDifficulty::Intermediate,
            Difficulty::Hard => GameDifficulty::Hard,
            Difficulty::Expert => GameDifficulty::Expert,
            Difficulty::Master => GameDifficulty::Master,
            Difficulty::Extreme => GameDifficulty::Extreme,
        }
    }
}

/// A hint for the player
#[derive(Debug, Clone, uniffi::Record)]
pub struct GameHint {
    /// Row of the cell (0-8)
    pub row: u8,
    /// Column of the cell (0-8)
    pub col: u8,
    /// The value to set (if applicable)
    pub value: Option<u8>,
    /// Values to eliminate (if applicable)
    pub eliminate: Vec<u8>,
    /// Human-readable explanation
    pub explanation: String,
    /// The technique name
    pub technique: String,
}

impl From<Hint> for GameHint {
    fn from(hint: Hint) -> Self {
        let (row, col, value, eliminate) = match hint.hint_type {
            HintType::SetValue { pos, value } => (pos.row as u8, pos.col as u8, Some(value), vec![]),
            HintType::EliminateCandidates { pos, values } => {
                (pos.row as u8, pos.col as u8, None, values)
            }
        };

        GameHint {
            row,
            col,
            value,
            eliminate,
            explanation: hint.explanation,
            technique: hint.technique.to_string(),
        }
    }
}

/// Cell state for UI rendering
#[derive(Debug, Clone, uniffi::Record)]
pub struct CellState {
    /// Row position (0-8)
    pub row: u8,
    /// Column position (0-8)
    pub col: u8,
    /// Current value (0 if empty)
    pub value: u8,
    /// Whether this is a given (puzzle) cell
    pub is_given: bool,
    /// Candidate values (pencil marks)
    pub candidates: Vec<u8>,
    /// Whether this cell has a conflict
    pub has_conflict: bool,
}

/// The main Sudoku game interface for mobile platforms
#[derive(uniffi::Object)]
pub struct SudokuGame {
    grid: Mutex<Grid>,
    solution: Mutex<Grid>,
    difficulty: Mutex<Difficulty>,
    undo_stack: Mutex<Vec<(usize, usize, Option<u8>)>>,
    redo_stack: Mutex<Vec<(usize, usize, Option<u8>)>>,
    hints_used: Mutex<usize>,
    mistakes: Mutex<usize>,
}

#[uniffi::export]
impl SudokuGame {
    /// Create a new classic Sudoku game with the specified difficulty
    #[uniffi::constructor]
    pub fn new_classic(difficulty: GameDifficulty) -> Arc<Self> {
        let diff: Difficulty = difficulty.into();
        let mut generator = Generator::new();
        let grid = generator.generate(diff);

        let solver = Solver::new();
        let solution = solver.solve(&grid).expect("Generated puzzle should be solvable");

        Arc::new(Self {
            grid: Mutex::new(grid),
            solution: Mutex::new(solution),
            difficulty: Mutex::new(diff),
            undo_stack: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
            hints_used: Mutex::new(0),
            mistakes: Mutex::new(0),
        })
    }

    /// Create a game from a puzzle string (81 characters, 0 or . for empty)
    #[uniffi::constructor]
    pub fn from_string(puzzle: String) -> Option<Arc<Self>> {
        let grid = Grid::from_string(&puzzle)?;
        let solver = Solver::new();
        let solution = solver.solve(&grid)?;
        let difficulty = solver.rate_difficulty(&grid);

        Some(Arc::new(Self {
            grid: Mutex::new(grid),
            solution: Mutex::new(solution),
            difficulty: Mutex::new(difficulty),
            undo_stack: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
            hints_used: Mutex::new(0),
            mistakes: Mutex::new(0),
        }))
    }

    /// Make a move: place a value at a position
    pub fn make_move(&self, row: u8, col: u8, value: u8) -> MoveResult {
        if value < 1 || value > 9 {
            return MoveResult::InvalidValue;
        }

        let pos = Position::new(row as usize, col as usize);
        let mut grid = self.grid.lock().unwrap();
        let solution = self.solution.lock().unwrap();

        if grid.cell(pos).is_given() {
            return MoveResult::CannotModifyGiven;
        }

        // Save for undo
        let old_value = grid.get(pos);
        self.undo_stack.lock().unwrap().push((row as usize, col as usize, old_value));
        self.redo_stack.lock().unwrap().clear();

        // Check if correct
        let is_correct = solution.get(pos) == Some(value);
        if !is_correct {
            *self.mistakes.lock().unwrap() += 1;
        }

        // Make the move
        grid.set_cell_unchecked(pos, Some(value));
        grid.recalculate_candidates();

        // Check for conflicts
        let values = grid.values();
        let has_conflict = Self::check_conflict(&values, pos, value);

        if has_conflict {
            return MoveResult::Conflict;
        }

        if grid.is_complete() && grid.validate().is_valid {
            return MoveResult::Complete;
        }

        MoveResult::Success
    }

    /// Clear a cell
    pub fn clear_cell(&self, row: u8, col: u8) -> MoveResult {
        let pos = Position::new(row as usize, col as usize);
        let mut grid = self.grid.lock().unwrap();

        if grid.cell(pos).is_given() {
            return MoveResult::CannotModifyGiven;
        }

        let old_value = grid.get(pos);
        if old_value.is_none() {
            return MoveResult::Success;
        }

        self.undo_stack.lock().unwrap().push((row as usize, col as usize, old_value));
        self.redo_stack.lock().unwrap().clear();

        grid.set_cell_unchecked(pos, None);
        grid.recalculate_candidates();

        MoveResult::Success
    }

    /// Toggle a candidate (pencil mark)
    pub fn toggle_candidate(&self, row: u8, col: u8, value: u8) -> bool {
        if value < 1 || value > 9 {
            return false;
        }

        let pos = Position::new(row as usize, col as usize);
        let mut grid = self.grid.lock().unwrap();

        let cell = grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }

        grid.cell_mut(pos).toggle_candidate(value);
        true
    }

    /// Undo the last move
    pub fn undo(&self) -> bool {
        let mut undo_stack = self.undo_stack.lock().unwrap();
        if let Some((row, col, old_value)) = undo_stack.pop() {
            let pos = Position::new(row, col);
            let mut grid = self.grid.lock().unwrap();
            let current_value = grid.get(pos);

            self.redo_stack.lock().unwrap().push((row, col, current_value));

            grid.set_cell_unchecked(pos, old_value);
            grid.recalculate_candidates();
            true
        } else {
            false
        }
    }

    /// Redo the last undone move
    pub fn redo(&self) -> bool {
        let mut redo_stack = self.redo_stack.lock().unwrap();
        if let Some((row, col, value)) = redo_stack.pop() {
            let pos = Position::new(row, col);
            let mut grid = self.grid.lock().unwrap();
            let current_value = grid.get(pos);

            self.undo_stack.lock().unwrap().push((row, col, current_value));

            grid.set_cell_unchecked(pos, value);
            grid.recalculate_candidates();
            true
        } else {
            false
        }
    }

    /// Get a hint
    pub fn get_hint(&self) -> Option<GameHint> {
        let grid = self.grid.lock().unwrap();
        let solver = Solver::new();

        if let Some(hint) = solver.get_hint(&grid) {
            *self.hints_used.lock().unwrap() += 1;
            Some(hint.into())
        } else {
            None
        }
    }

    /// Get the current value at a position (0 if empty)
    pub fn get_value(&self, row: u8, col: u8) -> u8 {
        let pos = Position::new(row as usize, col as usize);
        let grid = self.grid.lock().unwrap();
        grid.get(pos).unwrap_or(0)
    }

    /// Get candidates at a position
    pub fn get_candidates(&self, row: u8, col: u8) -> Vec<u8> {
        let pos = Position::new(row as usize, col as usize);
        let grid = self.grid.lock().unwrap();
        grid.get_candidates(pos).to_vec()
    }

    /// Check if a cell is given
    pub fn is_given(&self, row: u8, col: u8) -> bool {
        let pos = Position::new(row as usize, col as usize);
        let grid = self.grid.lock().unwrap();
        grid.cell(pos).is_given()
    }

    /// Get all cell states (for efficient bulk rendering)
    pub fn get_all_cells(&self) -> Vec<CellState> {
        let grid = self.grid.lock().unwrap();
        let values = grid.values();

        let mut cells = Vec::with_capacity(81);
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                let cell = grid.cell(pos);
                let value = cell.value().unwrap_or(0);
                let has_conflict = if value > 0 {
                    Self::check_conflict(&values, pos, value)
                } else {
                    false
                };

                cells.push(CellState {
                    row: row as u8,
                    col: col as u8,
                    value,
                    is_given: cell.is_given(),
                    candidates: cell.candidates().to_vec(),
                    has_conflict,
                });
            }
        }
        cells
    }

    /// Get the difficulty level
    pub fn get_difficulty(&self) -> GameDifficulty {
        (*self.difficulty.lock().unwrap()).into()
    }

    /// Get the number of hints used
    pub fn get_hints_used(&self) -> u32 {
        *self.hints_used.lock().unwrap() as u32
    }

    /// Get the number of mistakes made
    pub fn get_mistakes(&self) -> u32 {
        *self.mistakes.lock().unwrap() as u32
    }

    /// Check if the puzzle is complete
    pub fn is_complete(&self) -> bool {
        let grid = self.grid.lock().unwrap();
        grid.is_complete() && grid.validate().is_valid
    }

    /// Serialize the game state for saving
    pub fn serialize(&self) -> String {
        let grid = self.grid.lock().unwrap();
        let solution = self.solution.lock().unwrap();
        let difficulty = self.difficulty.lock().unwrap();

        serde_json::json!({
            "puzzle": grid.to_string_compact(),
            "solution": solution.to_string_compact(),
            "difficulty": format!("{:?}", *difficulty),
            "hints_used": *self.hints_used.lock().unwrap(),
            "mistakes": *self.mistakes.lock().unwrap(),
        })
        .to_string()
    }

    /// Deserialize a saved game state
    #[uniffi::constructor]
    pub fn deserialize(json: String) -> Option<Arc<Self>> {
        let data: serde_json::Value = serde_json::from_str(&json).ok()?;

        let puzzle_str = data["puzzle"].as_str()?;
        let solution_str = data["solution"].as_str()?;

        let grid = Grid::from_string(puzzle_str)?;
        let solution = Grid::from_string(solution_str)?;

        let difficulty = match data["difficulty"].as_str()? {
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

        let hints_used = data["hints_used"].as_u64().unwrap_or(0) as usize;
        let mistakes = data["mistakes"].as_u64().unwrap_or(0) as usize;

        Some(Arc::new(Self {
            grid: Mutex::new(grid),
            solution: Mutex::new(solution),
            difficulty: Mutex::new(difficulty),
            undo_stack: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
            hints_used: Mutex::new(hints_used),
            mistakes: Mutex::new(mistakes),
        }))
    }
}

impl SudokuGame {
    fn check_conflict(values: &[[Option<u8>; 9]; 9], pos: Position, value: u8) -> bool {
        // Check row
        for col in 0..9 {
            if col != pos.col && values[pos.row][col] == Some(value) {
                return true;
            }
        }

        // Check column
        for row in 0..9 {
            if row != pos.row && values[row][pos.col] == Some(value) {
                return true;
            }
        }

        // Check box
        let box_row = (pos.row / 3) * 3;
        let box_col = (pos.col / 3) * 3;
        for row in box_row..box_row + 3 {
            for col in box_col..box_col + 3 {
                if (row != pos.row || col != pos.col) && values[row][col] == Some(value) {
                    return true;
                }
            }
        }

        false
    }
}
