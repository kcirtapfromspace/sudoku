//! Tests for WASM Sudoku game functions

#[cfg(test)]
mod tests {
    use crate::game::{GameState, InputMode, ScreenState, MAX_MISTAKES};
    use sudoku_core::{Difficulty, Position};

    #[test]
    fn test_game_state_new() {
        let state = GameState::new(Difficulty::Easy);
        assert_eq!(state.screen(), ScreenState::Playing);
        assert_eq!(state.mode(), InputMode::Normal);
        assert_eq!(state.mistakes(), 0);
        assert_eq!(state.hints_used(), 0);
        assert!(!state.is_complete());
        assert!(!state.is_game_over());
        assert!(!state.is_paused());
    }

    #[test]
    fn test_game_state_difficulty_levels() {
        for difficulty in [
            Difficulty::Beginner,
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Intermediate,
            Difficulty::Hard,
            Difficulty::Expert,
        ] {
            let state = GameState::new(difficulty);
            assert_eq!(state.difficulty(), difficulty);
            assert_eq!(state.screen(), ScreenState::Playing);
        }
    }

    #[test]
    fn test_cursor_navigation() {
        let mut state = GameState::new(Difficulty::Easy);

        // Initial cursor should be at center (4, 4)
        assert_eq!(state.cursor(), Position::new(4, 4));

        // Move up
        state.handle_key("ArrowUp", false, false);
        assert_eq!(state.cursor(), Position::new(3, 4));

        // Move down
        state.handle_key("ArrowDown", false, false);
        assert_eq!(state.cursor(), Position::new(4, 4));

        // Move left
        state.handle_key("ArrowLeft", false, false);
        assert_eq!(state.cursor(), Position::new(4, 3));

        // Move right
        state.handle_key("ArrowRight", false, false);
        assert_eq!(state.cursor(), Position::new(4, 4));
    }

    #[test]
    fn test_vim_navigation() {
        let mut state = GameState::new(Difficulty::Easy);

        // h, j, k, l should work like vim
        state.handle_key("k", false, false); // up
        assert_eq!(state.cursor().row, 3);

        state.handle_key("j", false, false); // down
        assert_eq!(state.cursor().row, 4);

        state.handle_key("h", false, false); // left
        assert_eq!(state.cursor().col, 3);

        state.handle_key("l", false, false); // right
        assert_eq!(state.cursor().col, 4);
    }

    #[test]
    fn test_cursor_boundary() {
        let mut state = GameState::new(Difficulty::Easy);

        // Move to top-left
        for _ in 0..10 {
            state.handle_key("ArrowUp", false, false);
            state.handle_key("ArrowLeft", false, false);
        }
        assert_eq!(state.cursor(), Position::new(0, 0));

        // Can't go past boundary
        state.handle_key("ArrowUp", false, false);
        state.handle_key("ArrowLeft", false, false);
        assert_eq!(state.cursor(), Position::new(0, 0));

        // Move to bottom-right
        for _ in 0..10 {
            state.handle_key("ArrowDown", false, false);
            state.handle_key("ArrowRight", false, false);
        }
        assert_eq!(state.cursor(), Position::new(8, 8));
    }

    #[test]
    fn test_mode_toggle() {
        let mut state = GameState::new(Difficulty::Easy);

        assert_eq!(state.mode(), InputMode::Normal);

        // Toggle to candidate mode
        state.handle_key("c", false, false);
        assert_eq!(state.mode(), InputMode::Candidate);

        // Toggle back to normal
        state.handle_key("c", false, false);
        assert_eq!(state.mode(), InputMode::Normal);
    }

    #[test]
    fn test_pause_toggle() {
        let mut state = GameState::new(Difficulty::Easy);

        assert!(!state.is_paused());
        assert_eq!(state.screen(), ScreenState::Playing);

        // Pause
        state.handle_key("p", false, false);
        assert!(state.is_paused());
        assert_eq!(state.screen(), ScreenState::Paused);

        // Unpause
        state.handle_key("p", false, false);
        assert!(!state.is_paused());
        assert_eq!(state.screen(), ScreenState::Playing);
    }

    #[test]
    fn test_mistakes_limit() {
        let state = GameState::new(Difficulty::Easy);
        assert_eq!(MAX_MISTAKES, 3);
        assert!(!state.is_game_over());
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new(Difficulty::Medium);

        // Serialize
        let serialized = state.to_serializable();
        assert_eq!(serialized.difficulty, "Medium");
        assert_eq!(serialized.cursor_row, 4);
        assert_eq!(serialized.cursor_col, 4);
        assert_eq!(serialized.mistakes, 0);

        // Deserialize
        let restored = GameState::from_serializable(serialized);
        assert_eq!(restored.difficulty(), Difficulty::Medium);
        assert_eq!(restored.cursor(), Position::new(4, 4));
    }

    #[test]
    fn test_elapsed_time() {
        let state = GameState::new(Difficulty::Easy);

        // Initially should be 0 or very close
        assert!(state.elapsed_secs() < 2);

        // Format should be MM:SS
        let time_str = state.elapsed_string();
        assert!(time_str.contains(':'));
        assert_eq!(time_str.len(), 5);
    }

    #[test]
    fn test_box_navigation() {
        let mut state = GameState::new(Difficulty::Easy);

        // Start at center (4, 4) which is center of center box
        assert_eq!(state.cursor(), Position::new(4, 4));

        // Jump to box above (w)
        state.handle_key("w", false, false);
        assert_eq!(state.cursor().row, 1); // Center row of top-center box

        // Jump to box below (s)
        state.handle_key("s", false, false);
        assert_eq!(state.cursor().row, 4); // Back to center

        // Jump to box left (a)
        state.handle_key("a", false, false);
        assert_eq!(state.cursor().col, 1); // Center col of center-left box

        // Jump to box right (d)
        state.handle_key("d", false, false);
        assert_eq!(state.cursor().col, 4); // Back to center
    }

    #[test]
    fn test_grid_has_values() {
        let state = GameState::new(Difficulty::Easy);

        // Should have some given values
        let grid = state.grid();
        let values = grid.values();

        let mut filled_count = 0;
        for row in 0..9 {
            for col in 0..9 {
                if values[row][col].is_some() {
                    filled_count += 1;
                }
            }
        }

        // Easy puzzle should have at least 35 given cells
        assert!(filled_count >= 35);
    }

    #[test]
    fn test_solution_is_valid() {
        let state = GameState::new(Difficulty::Easy);
        let solution = state.solution();

        // Solution should be complete
        assert!(solution.is_complete());

        // Solution should be valid
        let validation = solution.validate();
        assert!(validation.is_valid);
    }

    #[test]
    fn test_completed_numbers() {
        let state = GameState::new(Difficulty::Easy);
        let completed = state.completed_numbers();

        // Array should have 9 elements
        assert_eq!(completed.len(), 9);
    }

    #[test]
    fn test_highlight_same_row_col_box() {
        let state = GameState::new(Difficulty::Easy);

        // Cursor is at (4, 4)
        // Same row should be highlighted
        assert!(state.is_highlighted(Position::new(4, 0)));
        assert!(state.is_highlighted(Position::new(4, 8)));

        // Same column should be highlighted
        assert!(state.is_highlighted(Position::new(0, 4)));
        assert!(state.is_highlighted(Position::new(8, 4)));

        // Same box should be highlighted (center box: rows 3-5, cols 3-5)
        assert!(state.is_highlighted(Position::new(3, 3)));
        assert!(state.is_highlighted(Position::new(5, 5)));

        // Different row/col/box should not be highlighted
        assert!(!state.is_highlighted(Position::new(0, 0)));
        assert!(!state.is_highlighted(Position::new(8, 8)));
    }

    #[test]
    fn test_tick_updates_frame() {
        let mut state = GameState::new(Difficulty::Easy);
        let initial_frame = state.frame();

        state.tick();
        assert_eq!(state.frame(), initial_frame + 1);

        state.tick();
        state.tick();
        assert_eq!(state.frame(), initial_frame + 3);
    }

    #[test]
    fn test_new_game_menu() {
        let mut state = GameState::new(Difficulty::Easy);

        // Press 'n' to open new game menu
        state.handle_key("n", false, false);
        assert_eq!(state.screen(), ScreenState::Menu);

        // Press Escape to go back
        state.handle_key("Escape", false, false);
        assert_eq!(state.screen(), ScreenState::Playing);
    }

    #[test]
    fn test_quit_returns_false() {
        let mut state = GameState::new(Difficulty::Easy);

        // 'q' should return false to signal quit
        let should_continue = state.handle_key("q", false, false);
        assert!(!should_continue);
    }

    #[test]
    fn test_fill_candidates() {
        let mut state = GameState::new(Difficulty::Easy);

        // Find an empty cell
        let grid = state.grid();
        let values = grid.values();
        let mut empty_pos = None;

        for row in 0..9 {
            for col in 0..9 {
                if values[row][col].is_none() {
                    empty_pos = Some(Position::new(row, col));
                    break;
                }
            }
            if empty_pos.is_some() {
                break;
            }
        }

        if let Some(pos) = empty_pos {
            // Move cursor to empty cell
            while state.cursor() != pos {
                if state.cursor().row < pos.row {
                    state.handle_key("j", false, false);
                } else if state.cursor().row > pos.row {
                    state.handle_key("k", false, false);
                }
                if state.cursor().col < pos.col {
                    state.handle_key("l", false, false);
                } else if state.cursor().col > pos.col {
                    state.handle_key("h", false, false);
                }
            }

            // Fill candidates
            state.handle_key("f", false, false);

            // Cell should now have candidates
            let cell = state.grid().cell(pos);
            assert!(cell.candidates().count() > 0);
        }
    }
}
