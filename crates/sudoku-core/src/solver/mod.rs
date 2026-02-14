//! Solver orchestrator.
//!
//! Dispatches to three abstract engines (Fish, ALS, AIC) plus basic techniques,
//! uniqueness patterns, and backtracking.

mod types;
pub(crate) mod fabric;
pub(crate) mod explain;
mod basic;
mod fish_engine;
mod als_engine;
mod aic_engine;
mod uniqueness;
pub(crate) mod backtrack;

use crate::{Grid, Position};
use explain::{Finding, InferenceResult};
use fabric::{idx_to_pos, CandidateFabric};

pub use types::{Difficulty, Hint, HintType, Technique};

/// Unit struct solver — stateless, all state is per-call.
pub struct Solver;

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver {
    /// Create a new solver.
    pub fn new() -> Self {
        Self
    }

    /// Solve the puzzle, returning the solved grid if successful.
    pub fn solve(&self, grid: &Grid) -> Option<Grid> {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();
        if backtrack::solve_recursive(&mut working) {
            Some(working)
        } else {
            None
        }
    }

    /// Count solutions up to a limit.
    pub fn count_solutions(&self, grid: &Grid, limit: usize) -> usize {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();
        let mut count = 0;
        backtrack::count_solutions_recursive(&mut working, &mut count, limit);
        count
    }

    /// Check if the puzzle has exactly one solution.
    pub fn has_unique_solution(&self, grid: &Grid) -> bool {
        self.count_solutions(grid, 2) == 1
    }

    /// Get a hint for the current position.
    pub fn get_hint(&self, grid: &Grid) -> Option<Hint> {
        let mut working = grid.deep_clone();
        working.recalculate_candidates();

        if let Some(finding) = self.find_first_technique(&working) {
            return Some(finding.to_hint());
        }

        // Last resort: backtracking hint
        if let Some(finding) = backtrack::find_backtracking_hint(&working) {
            return Some(finding.to_hint());
        }

        None
    }

    /// Rate the difficulty of a puzzle.
    pub fn rate_difficulty(&self, grid: &Grid) -> Difficulty {
        let empty_count = grid.empty_positions().len();
        let mut working = grid.deep_clone();
        let max_tech = self.solve_with_techniques(&mut working);
        Self::technique_to_difficulty(max_tech, empty_count)
    }

    /// Rate the puzzle using the Sudoku Explainer (SE) numerical scale.
    pub fn rate_se(&self, grid: &Grid) -> f32 {
        let mut working = grid.deep_clone();
        let max_tech = self.solve_with_techniques(&mut working);
        max_tech.se_rating()
    }

    // ==================== Internal dispatch ====================

    /// Find the first applicable technique for a hint (does not mutate grid).
    fn find_first_technique(&self, grid: &Grid) -> Option<Finding> {
        let fab = CandidateFabric::from_grid(grid);

        // Phase 1: Basic
        if let Some(f) = basic::find_naked_single(&fab) { return Some(f); }
        if let Some(f) = basic::find_hidden_single(&fab) { return Some(f); }

        // Phase 2: Subsets
        if let Some(f) = basic::find_naked_subset(&fab, 2) { return Some(f); }
        if let Some(f) = basic::find_hidden_subset(&fab, 2) { return Some(f); }
        if let Some(f) = basic::find_naked_subset(&fab, 3) { return Some(f); }
        if let Some(f) = basic::find_hidden_subset(&fab, 3) { return Some(f); }

        // Phase 3: Intersections (size-1 fish)
        if let Some(f) = fish_engine::find_pointing_pair(&fab) { return Some(f); }
        if let Some(f) = fish_engine::find_box_line_reduction(&fab) { return Some(f); }

        // Phase 4: Fish (size 2+) + quads
        if let Some(f) = fish_engine::find_basic_fish(&fab, 2) { return Some(f); }
        if let Some(f) = fish_engine::find_finned_fish(&fab, 2) { return Some(f); }
        if let Some(f) = fish_engine::find_basic_fish(&fab, 3) { return Some(f); }
        if let Some(f) = fish_engine::find_finned_fish(&fab, 3) { return Some(f); }
        if let Some(f) = fish_engine::find_basic_fish(&fab, 4) { return Some(f); }
        if let Some(f) = fish_engine::find_finned_fish(&fab, 4) { return Some(f); }
        if let Some(f) = basic::find_naked_subset(&fab, 4) { return Some(f); }
        if let Some(f) = basic::find_hidden_subset(&fab, 4) { return Some(f); }

        // Phase 5: Uniqueness
        if let Some(f) = uniqueness::find_empty_rectangle(&fab) { return Some(f); }
        if let Some(f) = uniqueness::find_avoidable_rectangle(&fab) { return Some(f); }
        if let Some(f) = uniqueness::find_unique_rectangle(&fab) { return Some(f); }
        if let Some(f) = uniqueness::find_hidden_rectangle(&fab) { return Some(f); }

        // Phase 6: Master
        if let Some(f) = als_engine::find_xy_wing(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_xyz_wing(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_wxyz_wing(&fab) { return Some(f); }
        if let Some(f) = aic_engine::find_w_wing(&fab) { return Some(f); }
        // AIC family: shared link graph for X-Chain, 3D Medusa, AIC
        let graph = aic_engine::build_link_graph(&fab);
        if let Some(f) = aic_engine::find_x_chain(&fab, &graph) { return Some(f); }
        if let Some(f) = aic_engine::find_medusa(&fab, &graph) { return Some(f); }
        if let Some(f) = als_engine::find_sue_de_coq(&fab) { return Some(f); }
        if let Some(f) = aic_engine::find_aic(&fab, &graph) { return Some(f); }
        if let Some(f) = fish_engine::find_franken_fish(&fab) { return Some(f); }
        if let Some(f) = fish_engine::find_siamese_fish(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_als_xz(&fab) { return Some(f); }
        if let Some(f) = uniqueness::find_extended_unique_rectangle(&fab) { return Some(f); }
        if let Some(f) = uniqueness::find_bug(&fab) { return Some(f); }

        // Phase 7: Extreme
        if let Some(f) = als_engine::find_als_xy_wing(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_als_chain(&fab) { return Some(f); }
        if let Some(f) = fish_engine::find_mutant_fish(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_aligned_pair_exclusion(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_aligned_triplet_exclusion(&fab) { return Some(f); }
        if let Some(f) = als_engine::find_death_blossom(&fab) { return Some(f); }

        // Forcing chains need the Grid for propagation
        let propagate_singles = |g: &Grid, pos: Position, val: u8| -> (Grid, bool) {
            backtrack::propagate_singles(g, pos, val)
        };
        if let Some(f) = aic_engine::find_nishio_fc(grid, &propagate_singles) { return Some(f); }
        if let Some(f) = aic_engine::find_kraken_fish(grid, &propagate_singles) { return Some(f); }
        if let Some(f) = aic_engine::find_region_fc(grid, &propagate_singles) { return Some(f); }
        if let Some(f) = aic_engine::find_cell_fc(grid, &propagate_singles) { return Some(f); }
        // Dynamic FC uses full technique propagation
        let prop_full = |g: &Grid, pos: Position, val: u8| -> (Grid, bool) {
            propagate_full(g, pos, val)
        };
        if let Some(f) = aic_engine::find_dynamic_fc(grid, &prop_full) { return Some(f); }

        None
    }

    /// Solve the puzzle using human techniques, returning the hardest technique used.
    fn solve_with_techniques(&self, grid: &mut Grid) -> Technique {
        grid.recalculate_candidates();
        let mut max_technique = Technique::NakedSingle;

        while !grid.is_complete() {
            let fab = CandidateFabric::from_grid(grid);

            // Try techniques in priority order via dispatch table
            let finding = None
                // Phase 1: Basic
                .or_else(|| basic::find_naked_single(&fab))
                .or_else(|| basic::find_hidden_single(&fab))
                // Phase 2: Subsets
                .or_else(|| basic::find_naked_subset(&fab, 2))
                .or_else(|| basic::find_hidden_subset(&fab, 2))
                .or_else(|| basic::find_naked_subset(&fab, 3))
                .or_else(|| basic::find_hidden_subset(&fab, 3))
                // Phase 3: Intersections (size-1 fish)
                .or_else(|| fish_engine::find_pointing_pair(&fab))
                .or_else(|| fish_engine::find_box_line_reduction(&fab))
                // Phase 4: Fish (size 2+) + quads
                .or_else(|| fish_engine::find_basic_fish(&fab, 2))
                .or_else(|| fish_engine::find_finned_fish(&fab, 2))
                .or_else(|| fish_engine::find_basic_fish(&fab, 3))
                .or_else(|| fish_engine::find_finned_fish(&fab, 3))
                .or_else(|| fish_engine::find_basic_fish(&fab, 4))
                .or_else(|| fish_engine::find_finned_fish(&fab, 4))
                .or_else(|| basic::find_naked_subset(&fab, 4))
                .or_else(|| basic::find_hidden_subset(&fab, 4))
                // Phase 5: Uniqueness
                .or_else(|| uniqueness::find_empty_rectangle(&fab))
                .or_else(|| uniqueness::find_avoidable_rectangle(&fab))
                .or_else(|| uniqueness::find_unique_rectangle(&fab))
                .or_else(|| uniqueness::find_hidden_rectangle(&fab))
                // Phase 6: Master
                .or_else(|| als_engine::find_xy_wing(&fab))
                .or_else(|| als_engine::find_xyz_wing(&fab))
                .or_else(|| als_engine::find_wxyz_wing(&fab))
                .or_else(|| aic_engine::find_w_wing(&fab))
                // AIC family: shared link graph for X-Chain, 3D Medusa, AIC
                .or_else(|| {
                    let graph = aic_engine::build_link_graph(&fab);
                    None
                        .or_else(|| aic_engine::find_x_chain(&fab, &graph))
                        .or_else(|| aic_engine::find_medusa(&fab, &graph))
                        .or_else(|| als_engine::find_sue_de_coq(&fab))
                        .or_else(|| aic_engine::find_aic(&fab, &graph))
                })
                .or_else(|| fish_engine::find_franken_fish(&fab))
                .or_else(|| fish_engine::find_siamese_fish(&fab))
                .or_else(|| als_engine::find_als_xz(&fab))
                .or_else(|| uniqueness::find_extended_unique_rectangle(&fab))
                .or_else(|| uniqueness::find_bug(&fab))
                // Phase 7: Extreme
                .or_else(|| als_engine::find_als_xy_wing(&fab))
                .or_else(|| als_engine::find_als_chain(&fab))
                .or_else(|| fish_engine::find_mutant_fish(&fab))
                .or_else(|| als_engine::find_aligned_pair_exclusion(&fab))
                .or_else(|| als_engine::find_aligned_triplet_exclusion(&fab))
                .or_else(|| als_engine::find_death_blossom(&fab))
                // Forcing chains (singles propagation)
                .or_else(|| {
                    let prop = |g: &Grid, pos: Position, val: u8| -> (Grid, bool) {
                        backtrack::propagate_singles(g, pos, val)
                    };
                    None
                        .or_else(|| aic_engine::find_nishio_fc(grid, &prop))
                        .or_else(|| aic_engine::find_kraken_fish(grid, &prop))
                        .or_else(|| aic_engine::find_region_fc(grid, &prop))
                        .or_else(|| aic_engine::find_cell_fc(grid, &prop))
                })
                // Dynamic FC: full technique propagation
                .or_else(|| {
                    let prop_full = |g: &Grid, pos: Position, val: u8| -> (Grid, bool) {
                        propagate_full(g, pos, val)
                    };
                    aic_engine::find_dynamic_fc(grid, &prop_full)
                });

            match finding {
                Some(f) => {
                    if f.technique > max_technique {
                        max_technique = f.technique;
                    }
                    apply_finding(grid, &f);
                }
                None => {
                    // No technique found, use backtracking to finish
                    backtrack::solve_recursive(grid);
                    return Technique::Backtracking;
                }
            }
        }

        max_technique
    }

    /// Map a technique + puzzle characteristics to a difficulty level.
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
            | Technique::AvoidableRectangle
            | Technique::UniqueRectangle
            | Technique::HiddenRectangle => Difficulty::Expert,
            Technique::XYWing
            | Technique::XYZWing
            | Technique::WXYZWing
            | Technique::WWing
            | Technique::XChain
            | Technique::ThreeDMedusa
            | Technique::SueDeCoq
            | Technique::AIC
            | Technique::FrankenFish
            | Technique::SiameseFish
            | Technique::AlsXz
            | Technique::ExtendedUniqueRectangle
            | Technique::BivalueUniversalGrave => Difficulty::Master,
            Technique::AlsXyWing
            | Technique::AlsChain
            | Technique::MutantFish
            | Technique::AlignedPairExclusion
            | Technique::AlignedTripletExclusion
            | Technique::DeathBlossom
            | Technique::NishioForcingChain
            | Technique::KrakenFish
            | Technique::RegionForcingChain
            | Technique::CellForcingChain
            | Technique::DynamicForcingChain
            | Technique::Backtracking => Difficulty::Extreme,
        }
    }
}

/// Apply a Finding to a mutable Grid.
fn apply_finding(grid: &mut Grid, finding: &Finding) {
    match &finding.inference {
        InferenceResult::Placement { cell, value } => {
            let pos = idx_to_pos(*cell);
            grid.set_cell_unchecked(pos, Some(*value));
            grid.recalculate_candidates();
        }
        InferenceResult::Elimination { cell, values } => {
            let pos = idx_to_pos(*cell);
            for &v in values {
                grid.cell_mut(pos).remove_candidate(v);
            }
        }
    }
}

/// Propagate using the full technique set (for Dynamic Forcing Chains).
///
/// Makes an assumption (set cell value), then loops applying all techniques
/// except forcing chains (to avoid infinite recursion) until no more progress.
fn propagate_full(grid: &Grid, pos: Position, val: u8) -> (Grid, bool) {
    let mut g = grid.deep_clone();
    g.set_cell_unchecked(pos, Some(val));
    g.recalculate_candidates();

    for _ in 0..200 {
        if backtrack::has_contradiction(&g) {
            return (g, true);
        }
        if g.is_complete() {
            return (g, false);
        }

        let fab = CandidateFabric::from_grid(&g);

        // Try all techniques except forcing chains (avoids infinite recursion)
        let finding = None
            .or_else(|| basic::find_naked_single(&fab))
            .or_else(|| basic::find_hidden_single(&fab))
            .or_else(|| basic::find_naked_subset(&fab, 2))
            .or_else(|| basic::find_hidden_subset(&fab, 2))
            .or_else(|| basic::find_naked_subset(&fab, 3))
            .or_else(|| basic::find_hidden_subset(&fab, 3))
            .or_else(|| fish_engine::find_pointing_pair(&fab))
            .or_else(|| fish_engine::find_box_line_reduction(&fab))
            .or_else(|| fish_engine::find_basic_fish(&fab, 2))
            .or_else(|| fish_engine::find_finned_fish(&fab, 2))
            .or_else(|| fish_engine::find_basic_fish(&fab, 3))
            .or_else(|| fish_engine::find_finned_fish(&fab, 3))
            .or_else(|| fish_engine::find_basic_fish(&fab, 4))
            .or_else(|| fish_engine::find_finned_fish(&fab, 4))
            .or_else(|| basic::find_naked_subset(&fab, 4))
            .or_else(|| basic::find_hidden_subset(&fab, 4))
            .or_else(|| uniqueness::find_empty_rectangle(&fab))
            .or_else(|| uniqueness::find_avoidable_rectangle(&fab))
            .or_else(|| uniqueness::find_unique_rectangle(&fab))
            .or_else(|| uniqueness::find_hidden_rectangle(&fab))
            .or_else(|| als_engine::find_xy_wing(&fab))
            .or_else(|| als_engine::find_xyz_wing(&fab))
            .or_else(|| als_engine::find_wxyz_wing(&fab))
            .or_else(|| aic_engine::find_w_wing(&fab))
            .or_else(|| {
                let graph = aic_engine::build_link_graph(&fab);
                None
                    .or_else(|| aic_engine::find_x_chain(&fab, &graph))
                    .or_else(|| aic_engine::find_medusa(&fab, &graph))
                    .or_else(|| als_engine::find_sue_de_coq(&fab))
                    .or_else(|| aic_engine::find_aic(&fab, &graph))
            })
            .or_else(|| fish_engine::find_franken_fish(&fab))
            .or_else(|| fish_engine::find_siamese_fish(&fab))
            .or_else(|| als_engine::find_als_xz(&fab))
            .or_else(|| uniqueness::find_extended_unique_rectangle(&fab))
            .or_else(|| uniqueness::find_bug(&fab))
            .or_else(|| als_engine::find_als_xy_wing(&fab))
            .or_else(|| als_engine::find_als_chain(&fab))
            .or_else(|| fish_engine::find_mutant_fish(&fab))
            .or_else(|| als_engine::find_aligned_pair_exclusion(&fab))
            .or_else(|| als_engine::find_aligned_triplet_exclusion(&fab))
            .or_else(|| als_engine::find_death_blossom(&fab));
        // Note: forcing chains excluded to avoid infinite recursion

        match finding {
            Some(f) => apply_finding(&mut g, &f),
            None => break,
        }
    }

    let contradiction = backtrack::has_contradiction(&g);
    (g, contradiction)
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
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();
        let solver = Solver::new();
        let difficulty = solver.rate_difficulty(&grid);
        assert!(difficulty >= Difficulty::Easy);
    }

    #[test]
    fn test_solve_with_techniques_regression() {
        let puzzle =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let grid = Grid::from_string(puzzle).unwrap();
        let solver = Solver::new();
        let mut working = grid.deep_clone();
        let max_tech = solver.solve_with_techniques(&mut working);
        assert!(max_tech < Technique::Backtracking);
        assert!(working.is_complete());
    }

    /// Soundness test: verify that every elimination/placement returned by hints
    /// is consistent with the unique solution.
    #[test]
    fn test_hint_soundness() {
        let puzzles = [
            // Easy (naked/hidden singles)
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
            // Medium
            "020000600008020050500060020060000093003905100790000080050090004010070300006000010",
            // Arto Inkala (requires advanced techniques)
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400",
        ];

        let solver = Solver::new();

        for puzzle_str in &puzzles {
            let grid = Grid::from_string(puzzle_str).unwrap();
            let solution = match solver.solve(&grid) {
                Some(s) if s.is_complete() => s,
                _ => continue,
            };

            let mut working = grid.deep_clone();
            working.recalculate_candidates();

            let mut steps = 0;
            while !working.is_complete() && steps < 300 {
                let hint = match solver.get_hint(&working) {
                    Some(h) => h,
                    None => break,
                };

                match &hint.hint_type {
                    HintType::SetValue { pos, value } => {
                        let sol_val = solution.get(*pos);
                        assert_eq!(
                            sol_val,
                            Some(*value),
                            "Unsound placement by {:?}: ({},{}) = {}, solution has {:?}. Puzzle: {}",
                            hint.technique,
                            pos.row + 1, pos.col + 1, value, sol_val, puzzle_str
                        );
                        working.set_cell_unchecked(*pos, Some(*value));
                        working.recalculate_candidates();
                    }
                    HintType::EliminateCandidates { pos, values } => {
                        let sol_val = solution.get(*pos).expect("Position should have solution");
                        for &v in values {
                            assert_ne!(
                                v, sol_val,
                                "Unsound elimination by {:?}: removing {} from ({},{}) but solution needs it. Puzzle: {}",
                                hint.technique, v, pos.row + 1, pos.col + 1, puzzle_str
                            );
                        }
                        for &v in values {
                            working.cell_mut(*pos).remove_candidate(v);
                        }
                    }
                }
                steps += 1;
            }
        }
    }
}
