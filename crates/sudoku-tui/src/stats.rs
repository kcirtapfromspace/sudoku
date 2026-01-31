#![allow(dead_code)]

use crate::leaderboard::{self, LeaderboardManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sudoku_core::Difficulty;

/// Result of a completed game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    Win,
    Loss,
    Abandoned,
}

/// Record of a single played game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    /// Unique game ID
    pub id: u64,
    /// Hash of the puzzle (for identifying unique puzzles)
    pub puzzle_hash: String,
    /// The original puzzle string (for replay)
    pub puzzle: String,
    /// Difficulty level
    pub difficulty: Difficulty,
    /// Game result
    pub result: GameResult,
    /// Time to complete in seconds
    pub time_secs: u64,
    /// Number of hints used
    pub hints_used: usize,
    /// Number of mistakes made
    pub mistakes: usize,
    /// Total moves made
    pub moves_count: usize,
    /// Unix timestamp when game was completed
    pub timestamp: u64,
    /// Average time between moves in milliseconds (for anti-bot)
    pub avg_move_time_ms: u64,
    /// Minimum time between moves (for anti-bot)
    pub min_move_time_ms: u64,
    /// Standard deviation of move times (for anti-bot)
    pub move_time_std_dev: f32,
    /// Whether this record passed anti-bot verification
    pub verified: bool,
}

impl GameRecord {
    /// Calculate a score for this game (lower is better for time-based)
    /// Score formula: base_time + (hints * 30) + (mistakes * 15)
    /// Only verified wins count
    pub fn score(&self) -> Option<u64> {
        if self.result != GameResult::Win || !self.verified {
            return None;
        }
        Some(self.time_secs + (self.hints_used as u64 * 30) + (self.mistakes as u64 * 15))
    }
}

/// Statistics for a specific difficulty level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifficultyStats {
    pub total_games: usize,
    pub wins: usize,
    pub losses: usize,
    pub abandoned: usize,
    pub best_time_secs: Option<u64>,
    pub worst_time_secs: Option<u64>,
    pub total_time_secs: u64,
    pub total_hints: usize,
    pub total_mistakes: usize,
}

impl DifficultyStats {
    pub fn avg_time_secs(&self) -> Option<u64> {
        if self.wins > 0 {
            Some(self.total_time_secs / self.wins as u64)
        } else {
            None
        }
    }

    pub fn win_rate(&self) -> f32 {
        if self.total_games > 0 {
            self.wins as f32 / self.total_games as f32 * 100.0
        } else {
            0.0
        }
    }
}

/// Overall player statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    pub player_name: String,
    pub total_games: usize,
    pub total_wins: usize,
    pub total_losses: usize,
    pub total_abandoned: usize,
    pub current_streak: i32, // Positive = win streak, negative = loss streak
    pub best_streak: i32,
    pub by_difficulty: HashMap<String, DifficultyStats>,
    /// Whether secret difficulties are unlocked
    pub secret_unlocked: bool,
    /// Number of expert wins (for unlock tracking)
    pub expert_wins: usize,
    /// Total play time in seconds
    pub total_play_time_secs: u64,
    /// Difficulties that have been won at least once
    pub difficulties_won: Vec<String>,
    /// Whether a perfect game (Expert, 0 hints, 0 mistakes) was achieved
    pub perfect_game_achieved: bool,
    /// Whether speed demon (Hard < 5 min) was achieved
    pub speed_demon_achieved: bool,
    /// Whether no-notes master (Hard+ without notes) was achieved
    pub no_notes_master_achieved: bool,
    /// Unlock reason (for display)
    pub unlock_reason: Option<String>,
}

/// Number of expert wins required to unlock secret difficulties
pub const EXPERT_WINS_TO_UNLOCK: usize = 10;
/// Win streak required to unlock
pub const WIN_STREAK_TO_UNLOCK: usize = 5;
/// Total wins required to unlock (Century)
pub const CENTURY_WINS_TO_UNLOCK: usize = 100;
/// Total play time required to unlock in seconds (10 hours)
pub const MARATHON_TIME_TO_UNLOCK: u64 = 10 * 60 * 60;
/// Speed demon time threshold in seconds (5 minutes)
pub const SPEED_DEMON_TIME: u64 = 5 * 60;

impl PlayerStats {
    pub fn new(name: &str) -> Self {
        Self {
            player_name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn overall_win_rate(&self) -> f32 {
        if self.total_games > 0 {
            self.total_wins as f32 / self.total_games as f32 * 100.0
        } else {
            0.0
        }
    }

    pub fn get_difficulty_stats(&self, difficulty: Difficulty) -> DifficultyStats {
        self.by_difficulty
            .get(&format!("{:?}", difficulty))
            .cloned()
            .unwrap_or_default()
    }
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub player_name: String,
    pub score: u64,
    pub time_secs: u64,
    pub difficulty: Difficulty,
    pub hints_used: usize,
    pub mistakes: usize,
    pub timestamp: u64,
    pub puzzle_hash: String,
}

/// Anti-bot verification thresholds
pub struct AntiBot;

impl AntiBot {
    /// Minimum realistic time to solve by difficulty (seconds)
    const MIN_TIME_BEGINNER: u64 = 15;
    const MIN_TIME_EASY: u64 = 30;
    const MIN_TIME_MEDIUM: u64 = 60;
    const MIN_TIME_INTERMEDIATE: u64 = 90;
    const MIN_TIME_HARD: u64 = 120;
    const MIN_TIME_EXPERT: u64 = 180;
    const MIN_TIME_MASTER: u64 = 300;
    const MIN_TIME_EXTREME: u64 = 600;

    /// Minimum average move time (milliseconds) - humans can't click faster than ~100ms consistently
    const MIN_AVG_MOVE_TIME_MS: u64 = 150;

    /// Minimum move time (milliseconds) - single fastest move
    const MIN_SINGLE_MOVE_TIME_MS: u64 = 50;

    /// Minimum standard deviation (bots have very consistent timing)
    const MIN_STD_DEV: f32 = 100.0;

    /// Verify a game record for bot-like behavior
    pub fn verify(record: &GameRecord) -> VerificationResult {
        let mut issues = Vec::new();

        // Check minimum time based on difficulty
        let min_time = match record.difficulty {
            Difficulty::Beginner => Self::MIN_TIME_BEGINNER,
            Difficulty::Easy => Self::MIN_TIME_EASY,
            Difficulty::Medium => Self::MIN_TIME_MEDIUM,
            Difficulty::Intermediate => Self::MIN_TIME_INTERMEDIATE,
            Difficulty::Hard => Self::MIN_TIME_HARD,
            Difficulty::Expert => Self::MIN_TIME_EXPERT,
            Difficulty::Master => Self::MIN_TIME_MASTER,
            Difficulty::Extreme => Self::MIN_TIME_EXTREME,
        };

        if record.time_secs < min_time {
            issues.push(format!(
                "Time too fast: {}s (min {}s for {:?})",
                record.time_secs, min_time, record.difficulty
            ));
        }

        // Check average move time
        if record.avg_move_time_ms < Self::MIN_AVG_MOVE_TIME_MS && record.moves_count > 10 {
            issues.push(format!(
                "Avg move time too fast: {}ms (min {}ms)",
                record.avg_move_time_ms,
                Self::MIN_AVG_MOVE_TIME_MS
            ));
        }

        // Check minimum single move time
        if record.min_move_time_ms < Self::MIN_SINGLE_MOVE_TIME_MS && record.moves_count > 5 {
            issues.push(format!(
                "Fastest move too quick: {}ms (min {}ms)",
                record.min_move_time_ms,
                Self::MIN_SINGLE_MOVE_TIME_MS
            ));
        }

        // Check timing variance (bots are too consistent)
        if record.move_time_std_dev < Self::MIN_STD_DEV && record.moves_count > 15 {
            issues.push(format!(
                "Move timing too consistent: std_dev={:.1}ms (min {:.1}ms)",
                record.move_time_std_dev,
                Self::MIN_STD_DEV
            ));
        }

        // Check for suspicious hint/mistake patterns
        // If someone wins expert with 0 hints, 0 mistakes in minimum time, suspicious
        if record.difficulty == Difficulty::Expert
            && record.hints_used == 0
            && record.mistakes == 0
            && record.time_secs < min_time * 2
        {
            issues.push("Suspiciously perfect expert game".to_string());
        }

        VerificationResult {
            verified: issues.is_empty(),
            issues,
        }
    }
}

/// Result of anti-bot verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub verified: bool,
    pub issues: Vec<String>,
}

/// The main statistics manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsManager {
    /// Player stats
    pub player: PlayerStats,
    /// All game records (most recent first)
    pub history: Vec<GameRecord>,
    /// Local leaderboard entries (best scores, limited to top 100)
    pub leaderboard: Vec<LeaderboardEntry>,
    /// Next game ID
    next_id: u64,
    /// Remote leaderboard manager (not serialized)
    #[serde(skip)]
    remote_leaderboard: Option<Arc<LeaderboardManager>>,
}

impl Default for StatsManager {
    fn default() -> Self {
        Self::new("Player")
    }
}

impl StatsManager {
    pub fn new(player_name: &str) -> Self {
        Self {
            player: PlayerStats::new(player_name),
            history: Vec::new(),
            leaderboard: Vec::new(),
            next_id: 1,
            remote_leaderboard: Some(Arc::new(LeaderboardManager::auto())),
        }
    }

    /// Get the save file path
    fn save_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sudoku_stats.json")
    }

    /// Load stats from file
    pub fn load() -> Self {
        let mut stats: Self = match fs::read_to_string(Self::save_path()) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        };
        // Initialize remote leaderboard after deserialization
        stats.remote_leaderboard = Some(Arc::new(LeaderboardManager::auto()));
        stats
    }

    /// Save stats to file
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::save_path(), json);
        }
    }

    /// Record a completed game
    pub fn record_game(
        &mut self,
        puzzle: &str,
        difficulty: Difficulty,
        result: GameResult,
        time_secs: u64,
        hints_used: usize,
        mistakes: usize,
        move_times_ms: &[u64],
        notes_used: bool,
    ) -> &GameRecord {
        let puzzle_hash = Self::hash_puzzle(puzzle);

        // Calculate move timing stats
        let moves_count = move_times_ms.len();
        let (avg_move_time_ms, min_move_time_ms, move_time_std_dev) = if moves_count > 0 {
            let sum: u64 = move_times_ms.iter().sum();
            let avg = sum / moves_count as u64;
            let min = *move_times_ms.iter().min().unwrap_or(&0);

            // Calculate standard deviation
            let variance: f32 = move_times_ms
                .iter()
                .map(|&t| {
                    let diff = t as f32 - avg as f32;
                    diff * diff
                })
                .sum::<f32>()
                / moves_count as f32;
            let std_dev = variance.sqrt();

            (avg, min, std_dev)
        } else {
            (0, 0, 0.0)
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut record = GameRecord {
            id: self.next_id,
            puzzle_hash,
            puzzle: puzzle.to_string(),
            difficulty,
            result,
            time_secs,
            hints_used,
            mistakes,
            moves_count,
            timestamp,
            avg_move_time_ms,
            min_move_time_ms,
            move_time_std_dev,
            verified: false,
        };

        // Run anti-bot verification
        let verification = AntiBot::verify(&record);
        record.verified = verification.verified;

        self.next_id += 1;

        // Update player stats
        self.player.total_games += 1;
        self.player.total_play_time_secs += time_secs;

        match result {
            GameResult::Win => {
                self.player.total_wins += 1;
                if self.player.current_streak >= 0 {
                    self.player.current_streak += 1;
                } else {
                    self.player.current_streak = 1;
                }
                self.player.best_streak = self.player.best_streak.max(self.player.current_streak);

                // Track difficulty won
                let diff_key = format!("{:?}", difficulty);
                if !self.player.difficulties_won.contains(&diff_key) {
                    self.player.difficulties_won.push(diff_key);
                }

                // Track expert wins for unlock
                if difficulty >= Difficulty::Expert {
                    self.player.expert_wins += 1;
                }

                // Check for Perfect Game (Expert+, 0 hints, 0 mistakes)
                if difficulty >= Difficulty::Expert && hints_used == 0 && mistakes == 0 {
                    self.player.perfect_game_achieved = true;
                }

                // Check for Speed Demon (Hard+ under 5 minutes)
                if difficulty >= Difficulty::Hard && time_secs < SPEED_DEMON_TIME {
                    self.player.speed_demon_achieved = true;
                }

                // Check for No Notes Master (Hard+ without using notes)
                if difficulty >= Difficulty::Hard && !notes_used {
                    self.player.no_notes_master_achieved = true;
                }

                // Check all unlock conditions
                self.check_unlocks();
            }
            GameResult::Loss => {
                self.player.total_losses += 1;
                if self.player.current_streak <= 0 {
                    self.player.current_streak -= 1;
                } else {
                    self.player.current_streak = -1;
                }
            }
            GameResult::Abandoned => {
                self.player.total_abandoned += 1;
                self.player.current_streak = 0;
            }
        }

        // Update difficulty stats
        let diff_key = format!("{:?}", difficulty);
        let diff_stats = self.player.by_difficulty.entry(diff_key).or_default();
        diff_stats.total_games += 1;
        diff_stats.total_hints += hints_used;
        diff_stats.total_mistakes += mistakes;

        match result {
            GameResult::Win => {
                diff_stats.wins += 1;
                diff_stats.total_time_secs += time_secs;

                // Update best/worst times (only for wins)
                match diff_stats.best_time_secs {
                    Some(best) => diff_stats.best_time_secs = Some(best.min(time_secs)),
                    None => diff_stats.best_time_secs = Some(time_secs),
                }
                match diff_stats.worst_time_secs {
                    Some(worst) => diff_stats.worst_time_secs = Some(worst.max(time_secs)),
                    None => diff_stats.worst_time_secs = Some(time_secs),
                }
            }
            GameResult::Loss => diff_stats.losses += 1,
            GameResult::Abandoned => diff_stats.abandoned += 1,
        }

        // Add to history (most recent first)
        self.history.insert(0, record);

        // Limit history to last 1000 games
        if self.history.len() > 1000 {
            self.history.truncate(1000);
        }

        // Update leaderboard if it's a verified win
        if let Some(score) = self.history[0].score() {
            let entry = LeaderboardEntry {
                player_name: self.player.player_name.clone(),
                score,
                time_secs,
                difficulty,
                hints_used,
                mistakes,
                timestamp,
                puzzle_hash: self.history[0].puzzle_hash.clone(),
            };
            self.add_to_leaderboard(entry);
        }

        // Auto-save
        self.save();

        &self.history[0]
    }

    /// Add entry to leaderboard (maintains sorted order, top 100)
    fn add_to_leaderboard(&mut self, entry: LeaderboardEntry) {
        // Find insertion point (sorted by score ascending - lower is better)
        let pos = self
            .leaderboard
            .iter()
            .position(|e| e.score > entry.score)
            .unwrap_or(self.leaderboard.len());

        self.leaderboard.insert(pos, entry);

        // Keep only top 100
        if self.leaderboard.len() > 100 {
            self.leaderboard.truncate(100);
        }
    }

    /// Get leaderboard filtered by difficulty
    pub fn leaderboard_by_difficulty(&self, difficulty: Difficulty) -> Vec<&LeaderboardEntry> {
        self.leaderboard
            .iter()
            .filter(|e| e.difficulty == difficulty)
            .collect()
    }

    /// Get recent games
    pub fn recent_games(&self, limit: usize) -> &[GameRecord] {
        let end = limit.min(self.history.len());
        &self.history[..end]
    }

    /// Get a game record by ID for replay
    pub fn get_game(&self, id: u64) -> Option<&GameRecord> {
        self.history.iter().find(|g| g.id == id)
    }

    /// Simple hash of puzzle string
    fn hash_puzzle(puzzle: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        puzzle.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Set player name
    pub fn set_player_name(&mut self, name: &str) {
        self.player.player_name = name.to_string();
        self.save();
    }

    /// Unlock secret difficulties (via Konami code)
    pub fn unlock_secrets(&mut self) {
        self.player.secret_unlocked = true;
        self.save();
    }

    /// Check if secret difficulties are unlocked
    pub fn secrets_unlocked(&self) -> bool {
        self.player.secret_unlocked
    }

    /// Get expert wins progress toward unlock
    pub fn expert_wins_progress(&self) -> (usize, usize) {
        (self.player.expert_wins, EXPERT_WINS_TO_UNLOCK)
    }

    /// Check all unlock conditions and unlock secrets if any are met
    fn check_unlocks(&mut self) {
        if self.player.secret_unlocked {
            return; // Already unlocked
        }

        // Method 1: Perfect Game (Expert+, 0 hints, 0 mistakes)
        if self.player.perfect_game_achieved {
            self.player.secret_unlocked = true;
            self.player.unlock_reason =
                Some("Perfect Game: Won Expert+ with no hints or mistakes!".to_string());
            return;
        }

        // Method 2: Speed Demon (Hard+ under 5 minutes)
        if self.player.speed_demon_achieved {
            self.player.secret_unlocked = true;
            self.player.unlock_reason =
                Some("Speed Demon: Won Hard+ in under 5 minutes!".to_string());
            return;
        }

        // Method 3: Win Streak (5 consecutive wins)
        if self.player.best_streak >= WIN_STREAK_TO_UNLOCK as i32 {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some(format!(
                "Win Streak: {} consecutive wins!",
                WIN_STREAK_TO_UNLOCK
            ));
            return;
        }

        // Method 4: Completionist (win on all 4 standard difficulties)
        let standard_diffs = [
            "Beginner",
            "Easy",
            "Medium",
            "Intermediate",
            "Hard",
            "Expert",
        ];
        let all_standard_won = standard_diffs
            .iter()
            .all(|d| self.player.difficulties_won.contains(&d.to_string()));
        if all_standard_won {
            self.player.secret_unlocked = true;
            self.player.unlock_reason =
                Some("Completionist: Won on all standard difficulties!".to_string());
            return;
        }

        // Method 5: Marathon (10+ hours total play time)
        if self.player.total_play_time_secs >= MARATHON_TIME_TO_UNLOCK {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some("Marathon: 10+ hours of total play time!".to_string());
            return;
        }

        // Method 6: Century (100 total wins)
        if self.player.total_wins >= CENTURY_WINS_TO_UNLOCK {
            self.player.secret_unlocked = true;
            self.player.unlock_reason =
                Some(format!("Century: {} total wins!", CENTURY_WINS_TO_UNLOCK));
            return;
        }

        // Method 7: No Notes Master (Hard+ without using notes)
        if self.player.no_notes_master_achieved {
            self.player.secret_unlocked = true;
            self.player.unlock_reason =
                Some("No Notes Master: Won Hard+ without using notes!".to_string());
            return;
        }

        // Method 8: Expert Wins (10 wins on Expert)
        if self.player.expert_wins >= EXPERT_WINS_TO_UNLOCK {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some(format!(
                "Expert Master: {} wins on Expert difficulty!",
                EXPERT_WINS_TO_UNLOCK
            ));
            return;
        }

        // Method 9: Secret Date (September 9th - 9/9)
        if Self::check_secret_date() {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some("Secret Date: Playing on 9/9!".to_string());
        }
    }

    /// Check if today is September 9th (9/9)
    fn check_secret_date() -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            // Get the current day/month (rough calculation)
            let secs = duration.as_secs();
            let days_since_epoch = secs / 86400;

            // Calculate year, month, day from days since epoch
            // This is a simplified calculation - good enough for 9/9 check
            let mut days = days_since_epoch as i64;
            let mut year = 1970i64;

            loop {
                let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    366
                } else {
                    365
                };
                if days < days_in_year {
                    break;
                }
                days -= days_in_year;
                year += 1;
            }

            // Days of month for this year
            let is_leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            let months = if is_leap {
                [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            } else {
                [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            };

            let mut month = 0;
            for (i, &days_in_month) in months.iter().enumerate() {
                if days < days_in_month as i64 {
                    month = i + 1;
                    break;
                }
                days -= days_in_month as i64;
            }
            let day = days + 1;

            // September 9th = month 9, day 9
            return month == 9 && day == 9;
        }
        false
    }

    /// Unlock via Konami code (called from app)
    pub fn unlock_via_konami(&mut self) {
        if !self.player.secret_unlocked {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some("Konami Code: ↑↑↓↓←→←→BA".to_string());
            self.save();
        }
    }

    /// Unlock via reverse Konami (lose screen easter egg)
    pub fn unlock_via_reverse_konami(&mut self) {
        if !self.player.secret_unlocked {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some("Reverse Konami: AB→←→←↓↓↑↑".to_string());
            self.save();
        }
    }

    /// Unlock via "42" pattern (The Answer)
    pub fn unlock_via_the_answer(&mut self) {
        if !self.player.secret_unlocked {
            self.player.secret_unlocked = true;
            self.player.unlock_reason = Some("The Answer: 42!".to_string());
            self.save();
        }
    }

    // ==================== Remote Leaderboard Methods ====================

    /// Submit a score to the remote leaderboard
    pub fn submit_to_remote(&self, record: &GameRecord) {
        if let Some(ref manager) = self.remote_leaderboard {
            if let Some(score) = record.score() {
                let entry = leaderboard::LeaderboardEntry {
                    player_name: self.player.player_name.clone(),
                    score,
                    time_secs: record.time_secs,
                    difficulty: record.difficulty,
                    hints_used: record.hints_used,
                    mistakes: record.mistakes,
                    timestamp: record.timestamp,
                    puzzle_hash: record.puzzle_hash.clone(),
                    rank: None,
                    verified: record.verified,
                };

                // Submit in background (ignore errors for now)
                let _ = manager.submit_score(entry);
            }
        }
    }

    /// Get leaderboard from remote (with fallback to local)
    pub fn get_remote_leaderboard(
        &self,
        difficulty: Option<Difficulty>,
        limit: usize,
    ) -> Vec<leaderboard::LeaderboardEntry> {
        if let Some(ref manager) = self.remote_leaderboard {
            if let Ok(entries) = manager.get_leaderboard(difficulty, limit) {
                return entries;
            }
        }

        // Fallback to local leaderboard
        self.leaderboard
            .iter()
            .filter(|e| difficulty.is_none_or(|d| e.difficulty == d))
            .take(limit)
            .enumerate()
            .map(|(i, e)| leaderboard::LeaderboardEntry {
                player_name: e.player_name.clone(),
                score: e.score,
                time_secs: e.time_secs,
                difficulty: e.difficulty,
                hints_used: e.hints_used,
                mistakes: e.mistakes,
                timestamp: e.timestamp,
                puzzle_hash: e.puzzle_hash.clone(),
                rank: Some(i + 1),
                verified: true,
            })
            .collect()
    }

    /// Get player's rank on remote leaderboard
    pub fn get_remote_rank(&self, difficulty: Difficulty) -> Option<usize> {
        if let Some(ref manager) = self.remote_leaderboard {
            if let Ok(rank) = manager.get_player_rank(&self.player.player_name, difficulty) {
                return rank;
            }
        }
        None
    }

    /// Get the leaderboard backend status
    pub fn leaderboard_status(&self) -> leaderboard::LeaderboardStatus {
        if let Some(ref manager) = self.remote_leaderboard {
            manager.status()
        } else {
            leaderboard::LeaderboardStatus {
                backend_name: "None",
                is_available: false,
                using_fallback: false,
            }
        }
    }

    /// Check if using remote backend
    pub fn is_remote_leaderboard(&self) -> bool {
        if let Some(ref manager) = self.remote_leaderboard {
            manager.status().backend_name != "Local"
        } else {
            false
        }
    }
}

/// Format seconds as MM:SS or HH:MM:SS
pub fn format_time(secs: u64) -> String {
    if secs >= 3600 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}
