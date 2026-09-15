import XCTest
@testable import Sudoku

@MainActor
final class HistoryServiceTests: XCTestCase {
    func testHistoryPersistsReplaysResultsFiltersAndClear() throws {
        let defaults = ServiceFixtures.defaults()
        let history = GameHistoryManager(defaults: defaults)
        XCTAssertTrue(history.recentPuzzles.isEmpty)
        let first = history.recordPuzzleStart(puzzleString: ServiceFixtures.puzzle, difficulty: .medium)
        XCTAssertTrue(history.hasPlayed(puzzleString: ServiceFixtures.puzzle))
        XCTAssertFalse(history.hasPlayed(puzzleString: "other"))
        XCTAssertEqual(history.recordPuzzleStart(puzzleString: ServiceFixtures.puzzle, difficulty: .medium).playCount, 2)
        let second = history.recordPuzzleStart(puzzleString: String(ServiceFixtures.puzzle.reversed()), difficulty: .easy)
        XCTAssertEqual(history.recentPuzzles.first?.puzzleHash, second.puzzleHash)
        XCTAssertEqual(history.puzzles(for: .medium).map(\.puzzleHash), [first.puzzleHash])
        history.recordResult(puzzleHash: "missing", won: true, time: 10)
        history.recordResult(puzzleHash: first.puzzleHash, won: false, time: nil)
        history.recordResult(puzzleHash: first.puzzleHash, won: true, time: 90)
        history.recordResult(puzzleHash: first.puzzleHash, won: true, time: 120)
        XCTAssertEqual(history.stats.totalPuzzles, 2)
        XCTAssertEqual(history.stats.totalPlays, 3)
        XCTAssertEqual(history.stats.solvedPuzzles, 1)
        XCTAssertEqual(history.unsolvedPuzzles.map(\.puzzleHash), [second.puzzleHash])
        XCTAssertEqual(history.getPuzzle(hash: first.puzzleHash)?.bestTime, 90)
        let restored = GameHistoryManager(defaults: defaults)
        XCTAssertEqual(restored.stats.solvedPuzzles, 1)
        XCTAssertEqual(restored.puzzles.count, 2)
        restored.clearHistory()
        XCTAssertEqual(GameHistoryManager(defaults: defaults).stats.totalPuzzles, 0)
        XCTAssertTrue(restored.puzzles.isEmpty)
    }
    func testCorruptAndDuplicatePersistenceDoesNotCrash() throws {
        let defaults = ServiceFixtures.defaults()
        defaults.set(Data("invalid".utf8), forKey: "sudoku_puzzle_library")
        defaults.set(Data("invalid".utf8), forKey: "sudoku_library_stats")
        XCTAssertTrue(GameHistoryManager(defaults: defaults).puzzles.isEmpty)
        let old = PuzzleRecord(puzzleString: ServiceFixtures.puzzle, difficulty: .medium)
        var newer = old; newer.lastPlayedAt = old.lastPlayedAt.addingTimeInterval(20)
        defaults.set(try JSONEncoder().encode([newer, old, newer]), forKey: "sudoku_puzzle_library")
        let restored = GameHistoryManager(defaults: defaults)
        XCTAssertEqual(restored.puzzles.count, 1)
        XCTAssertEqual(restored.recentPuzzles[0].lastPlayedAt, newer.lastPlayedAt)
    }
    func testFingerprintIncludesOnlyGivens() {
        var cells = (0..<9).map { row in (0..<9).map { CellModel.empty(row: row, col: $0) } }
        cells[0][0].value = 5; cells[0][0].isGiven = true
        cells[0][1].value = 3
        let fingerprint = GameHistoryManager.extractPuzzleFingerprint(from: cells)
        XCTAssertEqual(fingerprint, "5" + String(repeating: ".", count: 80))
    }
}
