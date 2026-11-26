use super::{game_from_grid, game_from_string, rated_templates::TEMPLATES, SudokuGame};
use std::sync::Arc;
use sudoku_core::{Generator, GeneratorConfig};

/// Sudoku symmetries preserve uniqueness, but the solver can choose different
/// deductions after a permutation. Rate the resulting grid before accepting it.
fn permuted(template: &str) -> String {
    fn axis() -> Vec<usize> {
        let mut bands = [0, 1, 2];
        fastrand::shuffle(&mut bands);
        let mut positions = Vec::with_capacity(9);
        for band in bands {
            let mut rows = [3 * band, 3 * band + 1, 3 * band + 2];
            fastrand::shuffle(&mut rows);
            positions.extend(rows);
        }
        positions
    }

    let rows = axis();
    let cols = axis();
    let mut digits = *b"123456789";
    fastrand::shuffle(&mut digits);
    let mut puzzle = String::with_capacity(81);
    for row in rows {
        for &col in &cols {
            let cell = template.as_bytes()[9 * row + col];
            puzzle.push(match cell {
                b'1'..=b'9' => digits[(cell - b'1') as usize] as char,
                _ => '.',
            });
        }
    }
    puzzle
}

pub(super) fn exact(target: f32) -> Option<Arc<SudokuGame>> {
    let (_, templates) = TEMPLATES.iter().find(|(rating, _)| *rating == target)?;
    let matches_rating = |game: &Arc<SudokuGame>| game.get_se_rating() == target;

    // Try a fresh grid without spending the engine's full search budget on a
    // rare technique. A candidate's named difficulty alone cannot qualify it.
    let mut config = GeneratorConfig::for_se_rating(target);
    config.max_attempts = 1;
    let grid = Generator::with_config(config).generate_with_config();
    if let Some(game) = game_from_grid(grid).filter(matches_rating) {
        return Some(game);
    }

    let mut choices = templates.to_vec();
    fastrand::shuffle(&mut choices);
    let matching = |puzzle| game_from_string(puzzle).filter(matches_rating);
    matching(permuted(choices[0])).or_else(|| {
        choices
            .into_iter()
            .find_map(|puzzle| matching(puzzle.to_owned()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sudoku_core::{Grid, Solver};

    #[test]
    fn every_offered_rating_has_unique_independently_rated_boards() {
        let solver = Solver::new();
        for &(rating, templates) in TEMPLATES {
            assert!(!templates.is_empty());
            for &puzzle in templates {
                let grid = Grid::from_string(puzzle).unwrap();
                assert!(solver.has_unique_solution(&grid), "{puzzle}");
                assert_eq!(solver.rate_se(&grid), rating, "{puzzle}");
            }
        }
    }
}
