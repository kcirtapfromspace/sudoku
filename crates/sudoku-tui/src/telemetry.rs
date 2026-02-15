//! Fire-and-forget telemetry that submits game results to the ukodus API.
//! Results populate the Galaxy visualization and leaderboards alongside web/iOS games.

use crate::stats::GameRecord;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use sudoku_core::canonical_puzzle_hash_str;

const API_ENDPOINT: &str = "https://ukodus.now/api/v1/results";
const TOKEN_ENDPOINT: &str = "https://ukodus.now/api/v1/token";

struct CachedToken {
    token: String,
    expires_at: u64, // Unix timestamp
}

static TOKEN_CACHE: Mutex<Option<CachedToken>> = Mutex::new(None);

/// Fetch an auth token from the server. Returns None if the server is unavailable
/// or doesn't support the token endpoint yet (migration period).
fn fetch_token(player_id: &str) -> Option<CachedToken> {
    let body = serde_json::json!({ "player_id": player_id });
    let resp = ureq::post(TOKEN_ENDPOINT)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(5))
        .send_string(&body.to_string())
        .ok()?;

    let body = resp.into_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let token = json["token"].as_str()?.to_string();
    let expires_at = json["expires_at"].as_u64()?;

    Some(CachedToken { token, expires_at })
}

/// Get a valid auth token, fetching a new one if needed.
/// Returns None during migration (server doesn't support tokens yet).
fn get_token(player_id: &str) -> Option<String> {
    let mut cache = TOKEN_CACHE.lock().unwrap();

    // Check if cached token is still valid (with 60s buffer)
    if let Some(ref cached) = *cache {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if cached.expires_at > now + 60 {
            return Some(cached.token.clone());
        }
    }

    // Fetch new token
    let new_token = fetch_token(player_id)?;
    let token_str = new_token.token.clone();
    *cache = Some(new_token);
    Some(token_str)
}

/// Get or create a persistent player UUID stored alongside stats.
fn player_id() -> String {
    let path = player_id_path();
    if let Ok(id) = std::fs::read_to_string(&path) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return id;
        }
    }
    // Generate a new UUID-like ID
    let id = format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        rand::random::<u32>(),
        rand::random::<u16>(),
        rand::random::<u16>(),
        rand::random::<u16>(),
        rand::random::<u64>() & 0xFFFF_FFFF_FFFF,
    );
    let _ = crate::persistence::atomic_write(&path, id.as_bytes());
    id
}

fn player_id_path() -> PathBuf {
    crate::persistence::app_data_dir().join("sudoku_player_id")
}

/// Submit a game result to the ukodus API. Spawns a background thread
/// so it never blocks the TUI. Failures are silently ignored.
pub fn submit_result(record: &GameRecord, se_rating: f32) {
    let puzzle_string = record.puzzle.clone();
    let puzzle_hash = canonical_puzzle_hash_str(&record.puzzle);
    let difficulty = format!("{:?}", record.difficulty);
    let result_str = match record.result {
        crate::stats::GameResult::Win => "Win",
        crate::stats::GameResult::Loss => "Loss",
        _ => return, // Don't submit abandoned games
    };
    let time_secs = record.time_secs;
    let hints_used = record.hints_used;
    let mistakes = record.mistakes;
    let moves_count = record.moves_count;
    let avg_move_time_ms = record.avg_move_time_ms;
    let min_move_time_ms = record.min_move_time_ms;
    let move_time_std_dev = record.move_time_std_dev;
    let short_code = record.short_code.clone();
    let pid = player_id();
    let version = env!("CARGO_PKG_VERSION");

    std::thread::spawn(move || {
        let mut body = serde_json::json!({
            "puzzle_hash": puzzle_hash,
            "puzzle_string": puzzle_string,
            "difficulty": difficulty,
            "se_rating": se_rating,
            "result": result_str,
            "time_secs": time_secs,
            "hints_used": hints_used,
            "mistakes": mistakes,
            "moves_count": moves_count,
            "avg_move_time_ms": avg_move_time_ms,
            "min_move_time_ms": min_move_time_ms,
            "move_time_std_dev": move_time_std_dev,
            "player_id": pid,
            "platform": "tui",
            "app_version": version,
        });

        if let Some(code) = short_code {
            body["short_code"] = serde_json::Value::String(code);
        }

        // Try to get auth token (None during migration = unauthenticated)
        let token = get_token(&pid);

        let mut req = ureq::post(API_ENDPOINT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10));

        if let Some(ref t) = token {
            req = req.set("Authorization", &format!("Bearer {}", t));
        }

        let resp = req.send_string(&body.to_string());

        match resp {
            Ok(_r) => {
                #[cfg(debug_assertions)]
                eprintln!("Telemetry: {} for {}", _r.status(), puzzle_hash);
            }
            Err(ureq::Error::Status(401, _)) => {
                // Token expired or invalid — clear cache so next submission refreshes
                *TOKEN_CACHE.lock().unwrap() = None;
                #[cfg(debug_assertions)]
                eprintln!("Telemetry: 401 — token expired, cleared cache");
            }
            Err(ureq::Error::Status(429, _resp)) => {
                #[cfg(debug_assertions)]
                {
                    let retry_after = _resp.header("Retry-After").unwrap_or("?");
                    eprintln!(
                        "Telemetry: 429 — rate limited, retry after {}s",
                        retry_after
                    );
                }
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("Telemetry error: {}", _e);
            }
        }
    });
}
