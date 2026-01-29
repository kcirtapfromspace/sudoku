import Foundation

/// Caches pre-generated puzzles for instant game start
actor PuzzleCache {
    static let shared = PuzzleCache()

    private var cache: [Difficulty: SudokuGame] = [:]
    private var generatingDifficulties: Set<Difficulty> = []

    private init() {}

    /// Prefetch puzzles for all difficulty levels
    func prefetchAll() async {
        await withTaskGroup(of: Void.self) { group in
            for difficulty in Difficulty.allCases {
                group.addTask {
                    await self.ensureCached(difficulty: difficulty)
                }
            }
        }
    }

    /// Get a cached puzzle, or generate one if not available
    func getPuzzle(difficulty: Difficulty) async -> SudokuGame {
        // If we have a cached puzzle, use it and start generating a replacement
        if let cached = cache[difficulty] {
            cache[difficulty] = nil

            // Start generating replacement in background
            Task {
                await ensureCached(difficulty: difficulty)
            }

            return cached
        }

        // No cached puzzle, generate one now
        return await generatePuzzle(difficulty: difficulty)
    }

    /// Ensure a puzzle is cached for the given difficulty
    private func ensureCached(difficulty: Difficulty) async {
        // Don't generate if already cached or already generating
        guard cache[difficulty] == nil,
              !generatingDifficulties.contains(difficulty) else {
            return
        }

        generatingDifficulties.insert(difficulty)
        let puzzle = await generatePuzzle(difficulty: difficulty)
        cache[difficulty] = puzzle
        generatingDifficulties.remove(difficulty)
    }

    /// Generate a puzzle on a background thread
    private func generatePuzzle(difficulty: Difficulty) async -> SudokuGame {
        await Task.detached(priority: .utility) {
            SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        }.value
    }

    /// Prefetch a specific difficulty (call during gameplay)
    nonisolated func prefetch(difficulty: Difficulty) {
        Task {
            await ensureCached(difficulty: difficulty)
        }
    }

    /// Get cache status for debugging
    func getCacheStatus() -> [Difficulty: Bool] {
        var status: [Difficulty: Bool] = [:]
        for difficulty in Difficulty.allCases {
            status[difficulty] = cache[difficulty] != nil
        }
        return status
    }
}
