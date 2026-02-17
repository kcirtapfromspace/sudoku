import XCTest
@testable import Sudoku

final class SudokuTests: XCTestCase {

    func testDifficultyAllCases() {
        XCTAssertEqual(Difficulty.allCases.count, 8)
    }

    func testCellModelCreation() {
        let cell = CellModel.empty(row: 0, col: 0)
        XCTAssertEqual(cell.row, 0)
        XCTAssertEqual(cell.col, 0)
        XCTAssertTrue(cell.isEmpty)
        XCTAssertFalse(cell.isGiven)
    }

    func testGameViewModelCreation() async {
        await MainActor.run {
            let game = GameViewModel(difficulty: .medium)
            XCTAssertEqual(game.difficulty, .medium)
            XCTAssertEqual(game.mistakes, 0)
            XCTAssertFalse(game.isComplete)
        }
    }

    func testGameStatistics() {
        var stats = GameStatistics()
        XCTAssertEqual(stats.gamesPlayed, 0)
        XCTAssertEqual(stats.winRate, 0)

        stats.recordWin(difficulty: .easy, time: 120)
        XCTAssertEqual(stats.gamesPlayed, 1)
        XCTAssertEqual(stats.gamesWon, 1)
        XCTAssertEqual(stats.winRate, 1.0)
        XCTAssertEqual(stats.currentStreak, 1)

        stats.recordLoss(time: 60)
        XCTAssertEqual(stats.gamesPlayed, 2)
        XCTAssertEqual(stats.gamesWon, 1)
        XCTAssertEqual(stats.currentStreak, 0)
    }

    func testInputModeToggle() {
        var mode = InputMode.normal
        XCTAssertEqual(mode, .normal)

        mode.toggle()
        XCTAssertEqual(mode, .candidate)

        mode.toggle()
        XCTAssertEqual(mode, .normal)
    }
}
