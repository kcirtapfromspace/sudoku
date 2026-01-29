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
    }

    // MARK: - Game Management

    func newGame(difficulty: Difficulty) {
        gameState = .loading

        // Generate puzzle on background thread
        Task {
            // Create the game asynchronously (engine generation happens off main thread)
            let game = await GameViewModel.createAsync(difficulty: difficulty)

            currentGame = game
            gameState = .playing
            saveCurrentGame()
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
        } else {
            statistics.recordLoss(time: time)
            gameState = .lost
        }

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

    // MARK: - Computed Properties

    var hasSavedGame: Bool {
        currentGame != nil && gameState != .won && gameState != .lost
    }
}
