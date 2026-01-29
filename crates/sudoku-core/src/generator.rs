use crate::{Difficulty, Grid, Position, Solver};
use serde::{Deserialize, Serialize};

/// Symmetry type for puzzle generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymmetryType {
    /// No symmetry
    None,
    /// 180-degree rotational symmetry
    Rotational180,
    /// 90-degree rotational symmetry
    Rotational90,
    /// Horizontal mirror symmetry
    Horizontal,
    /// Vertical mirror symmetry
    Vertical,
    /// Diagonal symmetry
    Diagonal,
}

impl Default for SymmetryType {
    fn default() -> Self {
        Self::Rotational180
    }
}

/// Configuration for puzzle generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Target difficulty
    pub difficulty: Difficulty,
    /// Symmetry type for cell removal
    pub symmetry: SymmetryType,
    /// Maximum attempts before giving up
    pub max_attempts: usize,
    /// Minimum number of givens
    pub min_givens: usize,
    /// Maximum number of givens
    pub max_givens: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Medium,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 100,
            min_givens: 22,
            max_givens: 35,
        }
    }
}

impl GeneratorConfig {
    pub fn beginner() -> Self {
        Self {
            difficulty: Difficulty::Beginner,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 30,
            min_givens: 45,
            max_givens: 55,
        }
    }

    pub fn easy() -> Self {
        Self {
            difficulty: Difficulty::Easy,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 50,
            min_givens: 36,
            max_givens: 45,
        }
    }

    pub fn medium() -> Self {
        Self {
            difficulty: Difficulty::Medium,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 100,
            min_givens: 32,
            max_givens: 38,
        }
    }

    pub fn intermediate() -> Self {
        Self {
            difficulty: Difficulty::Intermediate,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 150,
            min_givens: 28,
            max_givens: 34,
        }
    }

    pub fn hard() -> Self {
        Self {
            difficulty: Difficulty::Hard,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 200,
            min_givens: 24,
            max_givens: 30,
        }
    }

    pub fn expert() -> Self {
        Self {
            difficulty: Difficulty::Expert,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 500,
            min_givens: 22,
            max_givens: 26,
        }
    }

    pub fn master() -> Self {
        Self {
            difficulty: Difficulty::Master,
            symmetry: SymmetryType::Rotational180,
            max_attempts: 1000,
            min_givens: 20,
            max_givens: 24,
        }
    }

    pub fn extreme() -> Self {
        Self {
            difficulty: Difficulty::Extreme,
            symmetry: SymmetryType::None, // No symmetry for extreme puzzles
            max_attempts: 2000,
            min_givens: 17,
            max_givens: 22,
        }
    }
}

/// Sudoku puzzle generator
pub struct Generator {
    config: GeneratorConfig,
    rng: SimpleRng,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    /// Create a new generator with default configuration
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
            rng: SimpleRng::new(),
        }
    }

    /// Create a generator with custom configuration
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self {
            config,
            rng: SimpleRng::new(),
        }
    }

    /// Create a generator with a specific seed for reproducibility
    pub fn with_seed(seed: u64) -> Self {
        Self {
            config: GeneratorConfig::default(),
            rng: SimpleRng::with_seed(seed),
        }
    }

    /// Generate a puzzle with the configured difficulty
    pub fn generate(&mut self, difficulty: Difficulty) -> Grid {
        let config = match difficulty {
            Difficulty::Beginner => GeneratorConfig::beginner(),
            Difficulty::Easy => GeneratorConfig::easy(),
            Difficulty::Medium => GeneratorConfig::medium(),
            Difficulty::Intermediate => GeneratorConfig::intermediate(),
            Difficulty::Hard => GeneratorConfig::hard(),
            Difficulty::Expert => GeneratorConfig::expert(),
            Difficulty::Master => GeneratorConfig::master(),
            Difficulty::Extreme => GeneratorConfig::extreme(),
        };

        self.config = config;
        self.generate_with_config()
    }

    /// Generate a puzzle with the current configuration
    pub fn generate_with_config(&mut self) -> Grid {
        let solver = Solver::new();

        for _ in 0..self.config.max_attempts {
            // Generate a filled grid
            let mut grid = self.generate_filled_grid();

            // Remove cells while maintaining unique solution
            self.remove_cells(&mut grid, &solver);

            // Check difficulty
            let actual_difficulty = solver.rate_difficulty(&grid);

            // Accept if difficulty matches or is close
            if self.difficulty_acceptable(actual_difficulty) {
                let given_count = grid.given_count();
                if given_count >= self.config.min_givens && given_count <= self.config.max_givens {
                    return grid;
                }
            }
        }

        // If we couldn't hit target difficulty, return last attempt
        let mut grid = self.generate_filled_grid();
        self.remove_cells(&mut grid, &solver);
        grid
    }

    /// Check if the actual difficulty is acceptable
    fn difficulty_acceptable(&self, actual: Difficulty) -> bool {
        // Allow one level off from target (but not easier for high difficulties)
        let target = self.config.difficulty;
        match target {
            Difficulty::Beginner => actual == Difficulty::Beginner,
            Difficulty::Easy => actual == Difficulty::Beginner || actual == Difficulty::Easy,
            Difficulty::Medium => actual == Difficulty::Easy || actual == Difficulty::Medium,
            Difficulty::Intermediate => actual == Difficulty::Medium || actual == Difficulty::Intermediate,
            Difficulty::Hard => actual == Difficulty::Intermediate || actual == Difficulty::Hard,
            Difficulty::Expert => actual == Difficulty::Hard || actual == Difficulty::Expert,
            Difficulty::Master => actual == Difficulty::Expert || actual == Difficulty::Master,
            Difficulty::Extreme => actual == Difficulty::Master || actual == Difficulty::Extreme,
        }
    }

    /// Generate a completely filled valid grid
    fn generate_filled_grid(&mut self) -> Grid {
        let mut grid = Grid::new_classic();

        // Fill the diagonal boxes first (they don't affect each other)
        self.fill_box(&mut grid, 0, 0);
        self.fill_box(&mut grid, 3, 3);
        self.fill_box(&mut grid, 6, 6);

        // Solve the rest
        let solver = Solver::new();
        if let Some(solved) = solver.solve(&grid) {
            // Mark all cells as given
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    if let Some(value) = solved.get(pos) {
                        grid.set_given(pos, value);
                    }
                }
            }
            grid
        } else {
            // Fallback: try again
            self.generate_filled_grid()
        }
    }

    /// Fill a 3x3 box with random values
    fn fill_box(&mut self, grid: &mut Grid, start_row: usize, start_col: usize) {
        let mut values: Vec<u8> = (1..=9).collect();
        self.shuffle(&mut values);

        let mut idx = 0;
        for row in start_row..start_row + 3 {
            for col in start_col..start_col + 3 {
                grid.set_given(Position::new(row, col), values[idx]);
                idx += 1;
            }
        }
    }

    /// Remove cells while maintaining unique solution
    fn remove_cells(&mut self, grid: &mut Grid, solver: &Solver) {
        // Get all positions and shuffle them
        let mut positions: Vec<Position> = Position::all_9x9().collect();
        self.shuffle(&mut positions);

        // Track which positions have been tried
        let mut tried = [[false; 9]; 9];

        // Remove cells in symmetry pairs
        for pos in positions {
            if tried[pos.row][pos.col] {
                continue;
            }

            let symmetric_pos = self.symmetric_position(pos);
            tried[pos.row][pos.col] = true;
            if let Some(sym) = symmetric_pos {
                tried[sym.row][sym.col] = true;
            }

            // Store current values
            let value1 = grid.get(pos);
            let value2 = symmetric_pos.and_then(|p| grid.get(p));

            // Skip if already empty
            if value1.is_none() {
                continue;
            }

            // Temporarily remove
            grid.set_cell_unchecked(pos, None);
            if let Some(sym) = symmetric_pos {
                if sym != pos {
                    grid.set_cell_unchecked(sym, None);
                }
            }

            // Check if still has unique solution
            let mut test_grid = grid.deep_clone();
            // Convert remaining values to non-given for testing
            for row in 0..9 {
                for col in 0..9 {
                    let p = Position::new(row, col);
                    if let Some(v) = test_grid.get(p) {
                        test_grid.set_cell_unchecked(p, None);
                        test_grid.set_given(p, v);
                    }
                }
            }

            if solver.has_unique_solution(&test_grid) {
                // Keep removal, update givens
                for row in 0..9 {
                    for col in 0..9 {
                        let p = Position::new(row, col);
                        let cell = grid.cell_mut(p);
                        if cell.value().is_some() {
                            cell.set_given(true);
                        } else {
                            cell.set_given(false);
                        }
                    }
                }

                // Check if we have enough givens
                if grid.given_count() <= self.config.min_givens {
                    // Restore and stop
                    if let Some(v) = value1 {
                        grid.set_given(pos, v);
                    }
                    if let Some(sym) = symmetric_pos {
                        if sym != pos {
                            if let Some(v) = value2 {
                                grid.set_given(sym, v);
                            }
                        }
                    }
                    break;
                }
            } else {
                // Restore values
                if let Some(v) = value1 {
                    grid.set_given(pos, v);
                }
                if let Some(sym) = symmetric_pos {
                    if sym != pos {
                        if let Some(v) = value2 {
                            grid.set_given(sym, v);
                        }
                    }
                }
            }
        }

        // Clear non-given cells and recalculate candidates
        for row in 0..9 {
            for col in 0..9 {
                let pos = Position::new(row, col);
                if !grid.cell(pos).is_given() {
                    grid.set_cell_unchecked(pos, None);
                }
            }
        }
        grid.recalculate_candidates();
    }

    /// Get the symmetric position based on symmetry type
    fn symmetric_position(&self, pos: Position) -> Option<Position> {
        match self.config.symmetry {
            SymmetryType::None => None,
            SymmetryType::Rotational180 => Some(Position::new(8 - pos.row, 8 - pos.col)),
            SymmetryType::Rotational90 => Some(Position::new(pos.col, 8 - pos.row)),
            SymmetryType::Horizontal => Some(Position::new(8 - pos.row, pos.col)),
            SymmetryType::Vertical => Some(Position::new(pos.row, 8 - pos.col)),
            SymmetryType::Diagonal => Some(Position::new(pos.col, pos.row)),
        }
    }

    /// Shuffle a slice using Fisher-Yates
    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.rng.next_usize(i + 1);
            slice.swap(i, j);
        }
    }
}

/// Simple PRNG for no-std compatibility
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        // Use getrandom for WASM-compatible random seeding
        let mut seed_bytes = [0u8; 8];
        getrandom::getrandom(&mut seed_bytes).unwrap_or_else(|_| {
            // Fallback: use a static counter if getrandom fails
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let counter = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            seed_bytes = counter.to_le_bytes();
        });
        let seed = u64::from_le_bytes(seed_bytes);
        Self::with_seed(seed)
    }

    fn with_seed(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        // PCG-like PRNG
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let xorshifted = (((self.state >> 18) ^ self.state) >> 27) as u32;
        let rot = (self.state >> 59) as u32;
        (xorshifted.rotate_right(rot)) as u64
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_easy() {
        let mut generator = Generator::with_seed(42);
        let grid = generator.generate(Difficulty::Easy);

        assert!(grid.given_count() >= 30);
        assert!(grid.given_count() <= 50);

        let solver = Solver::new();
        assert!(solver.has_unique_solution(&grid));
    }

    #[test]
    fn test_generate_medium() {
        let mut generator = Generator::with_seed(42);
        let grid = generator.generate(Difficulty::Medium);

        let solver = Solver::new();
        assert!(solver.has_unique_solution(&grid));
    }

    #[test]
    fn test_symmetry() {
        let mut generator = Generator::with_seed(42);
        generator.config.symmetry = SymmetryType::Rotational180;
        let grid = generator.generate(Difficulty::Easy);

        // Check rotational symmetry
        for row in 0..9 {
            for col in 0..9 {
                let pos1 = Position::new(row, col);
                let pos2 = Position::new(8 - row, 8 - col);

                let has1 = grid.cell(pos1).is_given();
                let has2 = grid.cell(pos2).is_given();

                assert_eq!(has1, has2, "Symmetry broken at {:?} and {:?}", pos1, pos2);
            }
        }
    }
}
