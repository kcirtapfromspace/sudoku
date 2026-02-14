//! Fire-and-forget telemetry that submits game results to the ukodus API.
//! Results populate the Galaxy visualization and leaderboards alongside web/iOS games.

use crate::stats::GameRecord;
use std::path::PathBuf;

const API_ENDPOINT: &str = "https://ukodus.now/api/v1/results";

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
    let _ = std::fs::write(&path, &id);
    id
}

fn player_id_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sudoku_player_id")
}

/// DJB2 hash matching the JS `hashPuzzle()` and iOS `TelemetryService.hashPuzzle()`.
/// Uses Int32 wrapping arithmetic, output as 8-char zero-padded hex.
fn hash_puzzle(puzzle_string: &str) -> String {
    let mut hash: i32 = 0;
    for ch in puzzle_string.bytes() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(ch as i32);
    }
    format!("{:08x}", hash as u32)
}

/// Convert puzzle string from grid format ("." = empty) to API format ("0" = empty).
fn puzzle_to_api_format(puzzle: &str) -> String {
    puzzle.replace('.', "0")
}

/// Submit a game result to the ukodus API. Spawns a background thread
/// so it never blocks the TUI. Failures are silently ignored.
pub fn submit_result(record: &GameRecord) {
    let puzzle_string = puzzle_to_api_format(&record.puzzle);
    let puzzle_hash = hash_puzzle(&puzzle_string);
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
            "se_rating": 0.0,
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

        let resp = ureq::post(API_ENDPOINT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&body.to_string());

        #[cfg(debug_assertions)]
        match resp {
            Ok(r) => eprintln!("Telemetry: {} for {}", r.status(), puzzle_hash),
            Err(e) => eprintln!("Telemetry error: {}", e),
        }
        let _ = resp;
    });
}
