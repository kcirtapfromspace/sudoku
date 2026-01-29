use crate::animations::{LoseScreen, WinScreen};
use crate::game::Game;
use crate::stats::{GameResult, StatsManager};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use sudoku_core::{Difficulty, Hint, Position};

/// Maximum mistakes before game over
pub const MAX_MISTAKES: usize = 3;

/// Result of handling a key press
pub enum AppAction {
    Continue,
    Quit,
}

/// Current screen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    /// Normal gameplay
    Playing,
    /// Win celebration screen
    Win,
    /// Game over screen (too many mistakes)
    Lose,
    /// Statistics screen
    Stats,
    /// Leaderboard screen
    Leaderboard,
    /// Game history screen (for replay selection)
    History,
}

/// Input mode for the app
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InputMode {
    /// Normal mode: numbers set values
    Normal,
    /// Candidate mode: numbers toggle candidates
    Candidate,
    /// Menu mode: selecting options
    Menu,
}

/// Menu state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MenuState {
    None,
    NewGame,
    Difficulty,
    Theme,
    Confirm,
}

/// The main application state
pub struct App {
    /// Current game
    pub game: Game,
    /// Currently selected cell position
    pub cursor: Position,
    /// Current input mode
    pub mode: InputMode,
    /// Current menu state
    pub menu: MenuState,
    /// Selected menu item
    pub menu_selection: usize,
    /// Color theme
    pub theme: Theme,
    /// Current hint to display
    pub current_hint: Option<Hint>,
    /// Message to display
    pub message: Option<String>,
    /// Message timer
    message_timer: u32,
    /// Current screen state
    pub screen_state: ScreenState,
    /// Win screen animation
    pub win_screen: WinScreen,
    /// Lose screen animation
    pub lose_screen: LoseScreen,
    /// Whether to show valid candidate suggestions
    pub show_suggestions: bool,
    /// Whether to show naked singles (cells with only one candidate) as hints
    pub show_naked_singles: bool,
    /// Statistics manager
    pub stats: StatsManager,
    /// Whether current game has been recorded (to avoid double recording)
    game_recorded: bool,
    /// Selected difficulty for leaderboard filter
    pub leaderboard_difficulty: Difficulty,
    /// Scroll offset for history view
    pub history_scroll: usize,
    /// Konami code progress (for easter egg)
    konami_progress: usize,
    /// Reverse Konami code progress (lose screen easter egg)
    reverse_konami_progress: usize,
    /// "42" pattern progress (The Answer)
    the_answer_progress: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Konami code sequence: ↑↑↓↓←→←→BA
    const KONAMI_CODE: [KeyCode; 10] = [
        KeyCode::Up, KeyCode::Up, KeyCode::Down, KeyCode::Down,
        KeyCode::Left, KeyCode::Right, KeyCode::Left, KeyCode::Right,
        KeyCode::Char('b'), KeyCode::Char('a'),
    ];

    /// Reverse Konami code: AB→←→←↓↓↑↑
    const REVERSE_KONAMI: [KeyCode; 10] = [
        KeyCode::Char('a'), KeyCode::Char('b'),
        KeyCode::Right, KeyCode::Left, KeyCode::Right, KeyCode::Left,
        KeyCode::Down, KeyCode::Down, KeyCode::Up, KeyCode::Up,
    ];

    /// "The Answer" pattern: 42
    const THE_ANSWER: [KeyCode; 2] = [KeyCode::Char('4'), KeyCode::Char('2')];

    /// Create a new app with a medium difficulty game
    pub fn new() -> Self {
        Self {
            game: Game::new(Difficulty::Medium),
            cursor: Position::new(4, 4),
            mode: InputMode::Normal,
            menu: MenuState::None,
            menu_selection: 0,
            theme: Theme::dark(),
            current_hint: None,
            message: None,
            message_timer: 0,
            screen_state: ScreenState::Playing,
            win_screen: WinScreen::new(),
            lose_screen: LoseScreen::new(),
            show_suggestions: true,
            show_naked_singles: false, // Off by default - it's basically cheating!
            stats: StatsManager::load(),
            game_recorded: false,
            leaderboard_difficulty: Difficulty::Medium,
            history_scroll: 0,
            konami_progress: 0,
            reverse_konami_progress: 0,
            the_answer_progress: 0,
        }
    }

    /// Get the tick rate based on current screen
    pub fn get_tick_rate(&self) -> Duration {
        match self.screen_state {
            ScreenState::Win | ScreenState::Lose => Duration::from_millis(33), // 30 FPS for animations
            ScreenState::Playing | ScreenState::Stats | ScreenState::Leaderboard | ScreenState::History => {
                Duration::from_millis(100) // 10 FPS for normal screens
            }
        }
    }

    /// Update animations and timers (called every tick)
    pub fn tick(&mut self) {
        // Update message timer
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message = None;
            }
        }

        // Update animations based on screen state
        match self.screen_state {
            ScreenState::Win => {
                self.win_screen.update();
            }
            ScreenState::Lose => {
                self.lose_screen.update();
            }
            ScreenState::Playing => {
                // Check for win/lose conditions
                if self.game.is_completed() {
                    self.record_game(GameResult::Win);
                    self.screen_state = ScreenState::Win;
                    self.win_screen.reset();
                } else if self.game.mistakes() >= MAX_MISTAKES {
                    self.record_game(GameResult::Loss);
                    self.screen_state = ScreenState::Lose;
                    self.lose_screen.reset();
                }
            }
            ScreenState::Stats | ScreenState::Leaderboard | ScreenState::History => {
                // No animations for these screens
            }
        }
    }

    /// Record the current game to stats
    fn record_game(&mut self, result: GameResult) {
        if self.game_recorded {
            return;
        }
        self.game_recorded = true;

        self.stats.record_game(
            self.game.original_puzzle(),
            self.game.difficulty(),
            result,
            self.game.elapsed().as_secs(),
            self.game.hints_used(),
            self.game.mistakes(),
            self.game.move_times_ms(),
            self.game.notes_used(),
        );
    }

    /// Show a temporary message
    pub fn show_message(&mut self, msg: &str) {
        self.message = Some(msg.to_string());
        self.message_timer = 30; // ~3 seconds at 100ms poll
    }

    /// Handle a key press
    pub fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        // Handle based on screen state
        match self.screen_state {
            ScreenState::Win | ScreenState::Lose => self.handle_endgame_key(key),
            ScreenState::Stats => self.handle_stats_key(key),
            ScreenState::Leaderboard => self.handle_leaderboard_key(key),
            ScreenState::History => self.handle_history_key(key),
            ScreenState::Playing => {
                // Clear hint on any key
                if self.current_hint.is_some() {
                    self.current_hint = None;
                }

                match self.menu {
                    MenuState::None => self.handle_game_key(key),
                    MenuState::NewGame
                    | MenuState::Difficulty
                    | MenuState::Theme
                    | MenuState::Confirm => self.handle_menu_key(key),
                }
            }
        }
    }

    fn handle_endgame_key(&mut self, key: KeyEvent) -> AppAction {
        // Check for Konami code on win screen
        if self.screen_state == ScreenState::Win {
            if key.code == Self::KONAMI_CODE[self.konami_progress] {
                self.konami_progress += 1;
                if self.konami_progress >= Self::KONAMI_CODE.len() {
                    // Konami code completed! Start new game directly
                    self.konami_progress = 0;
                    self.game = Game::new(self.game.difficulty());
                    self.cursor = Position::new(4, 4);
                    self.game_recorded = false;
                    self.screen_state = ScreenState::Playing;
                    self.show_message("KONAMI! New game started!");
                    return AppAction::Continue;
                }
            } else if key.code == Self::KONAMI_CODE[0] {
                // Reset but count this as first key
                self.konami_progress = 1;
            } else {
                self.konami_progress = 0;
            }
        }

        // Check for reverse Konami code on lose screen (Method 10)
        if self.screen_state == ScreenState::Lose {
            if key.code == Self::REVERSE_KONAMI[self.reverse_konami_progress] {
                self.reverse_konami_progress += 1;
                if self.reverse_konami_progress >= Self::REVERSE_KONAMI.len() {
                    self.reverse_konami_progress = 0;
                    self.stats.unlock_via_reverse_konami();
                    self.show_message("REVERSE KONAMI! Secrets unlocked!");
                    return AppAction::Continue;
                }
            } else if key.code == Self::REVERSE_KONAMI[0] {
                self.reverse_konami_progress = 1;
            } else {
                self.reverse_konami_progress = 0;
            }
        }

        // Check for "42" pattern on any end screen (Method 11 - The Answer)
        if self.screen_state == ScreenState::Win || self.screen_state == ScreenState::Lose {
            if key.code == Self::THE_ANSWER[self.the_answer_progress] {
                self.the_answer_progress += 1;
                if self.the_answer_progress >= Self::THE_ANSWER.len() {
                    self.the_answer_progress = 0;
                    self.stats.unlock_via_the_answer();
                    self.show_message("42! The Answer to Life, the Universe, and Everything!");
                    return AppAction::Continue;
                }
            } else if key.code == Self::THE_ANSWER[0] {
                self.the_answer_progress = 1;
            } else {
                self.the_answer_progress = 0;
            }
        }

        match key.code {
            KeyCode::Char('q') => return AppAction::Quit,
            KeyCode::Char('n') => {
                // Start new game - go to difficulty selection menu
                self.screen_state = ScreenState::Playing;
                self.menu = MenuState::NewGame;
                self.menu_selection = 0;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Quick restart with same difficulty
                self.game = Game::new(self.game.difficulty());
                self.cursor = Position::new(4, 4);
                self.game_recorded = false;
                self.screen_state = ScreenState::Playing;
                self.show_message(&format!("New {} game", self.game.difficulty()));
            }
            KeyCode::Esc => {
                // Go back to the (finished) game view
                self.screen_state = ScreenState::Playing;
            }
            _ => {}
        }
        AppAction::Continue
    }

    fn handle_game_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            // Quit - record abandoned game if in progress
            KeyCode::Char('q') => {
                if !self.game.is_completed() && self.game.moves_count() > 0 {
                    self.record_game(GameResult::Abandoned);
                }
                return AppAction::Quit;
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1, 0),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1, 0),
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(0, -1),
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(0, 1),

            // Jump to box
            KeyCode::Char('w') => self.jump_box(-1, 0),
            KeyCode::Char('s') => self.jump_box(1, 0),
            KeyCode::Char('a') => self.jump_box(0, -1),
            KeyCode::Char('d') => self.jump_box(0, 1),

            // Number input
            KeyCode::Char(c @ '1'..='9') => {
                let value = c.to_digit(10).unwrap() as u8;
                if key.modifiers.contains(KeyModifiers::SHIFT) || self.mode == InputMode::Candidate
                {
                    self.game.toggle_candidate(self.cursor, value);
                } else {
                    let correct = self.game.set_value(self.cursor, value);
                    if !correct {
                        let remaining = MAX_MISTAKES.saturating_sub(self.game.mistakes());
                        if remaining > 0 {
                            self.show_message(&format!(
                                "Incorrect! {} {} left",
                                remaining,
                                if remaining == 1 { "chance" } else { "chances" }
                            ));
                        }
                    }
                }
            }

            // Clear cell (value and notes)
            KeyCode::Char('0') | KeyCode::Delete | KeyCode::Backspace => {
                if self.mode == InputMode::Candidate {
                    // In candidate mode, clear all candidates
                    if self.game.clear_candidates(self.cursor) {
                        self.show_message("Cleared notes");
                    }
                } else {
                    // In normal mode, clear the cell value
                    self.game.clear_cell(self.cursor);
                }
            }

            // Clear notes from cell (x) or all cells (Shift+X)
            KeyCode::Char('x') => {
                if self.game.clear_candidates(self.cursor) {
                    self.show_message("Cleared notes");
                }
            }
            KeyCode::Char('X') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                if self.game.clear_all_candidates() {
                    self.show_message("Cleared all notes");
                }
            }

            // Fill cell with valid candidates (f) or all cells (Shift+F)
            KeyCode::Char('f') => {
                if self.game.fill_candidates(self.cursor) {
                    self.show_message("Filled valid notes");
                }
            }
            KeyCode::Char('F') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                if self.game.fill_all_candidates() {
                    self.show_message("Filled all notes");
                }
            }

            // Mode toggle
            KeyCode::Char('c') => {
                self.mode = match self.mode {
                    InputMode::Normal => InputMode::Candidate,
                    InputMode::Candidate => InputMode::Normal,
                    InputMode::Menu => InputMode::Normal,
                };
                let mode_name = match self.mode {
                    InputMode::Normal => "Normal",
                    InputMode::Candidate => "Candidate",
                    InputMode::Menu => "Menu",
                };
                self.show_message(&format!("{} mode", mode_name));
            }

            // Undo/Redo
            KeyCode::Char('u') => {
                if self.game.undo() {
                    self.show_message("Undo");
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.game.redo() {
                    self.show_message("Redo");
                }
            }

            // Hint
            KeyCode::Char('?') => {
                if let Some(hint) = self.game.get_hint() {
                    self.current_hint = Some(hint);
                } else {
                    self.show_message("No hint available");
                }
            }

            // Apply hint
            KeyCode::Char('!') => {
                if let Some(pos) = self.game.apply_hint() {
                    self.cursor = pos;
                    self.show_message("Hint applied");
                }
            }

            // New game menu
            KeyCode::Char('n') => {
                self.menu = MenuState::NewGame;
                self.menu_selection = 0;
            }

            // Pause
            KeyCode::Char('p') => {
                self.game.toggle_pause();
                if self.game.is_paused() {
                    self.show_message("Paused");
                } else {
                    self.show_message("Resumed");
                }
            }

            // Theme toggle
            KeyCode::Char('t') => {
                self.menu = MenuState::Theme;
                self.menu_selection = 0;
            }

            // Toggle suggestions
            KeyCode::Char('v') => {
                self.show_suggestions = !self.show_suggestions;
                let state = if self.show_suggestions { "on" } else { "off" };
                self.show_message(&format!("Suggestions {}", state));
            }

            // Toggle naked singles display (shows answer for cells with only one candidate)
            KeyCode::Char('g') => {
                self.show_naked_singles = !self.show_naked_singles;
                let state = if self.show_naked_singles { "on" } else { "off" };
                self.show_message(&format!("Auto-fill hints {}", state));
            }

            // Save
            KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.save_game();
            }

            // Load
            KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.load_game();
            }

            // Stats screen
            KeyCode::Char('i') => {
                self.screen_state = ScreenState::Stats;
            }

            // Leaderboard
            KeyCode::Char('b') => {
                self.screen_state = ScreenState::Leaderboard;
            }

            // History/Replay
            KeyCode::Char('H') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.history_scroll = 0;
                self.screen_state = ScreenState::History;
            }

            _ => {}
        }

        AppAction::Continue
    }

    fn handle_menu_key(&mut self, key: KeyEvent) -> AppAction {
        // Check for Konami code on new game menu to unlock secrets
        if self.menu == MenuState::NewGame || self.menu == MenuState::Difficulty {
            if key.code == Self::KONAMI_CODE[self.konami_progress] {
                self.konami_progress += 1;
                if self.konami_progress >= Self::KONAMI_CODE.len() {
                    self.konami_progress = 0;
                    self.stats.unlock_via_konami();
                    self.show_message("SECRET LEVELS UNLOCKED!");
                    return AppAction::Continue;
                }
            } else if key.code == Self::KONAMI_CODE[0] {
                self.konami_progress = 1;
            } else {
                self.konami_progress = 0;
            }
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.menu = MenuState::None;
                self.konami_progress = 0;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                if self.menu_selection > 0 {
                    self.menu_selection -= 1;
                }
            }

            KeyCode::Down | KeyCode::Char('j') => {
                let max = match self.menu {
                    MenuState::NewGame | MenuState::Difficulty => {
                        if self.stats.secrets_unlocked() {
                            7 // All 8 difficulties (0-7)
                        } else {
                            5 // Standard 6 difficulties (0-5)
                        }
                    }
                    MenuState::Theme => 2,
                    MenuState::Confirm => 1,
                    MenuState::None => 0,
                };
                if self.menu_selection < max {
                    self.menu_selection += 1;
                }
            }

            KeyCode::Enter | KeyCode::Char(' ') => {
                match self.menu {
                    MenuState::NewGame | MenuState::Difficulty => {
                        let difficulty = self.get_difficulty_from_selection();
                        self.game = Game::new(difficulty);
                        self.cursor = Position::new(4, 4);
                        self.screen_state = ScreenState::Playing;
                        self.game_recorded = false;
                        self.show_message(&format!("New {} game", difficulty));
                        self.menu = MenuState::None;
                    }
                    MenuState::Theme => {
                        self.theme = match self.menu_selection {
                            0 => Theme::dark(),
                            1 => Theme::light(),
                            _ => Theme::high_contrast(),
                        };
                        self.menu = MenuState::None;
                    }
                    MenuState::Confirm => {
                        if self.menu_selection == 0 {
                            // Confirmed
                        }
                        self.menu = MenuState::None;
                    }
                    MenuState::None => {}
                }
            }

            _ => {}
        }

        AppAction::Continue
    }

    /// Get difficulty from current menu selection
    fn get_difficulty_from_selection(&self) -> Difficulty {
        if self.stats.secrets_unlocked() {
            match self.menu_selection {
                0 => Difficulty::Beginner,
                1 => Difficulty::Easy,
                2 => Difficulty::Medium,
                3 => Difficulty::Intermediate,
                4 => Difficulty::Hard,
                5 => Difficulty::Expert,
                6 => Difficulty::Master,
                _ => Difficulty::Extreme,
            }
        } else {
            match self.menu_selection {
                0 => Difficulty::Beginner,
                1 => Difficulty::Easy,
                2 => Difficulty::Medium,
                3 => Difficulty::Intermediate,
                4 => Difficulty::Hard,
                _ => Difficulty::Expert,
            }
        }
    }

    fn handle_stats_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.screen_state = ScreenState::Playing;
            }
            KeyCode::Char('b') => {
                self.screen_state = ScreenState::Leaderboard;
            }
            KeyCode::Char('H') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.history_scroll = 0;
                self.screen_state = ScreenState::History;
            }
            _ => {}
        }
        AppAction::Continue
    }

    fn handle_leaderboard_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.screen_state = ScreenState::Playing;
            }
            KeyCode::Char('i') => {
                self.screen_state = ScreenState::Stats;
            }
            // Change difficulty filter
            KeyCode::Left | KeyCode::Char('h') => {
                self.leaderboard_difficulty = self.prev_difficulty(self.leaderboard_difficulty);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.leaderboard_difficulty = self.next_difficulty(self.leaderboard_difficulty);
            }
            _ => {}
        }
        AppAction::Continue
    }

    fn prev_difficulty(&self, d: Difficulty) -> Difficulty {
        let levels = if self.stats.secrets_unlocked() {
            Difficulty::all_levels()
        } else {
            Difficulty::standard_levels()
        };
        let idx = levels.iter().position(|&x| x == d).unwrap_or(0);
        if idx == 0 {
            levels[levels.len() - 1]
        } else {
            levels[idx - 1]
        }
    }

    fn next_difficulty(&self, d: Difficulty) -> Difficulty {
        let levels = if self.stats.secrets_unlocked() {
            Difficulty::all_levels()
        } else {
            Difficulty::standard_levels()
        };
        let idx = levels.iter().position(|&x| x == d).unwrap_or(0);
        if idx >= levels.len() - 1 {
            levels[0]
        } else {
            levels[idx + 1]
        }
    }

    fn handle_history_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.screen_state = ScreenState::Playing;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.history_scroll = self.history_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.stats.history.len().saturating_sub(1);
                self.history_scroll = (self.history_scroll + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Replay selected game
                if let Some(record) = self.stats.history.get(self.history_scroll) {
                    if let Some(game) = Game::from_string(&record.puzzle) {
                        self.game = game;
                        self.cursor = Position::new(4, 4);
                        self.game_recorded = false;
                        self.screen_state = ScreenState::Playing;
                        self.show_message("Replaying game");
                    }
                }
            }
            KeyCode::Char('i') => {
                self.screen_state = ScreenState::Stats;
            }
            _ => {}
        }
        AppAction::Continue
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

        // Move to center of new box
        self.cursor = Position::new(new_box_row * 3 + 1, new_box_col * 3 + 1);
    }

    /// Get the save file path
    fn save_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sudoku_save.json")
    }

    /// Save the current game
    fn save_game(&mut self) {
        let json = self.game.serialize();
        match fs::write(Self::save_path(), json) {
            Ok(_) => self.show_message("Game saved"),
            Err(_) => self.show_message("Failed to save"),
        }
    }

    /// Load a saved game
    fn load_game(&mut self) {
        match fs::read_to_string(Self::save_path()) {
            Ok(json) => {
                if let Some(game) = Game::deserialize(&json) {
                    self.game = game;
                    self.cursor = Position::new(4, 4);
                    self.screen_state = ScreenState::Playing;
                    self.show_message("Game loaded");
                } else {
                    self.show_message("Invalid save file");
                }
            }
            Err(_) => self.show_message("No save file found"),
        }
    }

    /// Check if a position is highlighted (same row, col, or box as cursor)
    pub fn is_highlighted(&self, pos: Position) -> bool {
        pos.row == self.cursor.row
            || pos.col == self.cursor.col
            || pos.box_index() == self.cursor.box_index()
    }

    /// Check if a position has the same value as the cursor
    pub fn has_same_value(&self, pos: Position) -> bool {
        if let Some(cursor_value) = self.game.grid().get(self.cursor) {
            self.game.grid().get(pos) == Some(cursor_value)
        } else {
            false
        }
    }
}
