import XCTest
import UIKit
@testable import Sudoku

@MainActor
final class PuzzleConfirmationTests: XCTestCase {
    func testEditingSelectionAndConfidenceBoundaries() {
        let model = PuzzleConfirmationViewModel()
        XCTAssertEqual(model.digits.count, 81)
        XCTAssertEqual(model.puzzleString, String(repeating: "0", count: 81))
        XCTAssertFalse(model.canPlay)
        model.selectCell(at: 0); XCTAssertEqual(model.selectedCell, 0)
        model.selectCell(at: 0); XCTAssertNil(model.selectedCell)
        model.selectCell(at: -1); model.selectCell(at: 81); XCTAssertNil(model.selectedCell)
        model.setDigit(10, at: 0); model.setDigit(3, at: 81); model.setDigit(-1, at: 0)
        XCTAssertEqual(model.givenCount, 0)
        model.digits[0] = 5; model.confidences[0] = 0.69; model.notes[0] = [1, 2]
        XCTAssertTrue(model.isLowConfidence(at: 0))
        model.confidences[0] = 0.7; XCTAssertFalse(model.isLowConfidence(at: 0))
        model.validationResult = .valid
        model.setDigit(3, at: 0)
        XCTAssertEqual(model.digits[0], 3); XCTAssertEqual(model.confidences[0], 1)
        XCTAssertEqual(model.classifications[0], .given); XCTAssertTrue(model.notes[0].isEmpty)
        XCTAssertEqual(model.validationResult, .notValidated)
        model.setDigit(0, at: 0); XCTAssertEqual(model.classifications[0], .empty)
        model.confidences[0] = 0.2; XCTAssertFalse(model.isLowConfidence(at: 0))
        XCTAssertEqual(ImportMode.allCases.count, 2)
    }
    func testImportModesPreservePlayerMovesAndNotes() {
        let model = PuzzleConfirmationViewModel()
        for i in 0..<17 { model.setDigit((i % 9) + 1, at: i) }
        XCTAssertTrue(model.hasReasonableGivens)
        model.classifications[0] = .ambiguous
        model.digits[20] = 4; model.classifications[20] = .playerFilled
        model.notes[21] = [2, 7]
        XCTAssertEqual(model.strictGivenCount, 17); XCTAssertEqual(model.givenCount, 18)
        XCTAssertEqual(Array(model.givensOnlyString)[20], "0")
        XCTAssertEqual(model.playerMoves.first?.index, 20); XCTAssertEqual(model.playerMoves.first?.digit, 4)
        XCTAssertEqual(model.playerNotes.first?.notes, [2, 7])
        let continued = model.buildImportData()
        XCTAssertTrue(continued.isContinuing); XCTAssertEqual(continued.playerMoves.count, 1); XCTAssertEqual(continued.playerNotes.count, 1)
        model.importMode = .startFresh
        let fresh = model.buildImportData()
        XCTAssertFalse(fresh.isContinuing); XCTAssertTrue(fresh.playerMoves.isEmpty); XCTAssertTrue(fresh.playerNotes.isEmpty)
        for i in 17..<41 { model.setDigit(1, at: i) }
        XCTAssertFalse(model.hasReasonableGivens)
        model.setDigit(0, at: 40); XCTAssertTrue(model.hasReasonableGivens)
    }
    func testEveryValidationOutcomeAndRealEngine() {
        for result: PuzzleValidation in [.valid, .noSolution, .multipleSolutions, .invalidFormat(reason: "bad grid")] {
            let model = PuzzleConfirmationViewModel(validateString: { puzzle in
                XCTAssertEqual(puzzle.count, 81); return result
            })
            model.validate()
            switch result {
            case .valid: XCTAssertTrue(model.canPlay)
            case .noSolution: XCTAssertEqual(model.validationResult, .invalid(reason: "This puzzle has no solution. Check the digits for errors."))
            case .multipleSolutions: XCTAssertEqual(model.validationResult, .invalid(reason: "This puzzle has multiple solutions. It may be missing some digits."))
            case .invalidFormat: XCTAssertEqual(model.validationResult, .invalid(reason: "bad grid"))
            }
        }
        let real = PuzzleConfirmationViewModel()
        for (i, c) in ServiceFixtures.puzzle.enumerated() { real.setDigit(c.wholeNumberValue!, at: i) }
        real.validate(); XCTAssertTrue(real.canPlay)
    }
    func testOCRSuccessProgressTooFewDigitsAndError() async {
        for hasProgress in [false, true] {
            let model = PuzzleConfirmationViewModel(recognize: { _ in
                let cells = ServiceFixtures.puzzle.map { digit in
                    CellOCRResult(digit: digit.wholeNumberValue!, confidence: 0.9, classification: digit == "0" ? .empty : .given, notes: [])
                }
                return OCRResult(cells: cells, puzzleString: ServiceFixtures.puzzle, hasPlayerProgress: hasProgress)
            })
            let task = model.processImage(ServiceFixtures.image())
            XCTAssertTrue(model.isProcessing)
            await task.value
            XCTAssertFalse(model.isProcessing); XCTAssertTrue(model.canPlay)
            XCTAssertEqual(model.hasPlayerProgress, hasProgress)
            XCTAssertEqual(model.importMode, hasProgress ? .continuePuzzle : .startFresh)
            XCTAssertNil(model.errorMessage)
        }
        let sparse = PuzzleConfirmationViewModel(recognize: { _ in
            OCRResult(cells: Array(repeating: CellOCRResult(digit: 0, confidence: 1, classification: .empty, notes: []), count: 81), puzzleString: String(repeating: "0", count: 81), hasPlayerProgress: false)
        })
        await sparse.processImage(ServiceFixtures.image()).value
        XCTAssertTrue(sparse.errorMessage?.contains("Only 0 digits") == true); XCTAssertFalse(sparse.canPlay)
        let failed = PuzzleConfirmationViewModel(recognize: { _ in throw OCRError.noGridFound })
        await failed.processImage(UIImage()).value
        XCTAssertEqual(failed.errorMessage, OCRError.noGridFound.localizedDescription); XCTAssertFalse(failed.isProcessing)
        let defaultOCR = PuzzleConfirmationViewModel()
        await defaultOCR.processImage(UIImage()).value
        XCTAssertEqual(defaultOCR.errorMessage, OCRError.invalidImage.localizedDescription)
    }
}
