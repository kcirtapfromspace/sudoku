import UIKit
import Vision
import CoreImage
import CoreImage.CIFilterBuiltins

/// Per-cell OCR result with confidence score
struct CellOCRResult {
    let digit: Int  // 0 = empty, 1-9 = recognized digit
    let confidence: Float  // 0.0 - 1.0
}

/// Result from the full OCR pipeline
struct OCRResult {
    let cells: [CellOCRResult]  // 81 cells, row-major order
    let puzzleString: String    // 81-char string (digits 1-9, 0 for empty)
}

/// On-device OCR service for recognizing Sudoku puzzles from photos.
/// Uses Apple Vision framework for grid detection and text recognition.
final class PuzzleOCRService {

    /// Minimum confidence threshold to accept a digit recognition
    private static let confidenceThreshold: Float = 0.5
    private let ciContext = CIContext()

    /// Process a photo of a Sudoku puzzle and extract the digit grid.
    func recognizePuzzle(from image: UIImage) async throws -> OCRResult {
        guard let ciImage = CIImage(image: image) else {
            throw OCRError.invalidImage
        }

        // Step 1: Detect the grid rectangle
        let gridRect = try await detectGrid(in: ciImage)

        // Step 2: Perspective-correct to a square
        let corrected = try perspectiveCorrect(ciImage, to: gridRect)

        // Step 3: Extract 81 cell images
        let cellImages = extractCells(from: corrected)

        // Step 4: Recognize digit in each cell
        let cells = try await recognizeDigits(in: cellImages)

        // Step 5: Assemble puzzle string
        let puzzleString = cells.map { $0.digit == 0 ? "0" : "\($0.digit)" }.joined()

        return OCRResult(cells: cells, puzzleString: puzzleString)
    }

    // MARK: - Step 1: Grid Detection

    private func detectGrid(in image: CIImage) async throws -> VNRectangleObservation {
        let request = VNDetectRectanglesRequest()
        request.minimumAspectRatio = 0.7
        request.maximumAspectRatio = 1.3
        request.minimumSize = 0.2
        request.maximumObservations = 10
        request.minimumConfidence = 0.4

        let handler = VNImageRequestHandler(ciImage: image, options: [:])
        try handler.perform([request])

        guard let results = request.results, !results.isEmpty else {
            throw OCRError.noGridFound
        }

        // Score each candidate by internal grid structure, pick the best verified one
        let minimumGridScore: Float = 0.375
        var bestRect: VNRectangleObservation?
        var bestScore: Float = 0

        for rect in results {
            let score = Self.gridStructureScore(image: image, rect: rect, context: ciContext)
            if score > bestScore {
                bestScore = score
                bestRect = rect
            }
        }

        if let verified = bestRect, bestScore >= minimumGridScore {
            return verified
        }

        // Fallback: largest rectangle (user took photo deliberately)
        return results.max(by: { area(of: $0) < area(of: $1) })!
    }

    private func area(of rect: VNRectangleObservation) -> CGFloat {
        let w = hypot(rect.topRight.x - rect.topLeft.x, rect.topRight.y - rect.topLeft.y)
        let h = hypot(rect.bottomLeft.x - rect.topLeft.x, rect.bottomLeft.y - rect.topLeft.y)
        return w * h
    }

    /// Check whether a detected rectangle contains internal grid structure consistent
    /// with a Sudoku puzzle. Perspective-corrects the region to a small bitmap, then
    /// samples brightness at expected line positions (1/9 … 8/9) and compares with
    /// neighboring cell interiors. Returns fraction of the 16 expected lines detected.
    static func gridStructureScore(image: CIImage, rect: VNRectangleObservation, context: CIContext) -> Float {
        let imageSize = image.extent.size

        let filter = CIFilter.perspectiveCorrection()
        filter.inputImage = image
        filter.topLeft = CGPoint(x: rect.topLeft.x * imageSize.width, y: rect.topLeft.y * imageSize.height)
        filter.topRight = CGPoint(x: rect.topRight.x * imageSize.width, y: rect.topRight.y * imageSize.height)
        filter.bottomLeft = CGPoint(x: rect.bottomLeft.x * imageSize.width, y: rect.bottomLeft.y * imageSize.height)
        filter.bottomRight = CGPoint(x: rect.bottomRight.x * imageSize.width, y: rect.bottomRight.y * imageSize.height)

        guard let corrected = filter.outputImage else { return 0 }

        // Render to 180x180 bitmap for fast pixel sampling (20px per cell)
        let sz = 180
        let ext = corrected.extent
        guard ext.width > 0, ext.height > 0 else { return 0 }
        let translated = corrected.transformed(by: CGAffineTransform(translationX: -ext.origin.x, y: -ext.origin.y))
        let scaled = translated.transformed(by: CGAffineTransform(scaleX: CGFloat(sz) / ext.width, scaleY: CGFloat(sz) / ext.height))

        guard let cgImage = context.createCGImage(scaled, from: CGRect(x: 0, y: 0, width: sz, height: sz)),
              let dp = cgImage.dataProvider,
              let data = dp.data else { return 0 }

        let ptr = CFDataGetBytePtr(data)!
        let bpp = cgImage.bitsPerPixel / 8
        let bpr = cgImage.bytesPerRow

        func gray(_ x: Int, _ y: Int) -> Float {
            let cx = min(max(x, 0), sz - 1)
            let cy = min(max(y, 0), sz - 1)
            let off = cy * bpr + cx * bpp
            return (Float(ptr[off]) + Float(ptr[off + 1]) + Float(ptr[off + 2])) / (3.0 * 255.0)
        }

        let cell = sz / 9  // 20px per cell
        let half = cell / 2 // 10px — center of cell
        var found = 0

        for i in 1...8 {
            let pos = i * sz / 9

            // Vertical line: sample 9 points, compare with left/right cell centers
            var vHits = 0
            for s in 0..<9 {
                let sy = s * cell + half
                let lineVal = gray(pos, sy)
                let leftVal = gray(pos - half, sy)
                let rightVal = gray(pos + half, sy)
                if lineVal < (leftVal + rightVal) / 2.0 - 0.08 { vHits += 1 }
            }
            if vHits >= 5 { found += 1 }

            // Horizontal line: sample 9 points, compare with above/below cell centers
            var hHits = 0
            for s in 0..<9 {
                let sx = s * cell + half
                let lineVal = gray(sx, pos)
                let aboveVal = gray(sx, pos - half)
                let belowVal = gray(sx, pos + half)
                if lineVal < (aboveVal + belowVal) / 2.0 - 0.08 { hHits += 1 }
            }
            if hHits >= 5 { found += 1 }
        }

        return Float(found) / 16.0
    }

    // MARK: - Step 2: Perspective Correction

    private func perspectiveCorrect(_ image: CIImage, to rect: VNRectangleObservation) throws -> CIImage {
        let imageSize = image.extent.size

        // Convert normalized Vision coordinates to image coordinates
        func toImagePoint(_ p: CGPoint) -> CGPoint {
            CGPoint(x: p.x * imageSize.width, y: p.y * imageSize.height)
        }

        let filter = CIFilter.perspectiveCorrection()
        filter.inputImage = image
        filter.topLeft = toImagePoint(rect.topLeft)
        filter.topRight = toImagePoint(rect.topRight)
        filter.bottomLeft = toImagePoint(rect.bottomLeft)
        filter.bottomRight = toImagePoint(rect.bottomRight)

        guard let output = filter.outputImage else {
            throw OCRError.perspectiveCorrectionFailed
        }

        return output
    }

    // MARK: - Step 3: Cell Extraction

    private func extractCells(from image: CIImage) -> [CIImage] {
        let size = image.extent.size
        let cellW = size.width / 9.0
        let cellH = size.height / 9.0

        // Inset cells slightly to avoid grid lines
        let insetFraction: CGFloat = 0.12

        var cells: [CIImage] = []
        cells.reserveCapacity(81)

        for row in 0..<9 {
            for col in 0..<9 {
                // CIImage origin is bottom-left; row 0 is the top of the puzzle
                let flippedRow = 8 - row
                let x = CGFloat(col) * cellW
                let y = CGFloat(flippedRow) * cellH

                let insetX = cellW * insetFraction
                let insetY = cellH * insetFraction
                let cellRect = CGRect(
                    x: x + insetX,
                    y: y + insetY,
                    width: cellW - 2 * insetX,
                    height: cellH - 2 * insetY
                )

                let cropped = image.cropped(to: cellRect)
                cells.append(cropped)
            }
        }

        return cells
    }

    // MARK: - Step 4: Digit Recognition

    private func recognizeDigits(in cellImages: [CIImage]) async throws -> [CellOCRResult] {
        var results: [CellOCRResult] = []
        results.reserveCapacity(81)

        for cellImage in cellImages {
            let result = try await recognizeSingleDigit(in: cellImage, level: .fast)

            // If fast recognition has low confidence, retry with accurate
            if result.confidence < Self.confidenceThreshold && result.digit != 0 {
                let accurate = try await recognizeSingleDigit(in: cellImage, level: .accurate)
                results.append(accurate)
            } else {
                results.append(result)
            }
        }

        return results
    }

    private func recognizeSingleDigit(in cellImage: CIImage, level: VNRequestTextRecognitionLevel) async throws -> CellOCRResult {
        let request = VNRecognizeTextRequest()
        request.recognitionLevel = level
        request.usesLanguageCorrection = false
        // Only recognize single digits
        request.customWords = ["1", "2", "3", "4", "5", "6", "7", "8", "9"]

        let handler = VNImageRequestHandler(ciImage: cellImage, options: [:])
        try handler.perform([request])

        guard let results = request.results, !results.isEmpty else {
            return CellOCRResult(digit: 0, confidence: 1.0) // Empty cell (high confidence)
        }

        // Get the top candidate
        let top = results[0]
        guard let candidate = top.topCandidates(1).first else {
            return CellOCRResult(digit: 0, confidence: 1.0)
        }

        let text = candidate.string.trimmingCharacters(in: .whitespacesAndNewlines)

        // Must be a single digit 1-9
        if text.count == 1, let digit = Int(text), (1...9).contains(digit) {
            return CellOCRResult(digit: digit, confidence: candidate.confidence)
        }

        // Not a valid digit — treat as empty
        return CellOCRResult(digit: 0, confidence: 0.8)
    }
}

// MARK: - Errors

enum OCRError: LocalizedError {
    case invalidImage
    case noGridFound
    case perspectiveCorrectionFailed

    var errorDescription: String? {
        switch self {
        case .invalidImage:
            return "Could not process the image."
        case .noGridFound:
            return "No Sudoku grid found in the image. Make sure the entire grid is visible."
        case .perspectiveCorrectionFailed:
            return "Could not straighten the grid. Try taking the photo from directly above."
        }
    }
}
