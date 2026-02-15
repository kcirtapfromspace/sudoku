//! Leaderboard backend abstraction
//!
//! Supports different backends based on environment:
//! - Local: File-based storage for development
//! - Test: In-memory mock for testing
//! - Production: Remote HTTP API

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sudoku_core::Difficulty;

/// Environment configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Local development - file-based storage
    Local,
    /// Testing - in-memory mock
    Test,
    /// Production - remote API
    Production,
}

impl Environment {
    /// Detect environment from SUDOKU_ENV variable
    pub fn detect() -> Self {
        match std::env::var("SUDOKU_ENV").as_deref() {
            Ok("production") | Ok("prod") => Environment::Production,
            Ok("test") | Ok("testing") => Environment::Test,
            _ => Environment::Local,
        }
    }
}

/// Leaderboard entry for submission and retrieval
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
    /// Server-assigned rank (populated on retrieval)
    #[serde(default)]
    pub rank: Option<usize>,
    /// Whether this score was verified by anti-bot checks
    #[serde(default)]
    pub verified: bool,
}

/// Result type for leaderboard operations
pub type LeaderboardResult<T> = Result<T, LeaderboardError>;

/// Errors that can occur during leaderboard operations
#[derive(Debug, Clone)]
pub enum LeaderboardError {
    /// Network/connection error
    NetworkError(String),
    /// Server returned an error
    ServerError(String),
    /// Invalid response from server
    InvalidResponse(String),
    /// Local storage error
    StorageError(String),
    /// Score was rejected (e.g., failed anti-bot)
    ScoreRejected(String),
    /// Rate limited
    RateLimited,
    /// Not authenticated
    NotAuthenticated,
}

impl std::fmt::Display for LeaderboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::ServerError(e) => write!(f, "Server error: {}", e),
            Self::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            Self::StorageError(e) => write!(f, "Storage error: {}", e),
            Self::ScoreRejected(e) => write!(f, "Score rejected: {}", e),
            Self::RateLimited => write!(f, "Rate limited, try again later"),
            Self::NotAuthenticated => write!(f, "Not authenticated"),
        }
    }
}

/// Trait for leaderboard backends
pub trait LeaderboardBackend: Send + Sync {
    /// Submit a score to the leaderboard
    fn submit_score(&self, entry: LeaderboardEntry) -> LeaderboardResult<()>;

    /// Get top scores, optionally filtered by difficulty
    fn get_leaderboard(
        &self,
        difficulty: Option<Difficulty>,
        limit: usize,
        offset: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>>;

    /// Get a player's rank for a specific difficulty
    fn get_player_rank(
        &self,
        player_name: &str,
        difficulty: Difficulty,
    ) -> LeaderboardResult<Option<usize>>;

    /// Get a player's best scores
    fn get_player_scores(
        &self,
        player_name: &str,
        limit: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>>;

    /// Check if backend is available/connected
    fn is_available(&self) -> bool;

    /// Get backend name for display
    fn backend_name(&self) -> &'static str;
}

// ==================== Local File Backend ====================

/// Local file-based leaderboard for development
pub struct LocalLeaderboard {
    path: std::path::PathBuf,
    cache: Mutex<Option<LocalLeaderboardData>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LocalLeaderboardData {
    entries: Vec<LeaderboardEntry>,
}

impl LocalLeaderboard {
    pub fn new() -> Self {
        let path = crate::persistence::app_data_dir().join("sudoku_leaderboard.json");
        Self {
            path,
            cache: Mutex::new(None),
        }
    }

    fn load(&self) -> LocalLeaderboardData {
        let mut cache = self.cache.lock().unwrap();
        if let Some(ref data) = *cache {
            return data.clone();
        }

        let data = match std::fs::read_to_string(&self.path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => LocalLeaderboardData::default(),
        };

        *cache = Some(data.clone());
        data
    }

    fn save(&self, data: &LocalLeaderboardData) -> LeaderboardResult<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| LeaderboardError::StorageError(e.to_string()))?;
        crate::persistence::atomic_write(&self.path, json.as_bytes())
            .map_err(|e| LeaderboardError::StorageError(e.to_string()))?;
        *self.cache.lock().unwrap() = Some(data.clone());
        Ok(())
    }
}

impl Default for LocalLeaderboard {
    fn default() -> Self {
        Self::new()
    }
}

impl LeaderboardBackend for LocalLeaderboard {
    fn submit_score(&self, entry: LeaderboardEntry) -> LeaderboardResult<()> {
        let mut data = self.load();

        // Insert in sorted position (by score ascending - lower is better)
        let pos = data
            .entries
            .iter()
            .position(|e| e.score > entry.score)
            .unwrap_or(data.entries.len());

        data.entries.insert(pos, entry);

        // Keep only top 1000
        data.entries.truncate(1000);

        self.save(&data)
    }

    fn get_leaderboard(
        &self,
        difficulty: Option<Difficulty>,
        limit: usize,
        offset: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        let data = self.load();

        let filtered: Vec<LeaderboardEntry> = data
            .entries
            .into_iter()
            .filter(|e| difficulty.is_none_or(|d| e.difficulty == d))
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, mut e)| {
                e.rank = Some(offset + i + 1);
                e
            })
            .collect();

        Ok(filtered)
    }

    fn get_player_rank(
        &self,
        player_name: &str,
        difficulty: Difficulty,
    ) -> LeaderboardResult<Option<usize>> {
        let data = self.load();

        let rank = data
            .entries
            .iter()
            .filter(|e| e.difficulty == difficulty)
            .position(|e| e.player_name == player_name)
            .map(|pos| pos + 1);

        Ok(rank)
    }

    fn get_player_scores(
        &self,
        player_name: &str,
        limit: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        let data = self.load();

        let scores: Vec<LeaderboardEntry> = data
            .entries
            .into_iter()
            .filter(|e| e.player_name == player_name)
            .take(limit)
            .collect();

        Ok(scores)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn backend_name(&self) -> &'static str {
        "Local"
    }
}

// ==================== Mock Backend for Testing ====================

/// In-memory mock leaderboard for testing
pub struct MockLeaderboard {
    data: Mutex<Vec<LeaderboardEntry>>,
    available: Mutex<bool>,
}

impl MockLeaderboard {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(Vec::new()),
            available: Mutex::new(true),
        }
    }

    /// Set whether the backend should report as available
    pub fn set_available(&self, available: bool) {
        *self.available.lock().unwrap() = available;
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }

    /// Get entry count
    pub fn count(&self) -> usize {
        self.data.lock().unwrap().len()
    }
}

impl Default for MockLeaderboard {
    fn default() -> Self {
        Self::new()
    }
}

impl LeaderboardBackend for MockLeaderboard {
    fn submit_score(&self, entry: LeaderboardEntry) -> LeaderboardResult<()> {
        if !*self.available.lock().unwrap() {
            return Err(LeaderboardError::NetworkError("Mock unavailable".into()));
        }

        let mut data = self.data.lock().unwrap();
        let pos = data
            .iter()
            .position(|e| e.score > entry.score)
            .unwrap_or(data.len());
        data.insert(pos, entry);
        Ok(())
    }

    fn get_leaderboard(
        &self,
        difficulty: Option<Difficulty>,
        limit: usize,
        offset: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        if !*self.available.lock().unwrap() {
            return Err(LeaderboardError::NetworkError("Mock unavailable".into()));
        }

        let data = self.data.lock().unwrap();
        let filtered: Vec<LeaderboardEntry> = data
            .iter()
            .filter(|e| difficulty.is_none_or(|d| e.difficulty == d))
            .skip(offset)
            .take(limit)
            .cloned()
            .enumerate()
            .map(|(i, mut e)| {
                e.rank = Some(offset + i + 1);
                e
            })
            .collect();

        Ok(filtered)
    }

    fn get_player_rank(
        &self,
        player_name: &str,
        difficulty: Difficulty,
    ) -> LeaderboardResult<Option<usize>> {
        if !*self.available.lock().unwrap() {
            return Err(LeaderboardError::NetworkError("Mock unavailable".into()));
        }

        let data = self.data.lock().unwrap();
        let rank = data
            .iter()
            .filter(|e| e.difficulty == difficulty)
            .position(|e| e.player_name == player_name)
            .map(|pos| pos + 1);

        Ok(rank)
    }

    fn get_player_scores(
        &self,
        player_name: &str,
        limit: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        if !*self.available.lock().unwrap() {
            return Err(LeaderboardError::NetworkError("Mock unavailable".into()));
        }

        let data = self.data.lock().unwrap();
        let scores: Vec<LeaderboardEntry> = data
            .iter()
            .filter(|e| e.player_name == player_name)
            .take(limit)
            .cloned()
            .collect();

        Ok(scores)
    }

    fn is_available(&self) -> bool {
        *self.available.lock().unwrap()
    }

    fn backend_name(&self) -> &'static str {
        "Mock"
    }
}

// ==================== Remote HTTP Backend ====================

/// Remote HTTP API leaderboard for production
pub struct RemoteLeaderboard {
    base_url: String,
    api_key: Option<String>,
    client: Mutex<Option<()>>, // Placeholder for HTTP client
}

/// Configuration for remote backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.sudoku-game.example.com".to_string(),
            api_key: None,
            timeout_secs: 10,
        }
    }
}

impl RemoteLeaderboard {
    pub fn new(config: RemoteConfig) -> Self {
        Self {
            base_url: config.base_url,
            api_key: config.api_key,
            client: Mutex::new(None),
        }
    }

    pub fn from_env() -> Self {
        let base_url = std::env::var("SUDOKU_API_URL")
            .unwrap_or_else(|_| "https://api.sudoku-game.example.com".to_string());
        let api_key = std::env::var("SUDOKU_API_KEY").ok();

        Self {
            base_url,
            api_key,
            client: Mutex::new(None),
        }
    }

    /// Make an HTTP request (placeholder - would use reqwest/ureq in real implementation)
    fn request<T: for<'de> Deserialize<'de>>(
        &self,
        _method: &str,
        _endpoint: &str,
        _body: Option<&impl Serialize>,
    ) -> LeaderboardResult<T> {
        // In a real implementation, this would:
        // 1. Build the full URL
        // 2. Add authentication headers
        // 3. Make the HTTP request
        // 4. Parse the response
        //
        // For now, return an error indicating remote is not implemented
        Err(LeaderboardError::NetworkError(
            "Remote backend not fully implemented - add HTTP client dependency".to_string(),
        ))
    }
}

impl LeaderboardBackend for RemoteLeaderboard {
    fn submit_score(&self, entry: LeaderboardEntry) -> LeaderboardResult<()> {
        #[derive(Serialize)]
        struct SubmitRequest {
            entry: LeaderboardEntry,
        }

        #[derive(Deserialize)]
        struct SubmitResponse {
            success: bool,
            message: Option<String>,
        }

        let _response: SubmitResponse = self.request(
            "POST",
            "/api/v1/leaderboard/submit",
            Some(&SubmitRequest { entry }),
        )?;

        Ok(())
    }

    fn get_leaderboard(
        &self,
        difficulty: Option<Difficulty>,
        limit: usize,
        offset: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        #[derive(Serialize)]
        struct ListRequest {
            difficulty: Option<String>,
            limit: usize,
            offset: usize,
        }

        #[derive(Deserialize)]
        struct ListResponse {
            entries: Vec<LeaderboardEntry>,
        }

        let response: ListResponse = self.request(
            "GET",
            "/api/v1/leaderboard",
            Some(&ListRequest {
                difficulty: difficulty.map(|d| format!("{:?}", d)),
                limit,
                offset,
            }),
        )?;

        Ok(response.entries)
    }

    fn get_player_rank(
        &self,
        player_name: &str,
        difficulty: Difficulty,
    ) -> LeaderboardResult<Option<usize>> {
        #[derive(Deserialize)]
        struct RankResponse {
            rank: Option<usize>,
        }

        let response: RankResponse = self.request(
            "GET",
            &format!("/api/v1/leaderboard/rank/{}/{:?}", urlencoding::encode(player_name), difficulty),
            None::<&()>,
        )?;

        Ok(response.rank)
    }

    fn get_player_scores(
        &self,
        player_name: &str,
        limit: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        #[derive(Deserialize)]
        struct ScoresResponse {
            entries: Vec<LeaderboardEntry>,
        }

        let response: ScoresResponse = self.request(
            "GET",
            &format!("/api/v1/leaderboard/player/{}?limit={}", urlencoding::encode(player_name), limit),
            None::<&()>,
        )?;

        Ok(response.entries)
    }

    fn is_available(&self) -> bool {
        // In real implementation, would ping the server
        // For now, always return false since not implemented
        false
    }

    fn backend_name(&self) -> &'static str {
        "Remote"
    }
}

// ==================== Backend Factory ====================

/// Create the appropriate backend based on environment
pub fn create_backend(env: Environment) -> Arc<dyn LeaderboardBackend> {
    match env {
        Environment::Local => Arc::new(LocalLeaderboard::new()),
        Environment::Test => Arc::new(MockLeaderboard::new()),
        Environment::Production => Arc::new(RemoteLeaderboard::from_env()),
    }
}

/// Create backend with automatic environment detection
pub fn create_backend_auto() -> Arc<dyn LeaderboardBackend> {
    create_backend(Environment::detect())
}

// ==================== Leaderboard Manager ====================

/// High-level leaderboard manager with caching and fallback
pub struct LeaderboardManager {
    primary: Arc<dyn LeaderboardBackend>,
    fallback: Option<Arc<dyn LeaderboardBackend>>,
    cache: Mutex<LeaderboardCache>,
}

impl std::fmt::Debug for LeaderboardManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeaderboardManager")
            .field("primary", &self.primary.backend_name())
            .field(
                "fallback",
                &self.fallback.as_ref().map(|f| f.backend_name()),
            )
            .finish()
    }
}

#[derive(Debug, Default)]
struct LeaderboardCache {
    entries: HashMap<Option<Difficulty>, (Vec<LeaderboardEntry>, std::time::Instant)>,
    cache_ttl_secs: u64,
}

impl LeaderboardCache {
    fn new(ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            cache_ttl_secs: ttl_secs,
        }
    }

    fn get(&self, difficulty: Option<Difficulty>) -> Option<&Vec<LeaderboardEntry>> {
        self.entries.get(&difficulty).and_then(|(entries, time)| {
            if time.elapsed().as_secs() < self.cache_ttl_secs {
                Some(entries)
            } else {
                None
            }
        })
    }

    fn set(&mut self, difficulty: Option<Difficulty>, entries: Vec<LeaderboardEntry>) {
        self.entries
            .insert(difficulty, (entries, std::time::Instant::now()));
    }

    fn invalidate(&mut self) {
        self.entries.clear();
    }
}

impl LeaderboardManager {
    /// Create a new manager with the given primary backend
    pub fn new(primary: Arc<dyn LeaderboardBackend>) -> Self {
        Self {
            primary,
            fallback: None,
            cache: Mutex::new(LeaderboardCache::new(60)), // 1 minute cache
        }
    }

    /// Create with automatic environment detection
    pub fn auto() -> Self {
        let env = Environment::detect();
        let primary = create_backend(env);

        // Use local as fallback for production
        let fallback = if env == Environment::Production {
            Some(Arc::new(LocalLeaderboard::new()) as Arc<dyn LeaderboardBackend>)
        } else {
            None
        };

        Self {
            primary,
            fallback,
            cache: Mutex::new(LeaderboardCache::new(60)),
        }
    }

    /// Set a fallback backend
    pub fn with_fallback(mut self, fallback: Arc<dyn LeaderboardBackend>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// Get the active backend (primary if available, else fallback)
    fn active_backend(&self) -> &Arc<dyn LeaderboardBackend> {
        if self.primary.is_available() {
            &self.primary
        } else if let Some(ref fallback) = self.fallback {
            fallback
        } else {
            &self.primary
        }
    }

    /// Submit a score
    pub fn submit_score(&self, entry: LeaderboardEntry) -> LeaderboardResult<()> {
        let result = self.primary.submit_score(entry.clone());

        // If primary fails but we have fallback, try fallback
        if result.is_err() {
            if let Some(ref fallback) = self.fallback {
                let _ = fallback.submit_score(entry);
            }
        }

        // Invalidate cache on submit
        self.cache.lock().unwrap().invalidate();

        result
    }

    /// Get leaderboard with caching
    pub fn get_leaderboard(
        &self,
        difficulty: Option<Difficulty>,
        limit: usize,
    ) -> LeaderboardResult<Vec<LeaderboardEntry>> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(difficulty) {
                return Ok(cached.iter().take(limit).cloned().collect());
            }
        }

        // Fetch from backend
        let entries = self
            .active_backend()
            .get_leaderboard(difficulty, limit, 0)?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.set(difficulty, entries.clone());
        }

        Ok(entries)
    }

    /// Get player rank
    pub fn get_player_rank(
        &self,
        player_name: &str,
        difficulty: Difficulty,
    ) -> LeaderboardResult<Option<usize>> {
        self.active_backend()
            .get_player_rank(player_name, difficulty)
    }

    /// Get backend status info
    pub fn status(&self) -> LeaderboardStatus {
        LeaderboardStatus {
            backend_name: self.active_backend().backend_name(),
            is_available: self.primary.is_available(),
            using_fallback: !self.primary.is_available() && self.fallback.is_some(),
        }
    }
}

/// Status information about the leaderboard
#[derive(Debug, Clone)]
pub struct LeaderboardStatus {
    pub backend_name: &'static str,
    pub is_available: bool,
    pub using_fallback: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_backend() {
        let backend = MockLeaderboard::new();

        let entry = LeaderboardEntry {
            player_name: "TestPlayer".to_string(),
            score: 100,
            time_secs: 300,
            difficulty: Difficulty::Medium,
            hints_used: 0,
            mistakes: 0,
            timestamp: 0,
            puzzle_hash: "abc123".to_string(),
            rank: None,
            verified: true,
        };

        backend.submit_score(entry).unwrap();
        assert_eq!(backend.count(), 1);

        let leaderboard = backend.get_leaderboard(None, 10, 0).unwrap();
        assert_eq!(leaderboard.len(), 1);
        assert_eq!(leaderboard[0].rank, Some(1));
    }

    #[test]
    fn test_mock_unavailable() {
        let backend = MockLeaderboard::new();
        backend.set_available(false);

        assert!(!backend.is_available());

        let result = backend.get_leaderboard(None, 10, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_environment_detection() {
        // Default should be Local
        let env = Environment::detect();
        assert_eq!(env, Environment::Local);
    }

    #[test]
    fn test_local_backend() {
        let backend = LocalLeaderboard::new();
        assert!(backend.is_available());
        assert_eq!(backend.backend_name(), "Local");
    }
}
