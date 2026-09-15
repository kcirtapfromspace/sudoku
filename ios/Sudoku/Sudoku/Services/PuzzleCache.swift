import Foundation

/// Caches pre-generated puzzles for instant game start
actor PuzzleCache {
    static let shared = PuzzleCache()

    private var cache: [Difficulty: SudokuGame] = [:]
    private var generatingDifficulties: Set<Difficulty> = []

    private let fetch: @Sendable (Difficulty) async -> SudokuGame?
    private let generate: @Sendable (Difficulty) async -> SudokuGame

    init(fetch: @escaping @Sendable (Difficulty) async -> SudokuGame? = {
        await PuzzleAPIService.shared.fetchPuzzle(difficulty: $0)
    }, generate: @escaping @Sendable (Difficulty) async -> SudokuGame = { difficulty in
        await Task.detached(priority: .utility) {
            SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        }.value
    }) {
        self.fetch = fetch
        self.generate = generate
    }

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

        // For Hard+ difficulties, try fetching a pre-mined puzzle from the API.
        // This is much faster than local generation for these expensive difficulties.
        if PuzzleAPIService.eligibleDifficulties.contains(difficulty) {
            if let apiPuzzle = await fetch(difficulty) {
                // Start local generation in background as a cache warm-up
                Task {
                    await ensureCached(difficulty: difficulty)
                }
                return apiPuzzle
            }
        }

        // No cached puzzle and API unavailable — generate one now
        return await generatePuzzle(difficulty: difficulty)
    }

    /// Ensure a puzzle is cached for the given difficulty
    func ensureCached(difficulty: Difficulty) async {
        // Don't generate if already cached or already generating
        guard cache[difficulty] == nil,
              !generatingDifficulties.contains(difficulty) else {
            return
        }

        generatingDifficulties.insert(difficulty)

        // For Hard+ difficulties, try the API first for cache warm-up too
        if PuzzleAPIService.eligibleDifficulties.contains(difficulty),
           let apiPuzzle = await fetch(difficulty) {
            cache[difficulty] = apiPuzzle
            generatingDifficulties.remove(difficulty)
            return
        }

        let puzzle = await generatePuzzle(difficulty: difficulty)
        cache[difficulty] = puzzle
        generatingDifficulties.remove(difficulty)
    }

    /// Generate a puzzle on a background thread
    private func generatePuzzle(difficulty: Difficulty) async -> SudokuGame {
        await generate(difficulty)
    }

    /// Prefetch a specific difficulty (call during gameplay)
    @discardableResult
    nonisolated func prefetch(difficulty: Difficulty) -> Task<Void, Never> {
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
