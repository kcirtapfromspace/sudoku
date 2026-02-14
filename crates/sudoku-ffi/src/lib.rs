use std::sync::{Arc, Mutex};
use sudoku_core::{
    Difficulty, Generator, Grid, Hint, HintType, Polarity, Position, ProofCertificate, PuzzleId,
    Solver,
};

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

/// A cell position used in hint visualization
#[derive(Debug, Clone, uniffi::Record)]
pub struct HintCell {
    pub row: u8,
    pub col: u8,
}

/// Role of a cell in hint visualization
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum HintCellRole {
    None,
    Target,
    Involved,
    ChainOn,
    ChainOff,
    FishBase,
    FishCover,
    FishFin,
    UrFloor,
    UrRoof,
    AlsGroup,
}

impl HintCellRole {
    fn to_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Target => 1,
            Self::Involved => 2,
            Self::ChainOn => 3,
            Self::ChainOff => 4,
            Self::FishBase => 5,
            Self::FishCover => 6,
            Self::FishFin => 7,
            Self::UrFloor => 8,
            Self::UrRoof => 9,
            Self::AlsGroup => 10,
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
    /// Sudoku Explainer (SE) difficulty rating for this technique
    pub se_rating: f32,
    /// Cells involved in the reasoning (for highlighting)
    pub involved_cells: Vec<HintCell>,
}

impl From<Hint> for GameHint {
    fn from(hint: Hint) -> Self {
        let (row, col, value, eliminate) = match hint.hint_type {
            HintType::SetValue { pos, value } => {
                (pos.row as u8, pos.col as u8, Some(value), vec![])
            }
            HintType::EliminateCandidates { pos, values } => {
                (pos.row as u8, pos.col as u8, None, values)
            }
        };

        let se_rating = hint.technique.se_rating();
        let involved_cells = hint
            .involved_cells
            .iter()
            .map(|p| HintCell {
                row: p.row as u8,
                col: p.col as u8,
            })
            .collect();

        GameHint {
            row,
            col,
            value,
            eliminate,
            explanation: hint.explanation,
            technique: hint.technique.to_string(),
            se_rating,
            involved_cells,
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
    rated_difficulty: Mutex<Difficulty>,
    undo_stack: Mutex<Vec<(usize, usize, Option<u8>)>>,
    redo_stack: Mutex<Vec<(usize, usize, Option<u8>)>>,
    hints_used: Mutex<usize>,
    mistakes: Mutex<usize>,
    seed: Mutex<Option<u64>>,
    last_hint: Mutex<Option<Hint>>,
}

#[uniffi::export]
impl SudokuGame {
    /// Create a new classic Sudoku game with the specified difficulty
    #[uniffi::constructor]
    pub fn new_classic(difficulty: GameDifficulty) -> Arc<Self> {
        let diff: Difficulty = difficulty.into();
        let puzzle_id = PuzzleId::random(diff);
        let grid = puzzle_id.generate();

        let solver = Solver::new();
        let rated = solver.rate_difficulty(&grid);
        let solution = solver
            .solve(&grid)
            .expect("Generated puzzle should be solvable");

        Arc::new(Self {
            grid: Mutex::new(grid),
            solution: Mutex::new(solution),
            difficulty: Mutex::new(diff),
            rated_difficulty: Mutex::new(rated),
            undo_stack: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
            hints_used: Mutex::new(0),
            mistakes: Mutex::new(0),
            seed: Mutex::new(Some(puzzle_id.seed)),
            last_hint: Mutex::new(None),
        })
    }

    /// Create a new Sudoku game targeting a specific SE (Sudoku Explainer) rating
    #[uniffi::constructor]
    pub fn new_with_se_rating(target_se: f32) -> Arc<Self> {
        let mut generator = Generator::new();
        let grid = generator.generate_for_se(target_se);

        let solver = Solver::new();
        let rated = solver.rate_difficulty(&grid);
        let solution = solver
            .solve(&grid)
            .expect("Generated puzzle should be solvable");

        Arc::new(Self {
            grid: Mutex::new(grid),
            solution: Mutex::new(solution),
            difficulty: Mutex::new(rated),
            rated_difficulty: Mutex::new(rated),
            undo_stack: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
            hints_used: Mutex::new(0),
            mistakes: Mutex::new(0),
            seed: Mutex::new(None),
            last_hint: Mutex::new(None),
        })
    }

    /// Make a move: place a value at a position
    pub fn make_move(&self, row: u8, col: u8, value: u8) -> MoveResult {
        if !(1..=9).contains(&value) {
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
        self.undo_stack
            .lock()
            .unwrap()
            .push((row as usize, col as usize, old_value));
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

        self.undo_stack
            .lock()
            .unwrap()
            .push((row as usize, col as usize, old_value));
        self.redo_stack.lock().unwrap().clear();

        grid.set_cell_unchecked(pos, None);
        grid.recalculate_candidates();

        MoveResult::Success
    }

    /// Toggle a candidate (pencil mark)
    pub fn toggle_candidate(&self, row: u8, col: u8, value: u8) -> bool {
        if !(1..=9).contains(&value) {
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

            self.redo_stack
                .lock()
                .unwrap()
                .push((row, col, current_value));

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

            self.undo_stack
                .lock()
                .unwrap()
                .push((row, col, current_value));

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
            let game_hint: GameHint = hint.clone().into();
            *self.last_hint.lock().unwrap() = Some(hint);
            Some(game_hint)
        } else {
            None
        }
    }

    /// Get cell roles for hint visualization.
    /// Returns 81 u8 values (one per cell), encoding HintCellRole.
    /// detail_level: 0 = Summary (target + involved), 1 = ProofDetail (proof-specific roles).
    pub fn get_hint_cell_roles(&self, detail_level: u8) -> Vec<u8> {
        let hint_guard = self.last_hint.lock().unwrap();
        match hint_guard.as_ref() {
            Some(hint) => Self::compute_hint_roles(hint, detail_level)
                .iter()
                .map(|r| r.to_u8())
                .collect(),
            None => vec![0u8; 81],
        }
    }

    /// Clear the stored hint (call when user dismisses hint or selects a new cell)
    pub fn clear_hint(&self) {
        *self.last_hint.lock().unwrap() = None;
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

    /// Get the requested difficulty level
    pub fn get_difficulty(&self) -> GameDifficulty {
        (*self.difficulty.lock().unwrap()).into()
    }

    /// Get the actual rated difficulty of the generated puzzle.
    /// This may differ from the requested difficulty (one tier easier is accepted).
    pub fn get_rated_difficulty(&self) -> GameDifficulty {
        (*self.rated_difficulty.lock().unwrap()).into()
    }

    /// Get the Sudoku Explainer (SE) numerical rating for this puzzle
    pub fn get_se_rating(&self) -> f32 {
        let grid = self.grid.lock().unwrap();
        let solver = Solver::new();
        solver.rate_se(&grid)
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
        let rated_difficulty = self.rated_difficulty.lock().unwrap();

        serde_json::json!({
            "puzzle": grid.to_string_compact(),
            "solution": solution.to_string_compact(),
            "difficulty": format!("{:?}", *difficulty),
            "rated_difficulty": format!("{:?}", *rated_difficulty),
            "hints_used": *self.hints_used.lock().unwrap(),
            "mistakes": *self.mistakes.lock().unwrap(),
        })
        .to_string()
    }

    /// Get valid candidates for a cell (for ghost hints feature)
    pub fn get_valid_candidates(&self, row: u8, col: u8) -> Vec<u8> {
        let pos = Position::new(row as usize, col as usize);
        let grid = self.grid.lock().unwrap();
        grid.get_candidates(pos).iter().collect()
    }

    /// Check if a cell is a naked single (only one valid candidate)
    pub fn is_naked_single(&self, row: u8, col: u8) -> bool {
        let pos = Position::new(row as usize, col as usize);
        let grid = self.grid.lock().unwrap();
        let cell = grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }
        grid.get_candidates(pos).count() == 1
    }

    /// Fill candidates for a single cell with valid values
    pub fn fill_cell_candidates(&self, row: u8, col: u8) -> bool {
        let pos = Position::new(row as usize, col as usize);
        let mut grid = self.grid.lock().unwrap();
        let cell = grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }
        let valid = grid.get_candidates(pos);
        grid.cell_mut(pos).set_candidates(valid);
        true
    }

    /// Fill all empty cells with their valid candidates
    pub fn fill_all_candidates(&self) {
        let mut grid = self.grid.lock().unwrap();
        grid.recalculate_candidates();
    }

    /// Clear candidates from a single cell
    pub fn clear_cell_candidates(&self, row: u8, col: u8) -> bool {
        let pos = Position::new(row as usize, col as usize);
        let mut grid = self.grid.lock().unwrap();
        let cell = grid.cell(pos);
        if cell.is_given() || cell.is_filled() {
            return false;
        }
        grid.cell_mut(pos)
            .set_candidates(sudoku_core::BitSet::empty());
        true
    }

    /// Clear all candidates from all cells
    pub fn clear_all_candidates(&self) {
        let mut grid = self.grid.lock().unwrap();
        grid.clear_all_candidates();
    }

    /// Remove invalid candidates - keep only candidates that match the solution
    /// This is the "Check Notes" feature - removes wrong pencil marks
    pub fn remove_invalid_candidates(&self) {
        let mut grid = self.grid.lock().unwrap();
        let solution = self.solution.lock().unwrap();

        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                let cell = grid.cell(pos);
                if cell.is_filled() || cell.is_given() {
                    continue;
                }

                // Get the correct value for this cell
                if let Some(correct) = solution.get(pos) {
                    // Only keep the correct value as a candidate if it was already there
                    if cell.has_candidate(correct) {
                        grid.cell_mut(pos)
                            .set_candidates(sudoku_core::BitSet::single(correct));
                    } else {
                        // Cell had no candidates or didn't have the correct one
                        grid.cell_mut(pos)
                            .set_candidates(sudoku_core::BitSet::empty());
                    }
                }
            }
        }
    }

    /// Get the correct value for a cell (from solution)
    pub fn get_solution_value(&self, row: u8, col: u8) -> u8 {
        let pos = Position::new(row as usize, col as usize);
        let solution = self.solution.lock().unwrap();
        solution.get(pos).unwrap_or(0)
    }

    /// Check if the current value at a position is correct
    pub fn is_value_correct(&self, row: u8, col: u8) -> bool {
        let pos = Position::new(row as usize, col as usize);
        let grid = self.grid.lock().unwrap();
        let solution = self.solution.lock().unwrap();
        grid.get(pos) == solution.get(pos)
    }

    /// Get count of remaining empty cells
    pub fn get_empty_count(&self) -> u32 {
        let grid = self.grid.lock().unwrap();
        let mut count = 0u32;
        for row in 0..9 {
            for col in 0..9 {
                if grid.get(Position::new(row, col)).is_none() {
                    count += 1;
                }
            }
        }
        count
    }

    /// Get count of each number placed (for number completion indicator)
    pub fn get_number_counts(&self) -> Vec<u8> {
        let grid = self.grid.lock().unwrap();
        let values = grid.values();
        let mut counts = [0u8; 9];
        for row in &values {
            for v in row.iter().flatten() {
                if (1..=9).contains(v) {
                    counts[(v - 1) as usize] += 1;
                }
            }
        }
        counts.to_vec()
    }

    /// Check if can undo
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.lock().unwrap().is_empty()
    }

    /// Check if can redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.lock().unwrap().is_empty()
    }

    /// Get the puzzle as an 81-character string (givens as digits, empty as '0')
    pub fn get_puzzle_string(&self) -> String {
        let grid = self.grid.lock().unwrap();
        let values = grid.values();
        let mut result = String::with_capacity(81);
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if grid.cell(pos).is_given() {
                    if let Some(v) = values[row][col] {
                        result.push((b'0' + v) as char);
                    } else {
                        result.push('0');
                    }
                } else {
                    result.push('0');
                }
            }
        }
        result
    }

    /// Apply a hint automatically
    pub fn apply_hint(&self) -> Option<GameHint> {
        // Use stored hint if available, otherwise get a new one
        let hint = {
            let mut last = self.last_hint.lock().unwrap();
            if let Some(h) = last.take() {
                h
            } else {
                let grid = self.grid.lock().unwrap();
                let solver = Solver::new();
                solver.get_hint(&grid)?
            }
        };

        *self.hints_used.lock().unwrap() += 1;

        match &hint.hint_type {
            HintType::SetValue { pos, value } => {
                let mut grid = self.grid.lock().unwrap();
                let old_value = grid.get(*pos);
                self.undo_stack
                    .lock()
                    .unwrap()
                    .push((pos.row, pos.col, old_value));
                self.redo_stack.lock().unwrap().clear();
                grid.set_cell_unchecked(*pos, Some(*value));
                grid.recalculate_candidates();
            }
            HintType::EliminateCandidates { pos, values } => {
                let mut grid = self.grid.lock().unwrap();
                for value in values {
                    if grid.cell(*pos).has_candidate(*value) {
                        grid.cell_mut(*pos).remove_candidate(*value);
                    }
                }
            }
        }

        Some(hint.into())
    }

    /// Get the short code for this puzzle, if available
    pub fn get_short_code(&self) -> Option<String> {
        let seed = self.seed.lock().unwrap();
        let difficulty = *self.difficulty.lock().unwrap();
        seed.map(|s| {
            PuzzleId {
                difficulty,
                seed: s,
            }
            .to_short_code()
        })
    }
}

impl SudokuGame {
    /// Return cells belonging to a sector index.
    /// Convention: 0..8=rows, 9..17=cols, 18..26=boxes.
    fn sector_cells(sector: usize) -> Vec<usize> {
        if sector < 9 {
            let row = sector;
            (0..9).map(|col| row * 9 + col).collect()
        } else if sector < 18 {
            let col = sector - 9;
            (0..9).map(|row| row * 9 + col).collect()
        } else {
            let b = sector - 18;
            let br = (b / 3) * 3;
            let bc = (b % 3) * 3;
            let mut cells = Vec::with_capacity(9);
            for r in br..br + 3 {
                for c in bc..bc + 3 {
                    cells.push(r * 9 + c);
                }
            }
            cells
        }
    }

    /// Compute hint cell roles for all 81 cells.
    /// detail_level: 0 = Summary, 1 = ProofDetail.
    fn compute_hint_roles(hint: &Hint, detail_level: u8) -> [HintCellRole; 81] {
        let mut roles = [HintCellRole::None; 81];

        // Always mark the target cell
        let target_idx = match &hint.hint_type {
            HintType::SetValue { pos, .. } | HintType::EliminateCandidates { pos, .. } => {
                pos.row * 9 + pos.col
            }
        };
        roles[target_idx] = HintCellRole::Target;

        // Mark involved cells
        for pos in &hint.involved_cells {
            let idx = pos.row * 9 + pos.col;
            if matches!(roles[idx], HintCellRole::None) {
                roles[idx] = HintCellRole::Involved;
            }
        }

        // At ProofDetail (level 1), override with proof-specific roles
        if detail_level >= 1 {
            if let Some(ref proof) = hint.proof {
                match proof {
                    ProofCertificate::Fish {
                        base_sectors,
                        cover_sectors,
                        fins,
                        ..
                    } => {
                        for &s in base_sectors {
                            for idx in Self::sector_cells(s) {
                                if matches!(roles[idx], HintCellRole::Involved) {
                                    roles[idx] = HintCellRole::FishBase;
                                }
                            }
                        }
                        for &s in cover_sectors {
                            for idx in Self::sector_cells(s) {
                                if matches!(roles[idx], HintCellRole::Involved) {
                                    roles[idx] = HintCellRole::FishCover;
                                }
                            }
                        }
                        for &idx in fins {
                            if idx < 81 {
                                roles[idx] = HintCellRole::FishFin;
                            }
                        }
                    }
                    ProofCertificate::Aic { chain, .. } => {
                        for &(cell, _digit, polarity) in chain {
                            if cell < 81 {
                                roles[cell] = match polarity {
                                    Polarity::On => HintCellRole::ChainOn,
                                    Polarity::Off => HintCellRole::ChainOff,
                                };
                            }
                        }
                    }
                    ProofCertificate::Uniqueness {
                        floor_cells,
                        roof_cells,
                        ..
                    } => {
                        for &idx in floor_cells {
                            if idx < 81 {
                                roles[idx] = HintCellRole::UrFloor;
                            }
                        }
                        for &idx in roof_cells {
                            if idx < 81 {
                                roles[idx] = HintCellRole::UrRoof;
                            }
                        }
                    }
                    ProofCertificate::Als { als_chain, .. } => {
                        for als in als_chain {
                            for &idx in &als.cells {
                                if idx < 81 && !matches!(roles[idx], HintCellRole::Target) {
                                    roles[idx] = HintCellRole::AlsGroup;
                                }
                            }
                        }
                    }
                    ProofCertificate::Basic { .. }
                    | ProofCertificate::Forcing { .. }
                    | ProofCertificate::Backtracking => {}
                }
            }
        }

        // Ensure target stays as Target
        roles[target_idx] = HintCellRole::Target;
        roles
    }

    #[allow(clippy::needless_range_loop)]
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

// Free functions for creating games (UniFFI doesn't support associated functions that aren't constructors)

/// Create a game from a puzzle string (81 characters, 0 or . for empty)
#[uniffi::export]
pub fn game_from_string(puzzle: String) -> Option<Arc<SudokuGame>> {
    let grid = Grid::from_string(&puzzle)?;
    let solver = Solver::new();
    let solution = solver.solve(&grid)?;
    let difficulty = solver.rate_difficulty(&grid);

    Some(Arc::new(SudokuGame {
        grid: Mutex::new(grid.clone()),
        solution: Mutex::new(solution),
        difficulty: Mutex::new(difficulty),
        rated_difficulty: Mutex::new(solver.rate_difficulty(&grid)),
        undo_stack: Mutex::new(Vec::new()),
        redo_stack: Mutex::new(Vec::new()),
        hints_used: Mutex::new(0),
        mistakes: Mutex::new(0),
        seed: Mutex::new(None),
        last_hint: Mutex::new(None),
    }))
}

/// Create a game from a short code (e.g., "M1A2B3C4")
#[uniffi::export]
pub fn game_from_short_code(code: String) -> Option<Arc<SudokuGame>> {
    let puzzle_id = PuzzleId::from_short_code(&code)?;
    let grid = puzzle_id.generate();
    let solver = Solver::new();
    let solution = solver.solve(&grid)?;
    let rated = solver.rate_difficulty(&grid);

    Some(Arc::new(SudokuGame {
        grid: Mutex::new(grid),
        solution: Mutex::new(solution),
        difficulty: Mutex::new(puzzle_id.difficulty),
        rated_difficulty: Mutex::new(rated),
        undo_stack: Mutex::new(Vec::new()),
        redo_stack: Mutex::new(Vec::new()),
        hints_used: Mutex::new(0),
        mistakes: Mutex::new(0),
        seed: Mutex::new(Some(puzzle_id.seed)),
        last_hint: Mutex::new(None),
    }))
}

/// Helper to parse a difficulty string
fn parse_difficulty(s: &str) -> Difficulty {
    match s {
        "Beginner" => Difficulty::Beginner,
        "Easy" => Difficulty::Easy,
        "Medium" => Difficulty::Medium,
        "Intermediate" => Difficulty::Intermediate,
        "Hard" => Difficulty::Hard,
        "Expert" => Difficulty::Expert,
        "Master" => Difficulty::Master,
        "Extreme" => Difficulty::Extreme,
        _ => Difficulty::Medium,
    }
}

/// Deserialize a saved game state
#[uniffi::export]
pub fn game_deserialize(json: String) -> Option<Arc<SudokuGame>> {
    let data: serde_json::Value = serde_json::from_str(&json).ok()?;

    let puzzle_str = data["puzzle"].as_str()?;
    let solution_str = data["solution"].as_str()?;

    let grid = Grid::from_string(puzzle_str)?;
    let solution = Grid::from_string(solution_str)?;

    let difficulty = parse_difficulty(data["difficulty"].as_str()?);

    // Deserialize rated_difficulty if present, otherwise re-rate the puzzle
    let rated_difficulty = data["rated_difficulty"]
        .as_str()
        .map(parse_difficulty)
        .unwrap_or_else(|| {
            let solver = Solver::new();
            solver.rate_difficulty(&grid)
        });

    let hints_used = data["hints_used"].as_u64().unwrap_or(0) as usize;
    let mistakes = data["mistakes"].as_u64().unwrap_or(0) as usize;

    Some(Arc::new(SudokuGame {
        grid: Mutex::new(grid),
        solution: Mutex::new(solution),
        difficulty: Mutex::new(difficulty),
        rated_difficulty: Mutex::new(rated_difficulty),
        undo_stack: Mutex::new(Vec::new()),
        redo_stack: Mutex::new(Vec::new()),
        hints_used: Mutex::new(hints_used),
        mistakes: Mutex::new(mistakes),
        seed: Mutex::new(None),
        last_hint: Mutex::new(None),
    }))
}
