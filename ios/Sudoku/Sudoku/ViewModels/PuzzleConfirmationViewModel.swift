import SwiftUI

/// State management for the puzzle confirmation / editing screen.
@MainActor
final class PuzzleConfirmationViewModel: ObservableObject {
    /// The 9x9 grid of digit values (0 = empty, 1-9 = given)
    @Published var digits: [Int]

    /// Per-cell OCR confidence (0.0-1.0). Cells edited by the user get 1.0.
    @Published var confidences: [Float]

    /// Currently selected cell index (0-80), or nil
    @Published var selectedCell: Int?

    /// Validation state
    @Published var validationResult: ValidationState = .notValidated

    /// Whether OCR is in progress
    @Published var isProcessing = false

    /// Error message from OCR or validation
    @Published var errorMessage: String?

    /// Confidence threshold below which cells are highlighted
    static let lowConfidenceThreshold: Float = 0.7

    private let ocrService = PuzzleOCRService()

    init() {
        self.digits = Array(repeating: 0, count: 81)
        self.confidences = Array(repeating: 1.0, count: 81)
    }

    /// Run OCR on the captured image, then auto-validate if enough digits found
    func processImage(_ image: UIImage) {
        isProcessing = true
        errorMessage = nil
        validationResult = .notValidated

        Task {
            do {
                let result = try await ocrService.recognizePuzzle(from: image)
                digits = result.cells.map { $0.digit }
                confidences = result.cells.map { $0.confidence }
                isProcessing = false

                // If too few digits found, show a helpful error instead of an empty grid
                if givenCount < 10 {
                    errorMessage = "Only \(givenCount) digits recognized. Make sure the entire Sudoku grid is clearly visible."
                    return
                }

                // Auto-validate so the user doesn't have to tap an extra button
                validate()
            } catch {
                errorMessage = error.localizedDescription
                isProcessing = false
            }
        }
    }

    /// Set a digit in a cell (user editing)
    func setDigit(_ digit: Int, at index: Int) {
        guard (0...9).contains(digit), (0..<81).contains(index) else { return }
        digits[index] = digit
        confidences[index] = 1.0  // User-edited cells are fully confident
        validationResult = .notValidated  // Invalidate previous validation
    }

    /// Select a cell
    func selectCell(at index: Int) {
        selectedCell = (selectedCell == index) ? nil : index
    }

    /// Get the 81-character puzzle string
    var puzzleString: String {
        digits.map { "\($0)" }.joined()
    }

    /// Number of given (non-empty) cells
    var givenCount: Int {
        digits.filter { $0 != 0 }.count
    }

    /// Whether the puzzle has a reasonable number of givens (17-40)
    var hasReasonableGivens: Bool {
        givenCount >= 17 && givenCount <= 40
    }

    /// Validate the puzzle via FFI
    func validate() {
        let result = validatePuzzleString(puzzle: puzzleString)
        switch result {
        case .valid:
            validationResult = .valid
        case .noSolution:
            validationResult = .invalid(reason: "This puzzle has no solution. Check the digits for errors.")
        case .multipleSolutions:
            validationResult = .invalid(reason: "This puzzle has multiple solutions. It may be missing some digits.")
        case .invalidFormat(let reason):
            validationResult = .invalid(reason: reason)
        }
    }

    /// Whether the "Play" button should be enabled
    var canPlay: Bool {
        if case .valid = validationResult { return true }
        return false
    }

    /// Check if a cell has low confidence
    func isLowConfidence(at index: Int) -> Bool {
        confidences[index] < Self.lowConfidenceThreshold && digits[index] != 0
    }
}

// MARK: - Validation State

enum ValidationState: Equatable {
    case notValidated
    case valid
    case invalid(reason: String)
}
