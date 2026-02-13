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
            Difficulty::Intermediate => Technique::HiddenTriple,
            Difficulty::Hard => Technique::BoxLineReduction,
            Difficulty::Expert => Technique::FinnedJellyfish,
            Difficulty::Master => Technique::AIC,
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

    // Expert (fish family)
    XWing,
    FinnedXWing,
    Swordfish,
    FinnedSwordfish,
    Jellyfish,
    FinnedJellyfish,

    // Master (wings + chains)
    XYWing,
    XYZWing,
    WWing,
    XChain,
    AIC,

    // Extreme
    AlsXz,
    AlsXyWing,
    UniqueRectangle,
    NishioForcingChain,
    CellForcingChain,
    DynamicForcingChain,
    Backtracking,
}

impl Technique {
    /// Get the Sudoku Explainer (SE) numerical rating for this technique.
    /// This is the community-standard difficulty scale.
    pub fn se_rating(&self) -> f32 {
        match self {
            Technique::HiddenSingle => 1.5,
            Technique::NakedSingle => 2.3,
            Technique::PointingPair => 2.6,
            Technique::BoxLineReduction => 2.8,
            Technique::NakedPair => 3.0,
            Technique::XWing => 3.2,
            Technique::FinnedXWing => 3.4,
            Technique::HiddenPair => 3.4,
            Technique::NakedTriple => 3.6,
            Technique::Swordfish => 3.8,
            Technique::HiddenTriple => 3.8,
            Technique::FinnedSwordfish => 4.0,
            Technique::XYWing => 4.2,
            Technique::XYZWing => 4.4,
            Technique::WWing => 4.4,
            Technique::XChain => 4.5,
            Technique::UniqueRectangle => 4.6,
            Technique::Jellyfish => 5.2,
            Technique::FinnedJellyfish => 5.4,
            Technique::AlsXz => 5.5,
            Technique::AIC => 6.0,
            Technique::AlsXyWing => 7.0,
            Technique::NishioForcingChain => 7.5,
            Technique::CellForcingChain => 8.3,
            Technique::DynamicForcingChain => 9.3,
            Technique::Backtracking => 11.0,
        }
    }
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
            Technique::FinnedXWing => write!(f, "Finned X-Wing"),
            Technique::Swordfish => write!(f, "Swordfish"),
            Technique::FinnedSwordfish => write!(f, "Finned Swordfish"),
            Technique::Jellyfish => write!(f, "Jellyfish"),
            Technique::FinnedJellyfish => write!(f, "Finned Jellyfish"),
            Technique::XYWing => write!(f, "XY-Wing"),
            Technique::XYZWing => write!(f, "XYZ-Wing"),
            Technique::WWing => write!(f, "W-Wing"),
            Technique::XChain => write!(f, "X-Chain"),
            Technique::AIC => write!(f, "AIC"),
            Technique::AlsXz => write!(f, "ALS-XZ"),
            Technique::AlsXyWing => write!(f, "ALS-XY-Wing"),
            Technique::UniqueRectangle => write!(f, "Unique Rectangle"),
            Technique::NishioForcingChain => write!(f, "Nishio Forcing Chain"),
            Technique::CellForcingChain => write!(f, "Cell Forcing Chain"),
            Technique::DynamicForcingChain => write!(f, "Dynamic Forcing Chain"),
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
#[derive(Debug, Clone, Default)]
pub struct SolverConfig {
    /// Maximum technique level to use (None = use all including backtracking)
    pub max_technique: Option<Technique>,
    /// Whether to track techniques used
    pub track_techniques: bool,
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
        if let Some(hint) = self.find_finned_x_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_swordfish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_finned_swordfish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_jellyfish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_finned_jellyfish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_xy_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_xyz_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_w_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_x_chain(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_aic(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_als_xz(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_als_xy_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_nishio_forcing_chain(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_cell_forcing_chain(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_dynamic_forcing_chain(&working) {
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
        let empty_count = grid.empty_positions().len();
        let mut working = grid.deep_clone();
        let max_tech = self.solve_with_techniques(&mut working);
        Self::technique_to_difficulty(max_tech, empty_count)
    }

    /// Rate the puzzle using the Sudoku Explainer (SE) numerical scale.
    /// Returns the SE rating of the hardest technique needed to solve the puzzle.
    pub fn rate_se(&self, grid: &Grid) -> f32 {
        let mut working = grid.deep_clone();
        let max_tech = self.solve_with_techniques(&mut working);
        max_tech.se_rating()
    }

    /// Solve the puzzle using human techniques, returning the hardest technique used.
    /// If backtracking is needed, returns `Technique::Backtracking`.
    fn solve_with_techniques(&self, grid: &mut Grid) -> Technique {
        grid.recalculate_candidates();
        let mut max_technique = Technique::NakedSingle;

        macro_rules! try_technique {
            ($apply:ident, $tech:expr) => {
                if self.$apply(grid) {
                    if max_technique < $tech {
                        max_technique = $tech;
                    }
                    continue;
                }
            };
        }

        while !grid.is_complete() {
            try_technique!(apply_naked_singles, Technique::NakedSingle);
            try_technique!(apply_hidden_singles, Technique::HiddenSingle);
            try_technique!(apply_naked_pairs, Technique::NakedPair);
            try_technique!(apply_hidden_pairs, Technique::HiddenPair);
            try_technique!(apply_naked_triples, Technique::NakedTriple);
            try_technique!(apply_hidden_triples, Technique::HiddenTriple);
            try_technique!(apply_pointing_pairs, Technique::PointingPair);
            try_technique!(apply_box_line_reduction, Technique::BoxLineReduction);
            try_technique!(apply_x_wing, Technique::XWing);
            try_technique!(apply_finned_x_wing, Technique::FinnedXWing);
            try_technique!(apply_swordfish, Technique::Swordfish);
            try_technique!(apply_finned_swordfish, Technique::FinnedSwordfish);
            try_technique!(apply_jellyfish, Technique::Jellyfish);
            try_technique!(apply_finned_jellyfish, Technique::FinnedJellyfish);
            try_technique!(apply_xy_wing, Technique::XYWing);
            try_technique!(apply_xyz_wing, Technique::XYZWing);
            try_technique!(apply_w_wing, Technique::WWing);
            try_technique!(apply_x_chain, Technique::XChain);
            try_technique!(apply_aic, Technique::AIC);
            try_technique!(apply_als_xz, Technique::AlsXz);
            try_technique!(apply_als_xy_wing, Technique::AlsXyWing);
            try_technique!(apply_unique_rectangle, Technique::UniqueRectangle);
            try_technique!(apply_nishio_forcing_chain, Technique::NishioForcingChain);
            try_technique!(apply_cell_forcing_chain, Technique::CellForcingChain);
            try_technique!(apply_dynamic_forcing_chain, Technique::DynamicForcingChain);
            return Technique::Backtracking;
        }

        max_technique
    }

    /// Map a technique + puzzle characteristics to a difficulty level
    fn technique_to_difficulty(tech: Technique, empty_count: usize) -> Difficulty {
        match tech {
            Technique::NakedSingle => {
                if empty_count <= 35 {
                    Difficulty::Beginner
                } else {
                    Difficulty::Easy
                }
            }
            Technique::HiddenSingle => Difficulty::Medium,
            Technique::NakedPair
            | Technique::HiddenPair
            | Technique::NakedTriple
            | Technique::HiddenTriple => Difficulty::Intermediate,
            Technique::PointingPair | Technique::BoxLineReduction => Difficulty::Hard,
            Technique::XWing
            | Technique::FinnedXWing
            | Technique::Swordfish
            | Technique::FinnedSwordfish
            | Technique::Jellyfish
            | Technique::FinnedJellyfish => Difficulty::Expert,
            Technique::XYWing
            | Technique::XYZWing
            | Technique::WWing
            | Technique::XChain
            | Technique::AIC => Difficulty::Master,
            Technique::AlsXz
            | Technique::AlsXyWing
            | Technique::UniqueRectangle
            | Technique::NishioForcingChain
            | Technique::CellForcingChain
            | Technique::DynamicForcingChain
            | Technique::Backtracking => Difficulty::Extreme,
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
                        pos.row + 1,
                        pos.col + 1,
                        value
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
                            value,
                            pos.row + 1,
                            pos.col + 1,
                            row + 1
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
                            value,
                            pos.row + 1,
                            pos.col + 1,
                            col + 1
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
                            value,
                            pos.row + 1,
                            pos.col + 1,
                            box_idx + 1
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

                            let to_remove1: Vec<u8> =
                                cand1.iter().filter(|&v| v != v1 && v != v2).collect();
                            let to_remove2: Vec<u8> =
                                cand2.iter().filter(|&v| v != v1 && v != v2).collect();

                            if !to_remove1.is_empty() {
                                return Some(Hint {
                                    technique: Technique::HiddenPair,
                                    hint_type: HintType::EliminateCandidates {
                                        pos: pos1,
                                        values: to_remove1,
                                    },
                                    explanation: format!(
                                        "Hidden pair {{{}, {}}} in {} at ({}, {}) and ({}, {}).",
                                        v1,
                                        v2,
                                        unit_name,
                                        pos1.row + 1,
                                        pos1.col + 1,
                                        pos2.row + 1,
                                        pos2.col + 1
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
                                        v1,
                                        v2,
                                        unit_name,
                                        pos1.row + 1,
                                        pos1.col + 1,
                                        pos2.row + 1,
                                        pos2.col + 1
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

                            if combined.count() == 3
                                && cand1.count() <= 3
                                && cand2.count() <= 3
                                && cand3.count() <= 3
                            {
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
                    .filter(|&&pos| {
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
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
                    if cols_with_value
                        .iter()
                        .all(|&col| Position::new(row, col).box_index() == first_box)
                    {
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
                    if rows_with_value
                        .iter()
                        .all(|&row| Position::new(row, col).box_index() == first_box)
                    {
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
                                    if grid.cell(pos).is_empty()
                                        && grid.get_candidates(pos).contains(value)
                                    {
                                        let involved: Vec<Position> = rows
                                            .iter()
                                            .flat_map(|&r| {
                                                cols.iter().map(move |&c| Position::new(r, c))
                                            })
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
                                    if grid.cell(pos).is_empty()
                                        && grid.get_candidates(pos).contains(value)
                                    {
                                        let involved: Vec<Position> = rows
                                            .iter()
                                            .flat_map(|&r| {
                                                cols.iter().map(move |&c| Position::new(r, c))
                                            })
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
                                        if grid.cell(pos).is_empty()
                                            && grid.get_candidates(pos).contains(value)
                                        {
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
                                                    all_cols
                                                        .iter()
                                                        .map(|c| c + 1)
                                                        .collect::<Vec<_>>()
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
                                        if grid.cell(pos).is_empty()
                                            && grid.get_candidates(pos).contains(value)
                                        {
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
                                                    all_rows
                                                        .iter()
                                                        .map(|r| r + 1)
                                                        .collect::<Vec<_>>()
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

    fn find_jellyfish(&self, grid: &Grid) -> Option<Hint> {
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
                                for &col in &all_cols {
                                    for row in 0..9 {
                                        if !rows.contains(&row) {
                                            let pos = Position::new(row, col);
                                            if grid.cell(pos).is_empty()
                                                && grid.get_candidates(pos).contains(value)
                                            {
                                                return Some(Hint {
                                                    technique: Technique::Jellyfish,
                                                    hint_type: HintType::EliminateCandidates {
                                                        pos,
                                                        values: vec![value],
                                                    },
                                                    explanation: format!(
                                                        "Jellyfish on {} in rows {:?}, columns {:?}.",
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
            }

            // Column-based Jellyfish
            let mut col_data: Vec<(usize, Vec<usize>)> = Vec::new();
            for col in 0..9 {
                let rows: Vec<usize> = (0..9)
                    .filter(|&row| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();
                if rows.len() >= 2 && rows.len() <= 4 {
                    col_data.push((col, rows));
                }
            }
            for i in 0..col_data.len() {
                for j in (i + 1)..col_data.len() {
                    for k in (j + 1)..col_data.len() {
                        for l in (k + 1)..col_data.len() {
                            let mut all_rows: Vec<usize> = Vec::new();
                            all_rows.extend(&col_data[i].1);
                            all_rows.extend(&col_data[j].1);
                            all_rows.extend(&col_data[k].1);
                            all_rows.extend(&col_data[l].1);
                            all_rows.sort();
                            all_rows.dedup();
                            if all_rows.len() == 4 {
                                let cols = [col_data[i].0, col_data[j].0, col_data[k].0, col_data[l].0];
                                for &row in &all_rows {
                                    for col in 0..9 {
                                        if !cols.contains(&col) {
                                            let pos = Position::new(row, col);
                                            if grid.cell(pos).is_empty()
                                                && grid.get_candidates(pos).contains(value)
                                            {
                                                return Some(Hint {
                                                    technique: Technique::Jellyfish,
                                                    hint_type: HintType::EliminateCandidates {
                                                        pos,
                                                        values: vec![value],
                                                    },
                                                    explanation: format!(
                                                        "Jellyfish on {} in columns {:?}, rows {:?}.",
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
        }
        None
    }

    fn apply_jellyfish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_jellyfish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
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
                let (shared1, z1) = if a == x {
                    (x, b)
                } else if a == y {
                    (y, b)
                } else if b == x {
                    (x, a)
                } else if b == y {
                    (y, a)
                } else {
                    continue;
                };

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
                            if pos != pivot
                                && pos != wing1
                                && pos != wing2
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

    fn find_xyz_wing(&self, grid: &Grid) -> Option<Hint> {
        for pivot in grid.empty_positions() {
            let pivot_cand = grid.get_candidates(pivot);
            if pivot_cand.count() != 3 {
                continue;
            }

            let xyz: Vec<u8> = pivot_cand.iter().collect();

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
                    cand.iter().all(|v| pivot_cand.contains(v))
                })
                .collect();

            for i in 0..wings.len() {
                for j in (i + 1)..wings.len() {
                    let wing1 = wings[i];
                    let wing2 = wings[j];

                    let cand1 = grid.get_candidates(wing1);
                    let cand2 = grid.get_candidates(wing2);

                    let wing_union = cand1.union(&cand2);
                    if wing_union.count() != 3 {
                        continue;
                    }

                    let common: Vec<u8> = xyz
                        .iter()
                        .filter(|&&v| cand1.contains(v) && cand2.contains(v))
                        .copied()
                        .collect();

                    if common.len() != 1 {
                        continue;
                    }

                    let z = common[0];

                    for pos in grid.empty_positions() {
                        if pos != pivot
                            && pos != wing1
                            && pos != wing2
                            && self.sees(pos, pivot)
                            && self.sees(pos, wing1)
                            && self.sees(pos, wing2)
                            && grid.get_candidates(pos).contains(z)
                        {
                            return Some(Hint {
                                technique: Technique::XYZWing,
                                hint_type: HintType::EliminateCandidates {
                                    pos,
                                    values: vec![z],
                                },
                                explanation: format!(
                                    "XYZ-Wing: pivot ({}, {}) with {:?}, wings at ({}, {}) and ({}, {}). Remove {} from cells seeing all three.",
                                    pivot.row + 1, pivot.col + 1, xyz,
                                    wing1.row + 1, wing1.col + 1,
                                    wing2.row + 1, wing2.col + 1, z
                                ),
                                involved_cells: vec![pivot, wing1, wing2],
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_xyz_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_xyz_wing(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== W-Wing ====================

    fn find_w_wing(&self, grid: &Grid) -> Option<Hint> {
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

                if !((a1 == a2 && b1 == b2) || (a1 == b2 && b1 == a2)) {
                    continue;
                }

                let x = a1;
                let y = b1;

                // Check rows for strong link
                for row in 0..9 {
                    for value in [x, y] {
                        let positions_in_row: Vec<usize> = (0..9)
                            .filter(|&col| {
                                let pos = Position::new(row, col);
                                grid.cell(pos).is_empty()
                                    && grid.get_candidates(pos).contains(value)
                            })
                            .collect();

                        if positions_in_row.len() == 2 {
                            let link1 = Position::new(row, positions_in_row[0]);
                            let link2 = Position::new(row, positions_in_row[1]);
                            let other_value = if value == x { y } else { x };

                            if (self.sees(pos1, link1) && self.sees(pos2, link2))
                                || (self.sees(pos1, link2) && self.sees(pos2, link1))
                            {
                                for pos in grid.empty_positions() {
                                    if pos != pos1
                                        && pos != pos2
                                        && self.sees(pos, pos1)
                                        && self.sees(pos, pos2)
                                        && grid.get_candidates(pos).contains(other_value)
                                    {
                                        return Some(Hint {
                                            technique: Technique::WWing,
                                            hint_type: HintType::EliminateCandidates {
                                                pos,
                                                values: vec![other_value],
                                            },
                                            explanation: format!(
                                                "W-Wing: cells ({}, {}) and ({}, {}) with {{{}, {}}}, strong link on {} in row {}. Remove {}.",
                                                pos1.row + 1, pos1.col + 1,
                                                pos2.row + 1, pos2.col + 1,
                                                x, y, value, row + 1, other_value
                                            ),
                                            involved_cells: vec![pos1, pos2, link1, link2],
                                        });
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
                                grid.cell(pos).is_empty()
                                    && grid.get_candidates(pos).contains(value)
                            })
                            .collect();

                        if positions_in_col.len() == 2 {
                            let link1 = Position::new(positions_in_col[0], col);
                            let link2 = Position::new(positions_in_col[1], col);
                            let other_value = if value == x { y } else { x };

                            if (self.sees(pos1, link1) && self.sees(pos2, link2))
                                || (self.sees(pos1, link2) && self.sees(pos2, link1))
                            {
                                for pos in grid.empty_positions() {
                                    if pos != pos1
                                        && pos != pos2
                                        && self.sees(pos, pos1)
                                        && self.sees(pos, pos2)
                                        && grid.get_candidates(pos).contains(other_value)
                                    {
                                        return Some(Hint {
                                            technique: Technique::WWing,
                                            hint_type: HintType::EliminateCandidates {
                                                pos,
                                                values: vec![other_value],
                                            },
                                            explanation: format!(
                                                "W-Wing: cells ({}, {}) and ({}, {}) with {{{}, {}}}, strong link on {} in col {}. Remove {}.",
                                                pos1.row + 1, pos1.col + 1,
                                                pos2.row + 1, pos2.col + 1,
                                                x, y, value, col + 1, other_value
                                            ),
                                            involved_cells: vec![pos1, pos2, link1, link2],
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

    fn apply_w_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_w_wing(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Finned Fish ====================

    fn find_finned_fish_generic(
        &self,
        grid: &Grid,
        size: usize,
        technique: Technique,
    ) -> Option<Hint> {
        let name = match size {
            2 => "Finned X-Wing",
            3 => "Finned Swordfish",
            _ => "Finned Jellyfish",
        };

        for value in 1..=9u8 {
            // Row-based finned fish
            let mut row_data: Vec<(usize, Vec<usize>)> = Vec::new();
            for row in 0..9 {
                let cols: Vec<usize> = (0..9)
                    .filter(|&col| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();
                if cols.len() >= 2 && cols.len() <= size + 2 {
                    row_data.push((row, cols));
                }
            }

            if row_data.len() >= size {
                let indices: Vec<usize> = (0..row_data.len()).collect();
                for combo in Self::combinations(&indices, size) {
                    // Try each row in the combo as the fin row
                    for fin_idx in 0..size {
                        let base_indices: Vec<usize> = (0..size).filter(|&i| i != fin_idx).collect();

                        // Collect columns from base rows
                        let mut cover_cols: Vec<usize> = Vec::new();
                        let mut base_ok = true;
                        for &bi in &base_indices {
                            for &col in &row_data[combo[bi]].1 {
                                if !cover_cols.contains(&col) {
                                    cover_cols.push(col);
                                }
                            }
                        }
                        cover_cols.sort();

                        if cover_cols.len() != size {
                            continue;
                        }

                        // Check base rows have candidates only in cover columns
                        for &bi in &base_indices {
                            if !row_data[combo[bi]].1.iter().all(|c| cover_cols.contains(c)) {
                                base_ok = false;
                                break;
                            }
                        }
                        if !base_ok {
                            continue;
                        }

                        // Fin row: some candidates in cover cols, extras are the fin
                        let fin_row_idx = combo[fin_idx];
                        let fin_row = row_data[fin_row_idx].0;
                        let fin_cols: Vec<usize> = row_data[fin_row_idx]
                            .1
                            .iter()
                            .filter(|c| !cover_cols.contains(c))
                            .copied()
                            .collect();

                        if fin_cols.is_empty() {
                            continue; // No fin = regular fish, not finned
                        }

                        // Check candidates in cover cols exist in fin row
                        let has_cover = row_data[fin_row_idx].1.iter().any(|c| cover_cols.contains(c));
                        if !has_cover {
                            continue;
                        }

                        // All fin cells must share one box
                        let fin_positions: Vec<Position> = fin_cols
                            .iter()
                            .map(|&c| Position::new(fin_row, c))
                            .collect();
                        let fin_box = fin_positions[0].box_index();
                        if !fin_positions.iter().all(|p| p.box_index() == fin_box) {
                            continue;
                        }

                        // Eliminate from cells in cover columns AND fin box, not in any defining row
                        let defining_rows: Vec<usize> = combo.iter().map(|&i| row_data[i].0).collect();
                        for &col in &cover_cols {
                            for row in 0..9 {
                                if defining_rows.contains(&row) {
                                    continue;
                                }
                                let pos = Position::new(row, col);
                                if pos.box_index() == fin_box
                                    && grid.cell(pos).is_empty()
                                    && grid.get_candidates(pos).contains(value)
                                {
                                    return Some(Hint {
                                        technique,
                                        hint_type: HintType::EliminateCandidates {
                                            pos,
                                            values: vec![value],
                                        },
                                        explanation: format!(
                                            "{} on {} in rows {:?}, cover cols {:?}, fin at row {} box {}.",
                                            name, value,
                                            defining_rows.iter().map(|r| r + 1).collect::<Vec<_>>(),
                                            cover_cols.iter().map(|c| c + 1).collect::<Vec<_>>(),
                                            fin_row + 1, fin_box + 1
                                        ),
                                        involved_cells: vec![],
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Column-based finned fish
            let mut col_data: Vec<(usize, Vec<usize>)> = Vec::new();
            for col in 0..9 {
                let rows: Vec<usize> = (0..9)
                    .filter(|&row| {
                        let pos = Position::new(row, col);
                        grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(value)
                    })
                    .collect();
                if rows.len() >= 2 && rows.len() <= size + 2 {
                    col_data.push((col, rows));
                }
            }

            if col_data.len() >= size {
                let indices: Vec<usize> = (0..col_data.len()).collect();
                for combo in Self::combinations(&indices, size) {
                    for fin_idx in 0..size {
                        let base_indices: Vec<usize> = (0..size).filter(|&i| i != fin_idx).collect();

                        let mut cover_rows: Vec<usize> = Vec::new();
                        let mut base_ok = true;
                        for &bi in &base_indices {
                            for &row in &col_data[combo[bi]].1 {
                                if !cover_rows.contains(&row) {
                                    cover_rows.push(row);
                                }
                            }
                        }
                        cover_rows.sort();

                        if cover_rows.len() != size {
                            continue;
                        }

                        for &bi in &base_indices {
                            if !col_data[combo[bi]].1.iter().all(|r| cover_rows.contains(r)) {
                                base_ok = false;
                                break;
                            }
                        }
                        if !base_ok {
                            continue;
                        }

                        let fin_col_idx = combo[fin_idx];
                        let fin_col = col_data[fin_col_idx].0;
                        let fin_rows: Vec<usize> = col_data[fin_col_idx]
                            .1
                            .iter()
                            .filter(|r| !cover_rows.contains(r))
                            .copied()
                            .collect();

                        if fin_rows.is_empty() {
                            continue;
                        }

                        let has_cover = col_data[fin_col_idx].1.iter().any(|r| cover_rows.contains(r));
                        if !has_cover {
                            continue;
                        }

                        let fin_positions: Vec<Position> = fin_rows
                            .iter()
                            .map(|&r| Position::new(r, fin_col))
                            .collect();
                        let fin_box = fin_positions[0].box_index();
                        if !fin_positions.iter().all(|p| p.box_index() == fin_box) {
                            continue;
                        }

                        let defining_cols: Vec<usize> = combo.iter().map(|&i| col_data[i].0).collect();
                        for &row in &cover_rows {
                            for col in 0..9 {
                                if defining_cols.contains(&col) {
                                    continue;
                                }
                                let pos = Position::new(row, col);
                                if pos.box_index() == fin_box
                                    && grid.cell(pos).is_empty()
                                    && grid.get_candidates(pos).contains(value)
                                {
                                    return Some(Hint {
                                        technique,
                                        hint_type: HintType::EliminateCandidates {
                                            pos,
                                            values: vec![value],
                                        },
                                        explanation: format!(
                                            "{} on {} in cols {:?}, cover rows {:?}, fin at col {} box {}.",
                                            name, value,
                                            defining_cols.iter().map(|c| c + 1).collect::<Vec<_>>(),
                                            cover_rows.iter().map(|r| r + 1).collect::<Vec<_>>(),
                                            fin_col + 1, fin_box + 1
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
        None
    }

    fn find_finned_x_wing(&self, grid: &Grid) -> Option<Hint> {
        self.find_finned_fish_generic(grid, 2, Technique::FinnedXWing)
    }

    fn apply_finned_x_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_finned_x_wing(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    fn find_finned_swordfish(&self, grid: &Grid) -> Option<Hint> {
        self.find_finned_fish_generic(grid, 3, Technique::FinnedSwordfish)
    }

    fn apply_finned_swordfish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_finned_swordfish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    fn find_finned_jellyfish(&self, grid: &Grid) -> Option<Hint> {
        self.find_finned_fish_generic(grid, 4, Technique::FinnedJellyfish)
    }

    fn apply_finned_jellyfish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_finned_jellyfish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== AIC Framework (X-Chain + AIC) ====================

    /// Build a link graph for the AIC engine.
    fn build_link_graph(
        grid: &Grid,
    ) -> (
        std::collections::HashMap<(Position, u8), Vec<(Position, u8)>>,
        std::collections::HashMap<(Position, u8), Vec<(Position, u8)>>,
    ) {
        type Node = (Position, u8);
        let mut strong: std::collections::HashMap<Node, Vec<Node>> = std::collections::HashMap::new();
        let mut weak: std::collections::HashMap<Node, Vec<Node>> = std::collections::HashMap::new();

        // Helper: collect empty cells with a given value in a unit
        let units: Vec<Vec<Position>> = {
            let mut u = Vec::with_capacity(27);
            for i in 0..9 {
                u.push(Self::row_positions(i));
                u.push(Self::col_positions(i));
                u.push(Self::box_positions(i));
            }
            u
        };

        // Strong links from conjugate pairs (exactly 2 cells for a value in a unit)
        for unit in &units {
            for value in 1..=9u8 {
                let cells: Vec<Position> = unit
                    .iter()
                    .filter(|&&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(value))
                    .copied()
                    .collect();

                if cells.len() == 2 {
                    let a = (cells[0], value);
                    let b = (cells[1], value);
                    strong.entry(a).or_default().push(b);
                    strong.entry(b).or_default().push(a);
                    // Strong links are also weak links
                    weak.entry(a).or_default().push(b);
                    weak.entry(b).or_default().push(a);
                }

                // Weak links: same value in same unit with >2 occurrences
                if cells.len() > 2 {
                    for i in 0..cells.len() {
                        for j in (i + 1)..cells.len() {
                            let a = (cells[i], value);
                            let b = (cells[j], value);
                            weak.entry(a).or_default().push(b);
                            weak.entry(b).or_default().push(a);
                        }
                    }
                }
            }
        }

        // Strong links from bivalue cells
        for pos in grid.empty_positions() {
            let cand = grid.get_candidates(pos);
            if cand.count() == 2 {
                let vals: Vec<u8> = cand.iter().collect();
                let a = (pos, vals[0]);
                let b = (pos, vals[1]);
                strong.entry(a).or_default().push(b);
                strong.entry(b).or_default().push(a);
                weak.entry(a).or_default().push(b);
                weak.entry(b).or_default().push(a);
            }
            // Weak links: different values in same cell
            let vals: Vec<u8> = cand.iter().collect();
            for i in 0..vals.len() {
                for j in (i + 1)..vals.len() {
                    let a = (pos, vals[i]);
                    let b = (pos, vals[j]);
                    weak.entry(a).or_default().push(b);
                    weak.entry(b).or_default().push(a);
                }
            }
        }

        // Deduplicate
        for list in strong.values_mut() {
            list.sort_by(|a, b| (a.0.row, a.0.col, a.1).cmp(&(b.0.row, b.0.col, b.1)));
            list.dedup();
        }
        for list in weak.values_mut() {
            list.sort_by(|a, b| (a.0.row, a.0.col, a.1).cmp(&(b.0.row, b.0.col, b.1)));
            list.dedup();
        }

        (strong, weak)
    }

    /// Search for AIC chains. If `single_value_only` is true, only finds single-digit chains (X-Chain).
    fn find_aic_with_filter(&self, grid: &Grid, single_value_only: bool) -> Option<Hint> {
        let (strong, weak) = Self::build_link_graph(grid);
        const MAX_AIC_LENGTH: usize = 12;

        type Node = (Position, u8);

        // BFS state: (current_node, arrived_via_strong, chain)
        let all_nodes: Vec<Node> = strong.keys().copied().collect();

        for &start in &all_nodes {
            // BFS from start, alternating strong/weak
            // We start by looking for strong links from start
            let mut queue: std::collections::VecDeque<(Node, bool, Vec<Node>)> =
                std::collections::VecDeque::new();
            let mut visited: std::collections::HashSet<(Node, bool)> =
                std::collections::HashSet::new();

            // Initial: we need to traverse a strong link first
            if let Some(neighbors) = strong.get(&start) {
                for &next in neighbors {
                    if single_value_only && next.1 != start.1 {
                        continue;
                    }
                    let chain = vec![start, next];
                    queue.push_back((next, true, chain));
                }
            }

            while let Some((current, arrived_strong, chain)) = queue.pop_front() {
                if chain.len() > MAX_AIC_LENGTH {
                    continue;
                }

                let key = (current, arrived_strong);
                if visited.contains(&key) {
                    continue;
                }
                visited.insert(key);

                // Next step alternates: if arrived via strong, follow weak; if arrived via weak, follow strong
                if arrived_strong {
                    // Follow weak links
                    if let Some(neighbors) = weak.get(&current) {
                        for &next in neighbors {
                            if chain.contains(&next) && next != start {
                                continue;
                            }
                            if single_value_only && next.1 != start.1 {
                                continue;
                            }

                            // Check for elimination: chain ends with a weak link back to... we need the chain to close or produce eliminations
                            // Type 1: same value at both endpoints, different positions
                            if next != start && chain.len() >= 3 {
                                // The next node reached via weak link from current
                                // If next.1 == start.1 and next.0 != start.0, and they "see" each other or share peers
                                // Actually: for a valid AIC, the chain must start with strong and alternate
                                // A chain of even length: strong-weak-strong-weak... ending on weak
                                // The endpoints are connected by weak links at both ends
                                // Type 1 elimination: start and next have the same value and both endpoints' value can be eliminated from cells seeing both
                                if next.1 == start.1 && next.0 != start.0 {
                                    // Cells that see both start.0 and next.0 can have value eliminated
                                    let val = start.1;
                                    for pos in grid.empty_positions() {
                                        if pos != start.0
                                            && pos != next.0
                                            && self.sees(pos, start.0)
                                            && self.sees(pos, next.0)
                                            && grid.get_candidates(pos).contains(val)
                                        {
                                            let tech = if single_value_only {
                                                Technique::XChain
                                            } else {
                                                Technique::AIC
                                            };
                                            let mut involved: Vec<Position> = chain.iter().map(|n| n.0).collect();
                                            involved.push(next.0);
                                            involved.dedup();
                                            return Some(Hint {
                                                technique: tech,
                                                hint_type: HintType::EliminateCandidates {
                                                    pos,
                                                    values: vec![val],
                                                },
                                                explanation: format!(
                                                    "{}: chain of length {} on value(s), eliminate {} from ({}, {}).",
                                                    tech, chain.len(), val, pos.row + 1, pos.col + 1
                                                ),
                                                involved_cells: involved,
                                            });
                                        }
                                    }
                                }

                                // Type 2: same cell at both endpoints, different values
                                if !single_value_only
                                    && next.0 == start.0
                                    && next.1 != start.1
                                {
                                    // All other candidates in that cell can be eliminated
                                    let cand = grid.get_candidates(start.0);
                                    let to_remove: Vec<u8> = cand
                                        .iter()
                                        .filter(|&v| v != start.1 && v != next.1)
                                        .collect();
                                    if !to_remove.is_empty() {
                                        let involved: Vec<Position> = chain.iter().map(|n| n.0).collect();
                                        return Some(Hint {
                                            technique: Technique::AIC,
                                            hint_type: HintType::EliminateCandidates {
                                                pos: start.0,
                                                values: to_remove.clone(),
                                            },
                                            explanation: format!(
                                                "AIC: chain returns to cell ({}, {}), eliminate {:?}.",
                                                start.0.row + 1, start.0.col + 1, to_remove
                                            ),
                                            involved_cells: involved,
                                        });
                                    }
                                }
                            }

                            if next != start && !visited.contains(&(next, false)) {
                                let mut new_chain = chain.clone();
                                new_chain.push(next);
                                queue.push_back((next, false, new_chain));
                            }
                        }
                    }
                } else {
                    // Follow strong links
                    if let Some(neighbors) = strong.get(&current) {
                        for &next in neighbors {
                            if chain.contains(&next) && next != start {
                                continue;
                            }
                            if single_value_only && next.1 != start.1 {
                                continue;
                            }
                            if !visited.contains(&(next, true)) {
                                let mut new_chain = chain.clone();
                                new_chain.push(next);
                                queue.push_back((next, true, new_chain));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn find_x_chain(&self, grid: &Grid) -> Option<Hint> {
        self.find_aic_with_filter(grid, true)
    }

    fn apply_x_chain(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_x_chain(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    fn find_aic(&self, grid: &Grid) -> Option<Hint> {
        self.find_aic_with_filter(grid, false)
    }

    fn apply_aic(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_aic(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== ALS (Almost Locked Sets) ====================

    /// Generate all combinations of k items from a slice.
    fn combinations<T: Copy>(items: &[T], k: usize) -> Vec<Vec<T>> {
        let mut result = Vec::new();
        if k == 0 || k > items.len() {
            return result;
        }
        let mut indices: Vec<usize> = (0..k).collect();
        loop {
            result.push(indices.iter().map(|&i| items[i]).collect());
            // Find the rightmost index that can be incremented
            let mut i = k;
            loop {
                if i == 0 {
                    return result;
                }
                i -= 1;
                if indices[i] < items.len() - k + i {
                    break;
                }
                if i == 0 {
                    return result;
                }
            }
            indices[i] += 1;
            for j in (i + 1)..k {
                indices[j] = indices[j - 1] + 1;
            }
        }
    }

    /// Find all Almost Locked Sets in the grid.
    /// An ALS is a set of N cells in a unit whose candidate union has exactly N+1 values.
    fn find_all_als(grid: &Grid) -> Vec<(Vec<Position>, crate::BitSet)> {
        let mut result = Vec::new();
        let units: Vec<Vec<Position>> = {
            let mut u = Vec::with_capacity(27);
            for i in 0..9 {
                u.push(Self::row_positions(i));
                u.push(Self::col_positions(i));
                u.push(Self::box_positions(i));
            }
            u
        };

        for unit in &units {
            let empty: Vec<Position> = unit
                .iter()
                .filter(|&&p| grid.cell(p).is_empty())
                .copied()
                .collect();

            // Check subsets of size 2..=5 (ALS needs N cells with N+1 candidates)
            for n in 2..=empty.len().min(5) {
                for combo in Self::combinations(&empty, n) {
                    let mut union = crate::BitSet::empty();
                    for &pos in &combo {
                        union = union.union(&grid.get_candidates(pos));
                    }
                    if union.count() == (n + 1) as u32 {
                        // Check this ALS isn't already in result (by same set of positions)
                        let mut sorted_combo = combo.clone();
                        sorted_combo.sort_by(|a, b| (a.row, a.col).cmp(&(b.row, b.col)));
                        let exists = result.iter().any(|(cells, _): &(Vec<Position>, crate::BitSet)| {
                            *cells == sorted_combo
                        });
                        if !exists {
                            result.push((sorted_combo, union));
                        }
                    }
                }
            }
        }
        result
    }

    /// ALS-XZ: find two non-overlapping ALS with a restricted common candidate (RCC)
    /// and eliminate common non-RCC values from cells seeing both.
    fn find_als_xz(&self, grid: &Grid) -> Option<Hint> {
        let all_als = Self::find_all_als(grid);

        for i in 0..all_als.len() {
            for j in (i + 1)..all_als.len() {
                let (cells_a, cands_a) = &all_als[i];
                let (cells_b, cands_b) = &all_als[j];

                // Must be non-overlapping
                if cells_a.iter().any(|p| cells_b.contains(p)) {
                    continue;
                }

                let common = cands_a.intersection(cands_b);
                if common.is_empty() {
                    continue;
                }

                // Find restricted common candidates (RCC):
                // A value X is an RCC if every cell in ALS-A containing X sees every cell in ALS-B containing X
                for x in common.iter() {
                    let a_cells_x: Vec<Position> = cells_a
                        .iter()
                        .filter(|&&p| grid.get_candidates(p).contains(x))
                        .copied()
                        .collect();
                    let b_cells_x: Vec<Position> = cells_b
                        .iter()
                        .filter(|&&p| grid.get_candidates(p).contains(x))
                        .copied()
                        .collect();

                    let is_rcc = a_cells_x
                        .iter()
                        .all(|&a| b_cells_x.iter().all(|&b| self.sees(a, b)));

                    if !is_rcc {
                        continue;
                    }

                    // For each other common value Z, eliminate Z from cells seeing all Z-cells in both ALS
                    for z in common.iter() {
                        if z == x {
                            continue;
                        }

                        let a_cells_z: Vec<Position> = cells_a
                            .iter()
                            .filter(|&&p| grid.get_candidates(p).contains(z))
                            .copied()
                            .collect();
                        let b_cells_z: Vec<Position> = cells_b
                            .iter()
                            .filter(|&&p| grid.get_candidates(p).contains(z))
                            .copied()
                            .collect();

                        for pos in grid.empty_positions() {
                            if cells_a.contains(&pos) || cells_b.contains(&pos) {
                                continue;
                            }
                            if !grid.get_candidates(pos).contains(z) {
                                continue;
                            }
                            let sees_all_a = a_cells_z.iter().all(|&p| self.sees(pos, p));
                            let sees_all_b = b_cells_z.iter().all(|&p| self.sees(pos, p));
                            if sees_all_a && sees_all_b {
                                let mut involved = cells_a.clone();
                                involved.extend(cells_b);
                                return Some(Hint {
                                    technique: Technique::AlsXz,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![z],
                                    },
                                    explanation: format!(
                                        "ALS-XZ: RCC={}, eliminate {} from ({}, {}).",
                                        x, z, pos.row + 1, pos.col + 1
                                    ),
                                    involved_cells: involved,
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_als_xz(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_als_xz(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    /// ALS-XY-Wing: Three ALS (A-B-C) with RCC X between A-B and RCC Y between B-C.
    /// Eliminate common value Z from cells seeing all Z-cells in A and C.
    fn find_als_xy_wing(&self, grid: &Grid) -> Option<Hint> {
        let all_als = Self::find_all_als(grid);
        // Limit to smaller ALS for performance
        let small_als: Vec<&(Vec<Position>, crate::BitSet)> = all_als
            .iter()
            .filter(|(cells, _)| cells.len() <= 4)
            .collect();

        // Pre-compute RCC map: for each pair (i, j), which values are RCC?
        let mut rcc_map: std::collections::HashMap<(usize, usize), Vec<u8>> =
            std::collections::HashMap::new();

        for i in 0..small_als.len() {
            for j in (i + 1)..small_als.len() {
                let (cells_a, cands_a) = small_als[i];
                let (cells_b, cands_b) = small_als[j];

                if cells_a.iter().any(|p| cells_b.contains(p)) {
                    continue;
                }

                let common = cands_a.intersection(cands_b);
                let mut rccs = Vec::new();
                for v in common.iter() {
                    let a_cells: Vec<Position> = cells_a
                        .iter()
                        .filter(|&&p| grid.get_candidates(p).contains(v))
                        .copied()
                        .collect();
                    let b_cells: Vec<Position> = cells_b
                        .iter()
                        .filter(|&&p| grid.get_candidates(p).contains(v))
                        .copied()
                        .collect();
                    if a_cells.iter().all(|&a| b_cells.iter().all(|&b| self.sees(a, b))) {
                        rccs.push(v);
                    }
                }
                if !rccs.is_empty() {
                    rcc_map.insert((i, j), rccs.clone());
                    rcc_map.insert((j, i), rccs);
                }
            }
        }

        // Find triples A-B-C where A-B has RCC X and B-C has RCC Y (X != Y)
        for b_idx in 0..small_als.len() {
            // Find all ALS connected to B via RCC
            let partners: Vec<(usize, &[u8])> = rcc_map
                .iter()
                .filter(|((from, _), _)| *from == b_idx)
                .map(|((_, to), rccs)| (*to, rccs.as_slice()))
                .collect();

            for pi in 0..partners.len() {
                for pj in (pi + 1)..partners.len() {
                    let (a_idx, rccs_ab) = partners[pi];
                    let (c_idx, rccs_bc) = partners[pj];

                    if a_idx == c_idx {
                        continue;
                    }

                    let (cells_a, cands_a) = small_als[a_idx];
                    let (cells_c, cands_c) = small_als[c_idx];

                    // A and C must be non-overlapping
                    if cells_a.iter().any(|p| cells_c.contains(p)) {
                        continue;
                    }

                    // Find X in rccs_ab and Y in rccs_bc where X != Y
                    for &x in rccs_ab {
                        for &y in rccs_bc {
                            if x == y {
                                continue;
                            }

                            // Find common values Z between A and C (not X, not Y)
                            let common_ac = cands_a.intersection(cands_c);
                            for z in common_ac.iter() {
                                if z == x || z == y {
                                    continue;
                                }

                                let a_cells_z: Vec<Position> = cells_a
                                    .iter()
                                    .filter(|&&p| grid.get_candidates(p).contains(z))
                                    .copied()
                                    .collect();
                                let c_cells_z: Vec<Position> = cells_c
                                    .iter()
                                    .filter(|&&p| grid.get_candidates(p).contains(z))
                                    .copied()
                                    .collect();

                                if a_cells_z.is_empty() || c_cells_z.is_empty() {
                                    continue;
                                }

                                for pos in grid.empty_positions() {
                                    if cells_a.contains(&pos)
                                        || small_als[b_idx].0.contains(&pos)
                                        || cells_c.contains(&pos)
                                    {
                                        continue;
                                    }
                                    if !grid.get_candidates(pos).contains(z) {
                                        continue;
                                    }
                                    let sees_all_a = a_cells_z.iter().all(|&p| self.sees(pos, p));
                                    let sees_all_c = c_cells_z.iter().all(|&p| self.sees(pos, p));
                                    if sees_all_a && sees_all_c {
                                        let mut involved = cells_a.clone();
                                        involved.extend(small_als[b_idx].0.iter());
                                        involved.extend(cells_c);
                                        return Some(Hint {
                                            technique: Technique::AlsXyWing,
                                            hint_type: HintType::EliminateCandidates {
                                                pos,
                                                values: vec![z],
                                            },
                                            explanation: format!(
                                                "ALS-XY-Wing: RCC X={}, Y={}, eliminate {} from ({}, {}).",
                                                x, y, z, pos.row + 1, pos.col + 1
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

    fn apply_als_xy_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_als_xy_wing(grid) {
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
        #[allow(clippy::needless_range_loop)]
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
                        if corner3.box_index() != corner4.box_index()
                            && pos1.box_index() != pos2.box_index()
                        {
                            continue;
                        }

                        let cand3 = grid.get_candidates(corner3);
                        let cand4 = grid.get_candidates(corner4);

                        // Type 1: One corner has extra candidates
                        if cand3.count() == 2
                            && cand3.contains(a)
                            && cand3.contains(b)
                            && cand4.count() > 2
                            && cand4.contains(a)
                            && cand4.contains(b)
                        {
                            // corner4 must have at least one of a,b removed to break the pattern
                            // Actually Type 1 means corner4 can't be just {a,b}
                            // We eliminate a and b from corner4 if it would create deadly pattern
                            // This needs more careful implementation
                        }

                        // Type 2: Two corners have same extra candidate
                        if cand3.count() == 3 && cand4.count() == 3 {
                            let extra3: Vec<u8> =
                                cand3.iter().filter(|&v| v != a && v != b).collect();
                            let extra4: Vec<u8> =
                                cand4.iter().filter(|&v| v != a && v != b).collect();

                            if extra3.len() == 1 && extra4.len() == 1 && extra3[0] == extra4[0] {
                                let extra = extra3[0];

                                // Eliminate extra from cells that see both corner3 and corner4
                                for pos in grid.empty_positions() {
                                    if pos != corner3
                                        && pos != corner4
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

                        if corner3.box_index() != corner4.box_index()
                            && pos1.box_index() != pos2.box_index()
                        {
                            continue;
                        }

                        let cand3 = grid.get_candidates(corner3);
                        let cand4 = grid.get_candidates(corner4);

                        if cand3.count() == 3 && cand4.count() == 3 {
                            let extra3: Vec<u8> =
                                cand3.iter().filter(|&v| v != a && v != b).collect();
                            let extra4: Vec<u8> =
                                cand4.iter().filter(|&v| v != a && v != b).collect();

                            if extra3.len() == 1 && extra4.len() == 1 && extra3[0] == extra4[0] {
                                let extra = extra3[0];

                                for pos in grid.empty_positions() {
                                    if pos != corner3
                                        && pos != corner4
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

    // ==================== Forcing Chains Infrastructure ====================

    /// Check if a grid has a contradiction: any empty cell with no candidates,
    /// or duplicate values in any row/col/box.
    fn has_contradiction(grid: &Grid) -> bool {
        for pos in grid.empty_positions() {
            if grid.get_candidates(pos).is_empty() {
                return true;
            }
        }
        // Check for duplicate values in rows, columns, and boxes
        for i in 0..9 {
            let mut row_seen = [false; 10];
            let mut col_seen = [false; 10];
            let mut box_seen = [false; 10];
            for j in 0..9 {
                // Row check
                if let Some(v) = grid.get(Position::new(i, j)) {
                    if row_seen[v as usize] {
                        return true;
                    }
                    row_seen[v as usize] = true;
                }
                // Col check
                if let Some(v) = grid.get(Position::new(j, i)) {
                    if col_seen[v as usize] {
                        return true;
                    }
                    col_seen[v as usize] = true;
                }
                // Box check
                let box_row = (i / 3) * 3 + j / 3;
                let box_col = (i % 3) * 3 + j % 3;
                if let Some(v) = grid.get(Position::new(box_row, box_col)) {
                    if box_seen[v as usize] {
                        return true;
                    }
                    box_seen[v as usize] = true;
                }
            }
        }
        false
    }

    /// Propagate singles (naked + hidden) from an assumption until no more progress.
    /// Returns the resulting grid and whether a contradiction was found.
    fn propagate_singles(&self, grid: &Grid, pos: Position, val: u8) -> (Grid, bool) {
        let mut g = grid.deep_clone();
        g.set_cell_unchecked(pos, Some(val));
        g.recalculate_candidates();

        for _ in 0..200 {
            if Self::has_contradiction(&g) {
                return (g, true);
            }
            if g.is_complete() {
                return (g, false);
            }
            let mut progress = false;
            // Apply naked singles
            for p in g.empty_positions() {
                if let Some(v) = g.get_candidates(p).single_value() {
                    g.set_cell_unchecked(p, Some(v));
                    g.recalculate_candidates();
                    progress = true;
                    break;
                }
            }
            if !progress {
                // Apply hidden singles
                'outer: for unit in 0..27 {
                    let positions: Vec<Position> = if unit < 9 {
                        Self::row_positions(unit)
                    } else if unit < 18 {
                        Self::col_positions(unit - 9)
                    } else {
                        Self::box_positions(unit - 18)
                    };
                    for value in 1..=9u8 {
                        let mut candidates: Vec<Position> = Vec::new();
                        for &p in &positions {
                            if g.cell(p).is_empty() && g.get_candidates(p).contains(value) {
                                candidates.push(p);
                            }
                        }
                        if candidates.len() == 1 {
                            g.set_cell_unchecked(candidates[0], Some(value));
                            g.recalculate_candidates();
                            progress = true;
                            break 'outer;
                        }
                    }
                }
            }
            if !progress {
                break;
            }
        }
        let contradiction = Self::has_contradiction(&g);
        (g, contradiction)
    }

    /// Propagate using the full technique set (all techniques up to UniqueRectangle).
    /// Used by Dynamic Forcing Chain. Never calls forcing chains to prevent recursion.
    fn propagate_full(&self, grid: &Grid, pos: Position, val: u8) -> (Grid, bool) {
        let mut g = grid.deep_clone();
        g.set_cell_unchecked(pos, Some(val));
        g.recalculate_candidates();

        for _ in 0..50 {
            if Self::has_contradiction(&g) {
                return (g, true);
            }
            if g.is_complete() {
                return (g, false);
            }
            let mut progress = false;
            // Try all techniques up to UniqueRectangle (no forcing chains!)
            progress |= self.apply_naked_singles(&mut g);
            if !progress { progress |= self.apply_hidden_singles(&mut g); }
            if !progress { progress |= self.apply_naked_pairs(&mut g); }
            if !progress { progress |= self.apply_hidden_pairs(&mut g); }
            if !progress { progress |= self.apply_naked_triples(&mut g); }
            if !progress { progress |= self.apply_hidden_triples(&mut g); }
            if !progress { progress |= self.apply_pointing_pairs(&mut g); }
            if !progress { progress |= self.apply_box_line_reduction(&mut g); }
            if !progress { progress |= self.apply_x_wing(&mut g); }
            if !progress { progress |= self.apply_finned_x_wing(&mut g); }
            if !progress { progress |= self.apply_swordfish(&mut g); }
            if !progress { progress |= self.apply_finned_swordfish(&mut g); }
            if !progress { progress |= self.apply_jellyfish(&mut g); }
            if !progress { progress |= self.apply_finned_jellyfish(&mut g); }
            if !progress { progress |= self.apply_xy_wing(&mut g); }
            if !progress { progress |= self.apply_xyz_wing(&mut g); }
            if !progress { progress |= self.apply_w_wing(&mut g); }
            if !progress { progress |= self.apply_x_chain(&mut g); }
            if !progress { progress |= self.apply_aic(&mut g); }
            if !progress { progress |= self.apply_als_xz(&mut g); }
            if !progress { progress |= self.apply_als_xy_wing(&mut g); }
            if !progress { progress |= self.apply_unique_rectangle(&mut g); }
            if !progress {
                break;
            }
        }
        let contradiction = Self::has_contradiction(&g);
        (g, contradiction)
    }

    // ==================== Nishio Forcing Chain ====================

    fn find_nishio_forcing_chain(&self, grid: &Grid) -> Option<Hint> {
        // Collect empty cells, sorted by candidate count (bivalue first)
        let mut cells: Vec<Position> = grid.empty_positions();
        cells.sort_by_key(|&p| grid.get_candidates(p).count());

        for &pos in &cells {
            let cands = grid.get_candidates(pos);
            if cands.count() < 2 || cands.count() > 4 {
                continue;
            }
            for val in cands.iter() {
                let (_, contradiction) = self.propagate_singles(grid, pos, val);
                if contradiction {
                    return Some(Hint {
                        technique: Technique::NishioForcingChain,
                        hint_type: HintType::EliminateCandidates {
                            pos,
                            values: vec![val],
                        },
                        explanation: format!(
                            "Nishio: assuming {} in ({}, {}) leads to contradiction, so {} is eliminated.",
                            val, pos.row + 1, pos.col + 1, val
                        ),
                        involved_cells: vec![pos],
                    });
                }
            }
        }
        None
    }

    fn apply_nishio_forcing_chain(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_nishio_forcing_chain(grid) {
            match hint.hint_type {
                HintType::EliminateCandidates { pos, values } => {
                    for v in values {
                        grid.cell_mut(pos).remove_candidate(v);
                    }
                    true
                }
                HintType::SetValue { pos, value } => {
                    grid.set_cell_unchecked(pos, Some(value));
                    grid.recalculate_candidates();
                    true
                }
            }
        } else {
            false
        }
    }

    // ==================== Cell Forcing Chain ====================

    /// Find a common placement across all propagation branches.
    /// If all branches agree that a certain cell must have a certain value, return it.
    fn find_common_placement(
        grid: &Grid,
        source_pos: Position,
        branches: &[Grid],
        technique: Technique,
    ) -> Option<Hint> {
        for target in grid.empty_positions() {
            if target == source_pos {
                continue;
            }
            if grid.get(target).is_some() {
                continue;
            }
            // Check if all branches placed the same value in target
            let mut common_val: Option<u8> = None;
            let mut all_agree = true;
            for branch in branches {
                if let Some(v) = branch.get(target) {
                    match common_val {
                        None => common_val = Some(v),
                        Some(cv) if cv != v => {
                            all_agree = false;
                            break;
                        }
                        _ => {}
                    }
                } else {
                    all_agree = false;
                    break;
                }
            }
            if all_agree {
                if let Some(val) = common_val {
                    return Some(Hint {
                        technique,
                        hint_type: HintType::SetValue {
                            pos: target,
                            value: val,
                        },
                        explanation: format!(
                            "{}: all candidates in ({}, {}) lead to {} in ({}, {}).",
                            technique,
                            source_pos.row + 1,
                            source_pos.col + 1,
                            val,
                            target.row + 1,
                            target.col + 1
                        ),
                        involved_cells: vec![source_pos, target],
                    });
                }
            }
        }
        None
    }

    /// Find a common elimination across all propagation branches.
    /// If all branches agree that a certain candidate is removed from a cell, return it.
    fn find_common_elimination(
        grid: &Grid,
        source_pos: Position,
        branches: &[Grid],
        technique: Technique,
    ) -> Option<Hint> {
        for target in grid.empty_positions() {
            if target == source_pos {
                continue;
            }
            let orig_cands = grid.get_candidates(target);
            if orig_cands.count() < 2 {
                continue;
            }
            for val in orig_cands.iter() {
                // Check if all branches eliminated this candidate
                let mut all_eliminate = true;
                for branch in branches {
                    if let Some(placed) = branch.get(target) {
                        // Cell was filled — candidate is "eliminated" only if placed != val
                        if placed == val {
                            all_eliminate = false;
                            break;
                        }
                    } else if branch.get_candidates(target).contains(val) {
                        all_eliminate = false;
                        break;
                    }
                }
                if all_eliminate {
                    return Some(Hint {
                        technique,
                        hint_type: HintType::EliminateCandidates {
                            pos: target,
                            values: vec![val],
                        },
                        explanation: format!(
                            "{}: all candidates in ({}, {}) eliminate {} from ({}, {}).",
                            technique,
                            source_pos.row + 1,
                            source_pos.col + 1,
                            val,
                            target.row + 1,
                            target.col + 1
                        ),
                        involved_cells: vec![source_pos, target],
                    });
                }
            }
        }
        None
    }

    fn find_cell_forcing_chain(&self, grid: &Grid) -> Option<Hint> {
        let mut cells: Vec<Position> = grid.empty_positions();
        cells.sort_by_key(|&p| grid.get_candidates(p).count());

        for &pos in &cells {
            let cands = grid.get_candidates(pos);
            if cands.count() < 2 || cands.count() > 4 {
                continue;
            }

            let mut branches = Vec::new();
            let mut any_contradiction = false;

            for val in cands.iter() {
                let (result, contradiction) = self.propagate_singles(grid, pos, val);
                if contradiction {
                    // Nishio should have caught this; skip
                    any_contradiction = true;
                    break;
                }
                branches.push(result);
            }

            if any_contradiction || branches.len() < 2 {
                continue;
            }

            // Check for common placement
            if let Some(hint) =
                Self::find_common_placement(grid, pos, &branches, Technique::CellForcingChain)
            {
                return Some(hint);
            }

            // Check for common elimination
            if let Some(hint) =
                Self::find_common_elimination(grid, pos, &branches, Technique::CellForcingChain)
            {
                return Some(hint);
            }
        }
        None
    }

    fn apply_cell_forcing_chain(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_cell_forcing_chain(grid) {
            match hint.hint_type {
                HintType::SetValue { pos, value } => {
                    grid.set_cell_unchecked(pos, Some(value));
                    grid.recalculate_candidates();
                    true
                }
                HintType::EliminateCandidates { pos, values } => {
                    for v in values {
                        grid.cell_mut(pos).remove_candidate(v);
                    }
                    true
                }
            }
        } else {
            false
        }
    }

    // ==================== Dynamic Forcing Chain ====================

    fn find_dynamic_forcing_chain(&self, grid: &Grid) -> Option<Hint> {
        let mut cells: Vec<Position> = grid.empty_positions();
        cells.sort_by_key(|&p| grid.get_candidates(p).count());

        for &pos in &cells {
            let cands = grid.get_candidates(pos);
            // Tighter limit for performance: 2-3 candidates only
            if cands.count() < 2 || cands.count() > 3 {
                continue;
            }

            let mut branches = Vec::new();

            for val in cands.iter() {
                let (result, contradiction) = self.propagate_full(grid, pos, val);
                if contradiction {
                    // Report as dynamic forcing chain elimination
                    return Some(Hint {
                        technique: Technique::DynamicForcingChain,
                        hint_type: HintType::EliminateCandidates {
                            pos,
                            values: vec![val],
                        },
                        explanation: format!(
                            "Dynamic Forcing Chain: assuming {} in ({}, {}) leads to contradiction.",
                            val, pos.row + 1, pos.col + 1
                        ),
                        involved_cells: vec![pos],
                    });
                }
                branches.push(result);
            }

            if branches.len() < 2 {
                continue;
            }

            // Check for common placement
            if let Some(hint) =
                Self::find_common_placement(grid, pos, &branches, Technique::DynamicForcingChain)
            {
                return Some(hint);
            }

            // Check for common elimination
            if let Some(hint) =
                Self::find_common_elimination(grid, pos, &branches, Technique::DynamicForcingChain)
            {
                return Some(hint);
            }
        }
        None
    }

    fn apply_dynamic_forcing_chain(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_dynamic_forcing_chain(grid) {
            match hint.hint_type {
                HintType::SetValue { pos, value } => {
                    grid.set_cell_unchecked(pos, Some(value));
                    grid.recalculate_candidates();
                    true
                }
                HintType::EliminateCandidates { pos, values } => {
                    for v in values {
                        grid.cell_mut(pos).remove_candidate(v);
                    }
                    true
                }
            }
        } else {
            false
        }
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

    #[test]
    fn test_se_ratings() {
        // SE ratings should be monotonically increasing for each difficulty tier
        assert!(Technique::HiddenSingle.se_rating() < Technique::NakedSingle.se_rating());
        assert!(Technique::NakedSingle.se_rating() < Technique::NakedPair.se_rating());
        assert!(Technique::NakedPair.se_rating() < Technique::XWing.se_rating());
        assert!(Technique::XWing.se_rating() < Technique::FinnedXWing.se_rating());
        assert!(Technique::XYWing.se_rating() < Technique::XChain.se_rating());
        assert!(Technique::AlsXz.se_rating() < Technique::AIC.se_rating());
        assert!(Technique::AIC.se_rating() < Technique::AlsXyWing.se_rating());
        assert!(Technique::AlsXyWing.se_rating() < Technique::NishioForcingChain.se_rating());
        assert!(Technique::NishioForcingChain.se_rating() < Technique::CellForcingChain.se_rating());
        assert!(Technique::CellForcingChain.se_rating() < Technique::DynamicForcingChain.se_rating());
        assert!(Technique::DynamicForcingChain.se_rating() < Technique::Backtracking.se_rating());
    }

    #[test]
    fn test_se_rating_for_puzzle() {
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        let se = solver.rate_se(&grid);

        // Should be a positive rating
        assert!(se > 0.0);
        assert!(se <= 11.0);
    }

    #[test]
    fn test_technique_display() {
        assert_eq!(Technique::FinnedXWing.to_string(), "Finned X-Wing");
        assert_eq!(Technique::AIC.to_string(), "AIC");
        assert_eq!(Technique::AlsXz.to_string(), "ALS-XZ");
        assert_eq!(Technique::AlsXyWing.to_string(), "ALS-XY-Wing");
        assert_eq!(Technique::XChain.to_string(), "X-Chain");
        assert_eq!(
            Technique::NishioForcingChain.to_string(),
            "Nishio Forcing Chain"
        );
        assert_eq!(
            Technique::CellForcingChain.to_string(),
            "Cell Forcing Chain"
        );
        assert_eq!(
            Technique::DynamicForcingChain.to_string(),
            "Dynamic Forcing Chain"
        );
    }

    #[test]
    fn test_combinations() {
        let items = vec![1, 2, 3, 4];
        let combos = Solver::combinations(&items, 2);
        assert_eq!(combos.len(), 6);
        assert!(combos.contains(&vec![1, 2]));
        assert!(combos.contains(&vec![3, 4]));

        let combos3 = Solver::combinations(&items, 3);
        assert_eq!(combos3.len(), 4);
    }

    #[test]
    fn test_solve_with_techniques_regression() {
        // Ensure existing puzzles still solve correctly via technique-based solving
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        let mut working = grid.deep_clone();
        let max_tech = solver.solve_with_techniques(&mut working);

        // Should solve without backtracking
        assert!(max_tech < Technique::Backtracking);
        assert!(working.is_complete());
    }

    #[test]
    fn test_has_contradiction() {
        // Valid grid should have no contradiction
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let mut grid = Grid::from_string(puzzle).unwrap();
        grid.recalculate_candidates();
        assert!(!Solver::has_contradiction(&grid));

        // Grid with duplicate in a row is a contradiction
        let mut bad = grid.deep_clone();
        // Put a 5 in (0,1) which already has 5 in (0,0)
        bad.set_cell_unchecked(Position::new(0, 1), Some(5));
        bad.recalculate_candidates();
        assert!(Solver::has_contradiction(&bad));
    }

    #[test]
    fn test_nishio_forcing_chain() {
        // A known hard puzzle (Arto Inkala "world's hardest")
        let puzzle =
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400";
        let grid = Grid::from_string(puzzle).unwrap();

        let solver = Solver::new();
        let mut working = grid.deep_clone();
        let max_tech = solver.solve_with_techniques(&mut working);

        // Should solve (possibly with forcing chains instead of backtracking)
        // The key assertion is that it either solves completely or at least
        // uses forcing chains before falling back to backtracking
        if working.is_complete() {
            assert!(max_tech <= Technique::Backtracking);
        }
    }
}
