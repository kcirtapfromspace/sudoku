import Foundation
import Combine

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

        // Authenticate with Game Center
        gameCenter.authenticate()
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
}
