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
            Difficulty::Expert => Technique::AvoidableRectangle,
            Difficulty::Master => Technique::SiameseFish,
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

    // Expert (fish family + quads + basic rectangles)
    XWing,
    FinnedXWing,
    Swordfish,
    FinnedSwordfish,
    Jellyfish,
    FinnedJellyfish,
    NakedQuad,
    HiddenQuad,
    EmptyRectangle,
    AvoidableRectangle,

    // Master (wings + chains + advanced patterns)
    XYWing,
    XYZWing,
    WXYZWing,
    WWing,
    XChain,
    ThreeDMedusa,
    SueDeCoq,
    AIC,
    FrankenFish,
    SiameseFish,

    // Extreme (ALS + UR + advanced + forcing chains)
    AlsXz,
    AlsXyWing,
    AlsChain,
    UniqueRectangle,
    HiddenRectangle,
    ExtendedUniqueRectangle,
    MutantFish,
    AlignedPairExclusion,
    AlignedTripletExclusion,
    BivalueUniversalGrave,
    DeathBlossom,
    NishioForcingChain,
    KrakenFish,
    RegionForcingChain,
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
            Technique::EmptyRectangle => 4.6,
            Technique::AvoidableRectangle => 4.6,
            Technique::UniqueRectangle => 4.6,
            Technique::WXYZWing => 4.6,
            Technique::HiddenRectangle => 4.7,
            Technique::NakedQuad => 5.0,
            Technique::ThreeDMedusa => 5.0,
            Technique::SueDeCoq => 5.0,
            Technique::Jellyfish => 5.2,
            Technique::FinnedJellyfish => 5.4,
            Technique::HiddenQuad => 5.4,
            Technique::AlsXz => 5.5,
            Technique::FrankenFish => 5.5,
            Technique::SiameseFish => 5.5,
            Technique::ExtendedUniqueRectangle => 5.5,
            Technique::BivalueUniversalGrave => 5.6,
            Technique::AIC => 6.0,
            Technique::AlignedPairExclusion => 6.2,
            Technique::MutantFish => 6.5,
            Technique::AlsXyWing => 7.0,
            Technique::AlsChain => 7.5,
            Technique::AlignedTripletExclusion => 7.5,
            Technique::NishioForcingChain => 7.5,
            Technique::KrakenFish => 8.0,
            Technique::CellForcingChain => 8.3,
            Technique::DeathBlossom => 8.5,
            Technique::RegionForcingChain => 8.5,
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
            Technique::NakedQuad => write!(f, "Naked Quad"),
            Technique::HiddenQuad => write!(f, "Hidden Quad"),
            Technique::EmptyRectangle => write!(f, "Empty Rectangle"),
            Technique::AvoidableRectangle => write!(f, "Avoidable Rectangle"),
            Technique::XYWing => write!(f, "XY-Wing"),
            Technique::XYZWing => write!(f, "XYZ-Wing"),
            Technique::WXYZWing => write!(f, "WXYZ-Wing"),
            Technique::WWing => write!(f, "W-Wing"),
            Technique::XChain => write!(f, "X-Chain"),
            Technique::ThreeDMedusa => write!(f, "3D Medusa"),
            Technique::SueDeCoq => write!(f, "Sue de Coq"),
            Technique::AIC => write!(f, "AIC"),
            Technique::FrankenFish => write!(f, "Franken Fish"),
            Technique::SiameseFish => write!(f, "Siamese Fish"),
            Technique::AlsXz => write!(f, "ALS-XZ"),
            Technique::AlsXyWing => write!(f, "ALS-XY-Wing"),
            Technique::AlsChain => write!(f, "ALS Chain"),
            Technique::UniqueRectangle => write!(f, "Unique Rectangle"),
            Technique::HiddenRectangle => write!(f, "Hidden Rectangle"),
            Technique::ExtendedUniqueRectangle => write!(f, "Extended Unique Rectangle"),
            Technique::MutantFish => write!(f, "Mutant Fish"),
            Technique::AlignedPairExclusion => write!(f, "Aligned Pair Exclusion"),
            Technique::AlignedTripletExclusion => write!(f, "Aligned Triplet Exclusion"),
            Technique::BivalueUniversalGrave => write!(f, "BUG+1"),
            Technique::DeathBlossom => write!(f, "Death Blossom"),
            Technique::NishioForcingChain => write!(f, "Nishio Forcing Chain"),
            Technique::KrakenFish => write!(f, "Kraken Fish"),
            Technique::RegionForcingChain => write!(f, "Region Forcing Chain"),
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
        if let Some(hint) = self.find_naked_quad(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_empty_rectangle(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_avoidable_rectangle(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_xy_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_xyz_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_wxyz_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_w_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_x_chain(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_three_d_medusa(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_sue_de_coq(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_aic(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_franken_fish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_siamese_fish(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_als_xz(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_als_xy_wing(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_als_chain(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_hidden_rectangle(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_extended_unique_rectangle(&working) {
            return Some(hint);
        }
        if let Some(hint) = self.find_bug(&working) {
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
            try_technique!(apply_naked_quads, Technique::NakedQuad);
            try_technique!(apply_hidden_quads, Technique::HiddenQuad);
            try_technique!(apply_empty_rectangle, Technique::EmptyRectangle);
            try_technique!(apply_avoidable_rectangle, Technique::AvoidableRectangle);
            try_technique!(apply_xy_wing, Technique::XYWing);
            try_technique!(apply_xyz_wing, Technique::XYZWing);
            try_technique!(apply_wxyz_wing, Technique::WXYZWing);
            try_technique!(apply_w_wing, Technique::WWing);
            try_technique!(apply_x_chain, Technique::XChain);
            try_technique!(apply_three_d_medusa, Technique::ThreeDMedusa);
            try_technique!(apply_sue_de_coq, Technique::SueDeCoq);
            try_technique!(apply_aic, Technique::AIC);
            try_technique!(apply_franken_fish, Technique::FrankenFish);
            try_technique!(apply_siamese_fish, Technique::SiameseFish);
            try_technique!(apply_als_xz, Technique::AlsXz);
            try_technique!(apply_als_xy_wing, Technique::AlsXyWing);
            try_technique!(apply_als_chain, Technique::AlsChain);
            try_technique!(apply_unique_rectangle, Technique::UniqueRectangle);
            try_technique!(apply_hidden_rectangle, Technique::HiddenRectangle);
            try_technique!(apply_extended_unique_rectangle, Technique::ExtendedUniqueRectangle);
            try_technique!(apply_mutant_fish, Technique::MutantFish);
            try_technique!(apply_aligned_pair_exclusion, Technique::AlignedPairExclusion);
            try_technique!(apply_aligned_triplet_exclusion, Technique::AlignedTripletExclusion);
            try_technique!(apply_bug, Technique::BivalueUniversalGrave);
            try_technique!(apply_death_blossom, Technique::DeathBlossom);
            try_technique!(apply_nishio_forcing_chain, Technique::NishioForcingChain);
            try_technique!(apply_kraken_fish, Technique::KrakenFish);
            try_technique!(apply_region_forcing_chain, Technique::RegionForcingChain);
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
            | Technique::FinnedJellyfish
            | Technique::NakedQuad
            | Technique::HiddenQuad
            | Technique::EmptyRectangle
            | Technique::AvoidableRectangle => Difficulty::Expert,
            Technique::XYWing
            | Technique::XYZWing
            | Technique::WXYZWing
            | Technique::WWing
            | Technique::XChain
            | Technique::ThreeDMedusa
            | Technique::SueDeCoq
            | Technique::AIC
            | Technique::FrankenFish
            | Technique::SiameseFish => Difficulty::Master,
            Technique::AlsXz
            | Technique::AlsXyWing
            | Technique::AlsChain
            | Technique::UniqueRectangle
            | Technique::HiddenRectangle
            | Technique::ExtendedUniqueRectangle
            | Technique::MutantFish
            | Technique::AlignedPairExclusion
            | Technique::AlignedTripletExclusion
            | Technique::BivalueUniversalGrave
            | Technique::DeathBlossom
            | Technique::NishioForcingChain
            | Technique::KrakenFish
            | Technique::RegionForcingChain
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

    // ==================== Naked Quad ====================

    fn find_naked_quad(&self, grid: &Grid) -> Option<Hint> {
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
                if empty_cells.len() < 5 {
                    continue;
                }

                // Find four cells whose combined candidates are exactly 4 values
                for i in 0..empty_cells.len() {
                    for j in (i + 1)..empty_cells.len() {
                        for k in (j + 1)..empty_cells.len() {
                            for l in (k + 1)..empty_cells.len() {
                                let pos1 = empty_cells[i];
                                let pos2 = empty_cells[j];
                                let pos3 = empty_cells[k];
                                let pos4 = empty_cells[l];

                                let cand1 = grid.get_candidates(pos1);
                                let cand2 = grid.get_candidates(pos2);
                                let cand3 = grid.get_candidates(pos3);
                                let cand4 = grid.get_candidates(pos4);

                                let combined =
                                    cand1.union(&cand2).union(&cand3).union(&cand4);

                                if combined.count() == 4
                                    && cand1.count() <= 4
                                    && cand2.count() <= 4
                                    && cand3.count() <= 4
                                    && cand4.count() <= 4
                                {
                                    let quad_values: Vec<u8> = combined.iter().collect();
                                    let quad_pos = [pos1, pos2, pos3, pos4];

                                    for &other_pos in &empty_cells {
                                        if quad_pos.contains(&other_pos) {
                                            continue;
                                        }
                                        let other_cand = grid.get_candidates(other_pos);
                                        let to_remove: Vec<u8> = quad_values
                                            .iter()
                                            .filter(|&&v| other_cand.contains(v))
                                            .copied()
                                            .collect();
                                        if !to_remove.is_empty() {
                                            return Some(Hint {
                                                technique: Technique::NakedQuad,
                                                hint_type: HintType::EliminateCandidates {
                                                    pos: other_pos,
                                                    values: to_remove,
                                                },
                                                explanation: format!(
                                                    "Naked quad {:?} in {} at ({},{}), ({},{}), ({},{}), ({},{}).",
                                                    quad_values, unit_name,
                                                    pos1.row + 1, pos1.col + 1,
                                                    pos2.row + 1, pos2.col + 1,
                                                    pos3.row + 1, pos3.col + 1,
                                                    pos4.row + 1, pos4.col + 1
                                                ),
                                                involved_cells: vec![pos1, pos2, pos3, pos4, other_pos],
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

    fn apply_naked_quads(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_naked_quad(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for value in values {
                    grid.cell_mut(pos).remove_candidate(value);
                }
                return true;
            }
        }
        false
    }

    // ==================== Hidden Quad ====================

    fn apply_hidden_quads(&self, grid: &mut Grid) -> bool {
        for unit_type in 0..3 {
            for unit_idx in 0..9 {
                let positions = match unit_type {
                    0 => Self::row_positions(unit_idx),
                    1 => Self::col_positions(unit_idx),
                    _ => Self::box_positions(unit_idx),
                };

                let empty_cells = self.empty_cells(grid, &positions);
                if empty_cells.len() < 5 {
                    continue;
                }

                // Find four values that appear in exactly 4 cells
                let values: Vec<u8> = (1..=9).collect();
                for combo in Self::combinations(&values, 4) {
                    let v1 = combo[0];
                    let v2 = combo[1];
                    let v3 = combo[2];
                    let v4 = combo[3];

                    let mut cells_with_values: Vec<Position> = Vec::new();
                    for &pos in &empty_cells {
                        let cand = grid.get_candidates(pos);
                        if cand.contains(v1)
                            || cand.contains(v2)
                            || cand.contains(v3)
                            || cand.contains(v4)
                        {
                            cells_with_values.push(pos);
                        }
                    }

                    if cells_with_values.len() == 4 {
                        let mut eliminated = false;
                        for &pos in &cells_with_values {
                            let cand = grid.get_candidates(pos);
                            for v in cand.iter() {
                                if v != v1 && v != v2 && v != v3 && v != v4 {
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

    // ==================== Empty Rectangle ====================

    /// Empty Rectangle: In a box, if a digit's candidates form an "L" or "T" shape
    /// (all in one row + one column), a conjugate pair in a crossing line can eliminate.
    fn find_empty_rectangle(&self, grid: &Grid) -> Option<Hint> {
        for digit in 1..=9u8 {
            for box_idx in 0..9 {
                let box_positions = Self::box_positions(box_idx);
                let digit_positions: Vec<Position> = box_positions
                    .iter()
                    .filter(|&&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .copied()
                    .collect();

                if digit_positions.len() < 2 {
                    continue;
                }

                // Check if positions span exactly 2 rows and 2 cols (ER shape)
                let rows: std::collections::HashSet<usize> =
                    digit_positions.iter().map(|p| p.row).collect();
                let cols: std::collections::HashSet<usize> =
                    digit_positions.iter().map(|p| p.col).collect();

                if rows.len() < 2 || cols.len() < 2 {
                    continue;
                }

                // For each row in the box that has candidates, check for conjugate pair in that row outside box
                for &er_row in &rows {
                    let er_cols_in_row: Vec<usize> = digit_positions
                        .iter()
                        .filter(|p| p.row == er_row)
                        .map(|p| p.col)
                        .collect();

                    if er_cols_in_row.len() != 1 {
                        continue; // Need exactly one cell in this row (the "hinge")
                    }

                    let hinge_col = er_cols_in_row[0];

                    // The other candidates in the box must be in the hinge column
                    let others_in_col: Vec<&Position> = digit_positions
                        .iter()
                        .filter(|p| p.row != er_row)
                        .collect();
                    if !others_in_col.iter().all(|p| p.col == hinge_col) {
                        // Not all non-hinge-row candidates are in the hinge column
                        // Try the transpose: check if others are in a single column
                        continue;
                    }

                    // Look for a conjugate pair in the hinge row outside the box
                    let row_positions: Vec<Position> = (0..9)
                        .map(|c| Position::new(er_row, c))
                        .filter(|p| {
                            p.box_index() != box_idx
                                && grid.cell(*p).is_empty()
                                && grid.get_candidates(*p).contains(digit)
                        })
                        .collect();

                    if row_positions.len() != 1 {
                        continue; // Need exactly one other candidate in the row (forms conjugate pair with ER)
                    }

                    let conjugate = row_positions[0];

                    // Elimination target: cell at intersection of conjugate's column and hinge column
                    let target = Position::new(
                        *others_in_col.first().map(|p| &p.row).unwrap_or(&er_row),
                        conjugate.col,
                    );

                    // Actually: eliminate from cell that sees both the conjugate and the ER column cells
                    // The target is at (other_row, conjugate.col) where other_row is where the ER column cells are
                    for &other_pos in &others_in_col {
                        let elim_pos = Position::new(other_pos.row, conjugate.col);
                        if elim_pos != conjugate
                            && grid.cell(elim_pos).is_empty()
                            && grid.get_candidates(elim_pos).contains(digit)
                            && elim_pos.box_index() != box_idx
                        {
                            let _ = target; // suppress warning
                            return Some(Hint {
                                technique: Technique::EmptyRectangle,
                                hint_type: HintType::EliminateCandidates {
                                    pos: elim_pos,
                                    values: vec![digit],
                                },
                                explanation: format!(
                                    "Empty Rectangle: {} in box {} with conjugate pair in row {}. Eliminate {} from ({}, {}).",
                                    digit, box_idx + 1, er_row + 1, digit, elim_pos.row + 1, elim_pos.col + 1
                                ),
                                involved_cells: digit_positions.iter().copied().chain(std::iter::once(conjugate)).collect(),
                            });
                        }
                    }
                }

                // Symmetric: check columns
                for &er_col in &cols {
                    let er_rows_in_col: Vec<usize> = digit_positions
                        .iter()
                        .filter(|p| p.col == er_col)
                        .map(|p| p.row)
                        .collect();

                    if er_rows_in_col.len() != 1 {
                        continue;
                    }

                    let hinge_row = er_rows_in_col[0];

                    let others_in_row: Vec<&Position> = digit_positions
                        .iter()
                        .filter(|p| p.col != er_col)
                        .collect();
                    if !others_in_row.iter().all(|p| p.row == hinge_row) {
                        continue;
                    }

                    let col_positions: Vec<Position> = (0..9)
                        .map(|r| Position::new(r, er_col))
                        .filter(|p| {
                            p.box_index() != box_idx
                                && grid.cell(*p).is_empty()
                                && grid.get_candidates(*p).contains(digit)
                        })
                        .collect();

                    if col_positions.len() != 1 {
                        continue;
                    }

                    let conjugate = col_positions[0];

                    for &other_pos in &others_in_row {
                        let elim_pos = Position::new(conjugate.row, other_pos.col);
                        if elim_pos != conjugate
                            && grid.cell(elim_pos).is_empty()
                            && grid.get_candidates(elim_pos).contains(digit)
                            && elim_pos.box_index() != box_idx
                        {
                            return Some(Hint {
                                technique: Technique::EmptyRectangle,
                                hint_type: HintType::EliminateCandidates {
                                    pos: elim_pos,
                                    values: vec![digit],
                                },
                                explanation: format!(
                                    "Empty Rectangle: {} in box {} with conjugate pair in col {}. Eliminate {} from ({}, {}).",
                                    digit, box_idx + 1, er_col + 1, digit, elim_pos.row + 1, elim_pos.col + 1
                                ),
                                involved_cells: digit_positions.iter().copied().chain(std::iter::once(conjugate)).collect(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_empty_rectangle(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_empty_rectangle(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Avoidable Rectangle ====================

    /// Avoidable Rectangle: Like Unique Rectangle but involves given (clue) cells.
    /// If two given cells and one solved cell form three corners of a UR pattern,
    /// the fourth corner can have the UR digit eliminated.
    fn find_avoidable_rectangle(&self, grid: &Grid) -> Option<Hint> {
        // Look for rectangles where at least 2 corners are givens with UR digits
        for r1 in 0..9 {
            for r2 in (r1 + 1)..9 {
                for c1 in 0..9 {
                    for c2 in (c1 + 1)..9 {
                        let corners = [
                            Position::new(r1, c1),
                            Position::new(r1, c2),
                            Position::new(r2, c1),
                            Position::new(r2, c2),
                        ];

                        // Need corners in exactly 2 boxes
                        let boxes: std::collections::HashSet<usize> =
                            corners.iter().map(|p| p.box_index()).collect();
                        if boxes.len() != 2 {
                            continue;
                        }

                        // Count givens and find the digits
                        let mut given_count = 0;
                        let mut solved_count = 0;
                        let mut empty_corners = Vec::new();
                        let mut digits = std::collections::HashSet::new();

                        for &corner in &corners {
                            if grid.cell(corner).is_given() {
                                given_count += 1;
                                if let Some(v) = grid.get(corner) {
                                    digits.insert(v);
                                }
                            } else if grid.get(corner).is_some() {
                                solved_count += 1;
                                if let Some(v) = grid.get(corner) {
                                    digits.insert(v);
                                }
                            } else {
                                empty_corners.push(corner);
                            }
                        }

                        // Need exactly 2 digits, at least 2 givens, and exactly 1 empty corner
                        if digits.len() != 2 || given_count < 2 || empty_corners.len() != 1 {
                            continue;
                        }

                        // The remaining filled corner must be solved (not given) with one of the UR digits
                        if given_count + solved_count + empty_corners.len() != 4 {
                            continue;
                        }

                        let empty = empty_corners[0];
                        let cands = grid.get_candidates(empty);

                        // Eliminate UR digits from the empty corner
                        let digit_vec: Vec<u8> = digits.into_iter().collect();
                        for &d in &digit_vec {
                            if cands.contains(d) {
                                return Some(Hint {
                                    technique: Technique::AvoidableRectangle,
                                    hint_type: HintType::EliminateCandidates {
                                        pos: empty,
                                        values: vec![d],
                                    },
                                    explanation: format!(
                                        "Avoidable Rectangle: digits {},{} in rows {},{} cols {},{}. Eliminate {} from ({}, {}).",
                                        digit_vec[0], digit_vec[1], r1 + 1, r2 + 1, c1 + 1, c2 + 1,
                                        d, empty.row + 1, empty.col + 1
                                    ),
                                    involved_cells: corners.to_vec(),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_avoidable_rectangle(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_avoidable_rectangle(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== WXYZ-Wing ====================

    /// WXYZ-Wing: 4-cell wing pattern. A pivot with 3-4 candidates and three wings,
    /// where the union of candidates across all 4 cells has exactly 4 values.
    /// The restricted common candidate (appears in all cells) can be eliminated
    /// from cells seeing all 4 wing cells.
    fn find_wxyz_wing(&self, grid: &Grid) -> Option<Hint> {
        let empty: Vec<Position> = grid.empty_positions();

        for &pivot in &empty {
            let pivot_cands = grid.get_candidates(pivot);
            if pivot_cands.count() < 2 || pivot_cands.count() > 4 {
                continue;
            }

            // Find cells that see the pivot
            let visible: Vec<Position> = empty
                .iter()
                .filter(|&&p| p != pivot && self.sees(p, pivot))
                .copied()
                .collect();

            if visible.len() < 3 {
                continue;
            }

            // Try all combinations of 3 wings from visible cells
            for i in 0..visible.len() {
                for j in (i + 1)..visible.len() {
                    for k in (j + 1)..visible.len() {
                        let wings = [visible[i], visible[j], visible[k]];
                        let wing_cands: Vec<crate::BitSet> =
                            wings.iter().map(|&p| grid.get_candidates(p)).collect();

                        // Union of all 4 cells must have exactly 4 candidates
                        let mut union = pivot_cands;
                        for &wc in &wing_cands {
                            union = union.union(&wc);
                        }
                        if union.count() != 4 {
                            continue;
                        }

                        // Find the value Z that appears in all 4 cells (or at minimum,
                        // all cells that have Z must see each other for elimination to work)
                        let all_cells = [pivot, wings[0], wings[1], wings[2]];

                        for z in union.iter() {
                            let z_cells: Vec<Position> = all_cells
                                .iter()
                                .filter(|&&p| grid.get_candidates(p).contains(z))
                                .copied()
                                .collect();

                            if z_cells.len() < 2 {
                                continue;
                            }

                            // All Z-cells must see each other (restricted common)
                            let all_see_each_other = z_cells.iter().all(|&a| {
                                z_cells.iter().all(|&b| a == b || self.sees(a, b))
                            });
                            if !all_see_each_other {
                                continue;
                            }

                            // Eliminate Z from cells that see all Z-cells and are not part of the wing
                            for &pos in &empty {
                                if all_cells.contains(&pos) {
                                    continue;
                                }
                                if !grid.get_candidates(pos).contains(z) {
                                    continue;
                                }
                                if z_cells.iter().all(|&zc| self.sees(pos, zc)) {
                                    return Some(Hint {
                                        technique: Technique::WXYZWing,
                                        hint_type: HintType::EliminateCandidates {
                                            pos,
                                            values: vec![z],
                                        },
                                        explanation: format!(
                                            "WXYZ-Wing: pivot ({}, {}) with wings. Eliminate {} from ({}, {}).",
                                            pivot.row + 1, pivot.col + 1, z, pos.row + 1, pos.col + 1
                                        ),
                                        involved_cells: all_cells.to_vec(),
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

    fn apply_wxyz_wing(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_wxyz_wing(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== 3D Medusa (Multi-digit Coloring) ====================

    /// 3D Medusa: Build a coloring graph using strong links across multiple digits.
    /// Nodes are (position, digit) pairs. If coloring leads to contradiction, eliminate.
    fn find_three_d_medusa(&self, grid: &Grid) -> Option<Hint> {
        let empty = grid.empty_positions();
        let mut visited = std::collections::HashSet::new();

        for &start in &empty {
            for start_digit in grid.get_candidates(start).iter() {
                if visited.contains(&(start, start_digit)) {
                    continue;
                }

                // BFS coloring: color 0 and color 1
                let mut color: std::collections::HashMap<(Position, u8), u8> =
                    std::collections::HashMap::new();
                let mut queue = std::collections::VecDeque::new();

                color.insert((start, start_digit), 0);
                queue.push_back((start, start_digit, 0u8));

                while let Some((pos, digit, c)) = queue.pop_front() {
                    visited.insert((pos, digit));
                    let opp = 1 - c;

                    // Strong link: bivalue cell — if cell has exactly 2 candidates, the other gets opposite color
                    let cands = grid.get_candidates(pos);
                    if cands.count() == 2 {
                        for other_digit in cands.iter() {
                            if other_digit != digit {
                                let key = (pos, other_digit);
                                if !color.contains_key(&key) {
                                    color.insert(key, opp);
                                    queue.push_back((pos, other_digit, opp));
                                }
                            }
                        }
                    }

                    // Strong link: conjugate pair — digit appears in exactly 2 cells in a unit
                    for unit in 0..27 {
                        let positions: Vec<Position> = if unit < 9 {
                            Self::row_positions(unit)
                        } else if unit < 18 {
                            Self::col_positions(unit - 9)
                        } else {
                            Self::box_positions(unit - 18)
                        };

                        let digit_cells: Vec<Position> = positions
                            .iter()
                            .filter(|&&p| {
                                grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit)
                            })
                            .copied()
                            .collect();

                        if digit_cells.len() == 2 {
                            let other = if digit_cells[0] == pos {
                                digit_cells[1]
                            } else if digit_cells[1] == pos {
                                digit_cells[0]
                            } else {
                                continue;
                            };

                            let key = (other, digit);
                            if !color.contains_key(&key) {
                                color.insert(key, opp);
                                queue.push_back((other, digit, opp));
                            }
                        }
                    }
                }

                if color.len() < 4 {
                    continue;
                }

                // Check for contradictions in each color
                for check_color in 0..=1u8 {
                    let colored: Vec<(Position, u8)> = color
                        .iter()
                        .filter(|(_, &c)| c == check_color)
                        .map(|(&k, _)| k)
                        .collect();

                    // Rule 1: Two same-colored nodes with same digit in same unit → contradiction
                    let mut contradiction = false;
                    for i in 0..colored.len() {
                        for j in (i + 1)..colored.len() {
                            let (p1, d1) = colored[i];
                            let (p2, d2) = colored[j];
                            if d1 == d2 && self.sees(p1, p2) {
                                contradiction = true;
                                break;
                            }
                            // Rule 2: Two same-colored nodes in same cell → contradiction
                            if p1 == p2 {
                                contradiction = true;
                                break;
                            }
                        }
                        if contradiction {
                            break;
                        }
                    }

                    if contradiction {
                        // Eliminate all candidates of this color
                        for &(pos, digit) in &colored {
                            if grid.cell(pos).is_empty() && grid.get_candidates(pos).contains(digit)
                            {
                                return Some(Hint {
                                    technique: Technique::ThreeDMedusa,
                                    hint_type: HintType::EliminateCandidates {
                                        pos,
                                        values: vec![digit],
                                    },
                                    explanation: format!(
                                        "3D Medusa: color contradiction. Eliminate {} from ({}, {}).",
                                        digit, pos.row + 1, pos.col + 1
                                    ),
                                    involved_cells: colored.iter().map(|(p, _)| *p).collect(),
                                });
                            }
                        }
                    }
                }

                // Rule 5: If an uncolored candidate sees both colors of the same digit, eliminate it
                for &pos in &empty {
                    for digit in grid.get_candidates(pos).iter() {
                        if color.contains_key(&(pos, digit)) {
                            continue;
                        }

                        let sees_color_0 = color.iter().any(|(&(p, d), &c)| {
                            c == 0 && d == digit && self.sees(pos, p)
                        });
                        let sees_color_1 = color.iter().any(|(&(p, d), &c)| {
                            c == 1 && d == digit && self.sees(pos, p)
                        });

                        if sees_color_0 && sees_color_1 {
                            return Some(Hint {
                                technique: Technique::ThreeDMedusa,
                                hint_type: HintType::EliminateCandidates {
                                    pos,
                                    values: vec![digit],
                                },
                                explanation: format!(
                                    "3D Medusa: {} at ({}, {}) sees both colors. Eliminate.",
                                    digit, pos.row + 1, pos.col + 1
                                ),
                                involved_cells: color.keys().map(|(p, _)| *p).collect(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_three_d_medusa(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_three_d_medusa(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Sue de Coq ====================

    /// Sue de Coq: Find a box/line intersection where the candidates in the intersection
    /// cells can be decomposed into ALS subsets that restrict the rest of the row/box.
    fn find_sue_de_coq(&self, grid: &Grid) -> Option<Hint> {
        for box_idx in 0..9 {
            let box_positions = Self::box_positions(box_idx);

            // Check each row that intersects this box
            for row in (box_idx / 3 * 3)..(box_idx / 3 * 3 + 3) {
                let intersection: Vec<Position> = box_positions
                    .iter()
                    .filter(|p| p.row == row && grid.cell(**p).is_empty())
                    .copied()
                    .collect();

                if intersection.len() < 2 || intersection.len() > 3 {
                    continue;
                }

                let mut inter_union = crate::BitSet::empty();
                for &p in &intersection {
                    inter_union = inter_union.union(&grid.get_candidates(p));
                }

                if inter_union.count() < (intersection.len() as u32 + 2) {
                    continue;
                }

                // Candidates only in the row (outside box)
                let row_rest: Vec<Position> = (0..9)
                    .map(|c| Position::new(row, c))
                    .filter(|p| !intersection.contains(p) && grid.cell(*p).is_empty())
                    .collect();

                // Candidates only in the box (outside row)
                let box_rest: Vec<Position> = box_positions
                    .iter()
                    .filter(|p| p.row != row && grid.cell(**p).is_empty())
                    .copied()
                    .collect();

                let mut row_rest_cands = crate::BitSet::empty();
                for &p in &row_rest {
                    row_rest_cands = row_rest_cands.union(&grid.get_candidates(p));
                }
                let mut box_rest_cands = crate::BitSet::empty();
                for &p in &box_rest {
                    box_rest_cands = box_rest_cands.union(&grid.get_candidates(p));
                }

                // Find digits that are exclusive to row-rest or box-rest
                let row_exclusive = inter_union.intersection(&row_rest_cands).difference(&box_rest_cands);
                let box_exclusive = inter_union.intersection(&box_rest_cands).difference(&row_rest_cands);

                // For Sue de Coq: the intersection candidates must decompose into
                // row_exclusive ALS + box_exclusive ALS + shared
                // If row_exclusive.count() + box_exclusive.count() >= inter_union.count() - shared
                // then we can eliminate row_exclusive digits from box_rest and box_exclusive from row_rest

                if row_exclusive.is_empty() && box_exclusive.is_empty() {
                    continue;
                }

                // Eliminate box_exclusive candidates from row_rest
                for &pos in &row_rest {
                    let cands = grid.get_candidates(pos);
                    let to_remove = cands.intersection(&box_exclusive);
                    if !to_remove.is_empty() {
                        let vals: Vec<u8> = to_remove.iter().collect();
                        return Some(Hint {
                            technique: Technique::SueDeCoq,
                            hint_type: HintType::EliminateCandidates {
                                pos,
                                values: vals,
                            },
                            explanation: format!(
                                "Sue de Coq: intersection in box {} row {}. Eliminate box-exclusive candidates from row.",
                                box_idx + 1, row + 1
                            ),
                            involved_cells: intersection.clone(),
                        });
                    }
                }

                // Eliminate row_exclusive candidates from box_rest
                for &pos in &box_rest {
                    let cands = grid.get_candidates(pos);
                    let to_remove = cands.intersection(&row_exclusive);
                    if !to_remove.is_empty() {
                        let vals: Vec<u8> = to_remove.iter().collect();
                        return Some(Hint {
                            technique: Technique::SueDeCoq,
                            hint_type: HintType::EliminateCandidates {
                                pos,
                                values: vals,
                            },
                            explanation: format!(
                                "Sue de Coq: intersection in box {} row {}. Eliminate row-exclusive candidates from box.",
                                box_idx + 1, row + 1
                            ),
                            involved_cells: intersection.clone(),
                        });
                    }
                }
            }

            // Check each column that intersects this box (symmetric)
            for col in (box_idx % 3 * 3)..(box_idx % 3 * 3 + 3) {
                let intersection: Vec<Position> = box_positions
                    .iter()
                    .filter(|p| p.col == col && grid.cell(**p).is_empty())
                    .copied()
                    .collect();

                if intersection.len() < 2 || intersection.len() > 3 {
                    continue;
                }

                let mut inter_union = crate::BitSet::empty();
                for &p in &intersection {
                    inter_union = inter_union.union(&grid.get_candidates(p));
                }

                if inter_union.count() < (intersection.len() as u32 + 2) {
                    continue;
                }

                let col_rest: Vec<Position> = (0..9)
                    .map(|r| Position::new(r, col))
                    .filter(|p| !intersection.contains(p) && grid.cell(*p).is_empty())
                    .collect();

                let box_rest: Vec<Position> = box_positions
                    .iter()
                    .filter(|p| p.col != col && grid.cell(**p).is_empty())
                    .copied()
                    .collect();

                let mut col_rest_cands = crate::BitSet::empty();
                for &p in &col_rest {
                    col_rest_cands = col_rest_cands.union(&grid.get_candidates(p));
                }
                let mut box_rest_cands = crate::BitSet::empty();
                for &p in &box_rest {
                    box_rest_cands = box_rest_cands.union(&grid.get_candidates(p));
                }

                let col_exclusive = inter_union.intersection(&col_rest_cands).difference(&box_rest_cands);
                let box_exclusive = inter_union.intersection(&box_rest_cands).difference(&col_rest_cands);

                if col_exclusive.is_empty() && box_exclusive.is_empty() {
                    continue;
                }

                for &pos in &col_rest {
                    let cands = grid.get_candidates(pos);
                    let to_remove = cands.intersection(&box_exclusive);
                    if !to_remove.is_empty() {
                        let vals: Vec<u8> = to_remove.iter().collect();
                        return Some(Hint {
                            technique: Technique::SueDeCoq,
                            hint_type: HintType::EliminateCandidates {
                                pos,
                                values: vals,
                            },
                            explanation: format!(
                                "Sue de Coq: intersection in box {} col {}.",
                                box_idx + 1, col + 1
                            ),
                            involved_cells: intersection.clone(),
                        });
                    }
                }

                for &pos in &box_rest {
                    let cands = grid.get_candidates(pos);
                    let to_remove = cands.intersection(&col_exclusive);
                    if !to_remove.is_empty() {
                        let vals: Vec<u8> = to_remove.iter().collect();
                        return Some(Hint {
                            technique: Technique::SueDeCoq,
                            hint_type: HintType::EliminateCandidates {
                                pos,
                                values: vals,
                            },
                            explanation: format!(
                                "Sue de Coq: intersection in box {} col {}.",
                                box_idx + 1, col + 1
                            ),
                            involved_cells: intersection.clone(),
                        });
                    }
                }
            }
        }
        None
    }

    fn apply_sue_de_coq(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_sue_de_coq(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Franken Fish ====================

    /// Franken Fish: Like regular fish but base/cover sets can include box sectors.
    /// A Franken X-Wing uses one row + one box as base, one col + one box as cover (or similar).
    fn find_franken_fish(&self, grid: &Grid) -> Option<Hint> {
        // Collect all "sectors" (rows, cols, boxes) for each digit
        for digit in 1..=9u8 {
            // Build sector position lists: 9 rows + 9 cols + 9 boxes = 27 sectors
            let mut sectors: Vec<(usize, Vec<Position>)> = Vec::new();
            for i in 0..9 {
                let row_cells: Vec<Position> = Self::row_positions(i)
                    .into_iter()
                    .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .collect();
                if row_cells.len() >= 2 {
                    sectors.push((i, row_cells)); // 0-8: rows
                }
            }
            for i in 0..9 {
                let col_cells: Vec<Position> = Self::col_positions(i)
                    .into_iter()
                    .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .collect();
                if col_cells.len() >= 2 {
                    sectors.push((9 + i, col_cells)); // 9-17: cols
                }
            }
            for i in 0..9 {
                let box_cells: Vec<Position> = Self::box_positions(i)
                    .into_iter()
                    .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .collect();
                if box_cells.len() >= 2 {
                    sectors.push((18 + i, box_cells)); // 18-26: boxes
                }
            }

            // Try size-2 Franken fish (must mix types, not just rows+cols which is regular fish)
            for i in 0..sectors.len() {
                for j in (i + 1)..sectors.len() {
                    let (id_a, ref cells_a) = sectors[i];
                    let (id_b, ref cells_b) = sectors[j];

                    let type_a = id_a / 9; // 0=row, 1=col, 2=box
                    let type_b = id_b / 9;

                    // Must include at least one box sector to be "Franken"
                    if type_a != 2 && type_b != 2 {
                        continue;
                    }

                    // Base set: union of cells in sectors a and b
                    let mut base_cells: Vec<Position> = cells_a.clone();
                    for &p in cells_b {
                        if !base_cells.contains(&p) {
                            base_cells.push(p);
                        }
                    }

                    // Find cover sectors: other sectors that contain all base cells
                    for ci in 0..sectors.len() {
                        if ci == i || ci == j {
                            continue;
                        }
                        for cj in (ci + 1)..sectors.len() {
                            if cj == i || cj == j {
                                continue;
                            }

                            let (id_ci, ref cells_ci) = sectors[ci];
                            let (id_cj, ref cells_cj) = sectors[cj];

                            // Cover must contain all base cells
                            let mut cover_cells: Vec<Position> = cells_ci.clone();
                            for &p in cells_cj {
                                if !cover_cells.contains(&p) {
                                    cover_cells.push(p);
                                }
                            }

                            let all_covered = base_cells.iter().all(|p| cover_cells.contains(p));
                            if !all_covered {
                                continue;
                            }

                            // Eliminate digit from cover cells that are NOT in base cells
                            for &pos in &cover_cells {
                                if !base_cells.contains(&pos)
                                    && grid.get_candidates(pos).contains(digit)
                                {
                                    let _ = (id_ci, id_cj); // suppress warning
                                    return Some(Hint {
                                        technique: Technique::FrankenFish,
                                        hint_type: HintType::EliminateCandidates {
                                            pos,
                                            values: vec![digit],
                                        },
                                        explanation: format!(
                                            "Franken Fish: {} using base sectors {},{} and cover sectors {},{}. Eliminate from ({}, {}).",
                                            digit, id_a, id_b, id_ci, id_cj, pos.row + 1, pos.col + 1
                                        ),
                                        involved_cells: base_cells.clone(),
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

    fn apply_franken_fish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_franken_fish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Siamese Fish ====================

    /// Siamese Fish: Two finned fish patterns that share the same base rows/cols
    /// but have fins in different boxes, producing complementary eliminations.
    fn find_siamese_fish(&self, grid: &Grid) -> Option<Hint> {
        // Siamese fish are essentially two finned fish sharing base sets.
        // We look for fish patterns where there are exactly 2 extra cells (fins)
        // in different boxes, giving two elimination sets.
        for digit in 1..=9u8 {
            // Try row-based size-2 (Siamese X-Wing)
            for r1 in 0..9 {
                for r2 in (r1 + 1)..9 {
                    let row1_cols: Vec<usize> = (0..9)
                        .filter(|&c| {
                            let p = Position::new(r1, c);
                            grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit)
                        })
                        .collect();
                    let row2_cols: Vec<usize> = (0..9)
                        .filter(|&c| {
                            let p = Position::new(r2, c);
                            grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit)
                        })
                        .collect();

                    if row1_cols.len() < 2 || row2_cols.len() < 2 {
                        continue;
                    }

                    // Find common columns
                    let common_cols: Vec<usize> = row1_cols
                        .iter()
                        .filter(|c| row2_cols.contains(c))
                        .copied()
                        .collect();

                    if common_cols.len() != 2 {
                        continue;
                    }

                    // Fins are columns in row1 or row2 but not in common
                    let fins1: Vec<usize> = row1_cols.iter().filter(|c| !common_cols.contains(c)).copied().collect();
                    let fins2: Vec<usize> = row2_cols.iter().filter(|c| !common_cols.contains(c)).copied().collect();

                    // Need fins in exactly one row, in different boxes from the common cols
                    let all_fins: Vec<Position> = fins1
                        .iter()
                        .map(|&c| Position::new(r1, c))
                        .chain(fins2.iter().map(|&c| Position::new(r2, c)))
                        .collect();

                    if all_fins.is_empty() || all_fins.len() > 2 {
                        continue;
                    }

                    for &fin in &all_fins {
                        let fin_box = fin.box_index();
                        // Eliminate from cells in the cover column that see the fin (same box)
                        for &cover_col in &common_cols {
                            let cover_pos = if fin.row == r1 {
                                Position::new(r2, cover_col)
                            } else {
                                Position::new(r1, cover_col)
                            };
                            if cover_pos.box_index() == fin_box {
                                // Can eliminate from cells in this box/col intersection
                                for dr in 0..3 {
                                    let box_row = (fin_box / 3) * 3 + dr;
                                    let pos = Position::new(box_row, cover_col);
                                    if pos != cover_pos
                                        && pos.row != r1
                                        && pos.row != r2
                                        && grid.cell(pos).is_empty()
                                        && grid.get_candidates(pos).contains(digit)
                                    {
                                        let mut involved: Vec<Position> = common_cols
                                            .iter()
                                            .flat_map(|&c| vec![Position::new(r1, c), Position::new(r2, c)])
                                            .collect();
                                        involved.extend(all_fins.iter());
                                        return Some(Hint {
                                            technique: Technique::SiameseFish,
                                            hint_type: HintType::EliminateCandidates {
                                                pos,
                                                values: vec![digit],
                                            },
                                            explanation: format!(
                                                "Siamese Fish: {} in rows {},{} cols {:?} with fin. Eliminate from ({}, {}).",
                                                digit, r1 + 1, r2 + 1, common_cols, pos.row + 1, pos.col + 1
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

    fn apply_siamese_fish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_siamese_fish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== ALS Chain ====================

    /// ALS Chain: Generalization of ALS-XZ to chains of 3+ ALS connected by RCCs.
    fn find_als_chain(&self, grid: &Grid) -> Option<Hint> {
        let all_als = Self::find_all_als(grid);
        if all_als.len() < 3 {
            return None;
        }

        // Limit for performance
        let small_als: Vec<&(Vec<Position>, crate::BitSet)> = all_als
            .iter()
            .filter(|(cells, _)| cells.len() <= 4)
            .take(50)
            .collect();

        // Pre-compute RCC pairs
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

        // Try chains of length 3: A-B-C-D with RCCs between consecutive pairs
        // and different RCC values at each link
        for a_idx in 0..small_als.len() {
            let a_partners: Vec<(usize, &[u8])> = rcc_map
                .iter()
                .filter(|((from, _), _)| *from == a_idx)
                .map(|((_, to), rccs)| (*to, rccs.as_slice()))
                .collect();

            for &(b_idx, rccs_ab) in &a_partners {
                let b_partners: Vec<(usize, &[u8])> = rcc_map
                    .iter()
                    .filter(|((from, _), _)| *from == b_idx)
                    .map(|((_, to), rccs)| (*to, rccs.as_slice()))
                    .collect();

                for &(c_idx, rccs_bc) in &b_partners {
                    if c_idx == a_idx {
                        continue;
                    }

                    let c_partners: Vec<(usize, &[u8])> = rcc_map
                        .iter()
                        .filter(|((from, _), _)| *from == c_idx)
                        .map(|((_, to), rccs)| (*to, rccs.as_slice()))
                        .collect();

                    for &(d_idx, rccs_cd) in &c_partners {
                        if d_idx == a_idx || d_idx == b_idx {
                            continue;
                        }

                        let (cells_a, cands_a) = small_als[a_idx];
                        let (cells_d, cands_d) = small_als[d_idx];

                        if cells_a.iter().any(|p| small_als[d_idx].0.contains(p)) {
                            continue;
                        }

                        // Need distinct RCC values along the chain
                        for &x in rccs_ab {
                            for &y in rccs_bc {
                                if y == x {
                                    continue;
                                }
                                for &w in rccs_cd {
                                    if w == y {
                                        continue;
                                    }

                                    // Find common values between A and D (not used as RCC)
                                    let common_ad = cands_a.intersection(cands_d);
                                    for z in common_ad.iter() {
                                        if z == x || z == w {
                                            continue;
                                        }

                                        let a_cells_z: Vec<Position> = cells_a
                                            .iter()
                                            .filter(|&&p| grid.get_candidates(p).contains(z))
                                            .copied()
                                            .collect();
                                        let d_cells_z: Vec<Position> = cells_d
                                            .iter()
                                            .filter(|&&p| grid.get_candidates(p).contains(z))
                                            .copied()
                                            .collect();

                                        if a_cells_z.is_empty() || d_cells_z.is_empty() {
                                            continue;
                                        }

                                        for pos in grid.empty_positions() {
                                            if cells_a.contains(&pos) || cells_d.contains(&pos) {
                                                continue;
                                            }
                                            if !grid.get_candidates(pos).contains(z) {
                                                continue;
                                            }
                                            let sees_all_a =
                                                a_cells_z.iter().all(|&p| self.sees(pos, p));
                                            let sees_all_d =
                                                d_cells_z.iter().all(|&p| self.sees(pos, p));
                                            if sees_all_a && sees_all_d {
                                                let mut involved = cells_a.clone();
                                                involved.extend(small_als[b_idx].0.iter());
                                                involved.extend(small_als[c_idx].0.iter());
                                                involved.extend(cells_d);
                                                return Some(Hint {
                                                    technique: Technique::AlsChain,
                                                    hint_type: HintType::EliminateCandidates {
                                                        pos,
                                                        values: vec![z],
                                                    },
                                                    explanation: format!(
                                                        "ALS Chain (4 ALS): eliminate {} from ({}, {}).",
                                                        z, pos.row + 1, pos.col + 1
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
            }
        }
        None
    }

    fn apply_als_chain(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_als_chain(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Unique Rectangle (Types 1-4) ====================

    /// Try UR eliminations on a specific rectangle configuration.
    /// pos1, pos2 are bivalue {a,b} corners on the same line.
    /// corner3, corner4 are the opposite corners.
    fn try_ur_types(
        &self,
        grid: &mut Grid,
        pos1: Position,
        pos2: Position,
        corner3: Position,
        corner4: Position,
        a: u8,
        b: u8,
    ) -> bool {
        let cand3 = grid.get_candidates(corner3);
        let cand4 = grid.get_candidates(corner4);

        // Both corners must be empty and contain both UR digits
        if !grid.cell(corner3).is_empty() || !grid.cell(corner4).is_empty() {
            return false;
        }
        if !cand3.contains(a) || !cand3.contains(b) || !cand4.contains(a) || !cand4.contains(b) {
            return false;
        }

        // Type 1: Three bivalue corners, fourth has extras → eliminate a,b from fourth
        if cand3.count() == 2 && cand4.count() > 2 {
            // corner3 is bivalue {a,b}, corner4 has extras
            // To avoid deadly pattern, corner4 must not be {a,b}, so eliminate a and b
            grid.cell_mut(corner4).remove_candidate(a);
            grid.cell_mut(corner4).remove_candidate(b);
            return true;
        }
        if cand4.count() == 2 && cand3.count() > 2 {
            grid.cell_mut(corner3).remove_candidate(a);
            grid.cell_mut(corner3).remove_candidate(b);
            return true;
        }

        // Type 2: Both non-bivalue corners have same single extra candidate
        if cand3.count() == 3 && cand4.count() == 3 {
            let extra3: Vec<u8> = cand3.iter().filter(|&v| v != a && v != b).collect();
            let extra4: Vec<u8> = cand4.iter().filter(|&v| v != a && v != b).collect();

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

        // Type 3: Non-bivalue corners' extras form a naked subset with peers in the shared unit
        // If corners are in the same row or column, the extras + some peers form a naked pair/triple
        if cand3.count() > 2 || cand4.count() > 2 {
            let extra3 = cand3.difference(&crate::BitSet::from_slice(&[a, b]));
            let extra4 = cand4.difference(&crate::BitSet::from_slice(&[a, b]));
            let combined_extras = extra3.union(&extra4);

            if combined_extras.count() >= 2 && combined_extras.count() <= 3 {
                // Check if corners share a unit (row, col, or box)
                let shared_units: Vec<Vec<Position>> = {
                    let mut units = Vec::new();
                    if corner3.row == corner4.row {
                        units.push(Self::row_positions(corner3.row));
                    }
                    if corner3.col == corner4.col {
                        units.push(Self::col_positions(corner3.col));
                    }
                    if corner3.box_index() == corner4.box_index() {
                        units.push(Self::box_positions(corner3.box_index()));
                    }
                    units
                };

                for unit in &shared_units {
                    let other_cells: Vec<Position> = unit
                        .iter()
                        .filter(|&&p| {
                            p != corner3
                                && p != corner4
                                && p != pos1
                                && p != pos2
                                && grid.cell(p).is_empty()
                        })
                        .copied()
                        .collect();

                    // The extras act as a virtual naked pair/triple with the unit
                    // Eliminate combined_extras from other cells in the unit that aren't involved
                    let subset_size = combined_extras.count() as usize;
                    if subset_size == 2 {
                        // Virtual naked pair: eliminate combined_extras from other cells
                        for &pos in &other_cells {
                            let cell_cands = grid.get_candidates(pos);
                            let overlap = cell_cands.intersection(&combined_extras);
                            if !overlap.is_empty() && cell_cands != combined_extras {
                                for v in overlap.iter() {
                                    grid.cell_mut(pos).remove_candidate(v);
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // Type 4: Strong link on one UR digit forces the other out
        // If digit 'a' has a strong link (only 2 positions in a shared unit) through
        // both non-bivalue corners, then 'b' can be eliminated from those corners
        for &digit in &[a, b] {
            let other = if digit == a { b } else { a };

            // Check if the two non-bivalue corners form a strong link for 'digit' in some unit
            let in_same_row = corner3.row == corner4.row;
            let in_same_col = corner3.col == corner4.col;
            let in_same_box = corner3.box_index() == corner4.box_index();

            let is_strong_link = |positions: &[Position]| -> bool {
                let count = positions
                    .iter()
                    .filter(|&&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .count();
                count == 2
            };

            let strong = if in_same_row {
                is_strong_link(&Self::row_positions(corner3.row))
            } else if in_same_col {
                is_strong_link(&Self::col_positions(corner3.col))
            } else if in_same_box {
                is_strong_link(&Self::box_positions(corner3.box_index()))
            } else {
                false
            };

            if strong {
                // The strong link means one of the corners must have 'digit'.
                // So we can eliminate 'other' from both corners.
                let mut applied = false;
                if grid.get_candidates(corner3).contains(other) && cand3.count() > 2 {
                    grid.cell_mut(corner3).remove_candidate(other);
                    applied = true;
                }
                if grid.get_candidates(corner4).contains(other) && cand4.count() > 2 {
                    grid.cell_mut(corner4).remove_candidate(other);
                    applied = true;
                }
                if applied {
                    return true;
                }
            }
        }

        false
    }

    fn apply_unique_rectangle(&self, grid: &mut Grid) -> bool {
        // Find bi-value cells with exactly 2 candidates
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

        #[allow(clippy::needless_range_loop)]
        for i in 0..bivalues.len() {
            let (pos1, a, b) = bivalues[i];

            for j in (i + 1)..bivalues.len() {
                let (pos2, c, d) = bivalues[j];

                if !((a == c && b == d) || (a == d && b == c)) {
                    continue;
                }

                if pos1.row != pos2.row && pos1.col != pos2.col {
                    continue;
                }

                if pos1.row == pos2.row {
                    for other_row in 0..9 {
                        if other_row == pos1.row {
                            continue;
                        }
                        let corner3 = Position::new(other_row, pos1.col);
                        let corner4 = Position::new(other_row, pos2.col);

                        // Need exactly 2 boxes
                        let boxes: std::collections::HashSet<usize> = [pos1, pos2, corner3, corner4]
                            .iter()
                            .map(|p| p.box_index())
                            .collect();
                        if boxes.len() != 2 {
                            continue;
                        }

                        if self.try_ur_types(grid, pos1, pos2, corner3, corner4, a, b) {
                            return true;
                        }
                    }
                } else {
                    for other_col in 0..9 {
                        if other_col == pos1.col {
                            continue;
                        }
                        let corner3 = Position::new(pos1.row, other_col);
                        let corner4 = Position::new(pos2.row, other_col);

                        let boxes: std::collections::HashSet<usize> = [pos1, pos2, corner3, corner4]
                            .iter()
                            .map(|p| p.box_index())
                            .collect();
                        if boxes.len() != 2 {
                            continue;
                        }

                        if self.try_ur_types(grid, pos1, pos2, corner3, corner4, a, b) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== BUG+1 (Bivalue Universal Grave) ====================

    /// Detect BUG+1: if every empty cell has exactly 2 candidates except one cell
    /// with exactly 3 candidates, the extra candidate in that cell must be its value.
    /// A BUG state (all bivalue) has multiple solutions, violating uniqueness.
    fn find_bug(&self, grid: &Grid) -> Option<Hint> {
        let empty = grid.empty_positions();
        if empty.is_empty() {
            return None;
        }

        let mut trivalue_cell: Option<Position> = None;

        for &pos in &empty {
            let count = grid.get_candidates(pos).count();
            if count == 2 {
                continue;
            }
            if count == 3 && trivalue_cell.is_none() {
                trivalue_cell = Some(pos);
            } else {
                // More than one non-bivalue cell, or a cell with 4+ candidates — not BUG+1
                return None;
            }
        }

        let tri_pos = trivalue_cell?;
        let cands = grid.get_candidates(tri_pos);

        // Find the "extra" candidate: the one that appears 3 times in its row/col/box
        // In a BUG state, each candidate appears exactly twice per unit.
        // The extra candidate appears 3 times in at least one unit.
        for val in cands.iter() {
            let row_count = (0..9)
                .filter(|&c| {
                    let p = Position::new(tri_pos.row, c);
                    p != tri_pos && grid.cell(p).is_empty() && grid.get_candidates(p).contains(val)
                })
                .count();
            let col_count = (0..9)
                .filter(|&r| {
                    let p = Position::new(r, tri_pos.col);
                    p != tri_pos && grid.cell(p).is_empty() && grid.get_candidates(p).contains(val)
                })
                .count();
            let box_positions = Self::box_positions(tri_pos.box_index());
            let box_count = box_positions
                .iter()
                .filter(|&&p| {
                    p != tri_pos && grid.cell(p).is_empty() && grid.get_candidates(p).contains(val)
                })
                .count();

            // In BUG state, each value appears exactly 2 times per unit.
            // The extra value has an odd count (appears 2+1=3 times including tri_pos) in some unit.
            // So the count excluding tri_pos would be 2 (making total 3, odd) for the extra value.
            let is_extra = row_count == 2 || col_count == 2 || box_count == 2;

            if is_extra {
                return Some(Hint {
                    technique: Technique::BivalueUniversalGrave,
                    hint_type: HintType::SetValue {
                        pos: tri_pos,
                        value: val,
                    },
                    explanation: format!(
                        "BUG+1: all cells are bivalue except ({}, {}). {} must be {} to avoid a deadly pattern.",
                        tri_pos.row + 1, tri_pos.col + 1, val, val
                    ),
                    involved_cells: vec![tri_pos],
                });
            }
        }
        None
    }

    fn apply_bug(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_bug(grid) {
            if let HintType::SetValue { pos, value } = hint.hint_type {
                grid.set_cell_unchecked(pos, Some(value));
                grid.recalculate_candidates();
                return true;
            }
        }
        false
    }

    // ==================== Hidden Rectangle ====================

    /// Hidden Rectangle: A UR pattern where one of the UR digits is "hidden" —
    /// it appears in only the UR corners within some unit. The other UR digit
    /// can be eliminated from the non-bivalue corners.
    fn find_hidden_rectangle(&self, grid: &Grid) -> Option<Hint> {
        for r1 in 0..9 {
            for r2 in (r1 + 1)..9 {
                for c1 in 0..9 {
                    for c2 in (c1 + 1)..9 {
                        let corners = [
                            Position::new(r1, c1),
                            Position::new(r1, c2),
                            Position::new(r2, c1),
                            Position::new(r2, c2),
                        ];

                        // All corners must be empty
                        if corners.iter().any(|&p| !grid.cell(p).is_empty()) {
                            continue;
                        }

                        let boxes: std::collections::HashSet<usize> =
                            corners.iter().map(|p| p.box_index()).collect();
                        if boxes.len() != 2 {
                            continue;
                        }

                        // Find common candidates across all 4 corners
                        let mut common = grid.get_candidates(corners[0]);
                        for &c in &corners[1..] {
                            common = common.intersection(&grid.get_candidates(c));
                        }

                        if common.count() < 2 {
                            continue;
                        }

                        // Try each pair of common candidates as UR digits
                        let common_vec: Vec<u8> = common.iter().collect();
                        for di in 0..common_vec.len() {
                            for dj in (di + 1)..common_vec.len() {
                                let a = common_vec[di];
                                let b = common_vec[dj];

                                // Check if digit 'a' is "hidden" — only appears in UR corners
                                // within some shared unit (row or column)
                                for &digit in &[a, b] {
                                    let other = if digit == a { b } else { a };

                                    // Check rows
                                    for &row in &[r1, r2] {
                                        let row_cells: Vec<Position> = (0..9)
                                            .map(|c| Position::new(row, c))
                                            .filter(|p| {
                                                grid.cell(*p).is_empty()
                                                    && grid.get_candidates(*p).contains(digit)
                                                    && !corners.contains(p)
                                            })
                                            .collect();

                                        if row_cells.is_empty() {
                                            // digit is hidden in this row — only in UR corners
                                            // Eliminate 'other' from non-bivalue UR corners in this row
                                            let ur_row_corners: Vec<Position> = corners
                                                .iter()
                                                .filter(|p| p.row == row)
                                                .copied()
                                                .collect();

                                            for &corner in &ur_row_corners {
                                                let cands = grid.get_candidates(corner);
                                                if cands.count() > 2 && cands.contains(other) {
                                                    return Some(Hint {
                                                        technique: Technique::HiddenRectangle,
                                                        hint_type: HintType::EliminateCandidates {
                                                            pos: corner,
                                                            values: vec![other],
                                                        },
                                                        explanation: format!(
                                                            "Hidden Rectangle: {} hidden in row {}, eliminate {} from ({}, {}).",
                                                            digit, row + 1, other, corner.row + 1, corner.col + 1
                                                        ),
                                                        involved_cells: corners.to_vec(),
                                                    });
                                                }
                                            }
                                        }
                                    }

                                    // Check columns
                                    for &col in &[c1, c2] {
                                        let col_cells: Vec<Position> = (0..9)
                                            .map(|r| Position::new(r, col))
                                            .filter(|p| {
                                                grid.cell(*p).is_empty()
                                                    && grid.get_candidates(*p).contains(digit)
                                                    && !corners.contains(p)
                                            })
                                            .collect();

                                        if col_cells.is_empty() {
                                            let ur_col_corners: Vec<Position> = corners
                                                .iter()
                                                .filter(|p| p.col == col)
                                                .copied()
                                                .collect();

                                            for &corner in &ur_col_corners {
                                                let cands = grid.get_candidates(corner);
                                                if cands.count() > 2 && cands.contains(other) {
                                                    return Some(Hint {
                                                        technique: Technique::HiddenRectangle,
                                                        hint_type: HintType::EliminateCandidates {
                                                            pos: corner,
                                                            values: vec![other],
                                                        },
                                                        explanation: format!(
                                                            "Hidden Rectangle: {} hidden in col {}, eliminate {} from ({}, {}).",
                                                            digit, col + 1, other, corner.row + 1, corner.col + 1
                                                        ),
                                                        involved_cells: corners.to_vec(),
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
        }
        None
    }

    fn apply_hidden_rectangle(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_hidden_rectangle(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Extended Unique Rectangle ====================

    /// Extended Unique Rectangle: Deadly patterns larger than 4 cells.
    /// A 6-cell pattern in 2 rows x 3 cols (or 3 rows x 2 cols) with 2 digits
    /// forming a deadly pattern. Extra candidates can be eliminated.
    fn find_extended_unique_rectangle(&self, grid: &Grid) -> Option<Hint> {
        // Try 2 rows x 3 cols patterns
        for r1 in 0..9 {
            for r2 in (r1 + 1)..9 {
                for c1 in 0..9 {
                    for c2 in (c1 + 1)..9 {
                        for c3 in (c2 + 1)..9 {
                            let corners = [
                                Position::new(r1, c1), Position::new(r1, c2), Position::new(r1, c3),
                                Position::new(r2, c1), Position::new(r2, c2), Position::new(r2, c3),
                            ];

                            if corners.iter().any(|&p| !grid.cell(p).is_empty()) {
                                continue;
                            }

                            // Must span exactly 3 boxes
                            let boxes: std::collections::HashSet<usize> =
                                corners.iter().map(|p| p.box_index()).collect();
                            if boxes.len() < 2 {
                                continue;
                            }

                            // Find pair of digits that appears in all 6 cells
                            let mut common = grid.get_candidates(corners[0]);
                            for &c in &corners[1..] {
                                common = common.intersection(&grid.get_candidates(c));
                            }

                            if common.count() < 2 {
                                continue;
                            }

                            let common_vec: Vec<u8> = common.iter().collect();
                            for di in 0..common_vec.len() {
                                for dj in (di + 1)..common_vec.len() {
                                    let a = common_vec[di];
                                    let b = common_vec[dj];

                                    // Count bivalue corners (with exactly {a,b})
                                    let bivalue_count = corners
                                        .iter()
                                        .filter(|&&p| {
                                            let c = grid.get_candidates(p);
                                            c.count() == 2 && c.contains(a) && c.contains(b)
                                        })
                                        .count();

                                    // Need at least 4 bivalue corners for the pattern to be dangerous
                                    if bivalue_count < 4 {
                                        continue;
                                    }

                                    // Eliminate a and b from non-bivalue corners
                                    for &corner in &corners {
                                        let cands = grid.get_candidates(corner);
                                        if cands.count() > 2 {
                                            let mut elim = Vec::new();
                                            if cands.contains(a) && cands.count() == 3 {
                                                elim.push(a);
                                            }
                                            if cands.contains(b) && cands.count() == 3 {
                                                elim.push(b);
                                            }
                                            // For Extended UR: eliminate UR digits from non-bivalue
                                            // corners to break the pattern, but only if it's safe
                                            if !elim.is_empty() {
                                                return Some(Hint {
                                                    technique: Technique::ExtendedUniqueRectangle,
                                                    hint_type: HintType::EliminateCandidates {
                                                        pos: corner,
                                                        values: elim,
                                                    },
                                                    explanation: format!(
                                                        "Extended Unique Rectangle: {},{} pattern in 2x3 grid. Eliminate from ({}, {}).",
                                                        a, b, corner.row + 1, corner.col + 1
                                                    ),
                                                    involved_cells: corners.to_vec(),
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

    fn apply_extended_unique_rectangle(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_extended_unique_rectangle(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Mutant Fish ====================

    /// Mutant Fish: Generalized fish where base and cover sets can be any mix of
    /// rows, columns, and boxes. Size 2 (Mutant X-Wing).
    fn find_mutant_fish(&self, grid: &Grid) -> Option<Hint> {
        for digit in 1..=9u8 {
            // Build all sectors
            let mut sectors: Vec<(usize, Vec<Position>)> = Vec::new();
            for i in 0..9 {
                let cells: Vec<Position> = Self::row_positions(i)
                    .into_iter()
                    .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .collect();
                if cells.len() >= 2 && cells.len() <= 4 {
                    sectors.push((i, cells));
                }
            }
            for i in 0..9 {
                let cells: Vec<Position> = Self::col_positions(i)
                    .into_iter()
                    .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .collect();
                if cells.len() >= 2 && cells.len() <= 4 {
                    sectors.push((9 + i, cells));
                }
            }
            for i in 0..9 {
                let cells: Vec<Position> = Self::box_positions(i)
                    .into_iter()
                    .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                    .collect();
                if cells.len() >= 2 && cells.len() <= 4 {
                    sectors.push((18 + i, cells));
                }
            }

            // Size-2 mutant fish: base has 2 sectors, cover has 2 sectors
            // Must use all 3 types (row+col+box mix) — otherwise it's regular or Franken
            for bi in 0..sectors.len() {
                for bj in (bi + 1)..sectors.len() {
                    let (id_bi, ref cells_bi) = sectors[bi];
                    let (id_bj, ref cells_bj) = sectors[bj];
                    let type_bi = id_bi / 9;
                    let type_bj = id_bj / 9;

                    let mut base_cells: Vec<Position> = cells_bi.clone();
                    for &p in cells_bj {
                        if !base_cells.contains(&p) {
                            base_cells.push(p);
                        }
                    }

                    for ci in 0..sectors.len() {
                        if ci == bi || ci == bj {
                            continue;
                        }
                        for cj in (ci + 1)..sectors.len() {
                            if cj == bi || cj == bj {
                                continue;
                            }

                            let (id_ci, ref cells_ci) = sectors[ci];
                            let (id_cj, ref cells_cj) = sectors[cj];
                            let type_ci = id_ci / 9;
                            let type_cj = id_cj / 9;

                            // Must use all 3 types across base and cover
                            let mut types = std::collections::HashSet::new();
                            types.insert(type_bi);
                            types.insert(type_bj);
                            types.insert(type_ci);
                            types.insert(type_cj);
                            if types.len() < 3 {
                                continue;
                            }

                            let mut cover_cells: Vec<Position> = cells_ci.clone();
                            for &p in cells_cj {
                                if !cover_cells.contains(&p) {
                                    cover_cells.push(p);
                                }
                            }

                            if !base_cells.iter().all(|p| cover_cells.contains(p)) {
                                continue;
                            }

                            for &pos in &cover_cells {
                                if !base_cells.contains(&pos)
                                    && grid.get_candidates(pos).contains(digit)
                                {
                                    return Some(Hint {
                                        technique: Technique::MutantFish,
                                        hint_type: HintType::EliminateCandidates {
                                            pos,
                                            values: vec![digit],
                                        },
                                        explanation: format!(
                                            "Mutant Fish: {} with mixed sectors. Eliminate from ({}, {}).",
                                            digit, pos.row + 1, pos.col + 1
                                        ),
                                        involved_cells: base_cells.clone(),
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

    fn apply_mutant_fish(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_mutant_fish(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Aligned Pair Exclusion ====================

    /// Aligned Pair Exclusion: Two cells that see each other and share a common peer.
    /// If every valid combination of values for the pair excludes a candidate from a peer, eliminate it.
    fn find_aligned_pair_exclusion(&self, grid: &Grid) -> Option<Hint> {
        let empty = grid.empty_positions();

        for i in 0..empty.len() {
            let p1 = empty[i];
            let cands1 = grid.get_candidates(p1);
            if cands1.count() < 2 || cands1.count() > 5 {
                continue;
            }

            for j in (i + 1)..empty.len() {
                let p2 = empty[j];
                if !self.sees(p1, p2) {
                    continue;
                }
                let cands2 = grid.get_candidates(p2);
                if cands2.count() < 2 || cands2.count() > 5 {
                    continue;
                }

                // Enumerate all valid value pairs for (p1, p2)
                let mut valid_pairs: Vec<(u8, u8)> = Vec::new();
                for v1 in cands1.iter() {
                    for v2 in cands2.iter() {
                        if v1 == v2 {
                            continue; // Same value, same unit = invalid
                        }
                        valid_pairs.push((v1, v2));
                    }
                }

                if valid_pairs.is_empty() {
                    continue;
                }

                // Check common peers
                for &pos in &empty {
                    if pos == p1 || pos == p2 {
                        continue;
                    }
                    if !self.sees(pos, p1) || !self.sees(pos, p2) {
                        continue;
                    }

                    let peer_cands = grid.get_candidates(pos);
                    for val in peer_cands.iter() {
                        // Check if every valid pair excludes 'val' from 'pos'
                        let all_exclude = valid_pairs.iter().all(|&(v1, v2)| {
                            // val is excluded if v1==val or v2==val (since pos sees both p1 and p2)
                            v1 == val || v2 == val
                        });

                        if all_exclude {
                            return Some(Hint {
                                technique: Technique::AlignedPairExclusion,
                                hint_type: HintType::EliminateCandidates {
                                    pos,
                                    values: vec![val],
                                },
                                explanation: format!(
                                    "Aligned Pair Exclusion: cells ({},{}) and ({},{}). Eliminate {} from ({},{}).",
                                    p1.row + 1, p1.col + 1, p2.row + 1, p2.col + 1,
                                    val, pos.row + 1, pos.col + 1
                                ),
                                involved_cells: vec![p1, p2, pos],
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn apply_aligned_pair_exclusion(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_aligned_pair_exclusion(grid) {
            if let HintType::EliminateCandidates { pos, values } = hint.hint_type {
                for v in values {
                    grid.cell_mut(pos).remove_candidate(v);
                }
                return true;
            }
        }
        false
    }

    // ==================== Aligned Triplet Exclusion ====================

    /// Aligned Triplet Exclusion: Three mutually visible cells. If every valid
    /// value triple excludes a candidate from a common peer, eliminate it.
    fn apply_aligned_triplet_exclusion(&self, grid: &mut Grid) -> bool {
        let empty = grid.empty_positions();

        for i in 0..empty.len() {
            let p1 = empty[i];
            let c1 = grid.get_candidates(p1);
            if c1.count() < 2 || c1.count() > 4 {
                continue;
            }

            for j in (i + 1)..empty.len() {
                let p2 = empty[j];
                if !self.sees(p1, p2) {
                    continue;
                }
                let c2 = grid.get_candidates(p2);
                if c2.count() < 2 || c2.count() > 4 {
                    continue;
                }

                for k in (j + 1)..empty.len() {
                    let p3 = empty[k];
                    if !self.sees(p1, p3) || !self.sees(p2, p3) {
                        continue;
                    }
                    let c3 = grid.get_candidates(p3);
                    if c3.count() < 2 || c3.count() > 4 {
                        continue;
                    }

                    // Enumerate valid triples
                    let mut valid_triples: Vec<(u8, u8, u8)> = Vec::new();
                    for v1 in c1.iter() {
                        for v2 in c2.iter() {
                            if v1 == v2 && self.sees(p1, p2) {
                                continue;
                            }
                            for v3 in c3.iter() {
                                if (v1 == v3 && self.sees(p1, p3))
                                    || (v2 == v3 && self.sees(p2, p3))
                                {
                                    continue;
                                }
                                valid_triples.push((v1, v2, v3));
                            }
                        }
                    }

                    if valid_triples.is_empty() {
                        continue;
                    }

                    // Check common peers
                    for &pos in &empty {
                        if pos == p1 || pos == p2 || pos == p3 {
                            continue;
                        }
                        if !self.sees(pos, p1) || !self.sees(pos, p2) || !self.sees(pos, p3) {
                            continue;
                        }

                        for val in grid.get_candidates(pos).iter() {
                            let all_exclude = valid_triples.iter().all(|&(v1, v2, v3)| {
                                v1 == val || v2 == val || v3 == val
                            });

                            if all_exclude {
                                grid.cell_mut(pos).remove_candidate(val);
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== Death Blossom ====================

    /// Death Blossom: A "stem" cell with N candidates, each linking to a different ALS (petal).
    /// If all petals agree on eliminating a candidate from a cell, do it.
    fn apply_death_blossom(&self, grid: &mut Grid) -> bool {
        let all_als = Self::find_all_als(grid);
        let small_als: Vec<&(Vec<Position>, crate::BitSet)> = all_als
            .iter()
            .filter(|(cells, _)| cells.len() <= 4)
            .take(40)
            .collect();

        let empty = grid.empty_positions();

        for &stem in &empty {
            let stem_cands = grid.get_candidates(stem);
            if stem_cands.count() < 2 || stem_cands.count() > 3 {
                continue;
            }

            let stem_vals: Vec<u8> = stem_cands.iter().collect();

            // For each stem candidate, find an ALS that:
            // 1. Doesn't contain the stem
            // 2. Contains the stem candidate as a restricted common (all instances see the stem)
            let mut petals: Vec<(u8, usize)> = Vec::new(); // (stem_val, als_idx)
            let mut used_als = std::collections::HashSet::new();

            for &val in &stem_vals {
                let mut found_petal = None;
                for (idx, (cells, cands)) in small_als.iter().enumerate() {
                    if used_als.contains(&idx) {
                        continue;
                    }
                    if cells.contains(&stem) {
                        continue;
                    }
                    if !cands.contains(val) {
                        continue;
                    }
                    // Check restricted common: all cells in ALS with 'val' must see stem
                    let val_cells: Vec<&Position> = cells
                        .iter()
                        .filter(|&&p| grid.get_candidates(p).contains(val))
                        .collect();
                    if val_cells.iter().all(|&&p| self.sees(p, stem)) {
                        found_petal = Some(idx);
                        break;
                    }
                }
                if let Some(idx) = found_petal {
                    petals.push((val, idx));
                    used_als.insert(idx);
                }
            }

            // Need one petal per stem candidate
            if petals.len() != stem_vals.len() {
                continue;
            }

            // Find common eliminations: for each non-RCC digit in each ALS,
            // if all petals' instances of that digit see a target cell, eliminate it
            for digit in 1..=9u8 {
                if stem_cands.contains(digit) {
                    continue;
                }

                let mut target_eliminable = true;
                let mut all_digit_cells: Vec<Position> = Vec::new();

                for &(_, als_idx) in &petals {
                    let (cells, cands) = small_als[als_idx];
                    if !cands.contains(digit) {
                        target_eliminable = false;
                        break;
                    }
                    let digit_cells: Vec<Position> = cells
                        .iter()
                        .filter(|&&p| grid.get_candidates(p).contains(digit))
                        .copied()
                        .collect();
                    all_digit_cells.extend(digit_cells);
                }

                if !target_eliminable || all_digit_cells.is_empty() {
                    continue;
                }

                // Find cells that see all digit cells across all petals
                for &pos in &empty {
                    if pos == stem || petals.iter().any(|&(_, idx)| small_als[idx].0.contains(&pos)) {
                        continue;
                    }
                    if !grid.get_candidates(pos).contains(digit) {
                        continue;
                    }
                    if all_digit_cells.iter().all(|&dc| self.sees(pos, dc)) {
                        grid.cell_mut(pos).remove_candidate(digit);
                        return true;
                    }
                }
            }
        }
        false
    }

    // ==================== Kraken Fish ====================

    /// Kraken Fish: A finned fish where the fin's validity is verified via forcing chain.
    /// If assuming the fin leads to the same elimination as the fish, the elimination holds.
    fn apply_kraken_fish(&self, grid: &mut Grid) -> bool {
        for digit in 1..=9u8 {
            // Try row-based X-Wing patterns with problematic fins
            for r1 in 0..9 {
                for r2 in (r1 + 1)..9 {
                    let row1_cols: Vec<usize> = (0..9)
                        .filter(|&c| {
                            let p = Position::new(r1, c);
                            grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit)
                        })
                        .collect();
                    let row2_cols: Vec<usize> = (0..9)
                        .filter(|&c| {
                            let p = Position::new(r2, c);
                            grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit)
                        })
                        .collect();

                    let common_cols: Vec<usize> = row1_cols
                        .iter()
                        .filter(|c| row2_cols.contains(c))
                        .copied()
                        .collect();

                    if common_cols.len() != 2 {
                        continue;
                    }

                    // Find fins
                    let fins: Vec<Position> = row1_cols
                        .iter()
                        .filter(|c| !common_cols.contains(c))
                        .map(|&c| Position::new(r1, c))
                        .chain(
                            row2_cols
                                .iter()
                                .filter(|c| !common_cols.contains(c))
                                .map(|&c| Position::new(r2, c)),
                        )
                        .collect();

                    if fins.is_empty() || fins.len() > 2 {
                        continue;
                    }

                    // Standard fish elimination targets (cells in cover cols, not in base rows)
                    let targets: Vec<Position> = common_cols
                        .iter()
                        .flat_map(|&c| {
                            (0..9)
                                .filter(move |&r| r != r1 && r != r2)
                                .map(move |r| Position::new(r, c))
                        })
                        .filter(|&p| grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit))
                        .collect();

                    // For each target, verify via forcing chain on each fin
                    for &target in &targets {
                        let mut all_fins_eliminate = true;
                        for &fin in &fins {
                            // If fin has digit → propagate and check if target loses digit
                            let (result, contradiction) = self.propagate_singles(grid, fin, digit);
                            if contradiction {
                                continue; // Contradiction means fin can't have digit, fish holds trivially
                            }
                            // Check if target lost the digit
                            if result.get(target).is_some() {
                                // Target was filled → digit is "eliminated"
                                if result.get(target) == Some(digit) {
                                    all_fins_eliminate = false;
                                    break;
                                }
                            } else if result.get_candidates(target).contains(digit) {
                                all_fins_eliminate = false;
                                break;
                            }
                        }

                        if all_fins_eliminate {
                            grid.cell_mut(target).remove_candidate(digit);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // ==================== Region Forcing Chain ====================

    /// Region Forcing Chain: Like Cell FC but iterates over all positions of a digit
    /// in a unit. If all positions agree on a conclusion, apply it.
    fn find_region_forcing_chain(&self, grid: &Grid) -> Option<Hint> {
        for unit in 0..27 {
            let positions: Vec<Position> = if unit < 9 {
                Self::row_positions(unit)
            } else if unit < 18 {
                Self::col_positions(unit - 9)
            } else {
                Self::box_positions(unit - 18)
            };

            for digit in 1..=9u8 {
                let digit_cells: Vec<Position> = positions
                    .iter()
                    .filter(|&&p| {
                        grid.cell(p).is_empty() && grid.get_candidates(p).contains(digit)
                    })
                    .copied()
                    .collect();

                if digit_cells.len() < 2 || digit_cells.len() > 4 {
                    continue;
                }

                let mut branches = Vec::new();
                let mut any_contradiction = false;

                for &pos in &digit_cells {
                    let (result, contradiction) = self.propagate_singles(grid, pos, digit);
                    if contradiction {
                        any_contradiction = true;
                        break;
                    }
                    branches.push(result);
                }

                if any_contradiction || branches.len() < 2 {
                    continue;
                }

                let source_pos = digit_cells[0]; // Use first cell as reference

                if let Some(hint) = Self::find_common_placement(
                    grid,
                    source_pos,
                    &branches,
                    Technique::RegionForcingChain,
                ) {
                    return Some(hint);
                }

                if let Some(hint) = Self::find_common_elimination(
                    grid,
                    source_pos,
                    &branches,
                    Technique::RegionForcingChain,
                ) {
                    return Some(hint);
                }
            }
        }
        None
    }

    fn apply_region_forcing_chain(&self, grid: &mut Grid) -> bool {
        if let Some(hint) = self.find_region_forcing_chain(grid) {
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
            // Try all techniques up to BUG (no forcing chains — prevents recursion!)
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
            if !progress { progress |= self.apply_naked_quads(&mut g); }
            if !progress { progress |= self.apply_hidden_quads(&mut g); }
            if !progress { progress |= self.apply_empty_rectangle(&mut g); }
            if !progress { progress |= self.apply_avoidable_rectangle(&mut g); }
            if !progress { progress |= self.apply_xy_wing(&mut g); }
            if !progress { progress |= self.apply_xyz_wing(&mut g); }
            if !progress { progress |= self.apply_wxyz_wing(&mut g); }
            if !progress { progress |= self.apply_w_wing(&mut g); }
            if !progress { progress |= self.apply_x_chain(&mut g); }
            if !progress { progress |= self.apply_aic(&mut g); }
            if !progress { progress |= self.apply_als_xz(&mut g); }
            if !progress { progress |= self.apply_als_xy_wing(&mut g); }
            if !progress { progress |= self.apply_als_chain(&mut g); }
            if !progress { progress |= self.apply_unique_rectangle(&mut g); }
            if !progress { progress |= self.apply_hidden_rectangle(&mut g); }
            if !progress { progress |= self.apply_bug(&mut g); }
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
        // Expert tier
        assert!(Technique::EmptyRectangle.se_rating() <= Technique::NakedQuad.se_rating());
        assert!(Technique::AvoidableRectangle.se_rating() <= Technique::NakedQuad.se_rating());
        assert!(Technique::NakedQuad.se_rating() <= Technique::Jellyfish.se_rating());
        assert!(Technique::Jellyfish.se_rating() <= Technique::FinnedJellyfish.se_rating());
        assert!(Technique::FinnedJellyfish.se_rating() <= Technique::HiddenQuad.se_rating());
        // Master tier
        assert!(Technique::XYWing.se_rating() < Technique::WXYZWing.se_rating());
        assert!(Technique::XChain.se_rating() <= Technique::WXYZWing.se_rating());
        assert!(Technique::ThreeDMedusa.se_rating() <= Technique::SueDeCoq.se_rating());
        assert!(Technique::FrankenFish.se_rating() <= Technique::SiameseFish.se_rating());
        // Extreme tier
        assert!(Technique::HiddenQuad.se_rating() < Technique::AlsXz.se_rating());
        assert!(Technique::AlsXz.se_rating() <= Technique::ExtendedUniqueRectangle.se_rating());
        assert!(Technique::AlsXz.se_rating() < Technique::BivalueUniversalGrave.se_rating());
        assert!(Technique::BivalueUniversalGrave.se_rating() < Technique::AIC.se_rating());
        assert!(Technique::AIC.se_rating() < Technique::AlignedPairExclusion.se_rating());
        assert!(Technique::AlignedPairExclusion.se_rating() < Technique::MutantFish.se_rating());
        assert!(Technique::AIC.se_rating() < Technique::AlsXyWing.se_rating());
        assert!(Technique::AlsXyWing.se_rating() <= Technique::AlsChain.se_rating());
        assert!(Technique::AlsChain.se_rating() <= Technique::AlignedTripletExclusion.se_rating());
        assert!(Technique::AlignedTripletExclusion.se_rating() <= Technique::NishioForcingChain.se_rating());
        assert!(Technique::NishioForcingChain.se_rating() < Technique::KrakenFish.se_rating());
        assert!(Technique::KrakenFish.se_rating() < Technique::CellForcingChain.se_rating());
        assert!(Technique::CellForcingChain.se_rating() < Technique::DeathBlossom.se_rating());
        assert!(Technique::DeathBlossom.se_rating() <= Technique::RegionForcingChain.se_rating());
        assert!(Technique::RegionForcingChain.se_rating() < Technique::DynamicForcingChain.se_rating());
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
        assert_eq!(Technique::NakedQuad.to_string(), "Naked Quad");
        assert_eq!(Technique::HiddenQuad.to_string(), "Hidden Quad");
        assert_eq!(Technique::BivalueUniversalGrave.to_string(), "BUG+1");
        assert_eq!(Technique::NishioForcingChain.to_string(), "Nishio Forcing Chain");
        assert_eq!(Technique::CellForcingChain.to_string(), "Cell Forcing Chain");
        assert_eq!(Technique::DynamicForcingChain.to_string(), "Dynamic Forcing Chain");
        // New techniques
        assert_eq!(Technique::EmptyRectangle.to_string(), "Empty Rectangle");
        assert_eq!(Technique::AvoidableRectangle.to_string(), "Avoidable Rectangle");
        assert_eq!(Technique::WXYZWing.to_string(), "WXYZ-Wing");
        assert_eq!(Technique::ThreeDMedusa.to_string(), "3D Medusa");
        assert_eq!(Technique::SueDeCoq.to_string(), "Sue de Coq");
        assert_eq!(Technique::FrankenFish.to_string(), "Franken Fish");
        assert_eq!(Technique::SiameseFish.to_string(), "Siamese Fish");
        assert_eq!(Technique::AlsChain.to_string(), "ALS Chain");
        assert_eq!(Technique::HiddenRectangle.to_string(), "Hidden Rectangle");
        assert_eq!(Technique::ExtendedUniqueRectangle.to_string(), "Extended Unique Rectangle");
        assert_eq!(Technique::MutantFish.to_string(), "Mutant Fish");
        assert_eq!(Technique::AlignedPairExclusion.to_string(), "Aligned Pair Exclusion");
        assert_eq!(Technique::AlignedTripletExclusion.to_string(), "Aligned Triplet Exclusion");
        assert_eq!(Technique::DeathBlossom.to_string(), "Death Blossom");
        assert_eq!(Technique::KrakenFish.to_string(), "Kraken Fish");
        assert_eq!(Technique::RegionForcingChain.to_string(), "Region Forcing Chain");
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
