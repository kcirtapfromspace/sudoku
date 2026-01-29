import SwiftUI

@main
struct SudokuApp: App {
    @StateObject private var gameManager = GameManager()

    init() {
        // Start prefetching puzzles immediately on app launch
        Task(priority: .background) {
            await PuzzleCache.shared.prefetchAll()
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(gameManager)
        }
    }
}
