import Foundation
import CryptoKit

/// A record of a unique puzzle that has been played
struct PuzzleRecord: Codable, Identifiable {
    /// Unique identifier (SHA-256 hash of puzzle string)
    var id: String { puzzleHash }

    /// SHA-256 hash of the puzzle string (unique identifier)
    let puzzleHash: String

    /// 81-character puzzle string (givens as digits, empty as ".")
    let puzzleString: String

    /// Difficulty level of the puzzle
    let difficulty: Difficulty

    /// When this puzzle was first played
    let firstPlayedAt: Date

    /// When this puzzle was last played
    var lastPlayedAt: Date

    /// Number of times this puzzle has been played
    var playCount: Int

    /// Best completion time (nil if never won)
    var bestTime: TimeInterval?

    /// Number of wins on this puzzle
    var wins: Int

    /// Number of losses on this puzzle
    var losses: Int

    /// Whether this puzzle has been completed at least once
    var hasBeenSolved: Bool { wins > 0 }

    /// Create a new puzzle record
    init(puzzleString: String, difficulty: Difficulty, now: Date = Date()) {
        self.puzzleHash = Self.generateHash(from: puzzleString)
        self.puzzleString = puzzleString
        self.difficulty = difficulty
        self.firstPlayedAt = now
        self.lastPlayedAt = now
        self.playCount = 1
        self.bestTime = nil
        self.wins = 0
        self.losses = 0
    }

    /// Generate SHA-256 hash from puzzle string
    static func generateHash(from puzzleString: String) -> String {
        let data = Data(puzzleString.utf8)
        let hash = SHA256.hash(data: data)
        return hash.compactMap { String(format: "%02x", $0) }.joined()
    }

    /// Record a game result
    mutating func recordResult(won: Bool, time: TimeInterval?, now: Date = Date()) {
        lastPlayedAt = now

        if won {
            wins += 1
            if let time = time {
                if let currentBest = bestTime {
                    bestTime = min(currentBest, time)
                } else {
                    bestTime = time
                }
            }
        } else {
            losses += 1
        }
    }
}

/// Statistics about the puzzle library
struct PuzzleLibraryStats: Codable {
    var totalPuzzles: Int = 0
    var solvedPuzzles: Int = 0
    var totalPlays: Int = 0

    var completionRate: Double {
        guard totalPuzzles > 0 else { return 0 }
        return Double(solvedPuzzles) / Double(totalPuzzles)
    }
}
