import XCTest
import SwiftUI
import ViewInspector
@testable import Sudoku

@MainActor
final class ImportAndHistoryViewTests: XCTestCase {
    func testHistoryEmptyFilteredAndReplayStates() throws {
        let history = GameHistoryManager(defaults: UserDefaults(suiteName: "HistoryView.\(UUID().uuidString)")!)
        let manager = ViewTestFixture.manager()
        let empty = GameHistoryView(historyManager: history).environmentObject(manager)
        XCTAssertNoThrow(try empty.inspect().find(text: "No puzzles yet"))
        let record = history.recordPuzzleStart(puzzleString: ViewTestFixture.puzzle, difficulty: .medium)
        history.recordResult(puzzleHash: record.puzzleHash, won: true, time: 125)
        let matching = GameHistoryView(historyManager: history, selectedDifficulty: .medium).environmentObject(manager)
        let rows = try matching.inspect().findAll(PuzzleRowView.self)
        XCTAssertEqual(rows.count, 1)
        let row = try rows[0].actualView()
        XCTAssertEqual(row.puzzle.bestTime, 125)
        row.onReplay()
        XCTAssertEqual(manager.gameState, .playing)
        XCTAssertEqual(manager.currentGame?.cells[0][0].value, 5)
        let filtered = GameHistoryView(historyManager: history, selectedDifficulty: .easy).environmentObject(manager)
        XCTAssertNoThrow(try filtered.inspect().find(text: "No puzzles yet"))
        try ViewTestFixture.render(matching)
        let invalid = history.recordPuzzleStart(puzzleString: "invalid", difficulty: .hard)
        let invalidView = GameHistoryView(historyManager: history, selectedDifficulty: .hard).environmentObject(manager)
        let invalidRow = try invalidView.inspect().find(PuzzleRowView.self).actualView()
        XCTAssertEqual(invalidRow.puzzle.id, invalid.id)
        let previous = manager.currentGame
        invalidRow.onReplay()
        XCTAssertTrue(manager.currentGame === previous)
    }

    func testPuzzleRowsShowDifficultySolvedStatusBestTimeAndReplayAction() throws {
        for difficulty in Difficulty.allCases {
            for won in [true, false] {
                var record = PuzzleRecord(puzzleString: ViewTestFixture.puzzle, difficulty: difficulty)
                record.recordResult(won: won, time: won ? 125 : nil)
                var replayed = false
                let view = PuzzleRowView(puzzle: record, onReplay: { replayed = true })
                XCTAssertNoThrow(try view.inspect().find(text: difficulty.displayName))
                if won { XCTAssertNoThrow(try view.inspect().find(text: "Best: 2:05")) }
                else { XCTAssertThrowsError(try view.inspect().find(text: "Best: 2:05")) }
                try view.inspect().find(ViewType.Button.self).tap()
                XCTAssertTrue(replayed)
            }
        }
    }

    func testConfirmationNumberPadRequiresSelectionAndInvalidatesValidation() throws {
        let model = PuzzleConfirmationViewModel()
        let view = ConfirmationNumberPad(viewModel: model)
        XCTAssertTrue(try view.inspect().find(button: "5").isDisabled())
        model.selectCell(at: 2)
        model.validationResult = .valid
        try view.inspect().find(button: "5").tap()
        XCTAssertEqual(model.digits[2], 5)
        XCTAssertEqual(model.validationResult, .notValidated)
        try view.inspect().find(button: "C").tap()
        XCTAssertEqual(model.digits[2], 0)
    }

    func testConfirmationGridDisplaysClassificationsSelectionAndDetectedNotes() throws {
        let model = PuzzleConfirmationViewModel()
        model.digits[0] = 5; model.classifications[0] = .given
        model.digits[1] = 3; model.classifications[1] = .playerFilled
        model.digits[2] = 4; model.classifications[2] = .ambiguous
        model.digits[3] = 6; model.classifications[3] = .empty
        model.confidences[2] = 0.2
        model.notes[4] = [1, 7]
        model.selectedCell = 0
        let view = ConfirmationGridView(viewModel: model)
        try ViewTestFixture.render(view, size: CGSize(width: 360, height: 360))
        let texts = try view.inspect().findAll(ViewType.Text.self).map { try $0.string() }
        XCTAssertTrue(texts.contains("5"))
        XCTAssertTrue(texts.contains("3"))
        XCTAssertTrue(texts.contains("1"))
        model.importMode = .startFresh
        XCTAssertFalse(try view.inspect().findAll(ViewType.Text.self).map { try $0.string() }.contains("1"))
        model.selectedCell = nil
        try ViewTestFixture.render(view, size: CGSize(width: 360, height: 360))
    }

    func testConfirmationShowsProgressErrorAndEveryValidationStatus() throws {
        let model = PuzzleConfirmationViewModel()
        let view = PuzzleConfirmationView(image: UIImage(), viewModel: model, onPlay: { _ in })
        model.isProcessing = true
        XCTAssertNoThrow(try view.inspect().find(ViewType.ProgressView.self))
        model.isProcessing = false
        model.errorMessage = "Image has no grid"
        XCTAssertNoThrow(try view.inspect().find(text: "Image has no grid"))
        XCTAssertNoThrow(try view.inspect().find(button: "Try Again"))
        model.errorMessage = nil
        XCTAssertTrue(try view.inspect().find(button: "Play").isDisabled())
        model.validationResult = .invalid(reason: "Missing clues")
        XCTAssertNoThrow(try view.inspect().find(text: "Missing clues"))
        model.validationResult = .valid
        XCTAssertNoThrow(try view.inspect().find(text: "Valid puzzle with a unique solution"))
        XCTAssertFalse(try view.inspect().find(button: "Play").isDisabled())
        model.hasPlayerProgress = true
        model.importMode = .continuePuzzle
        XCTAssertNoThrow(try view.inspect().find(button: "Continue Puzzle"))
        model.digits[0] = 5
        model.confidences[0] = 0.2
        XCTAssertNoThrow(try view.inspect().find(text: "Uncertain cells highlighted"))
    }

    func testConfirmedImportCallsPlayWithReviewedValuesAndNotes() throws {
        let model = PuzzleConfirmationViewModel()
        for (index, value) in ViewTestFixture.puzzle.enumerated() { model.setDigit(value.wholeNumberValue!, at: index) }
        model.hasPlayerProgress = true
        model.importMode = .continuePuzzle
        model.notes[2] = [2, 4]
        var imported: ImportedPuzzleData?
        let view = PuzzleConfirmationView(image: UIImage(), viewModel: model, onPlay: { imported = $0 })
        try view.inspect().find(button: "Validate").tap()
        XCTAssertTrue(model.canPlay)
        try view.inspect().find(button: "Continue Puzzle").tap()
        XCTAssertEqual(imported?.givensString, ViewTestFixture.puzzle)
        XCTAssertEqual(imported?.playerNotes.first?.notes, [2, 4])
    }

    func testCameraPhotoPickerOnlyReportsValidImagesAndCancelDoesNotCapture() throws {
        var received: [UIImage] = []
        let view = CameraCaptureView(sourceType: .photoLibrary, onImageCaptured: { received.append($0) })
        let coordinator = view.makeCoordinator()
        let picker = UIImagePickerController()
        let image = UIImage()
        coordinator.imagePickerController(picker, didFinishPickingMediaWithInfo: [.originalImage: image])
        XCTAssertEqual(received.count, 1)
        XCTAssertTrue(received[0] === image)
        coordinator.imagePickerController(picker, didFinishPickingMediaWithInfo: [:])
        coordinator.imagePickerControllerDidCancel(picker)
        XCTAssertEqual(received.count, 1)
        ViewHosting.host(view: view)
        defer { ViewHosting.expel() }
        let controller = try view.viewController()
        XCTAssertEqual(controller.sourceType, .photoLibrary)
        XCTAssertFalse(controller.allowsEditing)
    }
}
