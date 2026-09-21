import SwiftUI

@main
struct SudokuApp: App {
    @StateObject private var gameManager: GameManager
    @Environment(\.scenePhase) private var scenePhase

    init() {
        let launch = AppLaunchConfiguration(arguments: ProcessInfo.processInfo.arguments,
                                            environment: ProcessInfo.processInfo.environment)
        _gameManager = StateObject(wrappedValue: launch.makeManager())
        if !launch.isTesting {
            Task(priority: .background) {
                await PuzzleCache.shared.prefetchAll()
            }
            PostHogService.shared.identify()
            PostHogService.shared.capture(event: "app_launched")
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(gameManager)
                .onChange(of: scenePhase) { newPhase in
                    if newPhase == .background || newPhase == .inactive {
                        gameManager.sceneBecameInactive()
                        PostHogService.shared.flush()
                    } else if newPhase == .active {
                        gameManager.sceneBecameActive()
                    }
                }
                .onOpenURL { url in
                    if let puzzle = PuzzleLink.extract(from: url.absoluteString) {
                        gameManager.loadSharedPuzzle(puzzle)
                    }
                }
        }
    }
}

struct AppLaunchConfiguration {
    let isTesting: Bool
    let resetState: Bool
    let initialPuzzle: String?

    init(arguments: [String], environment: [String: String]) {
        #if DEBUG
        isTesting = arguments.contains("--ui-testing") || environment["SUDOKU_TESTING"] == "1"
        resetState = isTesting && arguments.contains("--reset-state")
        if isTesting, let index = arguments.firstIndex(of: "--puzzle"), arguments.indices.contains(index + 1) {
            initialPuzzle = PuzzleLink.extract(from: arguments[index + 1])
        } else {
            initialPuzzle = nil
        }
        #else
        isTesting = false
        resetState = false
        initialPuzzle = nil
        #endif
    }

    @MainActor
    func makeManager() -> GameManager {
        guard isTesting else { return GameManager() }
        let suite = "com.ukodus.app.ui-testing"
        let defaults = UserDefaults(suiteName: suite)!
        if resetState { defaults.removePersistentDomain(forName: suite) }
        let history = GameHistoryManager(defaults: defaults)
        var dependencies = GameManager.Dependencies.isolated
        dependencies.recordStart = { _ = history.recordPuzzleStart(puzzleString: $0, difficulty: $1) }
        dependencies.recordResult = { history.recordResult(puzzleHash: $0, won: $1, time: $2) }
        let manager = GameManager(defaults: defaults, dependencies: dependencies, historyManager: history)
        manager.settings.cameraImportEnabled = true
        manager.settings.hapticsEnabled = false
        manager.settings.celebrationsEnabled = false
        if let initialPuzzle { manager.loadSharedPuzzle(initialPuzzle) }
        return manager
    }
}
