import UIKit
import Vision
import CoreImage
import CoreImage.CIFilterBuiltins

/// Classification of a cell's origin based on text color analysis
enum CellClassification {
    case empty
    case given         // Black/dark text — original puzzle clue
    case playerFilled  // Colored text (e.g. blue) — user-entered digit
    case ambiguous     // Could not determine
}

/// Per-cell OCR result with confidence score and classification
struct CellOCRResult {
    let digit: Int                      // 0 = empty, 1-9 = recognized digit
    let confidence: Float               // 0.0 - 1.0
    let classification: CellClassification
    let notes: Set<Int>                 // Detected pencil marks (empty for non-empty cells)
}

/// Result from the full OCR pipeline
struct OCRResult {
    let cells: [CellOCRResult]          // 81 cells, row-major order
    let puzzleString: String            // Givens-only (for "Start Fresh")
    let hasPlayerProgress: Bool         // Whether any player-filled or notes cells found
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

        // Step 3: Extract cell images (center-cropped for digits, full for notes)
        let centerCells = extractCells(from: corrected, insetFraction: 0.12)
        let fullCells = extractCells(from: corrected, insetFraction: 0.03)

        // Step 4: Recognize digit in each cell + classify color
        let digitResults = try await recognizeDigits(in: centerCells, fullCellImages: fullCells)

        // Step 5: Detect notes in empty cells
        let cells = try await detectNotes(digitResults: digitResults, fullCells: fullCells)

        // Step 6: Assemble results
        let hasProgress = cells.contains { $0.classification == .playerFilled || !$0.notes.isEmpty }

        let puzzleString = cells.map { cell -> String in
            if cell.digit != 0 && (cell.classification == .given || cell.classification == .ambiguous) {
                return "\(cell.digit)"
            }
            return "0"
        }.joined()

        return OCRResult(cells: cells, puzzleString: puzzleString, hasPlayerProgress: hasProgress)
    }

    // MARK: - Step 1: Grid Detection

    private func detectGrid(in image: CIImage) async throws -> VNRectangleObservation {
        // Primary attempt
        if let result = try await detectGridPrimary(in: image) {
            return result
        }

        // Enhanced retry: boost contrast and sharpen for thin pixel-based lines (screen photos)
        let enhanced = enhanceForGridDetection(image)
        if let result = try await detectGridEnhanced(in: enhanced, originalImage: image) {
            return result
        }

        throw OCRError.noGridFound
    }

    private func detectGridPrimary(in image: CIImage) throws -> VNRectangleObservation? {
        let request = VNDetectRectanglesRequest()
        request.minimumAspectRatio = 0.7
        request.maximumAspectRatio = 1.3
        request.minimumSize = 0.15
        request.maximumObservations = 10
        request.minimumConfidence = 0.4

        let handler = VNImageRequestHandler(ciImage: image, options: [:])
        try handler.perform([request])

        guard let results = request.results, !results.isEmpty else {
            return nil
        }

        let minimumGridScore: Float = 0.25
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
        return results.max(by: { area(of: $0) < area(of: $1) })
    }

    /// Pre-process image to enhance thin grid lines for screen photos
    private func enhanceForGridDetection(_ image: CIImage) -> CIImage {
        // Boost contrast
        let contrast = CIFilter.colorControls()
        contrast.inputImage = image
        contrast.contrast = 1.5
        contrast.brightness = 0.0
        contrast.saturation = 0.0

        guard let contrasted = contrast.outputImage else { return image }

        // Sharpen thin lines
        let sharpen = CIFilter.unsharpMask()
        sharpen.inputImage = contrasted
        sharpen.radius = 2.0
        sharpen.intensity = 1.5

        return sharpen.outputImage ?? contrasted
    }

    private func detectGridEnhanced(in enhanced: CIImage, originalImage: CIImage) throws -> VNRectangleObservation? {
        let request = VNDetectRectanglesRequest()
        request.minimumAspectRatio = 0.7
        request.maximumAspectRatio = 1.3
        request.minimumSize = 0.15
        request.maximumObservations = 10
        request.minimumConfidence = 0.3

        let handler = VNImageRequestHandler(ciImage: enhanced, options: [:])
        try handler.perform([request])

        guard let results = request.results, !results.isEmpty else {
            return nil
        }

        // Score against the original image for more accurate structure verification
        let minimumGridScore: Float = 0.25
        var bestRect: VNRectangleObservation?
        var bestScore: Float = 0

        for rect in results {
            let score = Self.gridStructureScore(image: originalImage, rect: rect, context: ciContext)
            if score > bestScore {
                bestScore = score
                bestRect = rect
            }
        }

        if let verified = bestRect, bestScore >= minimumGridScore {
            return verified
        }

        return results.max(by: { area(of: $0) < area(of: $1) })
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
        let scaled = translated.transformed(by: CGAffineTransform(scaleX: CGFloat(sz) / ext.width, y: CGFloat(sz) / ext.height))

        guard let cgImage = context.createCGImage(scaled, from: CGRect(x: 0, y: 0, width: sz, height: sz)),
              let dp = cgImage.dataProvider,
              let data = dp.data,
              let ptr = CFDataGetBytePtr(data) else { return 0 }
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

    private func extractCells(from image: CIImage, insetFraction: CGFloat) -> [CIImage] {
        let size = image.extent.size
        let cellW = size.width / 9.0
        let cellH = size.height / 9.0

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

    // MARK: - Step 4: Digit Recognition + Color Classification

    private func recognizeDigits(in centerCells: [CIImage], fullCellImages: [CIImage]) async throws -> [CellOCRResult] {
        var results: [CellOCRResult] = []
        results.reserveCapacity(81)

        for (index, cellImage) in centerCells.enumerated() {
            var result = try await recognizeSingleDigit(in: cellImage, level: .fast)

            // If fast recognition has low confidence, retry with accurate
            if result.confidence < Self.confidenceThreshold && result.digit != 0 {
                result = try await recognizeSingleDigit(in: cellImage, level: .accurate)
            }

            // Classify color for cells with a recognized digit
            let classification: CellClassification
            if result.digit == 0 {
                classification = .empty
            } else {
                classification = classifyCell(fullCellImages[index])
            }

            results.append(CellOCRResult(
                digit: result.digit,
                confidence: result.confidence,
                classification: classification,
                notes: []
            ))
        }

        return results
    }

    private struct RawDigitResult {
        let digit: Int
        let confidence: Float
    }

    private func recognizeSingleDigit(in cellImage: CIImage, level: VNRequestTextRecognitionLevel) throws -> RawDigitResult {
        let request = VNRecognizeTextRequest()
        request.recognitionLevel = level
        request.usesLanguageCorrection = false
        request.customWords = ["1", "2", "3", "4", "5", "6", "7", "8", "9"]

        let handler = VNImageRequestHandler(ciImage: cellImage, options: [:])
        try handler.perform([request])

        guard let results = request.results, !results.isEmpty else {
            return RawDigitResult(digit: 0, confidence: 1.0)
        }

        let top = results[0]
        guard let candidate = top.topCandidates(1).first else {
            return RawDigitResult(digit: 0, confidence: 1.0)
        }

        let text = candidate.string.trimmingCharacters(in: .whitespacesAndNewlines)

        if text.count == 1, let digit = Int(text), (1...9).contains(digit) {
            return RawDigitResult(digit: digit, confidence: candidate.confidence)
        }

        return RawDigitResult(digit: 0, confidence: 0.8)
    }

    // MARK: - Step 4b: Color Analysis

    /// Classify a cell as given (black text) or player-filled (colored text) by
    /// sampling the average HSB saturation of foreground pixels.
    private func classifyCell(_ cellImage: CIImage) -> CellClassification {
        let sz = 40
        let ext = cellImage.extent
        guard ext.width > 0, ext.height > 0 else { return .ambiguous }

        let translated = cellImage.transformed(by: CGAffineTransform(translationX: -ext.origin.x, y: -ext.origin.y))
        let scaled = translated.transformed(by: CGAffineTransform(scaleX: CGFloat(sz) / ext.width, y: CGFloat(sz) / ext.height))

        guard let cgImage = ciContext.createCGImage(scaled, from: CGRect(x: 0, y: 0, width: sz, height: sz)),
              let dp = cgImage.dataProvider,
              let data = dp.data,
              let ptr = CFDataGetBytePtr(data) else { return .ambiguous }
        let bpp = cgImage.bitsPerPixel / 8
        let bpr = cgImage.bytesPerRow

        // Compute average background brightness from corners
        var bgBrightness: Float = 0
        var bgCount: Float = 0
        let corners = [(0,0), (sz-1,0), (0,sz-1), (sz-1,sz-1)]
        for (cx, cy) in corners {
            let off = cy * bpr + cx * bpp
            let gray = (Float(ptr[off]) + Float(ptr[off+1]) + Float(ptr[off+2])) / (3.0 * 255.0)
            bgBrightness += gray
            bgCount += 1
        }
        bgBrightness /= bgCount

        // Sample foreground pixels (darker than background threshold)
        let fgThreshold = bgBrightness - 0.15
        var totalSaturation: Float = 0
        var fgCount = 0

        for y in 0..<sz {
            for x in 0..<sz {
                let off = y * bpr + x * bpp
                let r = Float(ptr[off]) / 255.0
                let g = Float(ptr[off+1]) / 255.0
                let b = Float(ptr[off+2]) / 255.0
                let gray = (r + g + b) / 3.0

                if gray < fgThreshold {
                    let maxC = max(r, g, b)
                    let minC = min(r, g, b)
                    let saturation = maxC > 0 ? (maxC - minC) / maxC : 0
                    totalSaturation += saturation
                    fgCount += 1
                }
            }
        }

        guard fgCount > 5 else { return .ambiguous }

        let avgSaturation = totalSaturation / Float(fgCount)

        if avgSaturation < 0.15 {
            return .given
        } else if avgSaturation > 0.3 {
            return .playerFilled
        } else {
            return .ambiguous
        }
    }

    // MARK: - Step 5: Notes Detection

    /// For cells where digit == 0, attempt to detect pencil marks by dividing the
    /// cell into a 3x3 sub-grid and running OCR on each sub-region.
    private func detectNotes(digitResults: [CellOCRResult], fullCells: [CIImage]) async throws -> [CellOCRResult] {
        var results = digitResults

        // Collect indices of empty cells that need notes detection
        var emptyIndices: [Int] = []
        for (i, cell) in digitResults.enumerated() {
            if cell.digit == 0 {
                emptyIndices.append(i)
            }
        }

        guard !emptyIndices.isEmpty else { return results }

        // Process empty cells in parallel using TaskGroup
        let notesResults: [(Int, Set<Int>)] = try await withThrowingTaskGroup(of: (Int, Set<Int>).self) { group in
            for idx in emptyIndices {
                let cellImage = fullCells[idx]
                group.addTask {
                    let notes = try self.detectNotesInCell(cellImage)
                    return (idx, notes)
                }
            }

            var collected: [(Int, Set<Int>)] = []
            for try await result in group {
                collected.append(result)
            }
            return collected
        }

        // Merge notes into results
        for (idx, notes) in notesResults {
            if !notes.isEmpty {
                let orig = results[idx]
                results[idx] = CellOCRResult(
                    digit: orig.digit,
                    confidence: orig.confidence,
                    classification: .empty,
                    notes: notes
                )
            }
        }

        return results
    }

    /// Detect pencil marks in a single cell by dividing it into a 3x3 sub-grid.
    /// Each sub-region corresponds to digit (row*3 + col + 1):
    /// [1][2][3]
    /// [4][5][6]
    /// [7][8][9]
    private func detectNotesInCell(_ cellImage: CIImage) throws -> Set<Int> {
        let ext = cellImage.extent
        guard ext.width > 10, ext.height > 10 else { return [] }

        let subW = ext.width / 3.0
        let subH = ext.height / 3.0

        var foundNotes: Set<Int> = []

        for subRow in 0..<3 {
            for subCol in 0..<3 {
                let expectedDigit = subRow * 3 + subCol + 1
                // CIImage is bottom-left origin, so flip subRow
                let flippedSubRow = 2 - subRow
                let subRect = CGRect(
                    x: ext.origin.x + CGFloat(subCol) * subW,
                    y: ext.origin.y + CGFloat(flippedSubRow) * subH,
                    width: subW,
                    height: subH
                )

                let subImage = cellImage.cropped(to: subRect)

                // Skip sub-regions with low pixel variance (obviously empty)
                if !hasSignificantContent(subImage) { continue }

                let request = VNRecognizeTextRequest()
                request.recognitionLevel = .fast
                request.usesLanguageCorrection = false
                request.customWords = ["\(expectedDigit)"]

                let handler = VNImageRequestHandler(ciImage: subImage, options: [:])
                try handler.perform([request])

                if let results = request.results,
                   let top = results.first,
                   let candidate = top.topCandidates(1).first {
                    let text = candidate.string.trimmingCharacters(in: .whitespacesAndNewlines)
                    if text == "\(expectedDigit)" && candidate.confidence > 0.3 {
                        foundNotes.insert(expectedDigit)
                    }
                }
            }
        }

        // Require at least 2 notes to reduce false positives
        return foundNotes.count >= 2 ? foundNotes : []
    }

    /// Quick check if a sub-region has enough pixel variance to contain text.
    private func hasSignificantContent(_ image: CIImage) -> Bool {
        let sz = 20
        let ext = image.extent
        guard ext.width > 0, ext.height > 0 else { return false }

        let translated = image.transformed(by: CGAffineTransform(translationX: -ext.origin.x, y: -ext.origin.y))
        let scaled = translated.transformed(by: CGAffineTransform(scaleX: CGFloat(sz) / ext.width, y: CGFloat(sz) / ext.height))

        guard let cgImage = ciContext.createCGImage(scaled, from: CGRect(x: 0, y: 0, width: sz, height: sz)),
              let dp = cgImage.dataProvider,
              let data = dp.data,
              let ptr = CFDataGetBytePtr(data) else { return false }
        let bpp = cgImage.bitsPerPixel / 8
        let bpr = cgImage.bytesPerRow

        var minVal: Float = 1.0
        var maxVal: Float = 0.0

        for y in 0..<sz {
            for x in 0..<sz {
                let off = y * bpr + x * bpp
                let gray = (Float(ptr[off]) + Float(ptr[off+1]) + Float(ptr[off+2])) / (3.0 * 255.0)
                minVal = min(minVal, gray)
                maxVal = max(maxVal, gray)
            }
        }

        return (maxVal - minVal) > 0.15
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
