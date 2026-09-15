import XCTest
@testable import Sudoku

@MainActor
final class GameplayTests: XCTestCase {
    static let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
    static let solution = "534678912672195348198342567859761423426853791713924856961537284287419635345286179"

    final class Clock {
        var date = Date(timeIntervalSince1970: 1000)
        func advance(_ seconds: TimeInterval) { date.addTimeInterval(seconds) }
    }

    func fixture(_ puzzle: String? = nil, clock: Clock? = nil) throws -> GameViewModel {
        let engine = try XCTUnwrap(gameFromString(puzzle: puzzle ?? Self.puzzle))
        let model = GameViewModel(cachedGame: engine, difficulty: .easy, now: { clock?.date ?? Date() })
        model.clearAllCandidates()
        model.resetUndoHistory()
        return model
    }

    func testInputSelectionAndHighlightsRespectGivensAndBounds() async throws {
        let game = try fixture()
        XCTAssertEqual(game.cells.count, 9)
        XCTAssertEqual(game.cells.flatMap { $0 }.count, 81)
        XCTAssertEqual(game.selectedValue, 0)
        XCTAssertFalse(game.isRelated(to: (0, 0)))
        XCTAssertFalse(game.hasSameValue(as: (0, 0)))
        game.enterNumber(4)
        game.clearSelectedCell()
        game.fillCandidatesForSelected()
        game.clearCandidatesForSelected()
        game.selectCell(row: -1, col: 10)
        XCTAssertNil(game.selectedCell)
        game.selectCell(row: 0, col: 0)
        game.enterNumber(4)
        game.clearSelectedCell()
        XCTAssertEqual(game.cells[0][0].value, 5)
        XCTAssertEqual(game.selectedValue, 5)
        XCTAssertTrue(game.isRelated(to: (0, 8)))
        XCTAssertTrue(game.isRelated(to: (8, 0)))
        XCTAssertTrue(game.isRelated(to: (2, 2)))
        XCTAssertFalse(game.isRelated(to: (8, 8)))
        XCTAssertFalse(game.isRelated(to: nil))
        XCTAssertTrue(game.hasSameValue(as: (1, 5)))
        XCTAssertFalse(game.hasSameValue(as: nil))
        game.selectCell(row: 0, col: 2)
        game.enterNumber(0)
        game.enterNumber(256)
        XCTAssertEqual(game.cells[0][2].value, 0)
        game.enterNumber(4)
        XCTAssertEqual(game.cells[0][2].value, 4)
        XCTAssertTrue(game.canUndo)
        XCTAssertFalse(game.canRedo)
        XCTAssertEqual(game.mistakes, 0)
        XCTAssertEqual(game.getPuzzleFingerprint(), Self.puzzle.replacingOccurrences(of: "0", with: "."))
        XCTAssertEqual(game.getPuzzleString(), game.getPuzzleFingerprint())
        XCTAssertEqual(game.puzzleHash, PuzzleRecord.generateHash(from: game.getPuzzleFingerprint()))
        XCTAssertNil(game.getShortCode())
        XCTAssertEqual(game.numberCounts.count, 9)
        XCTAssertTrue(game.completedNumbers.isEmpty)
        game.clearSelection()
        XCTAssertNil(game.selectedCell)
    }

    func testNotesEraseUndoRedoAndValueReplacementRestoreVisibleState() async throws {
        let game = try fixture()
        game.selectCell(row: 0, col: 2)
        game.inputMode = .candidate
        game.enterNumber(1)
        game.enterNumber(4)
        XCTAssertEqual(game.cells[0][2].candidates, [1, 4])
        game.undo()
        XCTAssertEqual(game.cells[0][2].candidates, [1])
        XCTAssertTrue(game.canRedo)
        game.redo()
        XCTAssertEqual(game.cells[0][2].candidates, [1, 4])
        game.clearSelectedCell()
        XCTAssertTrue(game.cells[0][2].candidates.isEmpty)
        game.undo()
        XCTAssertEqual(game.cells[0][2].candidates, [1, 4])
        game.inputMode = .normal
        game.enterNumber(4)
        XCTAssertTrue(game.cells[0][2].candidates.isEmpty)
        game.undo()
        XCTAssertEqual(game.cells[0][2].value, 0)
        XCTAssertEqual(game.cells[0][2].candidates, [1, 4])
        game.redo()
        game.clearSelectedCell()
        XCTAssertEqual(game.cells[0][2].value, 0)
        game.undo()
        XCTAssertEqual(game.cells[0][2].value, 4)
        game.resetUndoHistory()
        game.undo()
        game.redo()
        XCTAssertFalse(game.canUndo)
        XCTAssertFalse(game.canRedo)
    }

    func testTemporaryNotesRevertAndNewEditsDiscardRedo() async throws {
        let game = try fixture()
        game.selectCell(row: 0, col: 2)
        game.enterTemporaryNoteMode()
        game.enterTemporaryNoteMode()
        XCTAssertEqual(game.inputMode, .temporaryCandidate)
        game.enterNumber(4)
        XCTAssertEqual(game.inputMode, .normal)
        XCTAssertEqual(game.cells[0][2].candidates, [4])
        game.inputMode = .candidate
        game.enterTemporaryNoteMode()
        game.enterNumber(1)
        XCTAssertEqual(game.inputMode, .candidate)
        game.enterNumber(1)
        XCTAssertEqual(game.cells[0][2].candidates, [4])
        game.undo()
        game.enterNumber(2)
        XCTAssertFalse(game.canRedo)
        game.inputMode = .normal
        game.enterNumber(4)
        game.inputMode = .candidate
        game.enterNumber(1)
        XCTAssertEqual(game.cells[0][2].value, 4)
    }

    func testAutomaticAndManualCandidateToolsPreserveUndoAndValidity() async throws {
        let game = try fixture()
        game.selectCell(row: 0, col: 2)
        XCTAssertEqual(game.getValidCandidates(row: 0, col: 2), [1, 2, 4])
        XCTAssertFalse(game.isNakedSingle(row: 0, col: 2))
        game.fillCandidatesForSelected()
        XCTAssertEqual(game.cells[0][2].candidates, [1, 2, 4])
        game.clearCandidatesForSelected()
        XCTAssertTrue(game.cells[0][2].candidates.isEmpty)
        game.undo()
        XCTAssertEqual(game.cells[0][2].candidates, [1, 2, 4])
        game.fillAllCandidates()
        XCTAssertFalse(game.cells[8][0].candidates.isEmpty)
        game.inputMode = .candidate
        game.enterNumber(1)
        XCTAssertEqual(game.cells[0][2].candidates, [2, 4])
        game.checkNotes()
        XCTAssertEqual(game.cells[0][2].candidates, [4])
        game.undo()
        XCTAssertEqual(game.cells[0][2].candidates, [2, 4])
        game.clearAllCandidates()
        XCTAssertTrue(game.cells.flatMap { $0 }.allSatisfy { $0.candidates.isEmpty })
        game.selectCell(row: 0, col: 0)
        game.fillCandidatesForSelected()
        game.clearCandidatesForSelected()
        XCTAssertTrue(game.cells[0][0].candidates.isEmpty)
    }

    func testErrorsHonorConfiguredLimitAndUndoDoesNotRefundMistakes() async throws {
        let game = try fixture()
        game.configureMistakeLimit(enabled: true, limit: 2)
        game.selectCell(row: 0, col: 2)
        game.enterNumber(5)
        XCTAssertEqual(game.mistakes, 1)
        XCTAssertTrue(game.cells[0][2].hasConflict)
        game.undo()
        XCTAssertEqual(game.cells[0][2].value, 0)
        XCTAssertEqual(game.mistakes, 1)
        game.enterNumber(5)
        XCTAssertTrue(game.isGameOver)
        XCTAssertFalse(game.canUndo)
        game.enterNumber(4)
        game.clearSelectedCell()
        game.undo()
        game.redo()
        game.fillAllCandidates()
        game.clearAllCandidates()
        game.checkNotes()
        game.fillCandidatesForSelected()
        game.clearCandidatesForSelected()
        game.getHint()
        game.applyHint()
        XCTAssertEqual(game.cells[0][2].value, 5)
        game.configureMistakeLimit(enabled: false, limit: 0)
        XCTAssertEqual(game.maxMistakes, 1)
        XCTAssertFalse(game.isGameOver)
        game.enterNumber(5)
        XCTAssertEqual(game.mistakes, 3)
        XCTAssertFalse(game.isGameOver)
        game.configureMistakeLimit(enabled: true, limit: 100)
        XCTAssertEqual(game.maxMistakes, 10)
    }

    func testSavedGamePreservesEditableMovesNotesErrorsAndPausedElapsedTime() async throws {
        let clock = Clock()
        let game = try fixture(clock: clock)
        game.configureMistakeLimit(enabled: false, limit: 3)
        game.selectCell(row: 0, col: 2)
        game.enterNumber(5)
        game.selectCell(row: 0, col: 3)
        game.inputMode = .candidate
        game.enterNumber(6)
        clock.advance(65)
        let json = game.serialize()
        clock.advance(200)
        let restored = try XCTUnwrap(GameViewModel.deserialize(json, now: { clock.date }))
        XCTAssertEqual(restored.elapsedTime, 65)
        XCTAssertEqual(restored.elapsedTimeString, "01:05")
        XCTAssertEqual(restored.cells[0][2].value, 5)
        XCTAssertFalse(restored.cells[0][2].isGiven)
        XCTAssertEqual(restored.mistakes, 1)
        XCTAssertEqual(restored.cells[0][3].candidates, [6])
        XCTAssertEqual(restored.getPuzzleFingerprint(), game.getPuzzleFingerprint())
        restored.resume()
        clock.advance(5)
        restored.selectCell(row: 0, col: 2)
        restored.enterNumber(4)
        XCTAssertEqual(restored.cells[0][2].value, 4)
        XCTAssertEqual(restored.elapsedTime, 70)
        restored.selectCell(row: 0, col: 3)
        restored.clearSelectedCell()
        XCTAssertTrue(restored.cells[0][3].candidates.isEmpty)
        XCTAssertNil(GameViewModel.deserialize("not json"))
        XCTAssertNil(GameViewModel.deserialize("{}"))
        var corrupt = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any])
        corrupt["notes"] = [[[10]]]
        XCTAssertNil(GameViewModel.deserialize(String(decoding: try JSONSerialization.data(withJSONObject: corrupt), as: UTF8.self)))
        corrupt["playerValues"] = [99]
        XCTAssertNil(GameViewModel.deserialize(String(decoding: try JSONSerialization.data(withJSONObject: corrupt), as: UTF8.self)))
    }

    func testAutoNotesAndLegacySavesRestoreSafely() async throws {
        let game = try fixture()
        game.fillAllCandidates()
        let restored = try XCTUnwrap(GameViewModel.deserialize(game.serialize()))
        XCTAssertEqual(restored.cells[0][2].candidates, [1, 2, 4])
        let legacyEngine = try XCTUnwrap(gameFromString(puzzle: Self.puzzle))
        let legacy = try XCTUnwrap(GameViewModel.deserialize(legacyEngine.serialize()))
        XCTAssertEqual(legacy.difficulty, .medium)
        XCTAssertEqual(legacy.elapsedTime, 0)
        XCTAssertTrue(legacy.cells[0][2].candidates.isEmpty)
    }

    func testClockOnlyCountsActivePlayAndTerminalGamesCannotResume() async throws {
        let clock = Clock()
        let game = try fixture(clock: clock)
        clock.advance(10)
        game.pause()
        clock.advance(50)
        game.pause()
        XCTAssertEqual(game.elapsedTime, 10)
        game.resume()
        game.resume()
        clock.advance(7)
        XCTAssertEqual(game.elapsedTime, 17)
        game.fillAllExcept(count: 1)
        let last = try XCTUnwrap(game.cells.flatMap { $0 }.first { $0.isEmpty })
        game.selectCell(row: last.row, col: last.col)
        game.enterNumber(game.getSolution(row: last.row, col: last.col))
        XCTAssertTrue(game.isComplete)
        XCTAssertEqual(game.lastCelebration, .gameComplete)
        XCTAssertEqual(game.completedNumbers, Set(1...9))
        game.resume()
        clock.advance(20)
        XCTAssertEqual(game.elapsedTime, 17)
        game.clearCelebration()
        XCTAssertNil(game.lastCelebration)
    }

    func testHintsUpgradeDismissAndApplyVerifiedValues() async throws {
        let game = try fixture()
        game.applyHint()
        game.fillAllCandidates()
        game.getHint()
        XCTAssertNotNil(game.currentHint)
        XCTAssertEqual(game.hintDetailLevel, .summary)
        XCTAssertEqual(game.hintCellRoles.count, 81)
        let hint = try XCTUnwrap(game.currentHint)
        XCTAssertNotEqual(game.hintCellRole(row: hint.row, col: hint.col), .none)
        game.getHint()
        XCTAssertEqual(game.hintDetailLevel, .proofDetail)
        let filled = game.cells.flatMap { $0 }.filter { !$0.isEmpty }.count
        game.applyHint()
        XCTAssertEqual(game.cells.flatMap { $0 }.filter { !$0.isEmpty }.count, filled + 1)
        XCTAssertEqual(game.mistakes, 0)
        XCTAssertNil(game.currentHint)
        XCTAssertEqual(game.hintDetailLevel, .none)
        XCTAssertTrue(game.hintCellRoles.allSatisfy { $0 == .none })
        XCTAssertGreaterThan(game.hintsUsed, 0)
    }

    func testImportFiltersInvalidCoordinatesAndRestoresPlayerState() async throws {
        let game = try fixture()
        game.applyImportedMove(row: -1, col: 2, value: 4)
        game.applyImportedMove(row: 0, col: 2, value: 1000)
        game.applyImportedMove(row: 0, col: 2, value: 4)
        game.applyImportedMove(row: 0, col: 0, value: 9)
        XCTAssertEqual(game.cells[0][2].value, 4)
        XCTAssertEqual(game.cells[0][0].value, 5)
        game.applyImportedNotes(row: -1, col: 0, notes: [1])
        game.applyImportedNotes(row: 0, col: 0, notes: [1])
        game.applyImportedNotes(row: 0, col: 3, notes: [0, 6, 9, 10])
        XCTAssertEqual(game.cells[0][3].candidates, [6, 9])
        game.checkNotes()
        XCTAssertEqual(game.cells[0][3].candidates, [6])
    }

    func testRowColumnBoxCelebrationsAndDebugHelpers() async throws {
        let row = try fixture()
        XCTAssertNotNil(row.findEmptyCellInRow(0))
        row.fillRowExcept(row: 0, exceptCol: 2)
        row.selectCell(row: 0, col: 2)
        row.enterNumber(4)
        XCTAssertEqual(row.lastCelebration, .rowComplete(row: 0))
        XCTAssertNil(row.findEmptyCellInRow(0))
        let column = try fixture()
        XCTAssertNotNil(column.findEmptyCellInColumn(2))
        column.fillColumnExcept(col: 2, exceptRow: 0)
        column.selectCell(row: 0, col: 2)
        column.enterNumber(4)
        XCTAssertEqual(column.lastCelebration, .columnComplete(col: 2))
        XCTAssertNil(column.findEmptyCellInColumn(2))
        let box = try fixture()
        XCTAssertNotNil(box.findEmptyCellInBox(0))
        box.fillBoxExcept(boxIndex: 0, exceptRow: 0, exceptCol: 2)
        box.selectCell(row: 0, col: 2)
        box.enterNumber(4)
        XCTAssertEqual(box.lastCelebration, .boxComplete(boxIndex: 0))
        XCTAssertNil(box.findEmptyCellInBox(0))
        box.triggerRowCelebration(8)
        XCTAssertTrue(box.celebratingCells.contains("8-8"))
        box.triggerColumnCelebration(8)
        XCTAssertTrue(box.celebratingCells.contains("0-8"))
        box.triggerBoxCelebration(0)
        XCTAssertTrue(box.celebratingCells.contains("0-0"))
        try await Task.sleep(nanoseconds: 700_000_000)
        XCTAssertTrue(box.celebratingCells.isEmpty)
    }

    func testSequentialRowCompletionAscendingAndDescending() async throws {
        for ascending in [true, false] {
            var digits = Array(Self.solution)
            for col in 0..<9 { digits[col] = "0" }
            digits[80] = "0"
            let game = try fixture(String(digits))
            for value in ascending ? Array(1...9) : Array((1...9).reversed()) {
                let col = try XCTUnwrap(Array(Self.solution.prefix(9)).firstIndex(of: Character(String(value))))
                game.selectCell(row: 0, col: col)
                game.enterNumber(value)
            }
            XCTAssertEqual(game.lastCelebration, .rowComplete(row: 0, isSequential: true))
        }
    }

    func testAsyncGenerationProducesPlayableGame() async {
        let game = await GameViewModel.createAsync(difficulty: .beginner)
        XCTAssertEqual(game.difficulty, .beginner)
        XCTAssertFalse(game.isComplete)
        XCTAssertEqual(game.cells.count, 9)
    }
    func testHintRoleBridgeMatchesEngineWireContractAndUnknownValuesAreNeutral() async {
        // Values documented by HintCellRole::to_u8 in the FFI engine.
        let expected: [HintCellRole] = [.none, .target, .involved, .chainOn, .chainOff,
                                        .fishBase, .fishCover, .fishFin, .urFloor, .urRoof, .alsGroup]
        for (wireValue, role) in expected.enumerated() {
            XCTAssertEqual(GameViewModel.hintCellRole(from: UInt8(wireValue)), role)
        }
        XCTAssertEqual(GameViewModel.hintCellRole(from: 255), .none)
    }

    func testLegacyCounterAndUnknownDifficultyFallbacksRemainPlayable() async throws {
        let game = try fixture()
        game.selectCell(row: 0, col: 2)
        game.enterNumber(5)
        var dictionary = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(game.serialize().utf8)) as? [String: Any])
        dictionary.removeValue(forKey: "swiftMistakes")
        dictionary["swiftDifficulty"] = "Future difficulty"
        var restored = try XCTUnwrap(GameViewModel.deserialize(String(decoding: try JSONSerialization.data(withJSONObject: dictionary), as: UTF8.self)))
        XCTAssertEqual(restored.difficulty, .medium)
        XCTAssertEqual(restored.mistakes, 1)
        dictionary.removeValue(forKey: "mistakes")
        restored = try XCTUnwrap(GameViewModel.deserialize(String(decoding: try JSONSerialization.data(withJSONObject: dictionary), as: UTF8.self)))
        XCTAssertEqual(restored.mistakes, 1)
        XCTAssertFalse(restored.cells[0][2].isGiven)
    }

    func testAdvancedHintRetainsEliminationsAndProofInformation() async throws {
        // Reference puzzle from sudoku-core's advanced technique profile regression.
        let game = try fixture("030008002000190040108040000809060003400000790010920800061030000000000035000006000")
        game.fillAllCandidates()
        var elimination: HintModel?
        for _ in 0..<81 {
            game.getHint()
            guard let hint = game.currentHint else { break }
            if !hint.eliminate.isEmpty {
                elimination = hint
                game.getHint()
                XCTAssertEqual(game.hintDetailLevel, .proofDetail)
                break
            }
            game.applyHint()
        }
        let hint = try XCTUnwrap(elimination)
        XCTAssertNil(hint.value)
        XCTAssertTrue(hint.eliminate.allSatisfy { (1...9).contains($0) })
        XCTAssertFalse(hint.explanation.isEmpty)
        XCTAssertFalse(hint.involvedCells.isEmpty)
        XCTAssertGreaterThan(hint.seRating, 2)
    }

}
