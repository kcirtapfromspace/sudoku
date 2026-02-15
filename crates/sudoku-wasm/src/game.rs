//! Game state management for WASM Sudoku

use crate::animations::{LoseScreen, WinScreen};
use serde::{Deserialize, Serialize};
use sudoku_core::{
    BitSet, Difficulty, Generator, Grid, Hint, HintType, Position, PuzzleId, Solver,
};

/// Maximum mistakes before game over
pub const MAX_MISTAKES: usize = 3;

/// Estimated total puzzles in the puzzle universe (~10^30)
pub const TOTAL_PUZZLE_UNIVERSE: f64 = 1e30;

/// Level of hint detail shown to the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintDetailLevel {
    /// Technique name + involved cell highlighting
    Summary,
    /// Full proof coloring (AIC polarity, fish sectors, UR floor/roof, etc.)
    ProofDetail,
}

/// A single move recorded for anti-cheat replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveLogEntry {
    /// 0-indexed sequence number
    pub seq: u32,
    /// Milliseconds since game start (pauses excluded)
    pub ms: u32,
    /// Cell index: row*9 + col (0..80)
    pub cell: u8,
    /// What the player did
    pub action: MoveAction,
}

/// The action taken on a cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveAction {
    /// Player placed digit 1-9
    Place(u8),
    /// Player erased cell (stores old value)
    Clear(u8),
    /// Hint system placed digit
    Hint(u8),
    /// Undo restored cell to this value (None = cleared)
    Undo(Option<u8>),
    /// Redo restored cell to this value (None = cleared)
    Redo(Option<u8>),
}

/// Input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Normal,
    Candidate,
}

/// Screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenState {
    Playing,
    Paused,
    Win,
    Lose,
    Menu,
    Stats,
}

/// Player statistics for lifetime tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Total games played
    pub games_played: u32,
    /// Total games won
    pub games_won: u32,
    /// Total play time in seconds
    pub total_play_time_secs: u64,
    /// Current win streak
    pub current_streak: u32,
    /// Best win streak
    pub best_streak: u32,
    /// Best times by difficulty (in seconds)
    pub best_times: std::collections::HashMap<String, u32>,
}

impl PlayerStats {
    /// Record a game completion
    pub fn record_game(&mut self, won: bool, difficulty: Difficulty, time_secs: u32) {
        self.games_played += 1;
        self.total_play_time_secs += time_secs as u64;

        if won {
            self.games_won += 1;
            self.current_streak += 1;
            if self.current_streak > self.best_streak {
                self.best_streak = self.current_streak;
            }

            // Update best time for difficulty
            let diff_key = format!("{:?}", difficulty);
            let entry = self.best_times.entry(diff_key).or_insert(u32::MAX);
            if time_secs < *entry {
                *entry = time_secs;
            }
        } else {
            self.current_streak = 0;
        }
    }

    /// Calculate average solve time (for wins only)
    pub fn avg_solve_time_secs(&self) -> u64 {
        if self.games_won == 0 {
            300 // Default 5 minutes
        } else {
            self.total_play_time_secs / self.games_won as u64
        }
    }

    /// Calculate universe explored percentage text
    pub fn universe_explored_text(&self) -> String {
        if self.games_won == 0 {
            "0 / 10³⁰ puzzles (0%)".to_string()
        } else {
            let percentage = self.games_won as f64 / TOTAL_PUZZLE_UNIVERSE * 100.0;
            let exponent = percentage.log10().floor() as i32;
            format!("{} / 10³⁰ puzzles (10^{}%)", self.games_won, exponent)
        }
    }

    /// Get a cheeky note about universe progress
    pub fn universe_progress_note(&self) -> &'static str {
        match self.games_won {
            0 => "The puzzle universe awaits your first victory!",
            1 => "One small step for you, one giant... well, still tiny step for puzzlekind.",
            2..=9 => "You've made a dent! A very, very, very small dent.",
            10..=99 => "At this rate, you'll finish in approximately... never.",
            100..=999 => "Impressive dedication! The universe remains unimpressed.",
            _ => "A true puzzle warrior! The universe trembles (microscopically).",
        }
    }

    /// Calculate time to complete all puzzles
    pub fn time_to_complete_text(&self) -> String {
        if self.games_won == 0 {
            return "∞ years".to_string();
        }

        let avg_time = self.avg_solve_time_secs() as f64;
        let total_seconds = avg_time * TOTAL_PUZZLE_UNIVERSE;
        let years = total_seconds / (365.25 * 24.0 * 3600.0);

        let exponent = years.log10().floor() as i32;
        let mantissa = years / 10_f64.powi(exponent);

        format!("≈ {:.1} × 10^{} years", mantissa, exponent)
    }

    /// Get a cheeky note about time to complete
    pub fn time_note(&self) -> &'static str {
        if self.games_won == 0 {
            return "Complete a puzzle to see this stat!";
        }

        let avg_minutes = self.avg_solve_time_secs() as f64 / 60.0;
        if avg_minutes < 2.0 {
            "Speed demon! But even at light speed, you'd need multiple universe lifetimes."
        } else if avg_minutes < 5.0 {
            "Quick solver! The heat death of the universe called - it'll wait."
        } else if avg_minutes < 10.0 {
            "Solid pace! Only 10²² generations of your descendants needed to help."
        } else if avg_minutes < 20.0 {
            "Taking your time? Good strategy. You'll still need immortality though."
        } else if avg_minutes < 30.0 {
            "Thoughtful approach! The Sun will burn out first, but hey, no pressure."
        } else {
            "Savoring each puzzle! At this pace, new universes will form and die. Repeatedly."
        }
    }

    /// Win rate as percentage
    pub fn win_rate(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            self.games_won as f64 / self.games_played as f64 * 100.0
        }
    }

    /// Format total play time as HH:MM:SS
    pub fn total_time_formatted(&self) -> String {
        let secs = self.total_play_time_secs;
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }
}

/// Serializable game state for save/load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableState {
    pub puzzle: String,
    pub current: String,
    pub solution: String,
    pub difficulty: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub mode: InputMode,
    pub screen: ScreenState,
    pub elapsed_secs: u32,
    pub mistakes: usize,
    pub hints_used: usize,
    pub message: Option<String>,
    /// Whether secret difficulties are unlocked (backwards-compatible default)
    #[serde(default)]
    pub secrets_unlocked: bool,
}

/// The game state
pub struct GameState {
    /// Current grid (player's progress)
    grid: Grid,
    /// Original puzzle (for reference)
    puzzle: Grid,
    /// Solution
    solution: Grid,
    /// Difficulty level
    difficulty: Difficulty,
    /// Cursor position
    cursor: Position,
    /// Input mode
    mode: InputMode,
    /// Screen state
    screen: ScreenState,
    /// Start timestamp (ms since epoch)
    start_time: f64,
    /// Elapsed time when paused
    paused_elapsed: f64,
    /// Number of mistakes
    mistakes: usize,
    /// Number of hints used
    hints_used: usize,
    /// Current message to display
    message: Option<String>,
    /// Message timer (ticks remaining)
    message_timer: u32,
    /// Current hint
    current_hint: Option<Hint>,
    /// Hint detail level (Summary vs ProofDetail)
    hint_detail: HintDetailLevel,
    /// Undo stack (position, old value, old candidates)
    undo_stack: Vec<(Position, Option<u8>, BitSet)>,
    /// Redo stack (position, old value, old candidates)
    redo_stack: Vec<(Position, Option<u8>, BitSet)>,
    /// Animation frame counter
    frame: u32,
    /// Win screen animation
    win_screen: Option<WinScreen>,
    /// Lose screen animation
    lose_screen: Option<LoseScreen>,
    /// Show ghost hints (valid candidates as faded numbers)
    show_ghost_hints: bool,
    /// Show valid cells (highlight cells with only one valid number)
    show_valid_cells: bool,
    /// Player lifetime statistics
    player_stats: PlayerStats,
    /// Whether we've already recorded this game (prevent double-counting)
    game_recorded: bool,
    /// Seed for deterministic puzzle generation (from PuzzleId)
    seed: Option<u64>,
    /// Konami code progress tracker
    konami_progress: usize,
    /// Whether secret difficulties (Master/Extreme) are unlocked
    secrets_unlocked: bool,
    /// Cached SE (Sudoku Explainer) rating
    se_rating: f32,
    /// Move log for anti-cheat replay (not serialized into save state)
    move_log: Vec<MoveLogEntry>,
    /// Next sequence number for move log
    move_seq: u32,
}

/// Konami code sequence: Up Up Down Down Left Right Left Right B A
const KONAMI_SEQUENCE: [&str; 10] = [
    "ArrowUp",
    "ArrowUp",
    "ArrowDown",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowLeft",
    "ArrowRight",
    "b",
    "a",
];

impl GameState {
    /// Create a new game
    pub fn new(difficulty: Difficulty) -> Self {
        let puzzle_id = PuzzleId::random(difficulty);
        let puzzle = puzzle_id.generate();
        let seed = puzzle_id.seed;

        let mut grid = puzzle.deep_clone();
        grid.clear_all_candidates();

        let solver = Solver::new();
        let solution = solver.solve(&puzzle).expect("Puzzle should be solvable");
        let se_rating = solver.rate_se(&puzzle);

        Self {
            grid,
            puzzle,
            solution,
            difficulty,
            cursor: Position::new(4, 4),
            mode: InputMode::Normal,
            screen: ScreenState::Playing,
            start_time: Self::now(),
            paused_elapsed: 0.0,
            mistakes: 0,
            hints_used: 0,
            message: None,
            message_timer: 0,
            current_hint: None,
            hint_detail: HintDetailLevel::Summary,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame: 0,
            win_screen: None,
            lose_screen: None,
            show_ghost_hints: false,
            show_valid_cells: false,
            player_stats: PlayerStats::default(),
            game_recorded: false,
            seed: Some(seed),
            konami_progress: 0,
            secrets_unlocked: false,
            se_rating,
            move_log: Vec::new(),
            move_seq: 0,
        }
    }

    /// Create a new game preserving player stats and unlock state
    pub fn new_with_stats(difficulty: Difficulty, stats: PlayerStats) -> Self {
        let mut game = Self::new(difficulty);
        game.player_stats = stats;
        game
    }

    /// Create a new game preserving player stats, unlock state, and secrets
    pub fn new_preserving(difficulty: Difficulty, stats: PlayerStats, secrets: bool) -> Self {
        let mut game = Self::new(difficulty);
        game.player_stats = stats;
        game.secrets_unlocked = secrets;
        game
    }

    /// Create a game from an 81-character puzzle string
    pub fn from_puzzle_string(puzzle: &str) -> Option<Self> {
        let puzzle_grid = Grid::from_string(puzzle)?;
        let solver = Solver::new();
        let solution = solver.solve(&puzzle_grid)?;
        let difficulty = solver.rate_difficulty(&puzzle_grid);
        let se_rating = solver.rate_se(&puzzle_grid);

        let mut grid = puzzle_grid.deep_clone();
        grid.clear_all_candidates();

        Some(Self {
            grid,
            puzzle: puzzle_grid,
            solution,
            difficulty,
            cursor: Position::new(4, 4),
            mode: InputMode::Normal,
            screen: ScreenState::Playing,
            start_time: Self::now(),
            paused_elapsed: 0.0,
            mistakes: 0,
            hints_used: 0,
            message: None,
            message_timer: 0,
            current_hint: None,
            hint_detail: HintDetailLevel::Summary,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame: 0,
            win_screen: None,
            lose_screen: None,
            show_ghost_hints: false,
            show_valid_cells: false,
            player_stats: PlayerStats::default(),
            game_recorded: false,
            seed: None,
            konami_progress: 0,
            secrets_unlocked: false,
            se_rating,
            move_log: Vec::new(),
            move_seq: 0,
        })
    }

    /// Create a game from a short code (e.g., "M1A2B3C4")
    pub fn from_short_code(code: &str) -> Option<Self> {
        let puzzle_id = PuzzleId::from_short_code(code)?;
        let puzzle = puzzle_id.generate();
        let seed = puzzle_id.seed;
        let difficulty = puzzle_id.difficulty;

        let solver = Solver::new();
        let solution = solver.solve(&puzzle)?;
        let se_rating = solver.rate_se(&puzzle);

        let mut grid = puzzle.deep_clone();
        grid.clear_all_candidates();

        Some(Self {
            grid,
            puzzle,
            solution,
            difficulty,
            cursor: Position::new(4, 4),
            mode: InputMode::Normal,
            screen: ScreenState::Playing,
            start_time: Self::now(),
            paused_elapsed: 0.0,
            mistakes: 0,
            hints_used: 0,
            message: None,
            message_timer: 0,
            current_hint: None,
            hint_detail: HintDetailLevel::Summary,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame: 0,
            win_screen: None,
            lose_screen: None,
            show_ghost_hints: false,
            show_valid_cells: false,
            player_stats: PlayerStats::default(),
            game_recorded: false,
            seed: Some(seed),
            konami_progress: 0,
            secrets_unlocked: false,
            se_rating,
            move_log: Vec::new(),
            move_seq: 0,
        })
    }

    /// Get the puzzle as an 81-character string (givens as digits, empty as '.')
    pub fn puzzle_string(&self) -> String {
        self.puzzle.to_string_compact()
    }

    /// Get current timestamp in milliseconds
    fn now() -> f64 {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> u32 {
        if self.screen == ScreenState::Paused
            || self.screen == ScreenState::Win
            || self.screen == ScreenState::Lose
        {
            (self.paused_elapsed / 1000.0) as u32
        } else {
            let elapsed = Self::now() - self.start_time + self.paused_elapsed;
            (elapsed / 1000.0) as u32
        }
    }

    /// Get elapsed time in milliseconds (for move log timestamps)
    fn elapsed_ms(&self) -> u32 {
        let ms = if self.screen == ScreenState::Paused
            || self.screen == ScreenState::Win
            || self.screen == ScreenState::Lose
        {
            self.paused_elapsed
        } else {
            Self::now() - self.start_time + self.paused_elapsed
        };
        ms.max(0.0) as u32
    }

    /// Append a move to the log
    fn log_move(&mut self, pos: Position, action: MoveAction) {
        let entry = MoveLogEntry {
            seq: self.move_seq,
            ms: self.elapsed_ms(),
            cell: (pos.row * 9 + pos.col) as u8,
            action,
        };
        self.move_seq += 1;
        self.move_log.push(entry);
    }

    /// Get the move log as JSON for submission
    pub fn move_log_json(&self) -> String {
        serde_json::to_string(&self.move_log).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get formatted elapsed time
    pub fn elapsed_string(&self) -> String {
        let secs = self.elapsed_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Update game state (called each frame)
    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);

        // Update message timer
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message = None;
            }
        }

        // Check win/lose conditions
        if self.screen == ScreenState::Playing {
            if self.is_complete() {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Win;
                // Record the win
                if !self.game_recorded {
                    self.player_stats
                        .record_game(true, self.difficulty, self.elapsed_secs());
                    self.game_recorded = true;
                    self.check_gameplay_unlock();
                }
                // Create win screen animation
                let seed = (Self::now() * 1000.0) as u64;
                self.win_screen = Some(WinScreen::new(seed));
            } else if self.mistakes >= MAX_MISTAKES {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Lose;
                // Record the loss
                if !self.game_recorded {
                    self.player_stats
                        .record_game(false, self.difficulty, self.elapsed_secs());
                    self.game_recorded = true;
                }
                // Create lose screen animation
                let seed = (Self::now() * 1000.0) as u64;
                self.lose_screen = Some(LoseScreen::new(seed));
            }
        }

        // Update animation screens
        if let Some(ref mut win_screen) = self.win_screen {
            win_screen.update();
        }
        if let Some(ref mut lose_screen) = self.lose_screen {
            lose_screen.update();
        }
    }

    /// Handle keyboard input, returns true if game should continue
    pub fn handle_key(&mut self, key: &str, shift: bool, ctrl: bool) -> bool {
        // Clear hint on any key except "?" (which progresses hint detail)
        if key != "?" {
            self.current_hint = None;
            self.hint_detail = HintDetailLevel::Summary;
        }

        match self.screen {
            ScreenState::Win | ScreenState::Lose => self.handle_endgame_key(key),
            ScreenState::Paused => self.handle_paused_key(key),
            ScreenState::Menu => self.handle_menu_key(key),
            ScreenState::Stats => self.handle_stats_key(key),
            ScreenState::Playing => self.handle_playing_key(key, shift, ctrl),
        }
    }

    fn handle_endgame_key(&mut self, key: &str) -> bool {
        // Track Konami code
        if self.check_konami(key) {
            return true;
        }

        match key {
            "q" | "Q" | "Escape" => return false,
            "s" | "S" => self.screen = ScreenState::Stats,
            "n" | "N" | "Enter" | " " => {
                *self = GameState::new_preserving(
                    self.difficulty,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                );
            }
            "1" => {
                *self = GameState::new_preserving(
                    Difficulty::Beginner,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "2" => {
                *self = GameState::new_preserving(
                    Difficulty::Easy,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "3" => {
                *self = GameState::new_preserving(
                    Difficulty::Medium,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "4" => {
                *self = GameState::new_preserving(
                    Difficulty::Intermediate,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "5" => {
                *self = GameState::new_preserving(
                    Difficulty::Hard,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "6" => {
                *self = GameState::new_preserving(
                    Difficulty::Expert,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "7" if self.secrets_unlocked => {
                *self = GameState::new_preserving(
                    Difficulty::Master,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "8" if self.secrets_unlocked => {
                *self = GameState::new_preserving(
                    Difficulty::Extreme,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            _ => {}
        }
        true
    }

    fn handle_paused_key(&mut self, key: &str) -> bool {
        match key {
            "q" | "Escape" => return false,
            "s" => self.screen = ScreenState::Stats,
            "p" | " " | "Enter" => {
                self.screen = ScreenState::Playing;
                self.start_time = Self::now();
            }
            _ => {}
        }
        true
    }

    fn handle_menu_key(&mut self, key: &str) -> bool {
        // Track Konami code
        if self.check_konami(key) {
            return true;
        }

        match key {
            "Escape" => self.screen = ScreenState::Playing,
            "s" => self.screen = ScreenState::Stats,
            "1" => {
                *self = GameState::new_preserving(
                    Difficulty::Beginner,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "2" => {
                *self = GameState::new_preserving(
                    Difficulty::Easy,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "3" => {
                *self = GameState::new_preserving(
                    Difficulty::Medium,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "4" => {
                *self = GameState::new_preserving(
                    Difficulty::Intermediate,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "5" => {
                *self = GameState::new_preserving(
                    Difficulty::Hard,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "6" => {
                *self = GameState::new_preserving(
                    Difficulty::Expert,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "7" if self.secrets_unlocked => {
                *self = GameState::new_preserving(
                    Difficulty::Master,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            "8" if self.secrets_unlocked => {
                *self = GameState::new_preserving(
                    Difficulty::Extreme,
                    self.player_stats.clone(),
                    self.secrets_unlocked,
                )
            }
            _ => {}
        }
        true
    }

    fn handle_stats_key(&mut self, key: &str) -> bool {
        match key {
            "Escape" | "s" | " " | "Enter" => self.screen = ScreenState::Menu,
            "q" => return false,
            _ => {}
        }
        true
    }

    /// Check Konami code progress. Returns true if the key was consumed.
    fn check_konami(&mut self, key: &str) -> bool {
        if self.secrets_unlocked {
            return false;
        }
        if key == KONAMI_SEQUENCE[self.konami_progress] {
            self.konami_progress += 1;
            if self.konami_progress >= KONAMI_SEQUENCE.len() {
                self.konami_progress = 0;
                self.secrets_unlocked = true;
                self.show_message("Secrets unlocked! 7: Master  8: Extreme");
                return true;
            }
        } else if key == KONAMI_SEQUENCE[0] {
            self.konami_progress = 1;
        } else {
            self.konami_progress = 0;
        }
        false
    }

    fn handle_playing_key(&mut self, key: &str, shift: bool, ctrl: bool) -> bool {
        match key {
            // Quit
            "q" if !shift && !ctrl => return false,

            // Navigation
            "ArrowUp" | "k" => self.move_cursor(-1, 0),
            "ArrowDown" | "j" => self.move_cursor(1, 0),
            "ArrowLeft" | "h" => self.move_cursor(0, -1),
            "ArrowRight" | "l" => self.move_cursor(0, 1),

            // Box navigation
            "w" => self.jump_box(-1, 0),
            "s" if !shift => self.jump_box(1, 0),
            "a" => self.jump_box(0, -1),
            "d" => self.jump_box(0, 1),

            // Number input
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let value = key.parse::<u8>().unwrap();
                if shift || self.mode == InputMode::Candidate {
                    self.toggle_candidate(value);
                } else {
                    self.set_value(value);
                }
            }

            // Clear cell
            "0" | "Delete" | "Backspace" => {
                if self.mode == InputMode::Candidate {
                    self.clear_candidates();
                } else {
                    self.clear_cell();
                }
            }

            // Clear notes (x = current cell, X = all cells)
            "x" if !shift => self.clear_candidates(),
            "X" | "x" if shift => self.clear_all_candidates(),

            // Fill candidates (f = current cell, F = all cells)
            "f" if !shift => self.fill_candidates(),
            "F" | "f" if shift => self.fill_all_candidates(),

            // Mode toggle
            "c" => {
                self.mode = match self.mode {
                    InputMode::Normal => InputMode::Candidate,
                    InputMode::Candidate => InputMode::Normal,
                };
                let mode_name = match self.mode {
                    InputMode::Normal => "Normal",
                    InputMode::Candidate => "Candidate",
                };
                self.show_message(&format!("{} mode", mode_name));
            }

            // Undo/Redo
            "u" => {
                if self.undo() {
                    self.show_message("Undo");
                }
            }
            "r" if ctrl => {
                if self.redo() {
                    self.show_message("Redo");
                }
            }

            // Hint (progressive: first press = Summary, second = ProofDetail)
            "?" => {
                if self.current_hint.is_some() {
                    // Already showing a hint — upgrade to proof detail
                    self.hint_detail = HintDetailLevel::ProofDetail;
                } else if let Some(hint) = self.get_hint() {
                    self.current_hint = Some(hint);
                    self.hint_detail = HintDetailLevel::Summary;
                    self.hints_used += 1;
                } else {
                    self.show_message("No hint available");
                }
            }

            // Apply hint
            "!" => {
                if let Some(pos) = self.apply_hint() {
                    self.cursor = pos;
                    self.show_message("Hint applied");
                }
            }

            // New game
            "n" => self.screen = ScreenState::Menu,

            // Pause
            "p" => {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Paused;
            }

            // Stats
            "S" | "s" if shift => {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Stats;
            }

            // Ghost hints toggle
            "g" => {
                self.show_ghost_hints = !self.show_ghost_hints;
                let status = if self.show_ghost_hints { "ON" } else { "OFF" };
                self.show_message(&format!("Ghost hints: {}", status));
            }

            // Valid cells toggle
            "v" => {
                self.show_valid_cells = !self.show_valid_cells;
                let status = if self.show_valid_cells { "ON" } else { "OFF" };
                self.show_message(&format!("Valid cells: {}", status));
            }

            _ => {}
        }
        true
    }

    fn move_cursor(&mut self, row_delta: i32, col_delta: i32) {
        let new_row = (self.cursor.row as i32 + row_delta).clamp(0, 8) as usize;
        let new_col = (self.cursor.col as i32 + col_delta).clamp(0, 8) as usize;
        self.cursor = Position::new(new_row, new_col);
    }

    fn jump_box(&mut self, row_delta: i32, col_delta: i32) {
        let box_row = (self.cursor.row / 3) as i32;
        let box_col = (self.cursor.col / 3) as i32;

        let new_box_row = (box_row + row_delta).clamp(0, 2) as usize;
        let new_box_col = (box_col + col_delta).clamp(0, 2) as usize;

        self.cursor = Position::new(new_box_row * 3 + 1, new_box_col * 3 + 1);
    }

    fn set_value(&mut self, value: u8) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() {
            return;
        }

        // Check if correct
        let is_correct = self.solution.get(self.cursor) == Some(value);
        if !is_correct {
            self.mistakes += 1;
            let remaining = MAX_MISTAKES.saturating_sub(self.mistakes);
            if remaining > 0 {
                self.show_message(&format!(
                    "Incorrect! {} {} left",
                    remaining,
                    if remaining == 1 { "chance" } else { "chances" }
                ));
            }
        }

        // Save for undo (including current candidates so they can be restored)
        let old_value = self.grid.get(self.cursor);
        let old_candidates = self.grid.cell(self.cursor).candidates();
        self.undo_stack.push((self.cursor, old_value, old_candidates));
        self.redo_stack.clear();

        // Set the value and remove it from peer candidates
        self.grid.set_cell_unchecked(self.cursor, Some(value));
        self.grid.update_candidates_after_move(self.cursor, value);

        // Log the move
        self.log_move(self.cursor, MoveAction::Place(value));
    }

    fn clear_cell(&mut self) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.value().is_none() {
            return;
        }

        let old_value = self.grid.get(self.cursor);
        let old_candidates = self.grid.cell(self.cursor).candidates();
        self.undo_stack.push((self.cursor, old_value, old_candidates));
        self.redo_stack.clear();

        self.grid.set_cell_unchecked(self.cursor, None);

        // Log the clear (old_value is always Some here due to guard above)
        if let Some(v) = old_value {
            self.log_move(self.cursor, MoveAction::Clear(v));
        }
    }

    fn toggle_candidate(&mut self, value: u8) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.is_filled() {
            return;
        }
        self.grid.cell_mut(self.cursor).toggle_candidate(value);
    }

    fn clear_candidates(&mut self) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.is_filled() {
            return;
        }
        self.grid
            .cell_mut(self.cursor)
            .set_candidates(BitSet::empty());

        self.show_message("Cleared notes");
    }

    fn fill_candidates(&mut self) {
        let cell = self.grid.cell(self.cursor);
        if cell.is_given() || cell.is_filled() {
            return;
        }
        let valid = self.grid.compute_candidates(self.cursor);
        self.grid.cell_mut(self.cursor).set_candidates(valid);

        self.show_message("Filled valid notes");
    }

    fn fill_all_candidates(&mut self) {
        self.grid.recalculate_candidates();

        self.show_message("Filled all notes");
    }

    fn clear_all_candidates(&mut self) {
        self.grid.clear_all_candidates();

        self.show_message("Cleared all notes");
    }

    fn undo(&mut self) -> bool {
        if let Some((pos, old_value, old_candidates)) = self.undo_stack.pop() {
            let current_value = self.grid.get(pos);
            let current_candidates = self.grid.cell(pos).candidates();
            self.redo_stack.push((pos, current_value, current_candidates));

            self.grid.set_cell_unchecked(pos, old_value);
            // Restore the cell's own candidates from before the move
            if old_value.is_none() {
                self.grid.cell_mut(pos).set_candidates(old_candidates);
            }
            // If restoring a value, remove it from peer candidates
            if let Some(v) = old_value {
                self.grid.update_candidates_after_move(pos, v);
            }

            self.log_move(pos, MoveAction::Undo(old_value));
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        if let Some((pos, value, saved_candidates)) = self.redo_stack.pop() {
            let current_value = self.grid.get(pos);
            let current_candidates = self.grid.cell(pos).candidates();
            self.undo_stack.push((pos, current_value, current_candidates));

            self.grid.set_cell_unchecked(pos, value);
            // Restore the cell's candidates from the redo snapshot
            if value.is_none() {
                self.grid.cell_mut(pos).set_candidates(saved_candidates);
            }
            // If placing a value, remove it from peer candidates
            if let Some(v) = value {
                self.grid.update_candidates_after_move(pos, v);
            }

            self.log_move(pos, MoveAction::Redo(value));
            true
        } else {
            false
        }
    }

    fn get_hint(&self) -> Option<Hint> {
        let solver = Solver::new();
        solver.get_hint(&self.grid)
    }

    fn apply_hint(&mut self) -> Option<Position> {
        let solver = Solver::new();
        let hint = solver.get_next_placement(&self.grid)?;
        self.hints_used += 1;

        match hint.hint_type {
            HintType::SetValue { pos, value } => {
                // Use the original puzzle solution to determine the correct value.
                // get_next_placement() solves the *current* grid (which may contain
                // player mistakes), so its placement can disagree with the original
                // solution. Always trust self.solution to avoid false "mistake" counts.
                let correct_value = self.solution.get(pos).unwrap_or(value);
                self.cursor = pos;
                self.set_value(correct_value);

                // Reclassify the Place entry that set_value just logged as Hint
                if let Some(last) = self.move_log.last_mut() {
                    if let MoveAction::Place(v) = last.action {
                        last.action = MoveAction::Hint(v);
                    }
                }

                Some(pos)
            }
            HintType::EliminateCandidates { .. } => {
                // get_next_placement should always return SetValue, but
                // handle this defensively just in case
                None
            }
        }
    }

    fn show_message(&mut self, msg: &str) {
        self.message = Some(msg.to_string());
        self.message_timer = 90; // ~3 seconds at 30fps
    }

    // Getters
    pub fn grid(&self) -> &Grid {
        &self.grid
    }
    pub fn puzzle(&self) -> &Grid {
        &self.puzzle
    }
    pub fn solution(&self) -> &Grid {
        &self.solution
    }
    pub fn cursor(&self) -> Position {
        self.cursor
    }
    pub fn mode(&self) -> InputMode {
        self.mode
    }
    pub fn screen(&self) -> ScreenState {
        self.screen
    }
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }
    pub fn se_rating(&self) -> f32 {
        self.se_rating
    }
    pub fn mistakes(&self) -> usize {
        self.mistakes
    }
    pub fn hints_used(&self) -> usize {
        self.hints_used
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    pub fn current_hint(&self) -> Option<&Hint> {
        self.current_hint.as_ref()
    }
    pub fn hint_detail(&self) -> HintDetailLevel {
        self.hint_detail
    }
    pub fn frame(&self) -> u32 {
        self.frame
    }
    pub fn win_screen(&self) -> Option<&WinScreen> {
        self.win_screen.as_ref()
    }
    pub fn lose_screen(&self) -> Option<&LoseScreen> {
        self.lose_screen.as_ref()
    }
    pub fn show_ghost_hints(&self) -> bool {
        self.show_ghost_hints
    }
    pub fn show_valid_cells(&self) -> bool {
        self.show_valid_cells
    }
    pub fn player_stats(&self) -> &PlayerStats {
        &self.player_stats
    }
    pub fn seed(&self) -> Option<u64> {
        self.seed
    }
    pub fn secrets_unlocked(&self) -> bool {
        self.secrets_unlocked
    }

    /// Get the short code for the current puzzle (e.g., "M1A2B3C4")
    pub fn short_code(&self) -> Option<String> {
        self.seed.map(|s| {
            let id = PuzzleId {
                difficulty: self.difficulty,
                seed: s,
            };
            id.to_short_code()
        })
    }

    /// Get player stats as JSON for persistence
    pub fn stats_json(&self) -> String {
        serde_json::to_string(&self.player_stats).unwrap_or_default()
    }

    /// Load player stats from JSON
    pub fn load_stats_json(&mut self, json: &str) -> bool {
        if let Ok(stats) = serde_json::from_str(json) {
            self.player_stats = stats;
            self.check_gameplay_unlock();
            true
        } else {
            false
        }
    }

    /// Get ghost candidates for a cell (valid candidates computed from grid state)
    pub fn get_ghost_candidates(&self, pos: Position) -> Vec<u8> {
        if self.grid.cell(pos).is_filled() || self.grid.cell(pos).is_given() {
            return Vec::new();
        }
        self.grid.compute_candidates(pos).iter().collect()
    }

    /// Check if a cell has only one valid candidate (naked single)
    pub fn is_naked_single(&self, pos: Position) -> bool {
        if self.grid.cell(pos).is_filled() || self.grid.cell(pos).is_given() {
            return false;
        }
        self.grid.compute_candidates(pos).count() == 1
    }

    pub fn is_complete(&self) -> bool {
        self.grid.is_complete() && self.grid.validate().is_valid
    }

    pub fn is_game_over(&self) -> bool {
        self.mistakes >= MAX_MISTAKES
    }

    pub fn is_paused(&self) -> bool {
        self.screen == ScreenState::Paused
    }

    pub fn toggle_pause(&mut self) {
        match self.screen {
            ScreenState::Playing => {
                self.paused_elapsed += Self::now() - self.start_time;
                self.screen = ScreenState::Paused;
            }
            ScreenState::Paused => {
                self.start_time = Self::now();
                self.screen = ScreenState::Playing;
            }
            _ => {}
        }
    }

    /// Check if a cell has a conflict
    #[allow(clippy::needless_range_loop)]
    pub fn has_conflict(&self, pos: Position) -> bool {
        if let Some(value) = self.grid.get(pos) {
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

    /// Check if position is highlighted (same row/col/box as cursor)
    pub fn is_highlighted(&self, pos: Position) -> bool {
        pos.row == self.cursor.row
            || pos.col == self.cursor.col
            || pos.box_index() == self.cursor.box_index()
    }

    /// Check if position has same value as cursor
    pub fn has_same_value(&self, pos: Position) -> bool {
        if let Some(cursor_value) = self.grid.get(self.cursor) {
            self.grid.get(pos) == Some(cursor_value)
        } else {
            false
        }
    }

    /// Get completed numbers (all 9 placed)
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

    /// Convert to serializable format
    pub fn to_serializable(&self) -> SerializableState {
        SerializableState {
            puzzle: self.puzzle.to_string_compact(),
            current: self.grid.to_string_compact(),
            solution: self.solution.to_string_compact(),
            difficulty: format!("{:?}", self.difficulty),
            cursor_row: self.cursor.row,
            cursor_col: self.cursor.col,
            mode: self.mode,
            // Don't persist terminal states — on reload, go to menu instead
            screen: match self.screen {
                ScreenState::Win | ScreenState::Lose => ScreenState::Menu,
                other => other,
            },
            elapsed_secs: self.elapsed_secs(),
            mistakes: self.mistakes,
            hints_used: self.hints_used,
            message: self.message.clone(),
            secrets_unlocked: self.secrets_unlocked,
        }
    }

    /// Create from serializable format
    pub fn from_serializable(state: SerializableState) -> Self {
        let puzzle = Grid::from_string(&state.puzzle).unwrap_or_else(|| {
            let mut gen = Generator::new();
            gen.generate(Difficulty::Medium)
        });
        let grid = Grid::from_string(&state.current).unwrap_or_else(|| puzzle.deep_clone());
        let solution = Grid::from_string(&state.solution).unwrap_or_else(|| {
            let solver = Solver::new();
            solver.solve(&puzzle).unwrap_or_else(|| puzzle.deep_clone())
        });

        let difficulty = match state.difficulty.as_str() {
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

        Self {
            grid,
            puzzle,
            solution,
            difficulty,
            cursor: Position::new(state.cursor_row.min(8), state.cursor_col.min(8)),
            mode: state.mode,
            screen: state.screen,
            start_time: Self::now() - (state.elapsed_secs as f64 * 1000.0),
            paused_elapsed: 0.0,
            mistakes: state.mistakes,
            hints_used: state.hints_used,
            message: state.message,
            message_timer: 0,
            current_hint: None,
            hint_detail: HintDetailLevel::Summary,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            frame: 0,
            win_screen: None,
            lose_screen: None,
            show_ghost_hints: false,
            show_valid_cells: false,
            player_stats: PlayerStats::default(),
            game_recorded: false,
            seed: None,
            konami_progress: 0,
            secrets_unlocked: state.secrets_unlocked,
            se_rating: 0.0,
            move_log: Vec::new(),
            move_seq: 0,
        }
    }

    /// Auto-unlock secret difficulties based on gameplay achievements.
    /// Beating Expert (or higher) unlocks Master and Extreme.
    fn check_gameplay_unlock(&mut self) {
        if self.secrets_unlocked {
            return;
        }
        if self.player_stats.best_times.contains_key("Expert")
            || self.player_stats.best_times.contains_key("Master")
            || self.player_stats.best_times.contains_key("Extreme")
        {
            self.secrets_unlocked = true;
        }
    }

    /// Set secrets unlocked state (for JS persistence)
    pub fn set_secrets_unlocked(&mut self, unlocked: bool) {
        self.secrets_unlocked = unlocked;
    }
}
