import Foundation

/// Keeps one unplayed game ready per requested difficulty.
actor PuzzleCache {
    static let shared = PuzzleCache()

    private var ready: [Difficulty: SudokuGame] = [:]
    private var pending: [Difficulty: Task<Void, Never>] = [:]

    private let fetch: @Sendable (Difficulty) async -> SudokuGame?
    private let generate: @Sendable (Difficulty) async -> SudokuGame
    private let starter: @Sendable (Difficulty) -> SudokuGame?

    init(fetch: @escaping @Sendable (Difficulty) async -> SudokuGame? = {
        await PuzzleAPIService.shared.fetchPuzzle(difficulty: $0)
    }, generate: @escaping @Sendable (Difficulty) async -> SudokuGame = { difficulty in
        await Task.detached(priority: .utility) {
            SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        }.value
    }, starter: @escaping @Sendable (Difficulty) -> SudokuGame? = { StarterPuzzles.next(for: $0) }) {
        self.fetch = fetch
        self.generate = generate
        self.starter = starter
    }

    func getPuzzle(difficulty: Difficulty) async -> SudokuGame {
        while true {
            if let game = ready.removeValue(forKey: difficulty) {
                _ = prepare(difficulty)
                return game
            }
            let task = prepare(difficulty)
            if let game = starter(difficulty) { return game }
            // Multiple callers can join a warmup, but only one takes each mutable game.
            await task.value
        }
    }

    func ensureCached(difficulty: Difficulty) async {
        guard ready[difficulty] == nil else { return }
        await prepare(difficulty).value
    }

    private func prepare(_ difficulty: Difficulty) -> Task<Void, Never> {
        if let task = pending[difficulty] { return task }
        let task = Task {
            if PuzzleAPIService.eligibleDifficulties.contains(difficulty),
               let catalogGame = await fetch(difficulty) {
                ready[difficulty] = catalogGame
            } else {
                let game = await generate(difficulty)
                // The generator can return an easier tier when it exhausts its budget.
                ready[difficulty] = Difficulty.from(game.getRatedDifficulty()) == difficulty
                    ? game : starter(difficulty) ?? game
            }
            pending[difficulty] = nil
        }
        pending[difficulty] = task
        return task
    }

    @discardableResult
    nonisolated func prefetch(difficulty: Difficulty) -> Task<Void, Never> {
        Task { await ensureCached(difficulty: difficulty) }
    }
}
