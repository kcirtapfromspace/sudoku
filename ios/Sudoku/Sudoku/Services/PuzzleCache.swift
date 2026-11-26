import Foundation

/// Keeps one unplayed game ready per requested difficulty.
actor PuzzleCache {
    static let shared = PuzzleCache()

    private enum Request: Hashable {
        case tier(Difficulty)
        case rating(Float)
    }

    private var ready: [Request: SudokuGame] = [:]
    private var pending: [Request: Task<Void, Never>] = [:]

    private let fetch: @Sendable (Difficulty) async -> SudokuGame?
    private let generate: @Sendable (Difficulty) async -> SudokuGame
    private let generateSE: @Sendable (Float) async -> SudokuGame
    private let starter: @Sendable (Difficulty) -> SudokuGame?

    init(fetch: @escaping @Sendable (Difficulty) async -> SudokuGame? = {
        await PuzzleAPIService.shared.fetchPuzzle(difficulty: $0)
    }, generate: @escaping @Sendable (Difficulty) async -> SudokuGame = { difficulty in
        await Task.detached(priority: .utility) {
            SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        }.value
    }, generateSE: @escaping @Sendable (Float) async -> SudokuGame = { target in
        await Task.detached(priority: .utility) {
            SudokuGame.newWithSeRating(targetSe: target)
        }.value
    }, starter: @escaping @Sendable (Difficulty) -> SudokuGame? = { StarterPuzzles.next(for: $0) }) {
        self.fetch = fetch
        self.generate = generate
        self.generateSE = generateSE
        self.starter = starter
    }

    func getPuzzle(difficulty: Difficulty) async -> SudokuGame {
        await take(.tier(difficulty)) { starter(difficulty) }
    }

    func getPuzzle(targetSE: Float) async -> SudokuGame {
        await take(.rating(targetSE)) { StarterPuzzles.next(rating: targetSE) }
    }

    private func take(_ request: Request, reserve: () -> SudokuGame?) async -> SudokuGame {
        while true {
            if let game = ready.removeValue(forKey: request) {
                _ = prepare(request)
                return game
            }
            let task = prepare(request)
            if let game = reserve() { return game }
            // Multiple callers can join a warmup, but only one takes each mutable game.
            await task.value
        }
    }

    func ensureCached(difficulty: Difficulty) async {
        let request = Request.tier(difficulty)
        guard ready[request] == nil else { return }
        await prepare(request).value
    }

    private func prepare(_ request: Request) -> Task<Void, Never> {
        if let task = pending[request] { return task }
        let task = Task {
            switch request {
            case .rating(let target):
                let game = await generateSE(target)
                ready[request] = game.getSeRating() == target ? game : StarterPuzzles.next(rating: target) ?? game
            case .tier(let difficulty):
                if PuzzleAPIService.eligibleDifficulties.contains(difficulty),
                   let catalogGame = await fetch(difficulty) {
                    ready[request] = catalogGame
                } else {
                    let game = await generate(difficulty)
                    // The generator can return an easier tier when it exhausts its budget.
                    ready[request] = Difficulty.from(game.getRatedDifficulty()) == difficulty
                        ? game : starter(difficulty) ?? game
                }
            }
            pending[request] = nil
        }
        pending[request] = task
        return task
    }

    @discardableResult
    nonisolated func prefetch(difficulty: Difficulty) -> Task<Void, Never> {
        Task { await ensureCached(difficulty: difficulty) }
    }
}
