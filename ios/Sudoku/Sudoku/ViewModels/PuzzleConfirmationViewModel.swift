import SwiftUI

/// Import mode for camera-imported puzzles
enum ImportMode: String, CaseIterable {
    case continuePuzzle = "Continue Puzzle"
    case startFresh = "Start Fresh"
}

/// State management for the puzzle confirmation / editing screen.
@MainActor
final class PuzzleConfirmationViewModel: ObservableObject {
    /// The 9x9 grid of digit values (0 = empty, 1-9 = given)
    @Published var digits: [Int]

    /// Per-cell OCR confidence (0.0-1.0). Cells edited by the user get 1.0.
    @Published var confidences: [Float]

    /// Per-cell classification from color analysis
    @Published var classifications: [CellClassification]

    /// Per-cell detected pencil marks
    @Published var notes: [Set<Int>]

    /// Whether any player progress was detected in the photo
    @Published var hasPlayerProgress: Bool = false

    /// Current import mode selection
    @Published var importMode: ImportMode = .continuePuzzle

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

    private let recognize: (UIImage) async throws -> OCRResult
    private let validateString: (String) -> PuzzleValidation

    init(recognize: @escaping (UIImage) async throws -> OCRResult = {
        try await PuzzleOCRService().recognizePuzzle(from: $0)
    }, validateString: @escaping (String) -> PuzzleValidation = {
        validatePuzzleString(puzzle: $0)
    }) {
        self.recognize = recognize
        self.validateString = validateString
        self.digits = Array(repeating: 0, count: 81)
        self.confidences = Array(repeating: 1.0, count: 81)
        self.classifications = Array(repeating: .empty, count: 81)
        self.notes = Array(repeating: [], count: 81)
    }

    /// Run OCR on the captured image, then auto-validate if enough digits found
    @discardableResult
    func processImage(_ image: UIImage) -> Task<Void, Never> {
        isProcessing = true
        errorMessage = nil
        validationResult = .notValidated

        return Task {
            do {
                let result = try await recognize(image)
                digits = result.cells.map { $0.digit }
                confidences = result.cells.map { $0.confidence }
                classifications = result.cells.map { $0.classification }
                notes = result.cells.map { $0.notes }
                hasPlayerProgress = result.hasPlayerProgress
                importMode = result.hasPlayerProgress ? .continuePuzzle : .startFresh
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
        // User edits are treated as givens
        classifications[index] = digit == 0 ? .empty : .given
        notes[index] = []
        validationResult = .notValidated  // Invalidate previous validation
    }

    /// Select a cell
    func selectCell(at index: Int) {
        guard (0..<81).contains(index) else { return }
        selectedCell = (selectedCell == index) ? nil : index
    }

    /// 81-char string with only given/ambiguous digits (for validation and "Start Fresh")
    var givensOnlyString: String {
        (0..<81).map { i -> String in
            let digit = digits[i]
            let cls = classifications[i]
            if digit != 0 && (cls == .given || cls == .ambiguous) {
                return "\(digit)"
            }
            return "0"
        }.joined()
    }

    /// Full puzzle string including all recognized digits
    var puzzleString: String {
        digits.map { "\($0)" }.joined()
    }

    /// Number of given (non-empty) cells
    var givenCount: Int {
        digits.filter { $0 != 0 }.count
    }

    /// Number of cells classified as given
    var strictGivenCount: Int {
        (0..<81).filter { digits[$0] != 0 && (classifications[$0] == .given || classifications[$0] == .ambiguous) }.count
    }

    /// Whether the puzzle has a reasonable number of givens (17-40)
    var hasReasonableGivens: Bool {
        strictGivenCount >= 17 && strictGivenCount <= 40
    }

    /// Player moves to replay (cells classified as playerFilled with a digit)
    var playerMoves: [(index: Int, digit: Int)] {
        (0..<81).compactMap { i in
            if digits[i] != 0 && classifications[i] == .playerFilled {
                return (index: i, digit: digits[i])
            }
            return nil
        }
    }

    /// Player notes to restore
    var playerNotes: [(index: Int, notes: Set<Int>)] {
        (0..<81).compactMap { i in
            if !notes[i].isEmpty {
                return (index: i, notes: notes[i])
            }
            return nil
        }
    }

    /// Build the ImportedPuzzleData based on current state
    func buildImportData() -> ImportedPuzzleData {
        ImportedPuzzleData(
            givensString: givensOnlyString,
            playerMoves: importMode == .continuePuzzle ? playerMoves : [],
            playerNotes: importMode == .continuePuzzle ? playerNotes : [],
            isContinuing: importMode == .continuePuzzle
        )
    }

    /// Validate the puzzle via FFI — always validates against givens-only string
    func validate() {
        let result = validateString(givensOnlyString)
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
