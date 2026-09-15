import Foundation
import Combine

// MARK: - Demo Mode (Debug)

private enum DemoMode {
    static var isEnabled: Bool {
        #if DEBUG
        // When launching via `simctl`, set env vars using the `SIMCTL_CHILD_` prefix:
        // `SIMCTL_CHILD_SUDOKU_DEMO_MODE=1 xcrun simctl launch ...`
        return ProcessInfo.processInfo.environment["SUDOKU_DEMO_MODE"] == "1"
        #else
        return false
        #endif
    }
}

/// Manages game transitions and the side effects associated with a completed game.
@MainActor
class GameManager: ObservableObject {
    @MainActor
    struct Dependencies {
        var generate: (Difficulty) async -> SudokuGame
        var generateSE: (Float) async -> SudokuGame
        var prefetch: (Difficulty) -> Void
        var authenticate: () -> Void
        var recordStart: (String, Difficulty) -> Void
        var recordResult: (String, Bool, TimeInterval?) -> Void
        var submitWin: (GameViewModel, GameStatistics) -> Void
        var submitResult: (GameViewModel, Bool) -> Void
        var sleep: (TimeInterval) async -> Void = { seconds in
            try? await Task.sleep(nanoseconds: UInt64(max(0, seconds) * 1_000_000_000))
        }

        static var live: Dependencies {
            connected(cache: .shared, gameCenter: .shared, history: .shared, telemetry: .shared)
        }

        static func connected(cache: PuzzleCache, gameCenter: GameCenterManager,
                              history: GameHistoryManager, telemetry: TelemetryService) -> Dependencies {
            Dependencies(
                generate: { await cache.getPuzzle(difficulty: $0) },
                generateSE: { target in
                    await Task.detached(priority: .userInitiated) { SudokuGame.newWithSeRating(targetSe: target) }.value
                },
                prefetch: { cache.prefetch(difficulty: $0) },
                authenticate: { gameCenter.authenticate() },
                recordStart: { _ = history.recordPuzzleStart(puzzleString: $0, difficulty: $1) },
                recordResult: { history.recordResult(puzzleHash: $0, won: $1, time: $2) },
                submitWin: { game, statistics in
                    let center = gameCenter
                    center.submitScore(time: game.elapsedTime, difficulty: game.difficulty)
                    center.submitWinStreak(statistics.currentStreak)
                    center.checkAchievements(difficulty: game.difficulty, time: game.elapsedTime,
                                             mistakes: game.mistakes, currentStreak: statistics.currentStreak,
                                             totalWins: statistics.gamesWon)
                },
                submitResult: { _ = telemetry.submitResult(game: $0, won: $1) }
            )
        }

        /// A complete offline boundary for previews, UI automation and deterministic unit tests.
        static var isolated: Dependencies {
            let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
            return Dependencies(generate: { _ in gameFromString(puzzle: puzzle)! },
                                generateSE: { _ in gameFromString(puzzle: puzzle)! },
                                prefetch: { _ in }, authenticate: {}, recordStart: { _, _ in },
                                recordResult: { _, _, _ in }, submitWin: { _, _ in }, submitResult: { _, _ in })
        }
    }

    @Published var currentGame: GameViewModel? {
        didSet { applyGameSettings() }
    }
    @Published var statistics: GameStatistics
    @Published var settings: GameSettings {
        didSet { applyGameSettings() }
    }
    @Published var gameState: GameState = .menu

    let historyManager: GameHistoryManager
    private let defaults: UserDefaults
    private let dependencies: Dependencies
    private let now: () -> Date
    private let statisticsKey = "sudoku_statistics"
    private let settingsKey = "sudoku_settings"
    private let savedGameKey = "sudoku_saved_game"
    private var generationTask: Task<Void, Never>?
    private var automaticallyPaused = false
    #if DEBUG
    private var demoTask: Task<Void, Never>?
    #endif

    init(defaults: UserDefaults = .standard, dependencies: Dependencies? = nil,
         historyManager: GameHistoryManager? = nil, authenticateOnLaunch: Bool = true,
         now: @escaping () -> Date = Date.init) {
        self.defaults = defaults
        let history = historyManager ?? .shared
        self.historyManager = history
        self.dependencies = dependencies ?? .connected(cache: .shared, gameCenter: .shared, history: history, telemetry: .shared)
        self.now = now
        self.statistics = defaults.data(forKey: statisticsKey)
            .flatMap { try? JSONDecoder().decode(GameStatistics.self, from: $0) } ?? GameStatistics()
        self.settings = defaults.data(forKey: settingsKey)
            .flatMap { try? JSONDecoder().decode(GameSettings.self, from: $0) } ?? GameSettings()
        loadSavedGame()
        if authenticateOnLaunch && !DemoMode.isEnabled { self.dependencies.authenticate() }
        #if DEBUG
        if dependencies == nil { startDemoIfNeeded() }
        #endif
    }

    private func applyGameSettings() {
        currentGame?.configureMistakeLimit(enabled: settings.mistakeLimitEnabled, limit: settings.mistakeLimit)
    }

    @discardableResult
    func newGame(difficulty: Difficulty) -> Task<Void, Never> {
        generationTask?.cancel()
        currentGame?.pause()
        gameState = .loading
        let task = Task {
            let puzzle = await dependencies.generate(difficulty)
            guard !Task.isCancelled else { return }
            start(puzzle, difficulty: difficulty)
            let difficulties = Difficulty.allCases
            let index = difficulties.firstIndex(of: difficulty)!
            for nearby in max(0, index - 1)...min(difficulties.count - 1, index + 1) {
                dependencies.prefetch(difficulties[nearby])
            }
        }
        generationTask = task
        return task
    }

    @discardableResult
    func newGameWithSE(targetSE: Float) -> Task<Void, Never> {
        generationTask?.cancel()
        currentGame?.pause()
        gameState = .loading
        let task = Task {
            let puzzle = await dependencies.generateSE(targetSE)
            guard !Task.isCancelled else { return }
            start(puzzle, difficulty: Difficulty.from(puzzle.getRatedDifficulty()))
        }
        generationTask = task
        return task
    }

    private func start(_ puzzle: SudokuGame, difficulty: Difficulty, imported: ImportedPuzzleData? = nil) {
        let game = GameViewModel(cachedGame: puzzle, difficulty: difficulty, now: now)
        game.configureMistakeLimit(enabled: settings.mistakeLimitEnabled, limit: settings.mistakeLimit)
        if settings.autoFillCandidates && imported?.isContinuing != true {
            game.fillAllCandidates()
        } else {
            game.clearAllCandidates()
        }
        if let imported, imported.isContinuing {
            for move in imported.playerMoves {
                game.applyImportedMove(row: move.index / 9, col: move.index % 9, value: move.digit)
            }
            for note in imported.playerNotes {
                game.applyImportedNotes(row: note.index / 9, col: note.index % 9, notes: note.notes)
            }
        }
        game.resetUndoHistory()
        currentGame = game
        automaticallyPaused = false
        gameState = .playing
        saveCurrentGame()
        dependencies.recordStart(game.getPuzzleFingerprint(), difficulty)
    }

    func loadSharedPuzzle(_ puzzleString: String) {
        let puzzle = puzzleString.count == 8 ? gameFromShortCode(code: puzzleString) : gameFromString(puzzle: puzzleString)
        guard let puzzle else { return }
        generationTask?.cancel()
        start(puzzle, difficulty: Difficulty.from(puzzle.getRatedDifficulty()))
    }

    func loadImportedPuzzle(_ data: ImportedPuzzleData) {
        guard let puzzle = gameFromString(puzzle: data.givensString) else { return }
        generationTask?.cancel()
        start(puzzle, difficulty: Difficulty.from(puzzle.getRatedDifficulty()), imported: data)
    }

    func resumeGame() {
        guard let game = currentGame, !game.isComplete, !game.isGameOver else { return }
        automaticallyPaused = false
        game.resume()
        gameState = .playing
    }

    func pauseGame() {
        guard let game = currentGame, gameState == .playing else { return }
        game.pause()
        gameState = .paused
        saveCurrentGame()
    }

    func sceneBecameInactive() {
        guard gameState == .playing else { return }
        if let game = currentGame, game.isComplete || game.isGameOver {
            endGame(won: game.isComplete)
            return
        }
        pauseGame()
        automaticallyPaused = true
    }

    func sceneBecameActive() {
        guard automaticallyPaused else { return }
        resumeGame()
    }

    func endGame(won: Bool) {
        guard let game = currentGame, gameState == .playing || gameState == .paused,
              won ? game.isComplete : game.isGameOver else { return }
        game.pause()
        let time = game.elapsedTime
        if won {
            statistics.recordWin(difficulty: game.difficulty, time: time)
            gameState = .won
            dependencies.submitWin(game, statistics)
        } else {
            statistics.recordLoss(time: time)
            gameState = .lost
        }
        dependencies.submitResult(game, won)
        dependencies.recordResult(game.puzzleHash, won, won ? time : nil)
        saveStatistics()
        clearSavedGame()
    }

    func returnToMenu() {
        generationTask?.cancel()
        currentGame?.pause()
        saveCurrentGame()
        automaticallyPaused = false
        gameState = .menu
    }

    func quitGame() {
        generationTask?.cancel()
        clearSavedGame()
        currentGame = nil
        automaticallyPaused = false
        gameState = .menu
    }

    private func saveStatistics() {
        if let data = try? JSONEncoder().encode(statistics) { defaults.set(data, forKey: statisticsKey) }
    }

    func saveSettings() {
        if let data = try? JSONEncoder().encode(settings) { defaults.set(data, forKey: settingsKey) }
    }

    func saveCurrentGame() {
        guard let game = currentGame, gameState != .won, gameState != .lost else { return }
        defaults.set(game.serialize(), forKey: savedGameKey)
    }

    private func loadSavedGame() {
        guard let json = defaults.string(forKey: savedGameKey) else { return }
        currentGame = GameViewModel.deserialize(json, now: now)
        applyGameSettings()
        if let game = currentGame, game.isComplete || game.isGameOver {
            gameState = .playing
            endGame(won: game.isComplete)
        }
    }

    private func clearSavedGame() { defaults.removeObject(forKey: savedGameKey) }

    func resetStatistics() {
        statistics = GameStatistics()
        saveStatistics()
    }

    func unlockEasterEgg() {
        statistics.activateEasterEgg()
        saveStatistics()
    }

    var hasSavedGame: Bool {
        guard let game = currentGame else { return false }
        return !game.isComplete && !game.isGameOver
    }

    // MARK: - Demo Mode (Debug)

    #if DEBUG
    @discardableResult
    func startDemoIfNeeded(enabled: Bool? = nil) -> Task<Void, Never>? {
        guard enabled ?? DemoMode.isEnabled else { return nil }

        // Make demo recordings predictable and free of modals.
        clearSavedGame()
        currentGame = nil
        gameState = .menu

        // Settings that read well on video.
        settings.theme = .light
        settings.hapticsEnabled = false
        settings.timerVisible = true
        settings.ghostHintsEnabled = true
        settings.highlightValidCells = true
        settings.highlightRelatedCells = true
        settings.highlightSameNumbers = true
        settings.autoFillCandidates = false // keep ghost hints visible by default
        settings.celebrationsEnabled = true
        settings.showErrorsImmediately = true
        saveSettings()

        demoTask?.cancel()
        demoTask = Task { [weak self] in
            await self?.runDemoSequence()
        }
        return demoTask
    }

    private func runDemoSequence() async {
        await newGame(difficulty: .beginner).value
        guard !Task.isCancelled, let game = currentGame else { return }

        func nap(_ seconds: Double) async {
            await dependencies.sleep(seconds)
        }

        // Give the UI a beat to settle.
        await nap(0.8)

        // Drive a short, varied sequence of actions.
        game.selectCell(row: 4, col: 4)
        await nap(0.5)

        for _ in 0..<4 {
            if Task.isCancelled { return }
            game.getHint()
            await nap(0.7)
            game.applyHint()
            await nap(0.5)
        }

        // Show notes features briefly.
        game.inputMode = .candidate
        await nap(0.4)
        game.fillAllCandidates()
        await nap(0.9)
        game.checkNotes()
        await nap(0.8)
        game.clearAllCandidates()
        await nap(0.5)
        game.inputMode = .normal

        // Undo/redo to show history.
        game.undo()
        await nap(0.5)
        game.redo()
        await nap(0.7)

        // Pause overlay.
        pauseGame()
        await nap(1.1)
        resumeGame()
        await nap(0.8)

        // Flip to high contrast for a moment.
        settings.theme = .highContrast
        saveSettings()
        await nap(1.0)

        // Finish by rapidly applying hints until complete (or we run out of time).
        let deadline = now().addingTimeInterval(9.0)
        while !Task.isCancelled, !game.isComplete, now() < deadline {
            game.getHint()
            game.applyHint()
            await nap(0.15)
        }
    }
    #endif
}
