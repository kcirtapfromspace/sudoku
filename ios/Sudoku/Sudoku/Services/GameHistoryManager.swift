import Foundation

/// Manages the game history / puzzle library
@MainActor
class GameHistoryManager: ObservableObject {
    static let shared = GameHistoryManager()

    // MARK: - Published Properties

    @Published private(set) var puzzles: [String: PuzzleRecord] = [:]
    @Published private(set) var stats: PuzzleLibraryStats = PuzzleLibraryStats()

    // MARK: - Private Properties

    private let storageKey = "sudoku_puzzle_library"
    private let statsKey = "sudoku_library_stats"

    // MARK: - Initialization

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        load()
    }

    // MARK: - Public Methods

    /// Get all puzzles sorted by last played date (most recent first)
    var recentPuzzles: [PuzzleRecord] {
        puzzles.values.sorted { $0.lastPlayedAt > $1.lastPlayedAt }
    }

    /// Get puzzles filtered by difficulty
    func puzzles(for difficulty: Difficulty) -> [PuzzleRecord] {
        puzzles.values
            .filter { $0.difficulty == difficulty }
            .sorted { $0.lastPlayedAt > $1.lastPlayedAt }
    }

    /// Get unsolved puzzles
    var unsolvedPuzzles: [PuzzleRecord] {
        puzzles.values
            .filter { !$0.hasBeenSolved }
            .sorted { $0.lastPlayedAt > $1.lastPlayedAt }
    }

    /// Record a puzzle being started
    /// Returns the existing record if puzzle was played before, or creates a new one
    @discardableResult
    func recordPuzzleStart(puzzleString: String, difficulty: Difficulty) -> PuzzleRecord {
        let hash = PuzzleRecord.generateHash(from: puzzleString)

        if var existing = puzzles[hash] {
            existing.lastPlayedAt = Date()
            existing.playCount += 1
            puzzles[hash] = existing
            stats.totalPlays += 1
            save()
            return existing
        } else {
            let newRecord = PuzzleRecord(puzzleString: puzzleString, difficulty: difficulty)
            puzzles[hash] = newRecord
            stats.totalPuzzles += 1
            stats.totalPlays += 1
            save()
            return newRecord
        }
    }

    /// Record a game result
    func recordResult(puzzleHash: String, won: Bool, time: TimeInterval?) {
        guard var record = puzzles[puzzleHash] else { return }

        let wasAlreadySolved = record.hasBeenSolved
        record.recordResult(won: won, time: time)
        puzzles[puzzleHash] = record

        // Update stats if this is the first solve
        if won && !wasAlreadySolved {
            stats.solvedPuzzles += 1
        }

        save()
    }

    /// Get a puzzle by hash
    func getPuzzle(hash: String) -> PuzzleRecord? {
        puzzles[hash]
    }

    /// Check if a puzzle has been played before
    func hasPlayed(puzzleString: String) -> Bool {
        let hash = PuzzleRecord.generateHash(from: puzzleString)
        return puzzles[hash] != nil
    }

    /// Get the puzzle fingerprint (81-char string) from current game
    /// Call this on the GameViewModel to get its puzzle string
    static func extractPuzzleFingerprint(from cells: [[CellModel]]) -> String {
        var result = ""
        for row in 0..<9 {
            for col in 0..<9 {
                let cell = cells[row][col]
                if cell.isGiven {
                    result += "\(cell.value)"
                } else {
                    result += "."
                }
            }
        }
        return result
    }

    /// Clear all history (for debugging/reset)
    func clearHistory() {
        puzzles = [:]
        stats = PuzzleLibraryStats()
        save()
    }

    // MARK: - Persistence

    private func save() {
        // Save puzzles
        if let data = try? JSONEncoder().encode(Array(puzzles.values)) {
            defaults.set(data, forKey: storageKey)
        }

        // Save stats
        if let statsData = try? JSONEncoder().encode(stats) {
            defaults.set(statsData, forKey: statsKey)
        }
    }

    private func load() {
        // Load puzzles
        if let data = defaults.data(forKey: storageKey),
           let records = try? JSONDecoder().decode([PuzzleRecord].self, from: data) {
            puzzles = Dictionary(records.map { ($0.puzzleHash, $0) }, uniquingKeysWith: { first, second in
                first.lastPlayedAt > second.lastPlayedAt ? first : second
            })
        }

        // Load stats
        if let statsData = defaults.data(forKey: statsKey),
           let loadedStats = try? JSONDecoder().decode(PuzzleLibraryStats.self, from: statsData) {
            stats = loadedStats
        }
    }
}
