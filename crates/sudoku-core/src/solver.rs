use crate::{Grid, Position};
use serde::{Deserialize, Serialize};

/// Difficulty level of a puzzle
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Easy,
    Medium,
    Intermediate,
    Hard,
    Expert,
    Master,
    Extreme,
}

impl Difficulty {
    /// Get the maximum technique level allowed for this difficulty
    pub fn max_technique(&self) -> Technique {
        match self {
            Difficulty::Beginner => Technique::NakedSingle,
            Difficulty::Easy => Technique::NakedSingle,
            Difficulty::Medium => Technique::HiddenSingle,
            Difficulty::Intermediate => Technique::NakedTriple,
            Difficulty::Hard => Technique::BoxLineReduction,
            Difficulty::Expert => Technique::XWing,
            Difficulty::Master => Technique::XYWing,
            Difficulty::Extreme => Technique::Backtracking,
        }
    }

    /// Check if this is a secret/locked difficulty
    pub fn is_secret(&self) -> bool {
        matches!(self, Difficulty::Master | Difficulty::Extreme)
    }

    /// Get all standard (non-secret) difficulties
    pub fn standard_levels() -> &'static [Difficulty] {
        &[
            Difficulty::Beginner,
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Intermediate,
            Difficulty::Hard,
            Difficulty::Expert,
        ]
    }

    /// Get all difficulties including secret ones
    pub fn all_levels() -> &'static [Difficulty] {
        &[
            Difficulty::Beginner,
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Intermediate,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Master,
            Difficulty::Extreme,
        ]
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Beginner => write!(f, "Beginner"),
            Difficulty::Easy => write!(f, "Easy"),
            Difficulty::Medium => write!(f, "Medium"),
            Difficulty::Intermediate => write!(f, "Intermediate"),
            Difficulty::Hard => write!(f, "Hard"),
            Difficulty::Expert => write!(f, "Expert"),
            Difficulty::Master => write!(f, "Master"),
            Difficulty::Extreme => write!(f, "Extreme"),
        }
    }
}

/// Solving technique used (ordered by difficulty)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Technique {
    // Basic (Beginner/Easy)
    NakedSingle,
    HiddenSingle,

    // Intermediate
    NakedPair,
    HiddenPair,
    NakedTriple,
    HiddenTriple,

    // Hard
    PointingPair,
    BoxLineReduction,

    // Expert
    XWing,
    Swordfish,
    Jellyfish,

    // Master
    XYWing,
    XYZWing,
    WWing,
    SimpleColoring,
    XChain,

    // Extreme
    UniqueRectangle,
    Backtracking,
}

impl std::fmt::Display for Technique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Technique::NakedSingle => write!(f, "Naked Single"),
            Technique::HiddenSingle => write!(f, "Hidden Single"),
            Technique::NakedPair => write!(f, "Naked Pair"),
            Technique::HiddenPair => write!(f, "Hidden Pair"),
            Technique::NakedTriple => write!(f, "Naked Triple"),
            Technique::HiddenTriple => write!(f, "Hidden Triple"),
            Technique::PointingPair => write!(f, "Pointing Pair"),
            Technique::BoxLineReduction => write!(f, "Box/Line Reduction"),
            Technique::XWing => write!(f, "X-Wing"),
            Technique::Swordfish => write!(f, "Swordfish"),
            Technique::Jellyfish => write!(f, "Jellyfish"),
            Technique::XYWing => write!(f, "XY-Wing"),
            Technique::XYZWing => write!(f, "XYZ-Wing"),
            Technique::WWing => write!(f, "W-Wing"),
            Technique::SimpleColoring => write!(f, "Simple Coloring"),
            Technique::XChain => write!(f, "X-Chain"),
            Technique::UniqueRectangle => write!(f, "Unique Rectangle"),
            Technique::Backtracking => write!(f, "Backtracking"),
        }
    }
}

/// Type of hint provided
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HintType {
    /// Place this value in this cell
    SetValue { pos: Position, value: u8 },
    /// Remove these candidates from this cell
    EliminateCandidates { pos: Position, values: Vec<u8> },
}

/// A hint for the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    /// The technique used to find this hint
    pub technique: Technique,
    /// The type of hint
    pub hint_type: HintType,
    /// Explanation of the hint
    pub explanation: String,
    /// Cells involved in the reasoning
    pub involved_cells: Vec<Position>,
}

/// Configuration for the solver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum technique level to use (None = use all including backtracking)
    pub max_technique: Option<Technique>,
    /// Whether to track techniques used
    pub track_techniques: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_technique: None,
            track_techniques: false,
        }
    }
}

/// Sudoku solver with human-like techniques and backtracking fallback
pub struct Solver {
    #[allow(dead_code)]
    config: SolverConfig,
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver {
    /// Create a new solver with default configuration
    pub fn new() -> Self {
        Self {
            config: SolverConfig::default(),
        }
    }

    /// Create a solver with custom configuration
    pub fn with_config(config: SolverConfig) -> Self {
        Self { config }
    }

    /// Solve the puzzle, returning the solved grid if successful
    pub fn solve(&self, grid: &Grid) -> Option<Grid> {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();

        if self.solve_recursive(&mut working) {
            Some(working)
        } else {
            None
        }
    }

    /// Count solutions up to a limit
    pub fn count_solutions(&self, grid: &Grid, limit: usize) -> usize {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();
        let mut count = 0;
        self.count_solutions_recursive(&mut working, &mut count, limit);
        count
    }

    /// Check if the puzzle has exactly one solution
    pub fn has_unique_solution(&self, grid: &Grid) -> bool {
        self.count_solutions(grid, 2) == 1
    }

    /// Get a hint for the current position
    pub fn get_hint(&self, grid: &Grid) -> Option<Hint> {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();

        // Try techniques in order of difficulty
        if let Some(hint) = self.find_naked_single(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_hidden_single(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_naked_pair(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_hidden_pair(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_naked_triple(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_pointing_pair(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_box_line_reduction(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_x_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_swordfish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_xy_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_simple_coloring(&working) {
            return Some(hint);
        }

        // If no human technique found, give backtracking hint
        if let Some(solution) = self.solve(&working) {
            for pos in working.empty_positions() {
                if let Some(value) = solution.get(pos) {
                    return Some(Hint {
                        technique: Technique::Backtracking,
                        hint_type: HintType::SetValue { pos, value },
                        explanation: format!(
                            "The cell at ({}, {}) must be {}.",
                            pos.row + 1,
                            pos.col + 1,
                            value
                        ),
                        involved_cells: vec![pos],
                    });
                }
            }
        }

        None
    }

    /// Rate the difficulty of a puzzle
    pub fn rate_difficulty(&self, grid: &Grid) -> Difficulty {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();

        let mut max_technique = Technique::NakedSingle;
        let empty_count = grid.empty_positions().len();

        while !working.is_complete() {
            // Try techniques in order
            if self.apply_naked_singles(&mut working) {
                continue;
            }

            if self.apply_hidden_singles(&mut working) {
                if max_technique < Technique::HiddenSingle {
                    max_technique = Technique::HiddenSingle;
                }
                continue;
            }

            if self.apply_naked_pairs(&mut working) {
                if max_technique < Technique::NakedPair {
                    max_technique = Technique::NakedPair;
                }
                continue;
            }

            if self.apply_hidden_pairs(&mut working) {
                if max_technique < Technique::HiddenPair {
                    max_technique = Technique::HiddenPair;
                }
                continue;
            }

            if self.apply_naked_triples(&mut working) {
                if max_technique < Technique::NakedTriple {
                    max_technique = Technique::NakedTriple;
                }
                continue;
            }

            if self.apply_hidden_triples(&mut working) {
                if max_technique < Technique::HiddenTriple {
                    max_technique = Technique::HiddenTriple;
                }
                continue;
            }

            if self.apply_pointing_pairs(&mut working) {
                if max_technique < Technique::PointingPair {
                    max_technique = Technique::PointingPair;
                }
                continue;
            }

            if self.apply_box_line_reduction(&mut working) {
                if max_technique < Technique::BoxLineReduction {
                    max_technique = Technique::BoxLineReduction;
                }
                continue;
            }

            if self.apply_x_wing(&mut working) {
                if max_technique < Technique::XWing {
                    max_technique = Technique::XWing;
                }
                continue;
            }

            if self.apply_swordfish(&mut working) {
                if max_technique < Technique::Swordfish {
                    max_technique = Technique::Swordfish;
                }
                continue;
            }

            if self.apply_jellyfish(&mut working) {
                if max_technique < Technique::Jellyfish {
                    max_technique = Technique::Jellyfish;
                }
                continue;
            }

            if self.apply_xy_wing(&mut working) {
                if max_technique < Technique::XYWing {
                    max_technique = Technique::XYWing;
                }
                continue;
            }

            if self.apply_xyz_wing(&mut working) {
                if max_technique < Technique::XYZWing {
                    max_technique = Technique::XYZWing;
                }
                continue;
            }

            if self.apply_w_wing(&mut working) {
                if max_technique < Technique::WWing {
                    max_technique = Technique::WWing;
                }
                continue;
            }

            if self.apply_simple_coloring(&mut working) {
                if max_technique < Technique::SimpleColoring {
                    max_technique = Technique::SimpleColoring;
                }
                continue;
            }

            if self.apply_unique_rectangle(&mut working) {
                if max_technique < Technique::UniqueRectangle {
                    max_technique = Technique::UniqueRectangle;
                }
                continue;
            }

            // Need backtracking - extreme difficulty
            return Difficulty::Extreme;
        }

        // Determine difficulty based on technique and puzzle characteristics
        match max_technique {
            Technique::NakedSingle => {
                if empty_count <= 35 {
                    Difficulty::Beginner
                } else {
                    Difficulty::Easy
                }
            }
            Technique::HiddenSingle => Difficulty::Medium,
            Technique::NakedPair | Technique::HiddenPair |
            Technique::NakedTriple | Technique::HiddenTriple => Difficulty::Intermediate,
            Technique::PointingPair | Technique::BoxLineReduction => Difficulty::Hard,
            Technique::XWing | Technique::Swordfish | Technique::Jellyfish => Difficulty::Expert,
            Technique::XYWing | Technique::XYZWing | Technique::WWing |
            Technique::SimpleColoring | Technique::XChain => Difficulty::Master,
            _ => Difficulty::Extreme,
        }
    }

    // ==================== Helper Functions ====================

    /// Get all positions in a row
    fn row_positions(row: usize) -> Vec<Position> {
        (0..9).map(|col| Position::new(row, col)).collect()
    }

    /// Get all positions in a column
    fn col_positions(col: usize) -> Vec<Position> {
        (0..9).map(|row| Position::new(row, col)).collect()
    }

    /// Get all positions in a box
    fn box_positions(box_idx: usize) -> Vec<Position> {
        let box_row = (box_idx / 3) * 3;
        let box_col = (box_idx % 3) * 3;
        let mut positions = Vec::with_capacity(9);
        for dr in 0..3 {
            for dc in 0..3 {
                positions.push(Position::new(box_row + dr, box_col + dc));
            }
        }
        positions
    }

    /// Get empty cells from a list of positions
    fn empty_cells(&self, grid: &Grid, positions: &[Position]) -> Vec<Position> {
        positions
            .iter()
            .filter(|&&pos| grid.cell(pos).is_empty())
            .copied()
            .collect()
    }

    /// Check if two positions see each other (same row, col, or box)
    fn sees(&self, p1: Position, p2: Position) -> bool {
        p1.row == p2.row || p1.col == p2.col || p1.box_index() == p2.box_index()
    }

    // ==================== Naked Single ====================

    fn find_naked_single(&self, grid: &Grid) -> Option<Hint> {
        for pos in grid.empty_positions() {
            let candidates = grid.get_candidates(pos);
            if let Some(value) = candidates.single_value() {
                return Some(Hint {
                    technique: Technique::NakedSingle,
                    hint_type: HintType::SetValue { pos, value },
                    explanation: format!(
                        "Cell ({}, {}) can only be {} - it's the only candidate left.",
                        pos.row + 1, pos.col + 1, value
                    ),
                    involved_cells: vec![pos],
                });
            }
        }
        None
    }

    fn apply_naked_singles(&self, grid: &mut Grid) -> bool {
        let mut applied = false;
        loop {
            let mut found = false;
            for pos in grid.empty_positions() {
                let candidates = grid.get_candidates(pos);
                if let Some(value) = candidates.single_value() {
                    grid.set_cell_unchecked(pos, Some(value));
                    grid.recalculate_candidates();
                    found = true;
                    applied = true;
                    break;
                }
            }
            if !found {
                break;
            }
        }
        applied
    }

    // ==================== Hidden Single ====================

    fn find_hidden_single(&self, grid: &Grid) -> Option<Hint> {
        // Check rows
        for row in 0..9 {
            for value in 1..=9u8 {
                let mut possible_cols = Vec::new();
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                        possible_cols.push(col);
                    }
                }
                if possible_cols.len() == 1 {
                    let pos = Position::new(row, possible_cols[0]);
                    return Some(Hint {
                        technique: Technique::HiddenSingle,
                        hint_type: HintType::SetValue { pos, value },
                        explanation: format!(
                            "{} can only go in cell ({}, {}) in row {}.",
                            value, pos.row + 1, pos.col + 1, row + 1
                        ),
                        involved_cells: vec![pos],
                    });
                }
            }
        }

        // Check columns
        for col in 0..9 {
            for value in 1..=9u8 {
                let mut possible_rows = Vec::new();
                for row in 0..9 {
                    let pos = Position::new(row, col);
                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                        possible_rows.push(row);
                    }
                }
                if possible_rows.len() == 1 {
                    let pos = Position::new(possible_rows[0], col);
                    return Some(Hint {
                        technique: Technique::HiddenSingle,
                        hint_type: HintType::SetValue { pos, value },
                        explanation: format!(
                            "{} can only go in cell ({}, {}) in column {}.",
                            value, pos.row + 1, pos.col + 1, col + 1
                        ),
                        involved_cells: vec![pos],
                    });
                }
            }
        }

        // Check boxes
        for box_idx in 0..9 {
            let box_positions = Self::box_positions(box_idx);
            for value in 1..=9u8 {
                let mut possible_positions = Vec::new();
                for &pos in &box_positions {
                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                        possible_positions.push(pos);
                    }
                }
                if possible_positions.len() == 1 {
                    let pos = possible_positions[0];
                    return Some(Hint {
                        technique: Technique::HiddenSingle,
                        hint_type: HintType::SetValue { pos, value },
                        explanation: format!(
                            "{} can only go in cell ({}, {}) in box {}.",
                            value, pos.row + 1, pos.col + 1, box_idx + 1
                        ),
                        involved_cells: vec![pos],
                    });
                }
            }
        }

        None
    }

    fn apply_hidden_singles(&self, grid: &mut Grid) -> bool {
        let mut applied = false;
        loop {
            if let Some(hint) = self.find_hidden_single(grid) {
                if let HintType::SetValue { pos, value } = hint.hint_type {
                    grid.set_cell_unchecked(pos, Some(value));
                    grid.recalculate_candidates();
                    applied = true;
                    continue;
                }
            }
            break;
        }
        applied
    }

    // ==================== Naked Pair ====================

    fn find_naked_pair(&self, grid: &Grid) -> Option<Hint> {
        // Check all units (rows, columns, boxes)
        for unit_type in 0..3 {
            for unit_idx in 0..9 {
                let positions = match unit_type {
                    0 => Self::row_positions(unit_idx),
                    1 => Self::col_positions(unit_idx),
                    _ => Self::box_positions(unit_idx),
                };
                let unit_name = match unit_type {
                    0 => format!("row {}", unit_idx + 1),
                    1 => format!("column {}", unit_idx + 1),
                    _ => format!("box {}", unit_idx + 1),
                };

                let empty_cells = self.empty_cells(grid, &positions);

                for i in 0..empty_cells.len() {
                    for j in (i + 1)..empty_cells.len() {
                        let pos1 = empty_cells[i];
                        let pos2 = empty_cells[j];
                        let cand1 = grid.get_candidates(pos1);
                        let cand2 = grid.get_candidates(pos2);

                        if cand1.count() == 2 && cand1 == cand2 {
                            let pair_values: Vec<u8> = cand1.iter().collect();

                            // Check if it eliminates anything
                            for &other_pos in &empty_cells {
                                if other_pos != pos1 && other_pos != pos2 {
                                    let other_cand = grid.get_candidates(other_pos);
                                    let to_remove: Vec<u8> = pair_values
                                        .iter()
                                        .filter(|&&v| other_cand.contains(v))
                                        .copied()
                                        .collect();
                                    if !to_remove.is_empty() {
                                        return Some(Hint {
                                            technique: Technique::NakedPair,
                                            hint_type: HintType::EliminateCandidates {
                                                pos: other_pos,
                                                values: to_remove,
                                            },
                                            explanation: format!(
                                                "Cells ({}, {}) and ({}, {}) form a naked pair with {:?} in {}.",
                                                pos1.row + 1, pos1.col + 1,
                                                pos2.row + 1, pos2.col + 1,
                                                pair_values, unit_name
                                            ),
                                            involved_cells: vec![pos1, pos2, other_pos],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_naked_pairs(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_naked_pair(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Hidden Pair ====================

    fn find_hidden_pair(&self, grid: &Grid) -> Option<Hint> {
        for unit_type in 0..3 {
            for unit_idx in 0..9 {
                let positions = match unit_type {
                    0 => Self::row_positions(unit_idx),
                    1 => Self::col_positions(unit_idx),
                    _ => Self::box_positions(unit_idx),
                };
                let unit_name = match unit_type {
                    0 => format!("row {}", unit_idx + 1),
                    1 => format!("column {}", unit_idx + 1),
                    _ => format!("box {}", unit_idx + 1),
                };

                let empty_cells = self.empty_cells(grid, &positions);

                // Find values that appear in exactly 2 cells
                for v1 in 1..=8u8 {
                    for v2 in (v1 + 1)..=9u8 {
                        let cells_with_v1: Vec<Position> = empty_cells
                            .iter()
                            .filter(|&&pos| grid.get_candidates(pos).contains(v1))
                            .copied()
                            .collect();
                        let cells_with_v2: Vec<Position> = empty_cells
                            .iter()
                            .filter(|&&pos| grid.get_candidates(pos).contains(v2))
                            .copied()
                            .collect();

                        if cells_with_v1.len() == 2 && cells_with_v1 == cells_with_v2 {
                            let pos1 = cells_with_v1[0];
                            let pos2 = cells_with_v1[1];

                            // Check if either cell has more than just these two candidates
                            let cand1 = grid.get_candidates(pos1);
                            let cand2 = grid.get_candidates(pos2);

                            let to_remove1: Vec<u8> = cand1.iter().filter(|&v| v != v1 && v != v2).collect();
                            let to_remove2: Vec<u8> = cand2.iter().filter(|&v| v != v1 && v != v2).collect();

                            if !to_remove1.is_empty() {
                                return Some(Hint {
                                    technique: Technique::HiddenPair,
                                    hint_type: HintType::EliminateCandidates {
                                        pos: pos1,
                                        values: to_remove1,
                                    },
                                    explanation: format!(
                                        "Hidden pair {{{}, {}}} in {} at ({}, {}) and ({}, {}).",
                                        v1, v2, unit_name,
                                        pos1.row + 1, pos1.col + 1,
                                        pos2.row + 1, pos2.col + 1
                                    ),
                                    involved_cells: vec![pos1, pos2],
                                });
                            }
                            if !to_remove2.is_empty() {
                                return Some(Hint {
                                    technique: Technique::HiddenPair,
                                    hint_type: HintType::EliminateCandidates {
                                        pos: pos2,
                                        values: to_remove2,
                                    },
                                    explanation: format!(
                                        "Hidden pair {{{}, {}}} in {} at ({}, {}) and ({}, {}).",
                                        v1, v2, unit_name,
                                        pos1.row + 1, pos1.col + 1,
                                        pos2.row + 1, pos2.col + 1
                                    ),
                                    involved_cells: vec![pos1, pos2],
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_hidden_pairs(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_hidden_pair(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Naked Triple ====================

    fn find_naked_triple(&self, grid: &Grid) -> Option<Hint> {
        for unit_type in 0..3 {
            for unit_idx in 0..9 {
                let positions = match unit_type {
                    0 => Self::row_positions(unit_idx),
                    1 => Self::col_positions(unit_idx),
                    _ => Self::box_positions(unit_idx),
                };
                let unit_name = match unit_type {
                    0 => format!("row {}", unit_idx + 1),
                    1 => format!("column {}", unit_idx + 1),
                    _ => format!("box {}", unit_idx + 1),
                };

                let empty_cells = self.empty_cells(grid, &positions);
                if empty_cells.len() < 4 {
                    continue;
                }

                // Find three cells whose combined candidates are exactly 3 values
                for i in 0..empty_cells.len() {
                    for j in (i + 1)..empty_cells.len() {
                        for k in (j + 1)..empty_cells.len() {
                            let pos1 = empty_cells[i];
                            let pos2 = empty_cells[j];
                            let pos3 = empty_cells[k];

                            let cand1 = grid.get_candidates(pos1);
                            let cand2 = grid.get_candidates(pos2);
                            let cand3 = grid.get_candidates(pos3);

                            let combined = cand1.union(&cand2).union(&cand3);

                            if combined.count() == 3 &&
                               cand1.count() <= 3 && cand2.count() <= 3 && cand3.count() <= 3 {
                                let triple_values: Vec<u8> = combined.iter().collect();

                                // Check if it eliminates anything
                                for &other_pos in &empty_cells {
                                    if other_pos != pos1 && other_pos != pos2 && other_pos != pos3 {
                                        let other_cand = grid.get_candidates(other_pos);
                                        let to_remove: Vec<u8> = triple_values
                                            .iter()
                                            .filter(|&&v| other_cand.contains(v))
                                            .copied()
                                            .collect();
                                        if !to_remove.is_empty() {
                                            return Some(Hint {
                                                technique: Technique::NakedTriple,
                                                hint_type: HintType::EliminateCandidates {
                                                    pos: other_pos,
                                                    values: to_remove,
                                                },
                                                explanation: format!(
                                                    "Naked triple {:?} in {} at ({}, {}), ({}, {}), ({}, {}).",
                                                    triple_values, unit_name,
                                                    pos1.row + 1, pos1.col + 1,
                                                    pos2.row + 1, pos2.col + 1,
                                                    pos3.row + 1, pos3.col + 1
                                                ),
                                                involved_cells: vec![pos1, pos2, pos3, other_pos],
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_naked_triples(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_naked_triple(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Hidden Triple ====================

    fn apply_hidden_triples(&self, grid: &mut Grid) -> bool {
        for unit_type in 0..3 {
            for unit_idx in 0..9 {
                let positions = match unit_type {
                    0 => Self::row_positions(unit_idx),
                    1 => Self::col_positions(unit_idx),
                    _ => Self::box_positions(unit_idx),
                };

                let empty_cells = self.empty_cells(grid, &positions);
                if empty_cells.len() < 4 {
                    continue;
                }

                // Find three values that appear in exactly 3 cells
                for v1 in 1..=7u8 {
                    for v2 in (v1 + 1)..=8u8 {
                        for v3 in (v2 + 1)..=9u8 {
                            let mut cells_with_values: Vec<Position> = Vec::new();

                            for &pos in &empty_cells {
                                let cand = grid.get_candidates(pos);
                                if cand.contains(v1) || cand.contains(v2) || cand.contains(v3) {
                                    cells_with_values.push(pos);
                                }
                            }

                            if cells_with_values.len() == 3 {
                                // Found a hidden triple
                                let mut eliminated = false;
                                for &pos in &cells_with_values {
                                    let cand = grid.get_candidates(pos);
                                    for v in cand.iter() {
                                        if v != v1 && v != v2 && v != v3 {
                                            grid.cell_mut(pos).remove_candidate(v);
                                            eliminated = true;
                                        }
                                    }
                                }
                                if eliminated {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== Pointing Pair ====================

    fn find_pointing_pair(&self, grid: &Grid) -> Option<Hint> {
        for box_idx in 0..9 {
            let box_positions = Self::box_positions(box_idx);

            for value in 1..=9u8 {
                let cells_with_value: Vec<Position> = box_positions
                    .iter()
                    .filter(|&&pos| grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value))
                    .copied()
                    .collect();

                if cells_with_value.len() >= 2 && cells_with_value.len() <= 3 {
                    // Check if all in same row
                    let first_row = cells_with_value[0].row;
                    if cells_with_value.iter().all(|p| p.row == first_row) {
                        // Eliminate from rest of row
                        for col in 0..9 {
                            let pos = Position::new(first_row, col);
                            if pos.box_index() != box_idx
                                && grid.cell(pos).is_empty()
                                && grid.get_candidates(pos).contains(value)
                            {
                                return Some(Hint {
                                    technique: Technique::PointingPair,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![value],
                                    },
                                    explanation: format!(
                                        "In box {}, {} can only be in row {}. Remove from other cells in that row.",
                                        box_idx + 1, value, first_row + 1
                                    ),
                                    involved_cells: cells_with_value.clone(),
                                });
                            }
                        }
                    }

                    // Check if all in same column
                    let first_col = cells_with_value[0].col;
                    if cells_with_value.iter().all(|p| p.col == first_col) {
                        for row in 0..9 {
                            let pos = Position::new(row, first_col);
                            if pos.box_index() != box_idx
                                && grid.cell(pos).is_empty()
                                && grid.get_candidates(pos).contains(value)
                            {
                                return Some(Hint {
                                    technique: Technique::PointingPair,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![value],
                                    },
                                    explanation: format!(
                                        "In box {}, {} can only be in column {}. Remove from other cells in that column.",
                                        box_idx + 1, value, first_col + 1
                                    ),
                                    involved_cells: cells_with_value.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_pointing_pairs(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_pointing_pair(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Box/Line Reduction ====================

    fn find_box_line_reduction(&self, grid: &Grid) -> Option<Hint> {
        // Check rows
        for row in 0..9 {
            for value in 1..=9u8 {
                let mut cols_with_value = Vec::new();
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                        cols_with_value.push(col);
                    }
                }

                if cols_with_value.len() >= 2 && cols_with_value.len() <= 3 {
                    // Check if all in same box
                    let first_box = Position::new(row, cols_with_value[0]).box_index();
                    if cols_with_value.iter().all(|&col| Position::new(row, col).box_index() == first_box) {
                        // Eliminate from rest of box
                        let box_positions = Self::box_positions(first_box);
                        for &pos in &box_positions {
                            if pos.row != row
                                && grid.cell(pos).is_empty()
                                && grid.get_candidates(pos).contains(value)
                            {
                                return Some(Hint {
                                    technique: Technique::BoxLineReduction,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![value],
                                    },
                                    explanation: format!(
                                        "In row {}, {} is confined to box {}. Remove from other cells in that box.",
                                        row + 1, value, first_box + 1
                                    ),
                                    involved_cells: cols_with_value.iter().map(|&c| Position::new(row, c)).collect(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check columns
        for col in 0..9 {
            for value in 1..=9u8 {
                let mut rows_with_value = Vec::new();
                for row in 0..9 {
                    let pos = Position::new(row, col);
                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                        rows_with_value.push(row);
                    }
                }

                if rows_with_value.len() >= 2 && rows_with_value.len() <= 3 {
                    let first_box = Position::new(rows_with_value[0], col).box_index();
                    if rows_with_value.iter().all(|&row| Position::new(row, col).box_index() == first_box) {
                        let box_positions = Self::box_positions(first_box);
                        for &pos in &box_positions {
                            if pos.col != col
                                && grid.cell(pos).is_empty()
                                && grid.get_candidates(pos).contains(value)
                            {
                                return Some(Hint {
                                    technique: Technique::BoxLineReduction,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![value],
                                    },
                                    explanation: format!(
                                        "In column {}, {} is confined to box {}. Remove from other cells in that box.",
                                        col + 1, value, first_box + 1
                                    ),
                                    involved_cells: rows_with_value.iter().map(|&r| Position::new(r, col)).collect(),
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn apply_box_line_reduction(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_box_line_reduction(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== X-Wing ====================

    fn find_x_wing(&self, grid: &Grid) -> Option<Hint> {
        // Row-based X-Wing
        for value in 1..=9u8 {
            let mut row_pairs: Vec<(usize, Vec<usize>)> = Vec::new();

            for row in 0..9 {
                let cols: Vec<usize> = (0..9)
                    .filter(|&col| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();

                if cols.len() == 2 {
                    row_pairs.push((row, cols));
                }
            }

            for i in 0..row_pairs.len() {
                for j in (i + 1)..row_pairs.len() {
                    if row_pairs[i].1 == row_pairs[j].1 {
                        let cols = &row_pairs[i].1;
                        let rows = [row_pairs[i].0, row_pairs[j].0];

                        // Check if we can eliminate anything
                        for &col in cols {
                            for row in 0..9 {
                                if !rows.contains(&row) {
                                    let pos = Position::new(row, col);
                                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                                        let involved: Vec<Position> = rows
                                            .iter()
                                            .flat_map(|&r| cols.iter().map(move |&c| Position::new(r, c)))
                                            .collect();
                                        return Some(Hint {
                                            technique: Technique::XWing,
                                            hint_type: HintType::EliminateCandidates {
                                                pos,
                                                values: vec![value],
                                            },
                                            explanation: format!(
                                                "X-Wing on {} in rows {} and {}, columns {} and {}.",
                                                value, rows[0] + 1, rows[1] + 1, cols[0] + 1, cols[1] + 1
                                            ),
                                            involved_cells: involved,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Column-based X-Wing
            let mut col_pairs: Vec<(usize, Vec<usize>)> = Vec::new();

            for col in 0..9 {
                let rows: Vec<usize> = (0..9)
                    .filter(|&row| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();

                if rows.len() == 2 {
                    col_pairs.push((col, rows));
                }
            }

            for i in 0..col_pairs.len() {
                for j in (i + 1)..col_pairs.len() {
                    if col_pairs[i].1 == col_pairs[j].1 {
                        let rows = &col_pairs[i].1;
                        let cols = [col_pairs[i].0, col_pairs[j].0];

                        for &row in rows {
                            for col in 0..9 {
                                if !cols.contains(&col) {
                                    let pos = Position::new(row, col);
                                    if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                                        let involved: Vec<Position> = rows
                                            .iter()
                                            .flat_map(|&r| cols.iter().map(move |&c| Position::new(r, c)))
                                            .collect();
                                        return Some(Hint {
                                            technique: Technique::XWing,
                                            hint_type: HintType::EliminateCandidates {
                                                pos,
                                                values: vec![value],
                                            },
                                            explanation: format!(
                                                "X-Wing on {} in columns {} and {}, rows {} and {}.",
                                                value, cols[0] + 1, cols[1] + 1, rows[0] + 1, rows[1] + 1
                                            ),
                                            involved_cells: involved,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_x_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_x_wing(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Swordfish (3x3 Fish) ====================

    fn find_swordfish(&self, grid: &Grid) -> Option<Hint> {
        for value in 1..=9u8 {
            // Row-based Swordfish
            let mut row_data: Vec<(usize, Vec<usize>)> = Vec::new();

            for row in 0..9 {
                let cols: Vec<usize> = (0..9)
                    .filter(|&col| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();

                if cols.len() >= 2 && cols.len() <= 3 {
                    row_data.push((row, cols));
                }
            }

            // Find 3 rows where the union of columns is exactly 3
            for i in 0..row_data.len() {
                for j in (i + 1)..row_data.len() {
                    for k in (j + 1)..row_data.len() {
                        let mut all_cols: Vec<usize> = Vec::new();
                        all_cols.extend(&row_data[i].1);
                        all_cols.extend(&row_data[j].1);
                        all_cols.extend(&row_data[k].1);
                        all_cols.sort();
                        all_cols.dedup();

                        if all_cols.len() == 3 {
                            let rows = [row_data[i].0, row_data[j].0, row_data[k].0];

                            // Eliminate from other rows in these columns
                            for &col in &all_cols {
                                for row in 0..9 {
                                    if !rows.contains(&row) {
                                        let pos = Position::new(row, col);
                                        if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                                            return Some(Hint {
                                                technique: Technique::Swordfish,
                                                hint_type: HintType::EliminateCandidates {
                                                    pos,
                                                    values: vec![value],
                                                },
                                                explanation: format!(
                                                    "Swordfish on {} in rows {:?}, columns {:?}.",
                                                    value,
                                                    rows.iter().map(|r| r + 1).collect::<Vec<_>>(),
                                                    all_cols.iter().map(|c| c + 1).collect::<Vec<_>>()
                                                ),
                                                involved_cells: vec![],
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Column-based Swordfish
            let mut col_data: Vec<(usize, Vec<usize>)> = Vec::new();

            for col in 0..9 {
                let rows: Vec<usize> = (0..9)
                    .filter(|&row| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();

                if rows.len() >= 2 && rows.len() <= 3 {
                    col_data.push((col, rows));
                }
            }

            for i in 0..col_data.len() {
                for j in (i + 1)..col_data.len() {
                    for k in (j + 1)..col_data.len() {
                        let mut all_rows: Vec<usize> = Vec::new();
                        all_rows.extend(&col_data[i].1);
                        all_rows.extend(&col_data[j].1);
                        all_rows.extend(&col_data[k].1);
                        all_rows.sort();
                        all_rows.dedup();

                        if all_rows.len() == 3 {
                            let cols = [col_data[i].0, col_data[j].0, col_data[k].0];

                            for &row in &all_rows {
                                for col in 0..9 {
                                    if !cols.contains(&col) {
                                        let pos = Position::new(row, col);
                                        if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                                            return Some(Hint {
                                                technique: Technique::Swordfish,
                                                hint_type: HintType::EliminateCandidates {
                                                    pos,
                                                    values: vec![value],
                                                },
                                                explanation: format!(
                                                    "Swordfish on {} in columns {:?}, rows {:?}.",
                                                    value,
                                                    cols.iter().map(|c| c + 1).collect::<Vec<_>>(),
                                                    all_rows.iter().map(|r| r + 1).collect::<Vec<_>>()
                                                ),
                                                involved_cells: vec![],
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_swordfish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_swordfish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Jellyfish (4x4 Fish) ====================

    fn apply_jellyfish(&self, grid: &mut Grid) -> bool {
        for value in 1..=9u8 {
            // Row-based Jellyfish
            let mut row_data: Vec<(usize, Vec<usize>)> = Vec::new();

            for row in 0..9 {
                let cols: Vec<usize> = (0..9)
                    .filter(|&col| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();

                if cols.len() >= 2 && cols.len() <= 4 {
                    row_data.push((row, cols));
                }
            }

            // Find 4 rows where the union of columns is exactly 4
            for i in 0..row_data.len() {
                for j in (i + 1)..row_data.len() {
                    for k in (j + 1)..row_data.len() {
                        for l in (k + 1)..row_data.len() {
                            let mut all_cols: Vec<usize> = Vec::new();
                            all_cols.extend(&row_data[i].1);
                            all_cols.extend(&row_data[j].1);
                            all_cols.extend(&row_data[k].1);
                            all_cols.extend(&row_data[l].1);
                            all_cols.sort();
                            all_cols.dedup();

                            if all_cols.len() == 4 {
                                let rows = [row_data[i].0, row_data[j].0, row_data[k].0, row_data[l].0];
                                let mut eliminated = false;

                                for &col in &all_cols {
                                    for row in 0..9 {
                                        if !rows.contains(&row) {
                                            let pos = Position::new(row, col);
                                            if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value) {
                                                grid.cell_mut(pos).remove_candidate(value);
                                                eliminated = true;
                                            }
                                        }
                                    }
                                }

                                if eliminated {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== XY-Wing ====================

    fn find_xy_wing(&self, grid: &Grid) -> Option<Hint> {
        // Find cells with exactly 2 candidates (bi-value cells)
        let bivalues: Vec<(Position, u8, u8)> = grid
            .empty_positions()
            .into_iter()
            .filter_map(|pos| {
                let cand = grid.get_candidates(pos);
                if cand.count() == 2 {
                    let values: Vec<u8> = cand.iter().collect();
                    Some((pos, values[0], values[1]))
                } else {
                    None
                }
            })
            .collect();

        // Find XY-Wing pattern: pivot with XY, wing1 with XZ, wing2 with YZ
        for &(pivot, x, y) in &bivalues {
            for &(wing1, a, b) in &bivalues {
                if wing1 == pivot || !self.sees(pivot, wing1) {
                    continue;
                }

                // wing1 must share exactly one value with pivot
                let (shared1, z1) = if a == x { (x, b) }
                    else if a == y { (y, b) }
                    else if b == x { (x, a) }
                    else if b == y { (y, a) }
                    else { continue };

                for &(wing2, c, d) in &bivalues {
                    if wing2 == pivot || wing2 == wing1 || !self.sees(pivot, wing2) {
                        continue;
                    }

                    // wing2 must share the other value with pivot and have z1
                    let other_pivot_val = if shared1 == x { y } else { x };

                    let has_other = c == other_pivot_val || d == other_pivot_val;
                    let has_z = c == z1 || d == z1;

                    if has_other && has_z {
                        // Found XY-Wing! z1 can be eliminated from cells that see both wings
                        for pos in grid.empty_positions() {
                            if pos != pivot && pos != wing1 && pos != wing2
                                && self.sees(pos, wing1)
                                && self.sees(pos, wing2)
                                && grid.get_candidates(pos).contains(z1)
                            {
                                return Some(Hint {
                                    technique: Technique::XYWing,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![z1],
                                    },
                                    explanation: format!(
                                        "XY-Wing: pivot ({}, {}) with {}{}, wings at ({}, {}) and ({}, {}). Remove {} from cells seeing both wings.",
                                        pivot.row + 1, pivot.col + 1, x, y,
                                        wing1.row + 1, wing1.col + 1,
                                        wing2.row + 1, wing2.col + 1,
                                        z1
                                    ),
                                    involved_cells: vec![pivot, wing1, wing2],
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_xy_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_xy_wing(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== XYZ-Wing ====================

    fn apply_xyz_wing(&self, grid: &mut Grid) -> bool {
        // Find pivot cells with exactly 3 candidates
        for pivot in grid.empty_positions() {
            let pivot_cand = grid.get_candidates(pivot);
            if pivot_cand.count() != 3 {
                continue;
            }

            let xyz: Vec<u8> = pivot_cand.iter().collect();

            // Find two wing cells that see the pivot, each with 2 candidates that are subsets of pivot
            let wings: Vec<Position> = grid
                .empty_positions()
                .into_iter()
                .filter(|&pos| {
                    if pos == pivot || !self.sees(pivot, pos) {
                        return false;
                    }
                    let cand = grid.get_candidates(pos);
                    if cand.count() != 2 {
                        return false;
                    }
                    // Check if wing candidates are subset of pivot
                    cand.iter().all(|v| pivot_cand.contains(v))
                })
                .collect();

            for i in 0..wings.len() {
                for j in (i + 1)..wings.len() {
                    let wing1 = wings[i];
                    let wing2 = wings[j];

                    let cand1 = grid.get_candidates(wing1);
                    let cand2 = grid.get_candidates(wing2);

                    // The union of wing candidates should be all 3 pivot values
                    let wing_union = cand1.union(&cand2);
                    if wing_union.count() != 3 {
                        continue;
                    }

                    // Find the common value (z) that appears in both wings
                    let common: Vec<u8> = xyz.iter()
                        .filter(|&&v| cand1.contains(v) && cand2.contains(v))
                        .copied()
                        .collect();

                    if common.len() != 1 {
                        continue;
                    }

                    let z = common[0];

                    // Eliminate z from cells that see all three (pivot and both wings)
                    for pos in grid.empty_positions() {
                        if pos != pivot && pos != wing1 && pos != wing2
                            && self.sees(pos, pivot)
                            && self.sees(pos, wing1)
                            && self.sees(pos, wing2)
                            && grid.get_candidates(pos).contains(z)
                        {
                            grid.cell_mut(pos).remove_candidate(z);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== W-Wing ====================

    fn apply_w_wing(&self, grid: &mut Grid) -> bool {
        // Find bi-value cells with same two candidates
        let bivalues: Vec<(Position, u8, u8)> = grid
            .empty_positions()
            .into_iter()
            .filter_map(|pos| {
                let cand = grid.get_candidates(pos);
                if cand.count() == 2 {
                    let values: Vec<u8> = cand.iter().collect();
                    Some((pos, values[0], values[1]))
                } else {
                    None
                }
            })
            .collect();

        for i in 0..bivalues.len() {
            for j in (i + 1)..bivalues.len() {
                let (pos1, a1, b1) = bivalues[i];
                let (pos2, a2, b2) = bivalues[j];

                // Must have same candidates
                if !((a1 == a2 && b1 == b2) || (a1 == b2 && b1 == a2)) {
                    continue;
                }

                let x = a1;
                let y = b1;

                // Check if there's a strong link on either x or y connecting them
                // Strong link: only 2 places for a value in a row/column

                // Check rows for strong link
                for row in 0..9 {
                    for value in [x, y] {
                        let positions_in_row: Vec<usize> = (0..9)
                            .filter(|&col| {
                                let pos = Position::new(row, col);
                                grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                            })
                            .collect();

                        if positions_in_row.len() == 2 {
                            let link1 = Position::new(row, positions_in_row[0]);
                            let link2 = Position::new(row, positions_in_row[1]);

                            // Check if pos1 sees link1 and pos2 sees link2 (or vice versa)
                            let other_value = if value == x { y } else { x };

                            if (self.sees(pos1, link1) && self.sees(pos2, link2)) ||
                               (self.sees(pos1, link2) && self.sees(pos2, link1)) {
                                // W-Wing found! Eliminate other_value from cells seeing both pos1 and pos2
                                for pos in grid.empty_positions() {
                                    if pos != pos1 && pos != pos2
                                        && self.sees(pos, pos1)
                                        && self.sees(pos, pos2)
                                        && grid.get_candidates(pos).contains(other_value)
                                    {
                                        grid.cell_mut(pos).remove_candidate(other_value);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }

                // Check columns for strong link
                for col in 0..9 {
                    for value in [x, y] {
                        let positions_in_col: Vec<usize> = (0..9)
                            .filter(|&row| {
                                let pos = Position::new(row, col);
                                grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                            })
                            .collect();

                        if positions_in_col.len() == 2 {
                            let link1 = Position::new(positions_in_col[0], col);
                            let link2 = Position::new(positions_in_col[1], col);

                            let other_value = if value == x { y } else { x };

                            if (self.sees(pos1, link1) && self.sees(pos2, link2)) ||
                               (self.sees(pos1, link2) && self.sees(pos2, link1)) {
                                for pos in grid.empty_positions() {
                                    if pos != pos1 && pos != pos2
                                        && self.sees(pos, pos1)
                                        && self.sees(pos, pos2)
                                        && grid.get_candidates(pos).contains(other_value)
                                    {
                                        grid.cell_mut(pos).remove_candidate(other_value);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== Simple Coloring ====================

    fn find_simple_coloring(&self, grid: &Grid) -> Option<Hint> {
        for value in 1..=9u8 {
            // Build conjugate pairs (strong links) for this value
            let mut links: Vec<(Position, Position)> = Vec::new();

            // Row conjugates
            for row in 0..9 {
                let positions: Vec<Position> = (0..9)
                    .map(|col| Position::new(row, col))
                    .filter(|&pos| grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value))
                    .collect();
                if positions.len() == 2 {
                    links.push((positions[0], positions[1]));
                }
            }

            // Column conjugates
            for col in 0..9 {
                let positions: Vec<Position> = (0..9)
                    .map(|row| Position::new(row, col))
                    .filter(|&pos| grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value))
                    .collect();
                if positions.len() == 2 {
                    links.push((positions[0], positions[1]));
                }
            }

            // Box conjugates
            for box_idx in 0..9 {
                let positions: Vec<Position> = Self::box_positions(box_idx)
                    .into_iter()
                    .filter(|&pos| grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value))
                    .collect();
                if positions.len() == 2 {
                    links.push((positions[0], positions[1]));
                }
            }

            if links.is_empty() {
                continue;
            }

            // Build connected chains using colors (true/false)
            let mut colors: std::collections::HashMap<Position, bool> = std::collections::HashMap::new();

            // Start from first link
            let mut to_process: Vec<(Position, bool)> = vec![(links[0].0, true), (links[0].1, false)];

            while let Some((pos, color)) = to_process.pop() {
                if colors.contains_key(&pos) {
                    continue;
                }
                colors.insert(pos, color);

                // Find connected positions through links
                for &(p1, p2) in &links {
                    if p1 == pos && !colors.contains_key(&p2) {
                        to_process.push((p2, !color));
                    } else if p2 == pos && !colors.contains_key(&p1) {
                        to_process.push((p1, !color));
                    }
                }
            }

            if colors.len() < 2 {
                continue;
            }

            // Rule 2: If two cells of the same color see each other, that color is false
            let true_cells: Vec<Position> = colors.iter().filter(|&(_, &c)| c).map(|(&p, _)| p).collect();
            let false_cells: Vec<Position> = colors.iter().filter(|&(_, &c)| !c).map(|(&p, _)| p).collect();

            // Check if same-colored cells see each other
            for i in 0..true_cells.len() {
                for j in (i + 1)..true_cells.len() {
                    if self.sees(true_cells[i], true_cells[j]) {
                        // True color is invalid, eliminate from all true cells
                        for &pos in &true_cells {
                            if grid.get_candidates(pos).contains(value) {
                                return Some(Hint {
                                    technique: Technique::SimpleColoring,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![value],
                                    },
                                    explanation: format!(
                                        "Simple Coloring: two cells of same color see each other, eliminating {} from that color group.",
                                        value
                                    ),
                                    involved_cells: true_cells.clone(),
                                });
                            }
                        }
                    }
                }
            }

            for i in 0..false_cells.len() {
                for j in (i + 1)..false_cells.len() {
                    if self.sees(false_cells[i], false_cells[j]) {
                        for &pos in &false_cells {
                            if grid.get_candidates(pos).contains(value) {
                                return Some(Hint {
                                    technique: Technique::SimpleColoring,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![value],
                                    },
                                    explanation: format!(
                                        "Simple Coloring: two cells of same color see each other, eliminating {} from that color group.",
                                        value
                                    ),
                                    involved_cells: false_cells.clone(),
                                });
                            }
                        }
                    }
                }
            }

            // Rule 4: Eliminate from cells that see both colors
            for pos in grid.empty_positions() {
                if colors.contains_key(&pos) || !grid.get_candidates(pos).contains(value) {
                    continue;
                }

                let sees_true = true_cells.iter().any(|&p| self.sees(pos, p));
                let sees_false = false_cells.iter().any(|&p| self.sees(pos, p));

                if sees_true && sees_false {
                    return Some(Hint {
                        technique: Technique::SimpleColoring,
                        hint_type: HintType::EliminateCandidates {
                            pos,
                            values: vec![value],
                        },
                        explanation: format!(
                            "Simple Coloring: cell ({}, {}) sees both colors, so {} can be eliminated.",
                            pos.row + 1, pos.col + 1, value
                        ),
                        involved_cells: colors.keys().copied().collect(),
                    });
                }
            }
        }
        None
    }

    fn apply_simple_coloring(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_simple_coloring(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Unique Rectangle ====================

    fn apply_unique_rectangle(&self, grid: &mut Grid) -> bool {
        // Find bi-value cells
        let bivalues: Vec<(Position, u8, u8)> = grid
            .empty_positions()
            .into_iter()
            .filter_map(|pos| {
                let cand = grid.get_candidates(pos);
                if cand.count() == 2 {
                    let values: Vec<u8> = cand.iter().collect();
                    Some((pos, values[0], values[1]))
                } else {
                    None
                }
            })
            .collect();

        // Look for rectangles in two rows and two columns
        for i in 0..bivalues.len() {
            let (pos1, a, b) = bivalues[i];

            for j in (i + 1)..bivalues.len() {
                let (pos2, c, d) = bivalues[j];

                // Must have same candidates
                if !((a == c && b == d) || (a == d && b == c)) {
                    continue;
                }

                // Must be in same row but different columns, or same column but different rows
                if pos1.row != pos2.row && pos1.col != pos2.col {
                    continue;
                }

                // Find potential corners
                let mut corner3: Position;
                let mut corner4: Position;

                if pos1.row == pos2.row {
                    // Same row - look for another row
                    for other_row in 0..9 {
                        if other_row == pos1.row {
                            continue;
                        }

                        corner3 = Position::new(other_row, pos1.col);
                        corner4 = Position::new(other_row, pos2.col);

                        // Check if corners 3 and 4 are in same box (required for deadly pattern)
                        if corner3.box_index() != corner4.box_index() &&
                           pos1.box_index() != pos2.box_index() {
                            continue;
                        }

                        let cand3 = grid.get_candidates(corner3);
                        let cand4 = grid.get_candidates(corner4);

                        // Type 1: One corner has extra candidates
                        if cand3.count() == 2 && cand3.contains(a) && cand3.contains(b) &&
                           cand4.count() > 2 && cand4.contains(a) && cand4.contains(b) {
                            // corner4 must have at least one of a,b removed to break the pattern
                            // Actually Type 1 means corner4 can't be just {a,b}
                            // We eliminate a and b from corner4 if it would create deadly pattern
                            // This needs more careful implementation
                        }

                        // Type 2: Two corners have same extra candidate
                        if cand3.count() == 3 && cand4.count() == 3 {
                            let extra3: Vec<u8> = cand3.iter().filter(|&v| v != a && v != b).collect();
                            let extra4: Vec<u8> = cand4.iter().filter(|&v| v != a && v != b).collect();

                            if extra3.len() == 1 && extra4.len() == 1 && extra3[0] == extra4[0] {
                                let extra = extra3[0];

                                // Eliminate extra from cells that see both corner3 and corner4
                                for pos in grid.empty_positions() {
                                    if pos != corner3 && pos != corner4
                                        && self.sees(pos, corner3)
                                        && self.sees(pos, corner4)
                                        && grid.get_candidates(pos).contains(extra)
                                    {
                                        grid.cell_mut(pos).remove_candidate(extra);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Same column - look for another column
                    for other_col in 0..9 {
                        if other_col == pos1.col {
                            continue;
                        }

                        corner3 = Position::new(pos1.row, other_col);
                        corner4 = Position::new(pos2.row, other_col);

                        if corner3.box_index() != corner4.box_index() &&
                           pos1.box_index() != pos2.box_index() {
                            continue;
                        }

                        let cand3 = grid.get_candidates(corner3);
                        let cand4 = grid.get_candidates(corner4);

                        if cand3.count() == 3 && cand4.count() == 3 {
                            let extra3: Vec<u8> = cand3.iter().filter(|&v| v != a && v != b).collect();
                            let extra4: Vec<u8> = cand4.iter().filter(|&v| v != a && v != b).collect();

                            if extra3.len() == 1 && extra4.len() == 1 && extra3[0] == extra4[0] {
                                let extra = extra3[0];

                                for pos in grid.empty_positions() {
                                    if pos != corner3 && pos != corner4
                                        && self.sees(pos, corner3)
                                        && self.sees(pos, corner4)
                                        && grid.get_candidates(pos).contains(extra)
                                    {
                                        grid.cell_mut(pos).remove_candidate(extra);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== Backtracking Solver ====================

    fn solve_recursive(&self, grid: &mut Grid) -> bool {
        // First apply human techniques
        self.apply_naked_singles(grid);
        self.apply_hidden_singles(grid);

        if grid.is_complete() {
            return true;
        }

        // Find cell with minimum remaining values (MRV heuristic)
        let empty_positions = grid.empty_positions();
        if empty_positions.is_empty() {
            return false;
        }

        let best_pos = empty_positions
            .into_iter()
            .min_by_key(|&pos| grid.get_candidates(pos).count())
            .unwrap();

        let candidates = grid.get_candidates(best_pos);

        if candidates.is_empty() {
            return false;
        }

        for value in candidates.iter() {
            let mut test_grid = grid.deep_clone();
            test_grid.set_cell_unchecked(best_pos, Some(value));
            test_grid.recalculate_candidates();

            if test_grid.validate().is_valid && self.solve_recursive(&mut test_grid) {
                for row in 0..9 {
                    for col in 0..9 {
                        let pos = Position::new(row, col);
                        grid.set_cell_unchecked(pos, test_grid.get(pos));
                    }
                }
                return true;
            }
        }

        false
    }

    fn count_solutions_recursive(&self, grid: &mut Grid, count: &mut usize, limit: usize) {
        if *count >= limit {
            return;
        }

        self.apply_naked_singles(grid);
        self.apply_hidden_singles(grid);

        if grid.is_complete() {
            *count += 1;
            return;
        }

        let empty_positions = grid.empty_positions();
        if empty_positions.is_empty() {
            return;
        }

        let best_pos = empty_positions
            .into_iter()
            .min_by_key(|&pos| grid.get_candidates(pos).count())
            .unwrap();

        let candidates = grid.get_candidates(best_pos);

        if candidates.is_empty() {
            return;
        }

        for value in candidates.iter() {
            if *count >= limit {
                return;
            }

            let mut test_grid = grid.deep_clone();
            test_grid.set_cell_unchecked(best_pos, Some(value));
            test_grid.recalculate_candidates();

            if test_grid.validate().is_valid {
                self.count_solutions_recursive(&mut test_grid, count, limit);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_easy() {
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        let solution = solver.solve(&grid).unwrap();

        assert!(solution.is_complete());
    }

    #[test]
    fn test_unique_solution() {
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        assert!(solver.has_unique_solution(&grid));
    }

    #[test]
    fn test_multiple_solutions() {
        let puzzle =
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        assert!(!solver.has_unique_solution(&grid));
    }

    #[test]
    fn test_get_hint() {
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        let hint = solver.get_hint(&grid);

        assert!(hint.is_some());
    }

    #[test]
    fn test_difficulty_rating() {
        let easy =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(easy).unwrap();

        let solver = Solver::new();
        let difficulty = solver.rate_difficulty(&grid);

        assert!(difficulty >= Difficulty::Easy);
    }
}
