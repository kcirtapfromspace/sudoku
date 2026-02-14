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

/// Manages overall game state, persistence, and statistics
@MainActor
class GameManager: ObservableObject {
    // MARK: - Published Properties

    @Published var currentGame: GameViewModel?
    @Published var statistics: GameStatistics
    @Published var settings: GameSettings
    @Published var gameState: GameState = .menu

    // MARK: - Game Center

    let gameCenter = GameCenterManager.shared

    // MARK: - Private Properties

    private let statisticsKey = "sudoku_statistics"
    private let settingsKey = "sudoku_settings"
    private let savedGameKey = "sudoku_saved_game"

    #if DEBUG
    private var demoTask: Task<Void, Never>?
    #endif

    // MARK: - Initialization

    init() {
        // Load statistics
        if let data = UserDefaults.standard.data(forKey: statisticsKey),
           let stats = try? JSONDecoder().decode(GameStatistics.self, from: data) {
            self.statistics = stats
        } else {
            self.statistics = GameStatistics()
        }

        // Load settings
        if let data = UserDefaults.standard.data(forKey: settingsKey),
           let settings = try? JSONDecoder().decode(GameSettings.self, from: data) {
            self.settings = settings
        } else {
            self.settings = GameSettings()
        }

        // Try to load saved game
        loadSavedGame()

        // Authenticate with Game Center (skip in demo mode to avoid UI popups during recording).
        if !DemoMode.isEnabled {
            gameCenter.authenticate()
        }

        #if DEBUG
        startDemoIfNeeded()
        #endif
    }

    // MARK: - Game Management

    func newGame(difficulty: Difficulty) {
        gameState = .loading

        Task {
            // Get puzzle from cache (instant if prefetched)
            let puzzle = await PuzzleCache.shared.getPuzzle(difficulty: difficulty)
            let game = GameViewModel(cachedGame: puzzle, difficulty: difficulty)

            // Handle candidates based on setting
            // The Rust engine auto-calculates candidates, so we need to clear them
            // if the user doesn't want auto-fill
            if settings.autoFillCandidates {
                game.fillAllCandidates()
            } else {
                game.clearAllCandidates()
            }

            currentGame = game
            gameState = .playing
            saveCurrentGame()

            // Record puzzle in history
            let fingerprint = game.getPuzzleFingerprint()
            GameHistoryManager.shared.recordPuzzleStart(puzzleString: fingerprint, difficulty: difficulty)

            // Prefetch puzzles for nearby difficulties during gameplay
            prefetchNearbyDifficulties(current: difficulty)
        }
    }

    /// Prefetch puzzles for difficulties the user might choose next
    private func prefetchNearbyDifficulties(current: Difficulty) {
        let allCases = Difficulty.allCases
        guard let currentIndex = allCases.firstIndex(of: current) else { return }

        // Prefetch same difficulty (for "play again")
        PuzzleCache.shared.prefetch(difficulty: current)

        // Prefetch adjacent difficulties
        if currentIndex > 0 {
            PuzzleCache.shared.prefetch(difficulty: allCases[currentIndex - 1])
        }
        if currentIndex < allCases.count - 1 {
            PuzzleCache.shared.prefetch(difficulty: allCases[currentIndex + 1])
        }
    }

    func newGameWithSE(targetSE: Float) {
        gameState = .loading

        Task {
            let sudokuGame = await Task.detached(priority: .userInitiated) {
                SudokuGame.newWithSeRating(targetSe: targetSE)
            }.value

            // Derive difficulty from the rated result
            let ratedDifficulty = Difficulty.from(sudokuGame.getRatedDifficulty())

            let game = GameViewModel(cachedGame: sudokuGame, difficulty: ratedDifficulty)

            if settings.autoFillCandidates {
                game.fillAllCandidates()
            } else {
                game.clearAllCandidates()
            }

            currentGame = game
            gameState = .playing
            saveCurrentGame()

            let fingerprint = game.getPuzzleFingerprint()
            GameHistoryManager.shared.recordPuzzleStart(puzzleString: fingerprint, difficulty: ratedDifficulty)
        }
    }

    /// Load a shared puzzle from an 81-character string or 8-character short code
    func loadSharedPuzzle(_ puzzleString: String) {
        // Try short code first (8-char PuzzleId)
        let game: SudokuGame
        if puzzleString.count == 8, let shortCodeGame = gameFromShortCode(code: puzzleString) {
            game = shortCodeGame
        } else if let stringGame = gameFromString(puzzle: puzzleString) {
            game = stringGame
        } else {
            return
        }
        let rated = Difficulty.from(game.getRatedDifficulty())
        let viewModel = GameViewModel(cachedGame: game, difficulty: rated)

        if settings.autoFillCandidates {
            viewModel.fillAllCandidates()
        } else {
            viewModel.clearAllCandidates()
        }

        currentGame = viewModel
        gameState = .playing
        saveCurrentGame()

        let fingerprint = viewModel.getPuzzleFingerprint()
        GameHistoryManager.shared.recordPuzzleStart(puzzleString: fingerprint, difficulty: rated)
    }

    func resumeGame() {
        guard currentGame != nil else { return }
        gameState = .playing
    }

    func pauseGame() {
        gameState = .paused
        saveCurrentGame()
    }

    func endGame(won: Bool) {
        guard let game = currentGame else { return }

        let time = game.elapsedTime
        if won {
            statistics.recordWin(difficulty: game.difficulty, time: time)
            gameState = .won

            // Submit to Game Center
            gameCenter.submitScore(time: time, difficulty: game.difficulty)
            gameCenter.submitWinStreak(statistics.currentStreak)
            gameCenter.checkAchievements(
                difficulty: game.difficulty,
                time: time,
                mistakes: game.mistakes,
                currentStreak: statistics.currentStreak,
                totalWins: statistics.gamesWon
            )
        } else {
            statistics.recordLoss(time: time)
            gameState = .lost
        }

        // Submit to ukodus API (fire-and-forget)
        TelemetryService.shared.submitResult(game: game, won: won)

        // Record result in history
        GameHistoryManager.shared.recordResult(
            puzzleHash: game.puzzleHash,
            won: won,
            time: won ? time : nil
        )

        saveStatistics()
        clearSavedGame()
    }

    func returnToMenu() {
        saveCurrentGame()
        gameState = .menu
    }

    func quitGame() {
        clearSavedGame()
        currentGame = nil
        gameState = .menu
    }

    // MARK: - Persistence

    private func saveStatistics() {
        if let data = try? JSONEncoder().encode(statistics) {
            UserDefaults.standard.set(data, forKey: statisticsKey)
        }
    }

    func saveSettings() {
        if let data = try? JSONEncoder().encode(settings) {
            UserDefaults.standard.set(data, forKey: settingsKey)
        }
    }

    func saveCurrentGame() {
        guard let game = currentGame else { return }
        let saveData = game.serialize()
        UserDefaults.standard.set(saveData, forKey: savedGameKey)
    }

    private func loadSavedGame() {
        guard let saveData = UserDefaults.standard.string(forKey: savedGameKey) else { return }
        if let game = GameViewModel.deserialize(saveData) {
            currentGame = game
        }
    }

    private func clearSavedGame() {
        UserDefaults.standard.removeObject(forKey: savedGameKey)
    }

    func resetStatistics() {
        statistics = GameStatistics()
        saveStatistics()
    }

    /// Unlock Master and Extreme difficulties via easter egg
    func unlockEasterEgg() {
        statistics.activateEasterEgg()
        saveStatistics()
    }

    // MARK: - Computed Properties

    var hasSavedGame: Bool {
        currentGame != nil && gameState != .won && gameState != .lost
    }

    // MARK: - Demo Mode (Debug)

    #if DEBUG
    private func startDemoIfNeeded() {
        guard DemoMode.isEnabled else { return }

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
    }

    private func runDemoSequence() async {
        // Start a fresh puzzle.
        newGame(difficulty: .beginner)

        // Wait until the puzzle is loaded.
        while !Task.isCancelled {
            if gameState == .playing, currentGame != nil { break }
            try? await Task.sleep(nanoseconds: 100_000_000) // 0.1s
        }

        guard !Task.isCancelled, let game = currentGame else { return }

        func nap(_ seconds: Double) async {
            let ns = UInt64(max(0, seconds) * 1_000_000_000)
            try? await Task.sleep(nanoseconds: ns)
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
        let deadline = Date().addingTimeInterval(9.0)
        while !Task.isCancelled, !game.isComplete, Date() < deadline {
            game.getHint()
            game.applyHint()
            await nap(0.15)
        }
    }
    #endif
}
