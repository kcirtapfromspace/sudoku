import XCTest
@testable import Sudoku

@MainActor
final class GameManagerTests: XCTestCase {
    private func defaults() -> UserDefaults {
        let name = "Sudoku.GameManagerTests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: name)!
        defaults.removePersistentDomain(forName: name)
        addTeardownBlock { defaults.removePersistentDomain(forName: name) }
        return defaults
    }

    func testFreshAndCorruptStorageUseDefaultsAndNoSavedGame() async {
        let storage = defaults()
        storage.set(Data("bad".utf8), forKey: "sudoku_statistics")
        storage.set(Data("bad".utf8), forKey: "sudoku_settings")
        storage.set("bad", forKey: "sudoku_saved_game")
        let manager = GameManager(defaults: storage, dependencies: .isolated)
        XCTAssertEqual(manager.gameState, .menu)
        XCTAssertEqual(manager.statistics.gamesPlayed, 0)
        XCTAssertEqual(manager.settings.mistakeLimit, 3)
        XCTAssertFalse(manager.hasSavedGame)
        manager.pauseGame()
        manager.resumeGame()
        manager.endGame(won: true)
        manager.endGame(won: false)
        manager.saveCurrentGame()
        manager.sceneBecameInactive()
        manager.sceneBecameActive()
        XCTAssertEqual(manager.gameState, .menu)
    }

    func testNewGamesApplySettingsPrefetchNeighborsAndRecordStart() async throws {
        let storage = defaults()
        var dependencies = GameManager.Dependencies.isolated
        var started: [(String, Difficulty)] = []
        var prefetched: [Difficulty] = []
        var authenticated = 0
        dependencies.recordStart = { started.append(($0, $1)) }
        dependencies.prefetch = { prefetched.append($0) }
        dependencies.authenticate = { authenticated += 1 }
        let manager = GameManager(defaults: storage, dependencies: dependencies)
        XCTAssertEqual(authenticated, 1)
        manager.settings.autoFillCandidates = true
        manager.settings.mistakeLimitEnabled = false
        manager.settings.mistakeLimit = 7
        let task = manager.newGame(difficulty: .beginner)
        XCTAssertEqual(manager.gameState, .loading)
        await task.value
        XCTAssertEqual(manager.gameState, .playing)
        let game = try XCTUnwrap(manager.currentGame)
        XCTAssertEqual(game.maxMistakes, 7)
        XCTAssertFalse(game.mistakeLimitEnabled)
        XCTAssertFalse(game.cells[0][2].candidates.isEmpty)
        XCTAssertFalse(game.canUndo)
        XCTAssertEqual(started.last?.1, .beginner)
        XCTAssertEqual(prefetched, [.beginner, .easy])
        XCTAssertTrue(manager.hasSavedGame)
        XCTAssertNotNil(storage.string(forKey: "sudoku_saved_game"))
        prefetched.removeAll()
        manager.settings.autoFillCandidates = false
        await manager.newGame(difficulty: .extreme).value
        XCTAssertEqual(prefetched, [.master, .extreme])
        XCTAssertTrue(manager.currentGame!.cells[0][2].candidates.isEmpty)
        prefetched.removeAll()
        await manager.newGame(difficulty: .medium).value
        XCTAssertEqual(prefetched, [.easy, .medium, .intermediate])
        await manager.newGameWithSE(targetSE: 2).value
        XCTAssertEqual(manager.gameState, .playing)
        XCTAssertEqual(started.count, 4)
    }

    func testQuitCancelsPendingGenerationWithoutResurrectingGame() async {
        let manager = GameManager(defaults: defaults(), dependencies: .isolated)
        let first = manager.newGame(difficulty: .easy)
        let second = manager.newGameWithSE(targetSE: 2)
        manager.quitGame()
        await first.value
        await second.value
        XCTAssertNil(manager.currentGame)
        XCTAssertEqual(manager.gameState, .menu)
        XCTAssertFalse(manager.hasSavedGame)
    }

    func testPauseMenuLifecycleAndRelaunchOnlyCountActivePlay() async throws {
        let storage = defaults()
        let clock = GameplayTests.Clock()
        let manager = GameManager(defaults: storage, dependencies: .isolated, now: { clock.date })
        await manager.newGame(difficulty: .easy).value
        let game = try XCTUnwrap(manager.currentGame)
        clock.advance(10)
        manager.pauseGame()
        clock.advance(100)
        manager.sceneBecameActive()
        XCTAssertEqual(manager.gameState, .paused)
        XCTAssertEqual(game.elapsedTime, 10)
        manager.resumeGame()
        clock.advance(5)
        manager.sceneBecameInactive()
        manager.sceneBecameInactive()
        clock.advance(50)
        manager.sceneBecameActive()
        XCTAssertEqual(manager.gameState, .playing)
        clock.advance(5)
        manager.returnToMenu()
        clock.advance(100)
        XCTAssertEqual(game.elapsedTime, 20)
        XCTAssertTrue(manager.hasSavedGame)
        let restored = GameManager(defaults: storage, dependencies: .isolated, now: { clock.date })
        XCTAssertTrue(restored.hasSavedGame)
        XCTAssertEqual(restored.currentGame?.elapsedTime, 20)
        restored.resumeGame()
        clock.advance(2)
        XCTAssertEqual(restored.currentGame?.elapsedTime, 22)
        restored.quitGame()
        XCTAssertNil(storage.string(forKey: "sudoku_saved_game"))
    }

    func testSharedAndImportedPuzzlesPreserveMovesNotesAndRejectInvalidInput() async throws {
        let manager = GameManager(defaults: defaults(), dependencies: .isolated)
        manager.loadSharedPuzzle("invalid")
        manager.loadSharedPuzzle("????????")
        XCTAssertNil(manager.currentGame)
        manager.loadImportedPuzzle(ImportedPuzzleData(givensString: "bad", playerMoves: [], playerNotes: [], isContinuing: false))
        XCTAssertNil(manager.currentGame)
        manager.loadSharedPuzzle(GameplayTests.puzzle)
        let original = try XCTUnwrap(manager.currentGame)
        manager.loadSharedPuzzle("bad")
        XCTAssertTrue(manager.currentGame === original)
        manager.settings.autoFillCandidates = true
        manager.loadImportedPuzzle(ImportedPuzzleData(givensString: GameplayTests.puzzle,
            playerMoves: [(2, 4), (-1, 5)], playerNotes: [(3, [6, 9])], isContinuing: true))
        XCTAssertEqual(manager.currentGame?.cells[0][2].value, 4)
        XCTAssertEqual(manager.currentGame?.cells[0][3].candidates, [6, 9])
        XCTAssertTrue(manager.currentGame!.cells[8][0].candidates.isEmpty)
        manager.loadImportedPuzzle(ImportedPuzzleData(givensString: GameplayTests.puzzle,
            playerMoves: [(2, 4)], playerNotes: [(3, [6])], isContinuing: false))
        XCTAssertEqual(manager.currentGame?.cells[0][2].value, 0)
        XCTAssertEqual(manager.currentGame?.cells[0][2].candidates, [1, 2, 4])
    }

    func testVictoryIsVerifiedAndRecordedOnceWithFrozenTime() async throws {
        let storage = defaults()
        let clock = GameplayTests.Clock()
        var dependencies = GameManager.Dependencies.isolated
        var submissions: [Bool] = []
        var history: [(Bool, TimeInterval?)] = []
        var wins = 0
        dependencies.submitWin = { _, _ in wins += 1 }
        dependencies.submitResult = { _, won in submissions.append(won) }
        dependencies.recordResult = { _, won, time in history.append((won, time)) }
        let manager = GameManager(defaults: storage, dependencies: dependencies, now: { clock.date })
        await manager.newGame(difficulty: .easy).value
        manager.endGame(won: true)
        XCTAssertEqual(manager.statistics.gamesWon, 0)
        let game = try XCTUnwrap(manager.currentGame)
        clock.advance(30)
        game.fillAllExcept(count: 1)
        let last = try XCTUnwrap(game.cells.flatMap { $0 }.first { $0.isEmpty })
        game.selectCell(row: last.row, col: last.col)
        game.enterNumber(game.getSolution(row: last.row, col: last.col))
        clock.advance(10)
        manager.endGame(won: true)
        manager.endGame(won: true)
        manager.resumeGame()
        manager.saveCurrentGame()
        XCTAssertEqual(manager.gameState, .won)
        XCTAssertEqual(manager.statistics.gamesWon, 1)
        XCTAssertEqual(manager.statistics.totalPlayTime, 30)
        XCTAssertEqual(wins, 1)
        XCTAssertEqual(submissions, [true])
        XCTAssertEqual(history.count, 1)
        XCTAssertEqual(history.first?.1, 30)
        XCTAssertFalse(manager.hasSavedGame)
        XCTAssertNil(storage.string(forKey: "sudoku_saved_game"))
    }

    func testLossAndStatisticsSettingsPersistence() async throws {
        let storage = defaults()
        var dependencies = GameManager.Dependencies.isolated
        var result: Bool?
        dependencies.submitResult = { _, won in result = won }
        let manager = GameManager(defaults: storage, dependencies: dependencies)
        manager.settings.mistakeLimit = 1
        manager.settings.theme = .dark
        manager.saveSettings()
        await manager.newGame(difficulty: .easy).value
        let game = try XCTUnwrap(manager.currentGame)
        manager.endGame(won: false)
        XCTAssertEqual(manager.statistics.gamesPlayed, 0)
        game.selectCell(row: 0, col: 2)
        game.enterNumber(5)
        manager.pauseGame()
        manager.endGame(won: false)
        manager.endGame(won: false)
        XCTAssertEqual(manager.gameState, .lost)
        XCTAssertEqual(result, false)
        XCTAssertEqual(manager.statistics.gamesPlayed, 1)
        XCTAssertFalse(manager.hasSavedGame)
        manager.unlockEasterEgg()
        let restored = GameManager(defaults: storage, dependencies: .isolated)
        XCTAssertTrue(restored.statistics.easterEggUnlocked)
        XCTAssertEqual(restored.statistics.gamesPlayed, 1)
        XCTAssertEqual(restored.settings.theme, .dark)
        restored.resetStatistics()
        XCTAssertEqual(restored.statistics.gamesPlayed, 0)
        XCTAssertFalse(restored.statistics.easterEggUnlocked)
    }
    func testConnectedServicesRecordACompletedGameWithoutExternalEffects() async throws {
        let generated = await GameManager.Dependencies.live.generateSE(1.5)
        XCTAssertEqual(generated.getAllCells().count, 81)
        XCTAssertFalse(generated.isComplete())
        let cache = PuzzleCache(fetch: { _ in ServiceFixtures.game() }, generate: { _ in ServiceFixtures.game() })
        var centerDependencies = GameCenterManager.Dependencies()
        centerDependencies.authenticate = { $0(nil, nil, false, nil) }
        centerDependencies.submitScore = { _, _ in }
        centerDependencies.reportAchievement = { _ in }
        centerDependencies.rootController = { nil }
        let center = GameCenterManager(defaults: defaults(), dependencies: centerDependencies)
        let history = GameHistoryManager(defaults: defaults())
        let session = ServiceURLProtocol.session { request in try ServiceFixtures.response(request, status: 404) }
        let telemetry = TelemetryService(session: session, defaults: defaults())
        var posthogEvents: [String] = []
        let posthogSession = ServiceURLProtocol.session { request in
            if let data = request.httpBody ?? (request.httpBodyStream.flatMap { stream in
                stream.open(); defer { stream.close() }
                var buffer = Data(), bytes = [UInt8](repeating: 0, count: 1024)
                while stream.hasBytesAvailable {
                    let n = stream.read(&bytes, maxLength: bytes.count)
                    if n <= 0 { break }
                    buffer.append(bytes, count: n)
                }
                return buffer
            }),
               let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let batch = json["batch"] as? [[String: Any]] {
                for item in batch {
                    if let ev = item["event"] as? String { posthogEvents.append(ev) }
                }
            }
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }
        let posthog = PostHogService(apiKey: "test_key", session: posthogSession, defaults: defaults(), isEnabled: true)
        let dependencies = GameManager.Dependencies.connected(cache: cache, gameCenter: center, history: history, telemetry: telemetry, posthog: posthog)
        let manager = GameManager(defaults: defaults(), dependencies: dependencies)
        await manager.newGame(difficulty: .beginner).value
        XCTAssertEqual(history.stats.totalPlays, 1)
        let game = try XCTUnwrap(manager.currentGame)
        game.fillAllExcept(count: 1)
        let last = try XCTUnwrap(game.cells.flatMap { $0 }.first { $0.isEmpty })
        game.selectCell(row: last.row, col: last.col)
        game.enterNumber(game.getSolution(row: last.row, col: last.col))
        manager.endGame(won: true)
        XCTAssertEqual(history.stats.solvedPuzzles, 1)
        XCTAssertEqual(history.getPuzzle(hash: game.puzzleHash)?.playCount, 1)
        await manager.newGameWithSE(targetSE: 1.5).value
        XCTAssertEqual(manager.gameState, .playing)
        try await Task.sleep(nanoseconds: 50_000_000)
        XCTAssertTrue(posthogEvents.contains("game_started"))
        XCTAssertTrue(posthogEvents.contains("game_completed"))
    }

    func testDemoExercisesRecordedGameFlowWithControllableTimeAndCancellation() async throws {
        let clock = GameplayTests.Clock()
        var dependencies = GameManager.Dependencies.isolated
        await dependencies.sleep(0)
        dependencies.sleep = { seconds in
            clock.advance(seconds)
            await Task.yield()
        }
        let manager = GameManager(defaults: defaults(), dependencies: dependencies, now: { clock.date })
        XCTAssertNil(manager.startDemoIfNeeded(enabled: false))
        XCTAssertNil(manager.startDemoIfNeeded())
        let demo = try XCTUnwrap(manager.startDemoIfNeeded(enabled: true))
        await demo.value
        XCTAssertEqual(manager.settings.theme, .highContrast)
        XCTAssertFalse(manager.settings.hapticsEnabled)
        XCTAssertTrue(manager.settings.ghostHintsEnabled)
        XCTAssertEqual(manager.currentGame?.inputMode, .normal)
        XCTAssertNotNil(manager.currentGame)
        XCTAssertGreaterThan(manager.currentGame!.cells.flatMap { $0 }.filter { !$0.isGiven && !$0.isEmpty }.count, 0)
        let canceled = try XCTUnwrap(manager.startDemoIfNeeded(enabled: true))
        canceled.cancel()
        await canceled.value
        XCTAssertNotNil(manager.currentGame)
    }

    func testFinishedGameSavedBeforeContinueIsRecordedOnceAfterRelaunch() async throws {
        let storage = defaults()
        let manager = GameManager(defaults: storage, dependencies: .isolated)
        await manager.newGame(difficulty: .easy).value
        let game = try XCTUnwrap(manager.currentGame)
        game.fillAllExcept(count: 1)
        let last = try XCTUnwrap(game.cells.flatMap { $0 }.first { $0.isEmpty })
        game.selectCell(row: last.row, col: last.col)
        game.enterNumber(game.getSolution(row: last.row, col: last.col))
        manager.saveCurrentGame()
        XCTAssertNotNil(storage.string(forKey: "sudoku_saved_game"))
        let restored = GameManager(defaults: storage, dependencies: .isolated)
        XCTAssertEqual(restored.gameState, .won)
        XCTAssertEqual(restored.statistics.gamesWon, 1)
        XCTAssertNil(storage.string(forKey: "sudoku_saved_game"))
        let relaunched = GameManager(defaults: storage, dependencies: .isolated)
        XCTAssertEqual(relaunched.statistics.gamesWon, 1)
        XCTAssertFalse(relaunched.hasSavedGame)
    }

    func testBackgroundingAfterLastMistakeFinalizesWithoutAnUnresumablePause() async throws {
        let manager = GameManager(defaults: defaults(), dependencies: .isolated)
        manager.settings.mistakeLimit = 1
        await manager.newGame(difficulty: .easy).value
        let game = try XCTUnwrap(manager.currentGame)
        game.selectCell(row: 0, col: 2)
        game.enterNumber(5)
        manager.sceneBecameInactive()
        manager.sceneBecameActive()
        XCTAssertEqual(manager.gameState, .lost)
        XCTAssertEqual(manager.statistics.gamesPlayed, 1)
    }

    func testOwnedLibrariesStayIsolatedAndSurviveRelaunchWithRealHistoryCallbacks() async throws {
        func manager(storage: UserDefaults) -> GameManager {
            let history = GameHistoryManager(defaults: storage)
            var dependencies = GameManager.Dependencies.isolated
            dependencies.recordStart = { _ = history.recordPuzzleStart(puzzleString: $0, difficulty: $1) }
            dependencies.recordResult = { history.recordResult(puzzleHash: $0, won: $1, time: $2) }
            return GameManager(defaults: storage, dependencies: dependencies, historyManager: history)
        }
        let firstStorage = defaults()
        let first = manager(storage: firstStorage)
        let other = manager(storage: defaults())
        await first.newGame(difficulty: .easy).value
        let game = try XCTUnwrap(first.currentGame)
        XCTAssertEqual(first.historyManager.stats.totalPlays, 1)
        XCTAssertTrue(other.historyManager.recentPuzzles.isEmpty)
        first.settings.mistakeLimit = 1
        game.selectCell(row: 0, col: 2)
        game.enterNumber(5)
        first.endGame(won: false)
        XCTAssertEqual(first.historyManager.getPuzzle(hash: game.puzzleHash)?.losses, 1)
        let restored = manager(storage: firstStorage)
        XCTAssertEqual(restored.historyManager.stats.totalPlays, 1)
        XCTAssertEqual(restored.historyManager.getPuzzle(hash: game.puzzleHash)?.losses, 1)
        XCTAssertTrue(other.historyManager.recentPuzzles.isEmpty)
    }

    func testDefaultServiceAssemblyCanInitializeWithoutInteractiveAuthentication() async {
        let storage = defaults()
        let history = GameHistoryManager(defaults: storage)
        let manager = GameManager(defaults: storage, historyManager: history, authenticateOnLaunch: false)
        XCTAssertTrue(manager.historyManager === history)
        XCTAssertEqual(manager.gameState, .menu)
        XCTAssertNil(manager.currentGame)
        XCTAssertEqual(manager.statistics.gamesPlayed, 0)
    }

}
