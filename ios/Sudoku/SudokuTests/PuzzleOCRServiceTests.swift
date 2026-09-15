import XCTest
import UIKit
import Vision
import CoreImage
@testable import Sudoku

final class PuzzleOCRServiceTests: XCTestCase {
    private let fullRect = VNRectangleObservation(boundingBox: CGRect(x: 0, y: 0, width: 1, height: 1))
    private func grid() -> UIImage {
        ServiceFixtures.image { context in
            UIColor.black.setStroke(); context.cgContext.setLineWidth(3)
            for index in 0...9 {
                let position = CGFloat(index * 20)
                context.cgContext.move(to: CGPoint(x: position, y: 0)); context.cgContext.addLine(to: CGPoint(x: position, y: 180))
                context.cgContext.move(to: CGPoint(x: 0, y: position)); context.cgContext.addLine(to: CGPoint(x: 180, y: position))
            }
            context.cgContext.strokePath()
        }
    }
    private func ink(_ color: UIColor) -> CIImage {
        CIImage(image: ServiceFixtures.image { context in
            color.setFill(); context.fill(CGRect(x: 50, y: 50, width: 80, height: 80))
        })!
    }
    func testGridScoringPerspectiveExtractionAndVisionDetection() throws {
        let service = PuzzleOCRService()
        let image = CIImage(image: grid())!
        XCTAssertGreaterThan(PuzzleOCRService.gridStructureScore(image: image, rect: fullRect, context: CIContext()), 0.8)
        XCTAssertEqual(PuzzleOCRService.gridStructureScore(image: CIImage(image: ServiceFixtures.image())!, rect: fullRect, context: CIContext()), 0)
        let corrected = try service.perspectiveCorrect(image, to: fullRect)
        let cells = service.extractCells(from: corrected, insetFraction: 0.1)
        XCTAssertEqual(cells.count, 81)
        XCTAssertGreaterThan(cells[0].extent.minY, cells[72].extent.minY)
        XCTAssertLessThan(cells[0].extent.minX, cells[8].extent.minX)
        XCTAssertEqual(cells[0].extent.width, corrected.extent.width / 9 * 0.8, accuracy: 0.001)
        let framed = ServiceFixtures.image(size: CGSize(width: 240, height: 240)) { _ in
            self.grid().draw(in: CGRect(x: 30, y: 30, width: 180, height: 180))
        }
        XCTAssertFalse(try PuzzleOCRService.visionRectangles(CIImage(image: framed)!, 0.3).isEmpty)
        XCTAssertGreaterThan(service.enhanceForGridDetection(image).extent.width, 0)
    }
    func testPrimaryEnhancedFallbackAndNoGrid() throws {
        let image = CIImage(image: grid())!, blank = CIImage(image: ServiceFixtures.image())!
        let small = VNRectangleObservation(boundingBox: CGRect(x: 0, y: 0, width: 0.5, height: 0.5))
        let primary = PuzzleOCRService(detectRectangles: { _, _ in [small, self.fullRect] })
        XCTAssertNotNil(try primary.detectGridPrimary(in: image))
        XCTAssertEqual(try primary.detectGridPrimary(in: blank)?.boundingBox, fullRect.boundingBox)
        XCTAssertNotNil(try primary.detectGridEnhanced(in: image, originalImage: image))
        XCTAssertEqual(try primary.detectGridEnhanced(in: blank, originalImage: blank)?.boundingBox, fullRect.boundingBox)
        let enhanced = PuzzleOCRService(detectRectangles: { _, confidence in confidence == 0.4 ? [] : [self.fullRect] })
        XCTAssertEqual(try enhanced.detectGridSync(in: image).boundingBox, fullRect.boundingBox)
        let missing = PuzzleOCRService(detectRectangles: { _, _ in [] })
        XCTAssertThrowsError(try missing.detectGridSync(in: blank)) { XCTAssertEqual($0.localizedDescription, OCRError.noGridFound.localizedDescription) }
    }
    func testDigitParsingConfidenceRetryAndRealVision() throws {
        let image = ink(.black)
        let empty = PuzzleOCRService(recognizeText: { _, _, _ in nil })
        XCTAssertEqual(try empty.recognizeSingleDigit(in: image, level: .fast).digit, 0)
        for text in ["0", "10", "X", ""] {
            let service = PuzzleOCRService(recognizeText: { _, _, _ in .init(text: text, confidence: 1) })
            let result = try service.recognizeSingleDigit(in: image, level: .fast)
            XCTAssertEqual(result.digit, 0); XCTAssertEqual(result.confidence, 0.8)
        }
        var levels: [VNRequestTextRecognitionLevel] = []
        let retry = PuzzleOCRService(recognizeText: { _, level, _ in
            levels.append(level)
            return .init(text: " 5\n", confidence: level == .fast ? 0.3 : 0.95)
        })
        let result = try retry.recognizeDigits(in: [image], fullCellImages: [image])
        XCTAssertEqual(levels, [.fast, .accurate]); XCTAssertEqual(result[0].digit, 5); XCTAssertEqual(result[0].classification, .given)
        XCTAssertEqual(try empty.recognizeDigits(in: [image], fullCellImages: [image])[0].classification, .empty)
        let digitImage = CIImage(image: ServiceFixtures.image(size: CGSize(width: 600, height: 180)) { _ in
            ("123456789" as NSString).draw(at: CGPoint(x: 20, y: 30), withAttributes: [.font: UIFont.monospacedSystemFont(ofSize: 90, weight: .regular), .foregroundColor: UIColor.black])
        })!
        let recognized = try PuzzleOCRService.visionText(digitImage, .accurate, ["123456789"])
        XCTAssertEqual(recognized?.text.trimmingCharacters(in: .whitespacesAndNewlines), "123456789")
        XCTAssertNil(try PuzzleOCRService.visionText(CIImage(image: ServiceFixtures.image())!, .fast, []))
    }
    func testColorClassificationAndContentThresholds() {
        let service = PuzzleOCRService()
        XCTAssertEqual(service.classifyCell(ink(.black)), .given)
        XCTAssertEqual(service.classifyCell(ink(.blue)), .playerFilled)
        XCTAssertEqual(service.classifyCell(ink(UIColor(red: 0.3, green: 0.24, blue: 0.24, alpha: 1))), .ambiguous)
        XCTAssertEqual(service.classifyCell(CIImage(image: ServiceFixtures.image())!), .ambiguous)
        XCTAssertEqual(service.classifyCell(CIImage.empty()), .ambiguous)
        XCTAssertFalse(service.hasSignificantContent(CIImage.empty()))
        XCTAssertFalse(service.hasSignificantContent(CIImage(image: ServiceFixtures.image())!))
        XCTAssertTrue(service.hasSignificantContent(ink(.black)))
    }
    func testNotesRequireMultipleCorrectConfidentDigits() throws {
        let notesImage = CIImage(image: ServiceFixtures.image { context in
            UIColor.black.setFill()
            for row in 0..<3 { for col in 0..<3 { context.fill(CGRect(x: col * 60 + 20, y: row * 60 + 20, width: 20, height: 20)) } }
        })!
        let service = PuzzleOCRService(recognizeText: { _, _, words in .init(text: words[0], confidence: 0.9) })
        XCTAssertEqual(try service.detectNotesInCell(notesImage), Set(1...9))
        XCTAssertTrue(try service.detectNotesInCell(notesImage.cropped(to: CGRect(x: 0, y: 0, width: 5, height: 5))).isEmpty)
        XCTAssertTrue(try service.detectNotesInCell(CIImage(image: ServiceFixtures.image())!).isEmpty)
        let one = PuzzleOCRService(recognizeText: { _, _, words in words[0] == "1" ? .init(text: "1", confidence: 1) : nil })
        XCTAssertTrue(try one.detectNotesInCell(notesImage).isEmpty)
        let low = PuzzleOCRService(recognizeText: { _, _, _ in .init(text: "wrong", confidence: 0.2) })
        XCTAssertTrue(try low.detectNotesInCell(notesImage).isEmpty)
        let given = CellOCRResult(digit: 5, confidence: 1, classification: .given, notes: [])
        XCTAssertEqual(try service.detectNotes(digitResults: [given], fullCells: [notesImage])[0].digit, 5)
        let empty = CellOCRResult(digit: 0, confidence: 1, classification: .empty, notes: [])
        XCTAssertEqual(try service.detectNotes(digitResults: [empty], fullCells: [notesImage])[0].notes, Set(1...9))
    }
    func testAsyncPipelineSuccessAndErrors() async throws {
        let service = PuzzleOCRService(recognizeText: { _, _, _ in .init(text: "5", confidence: 1) }, detectRectangles: { _, _ in [self.fullRect] })
        let result = try await service.recognizePuzzle(from: grid())
        XCTAssertEqual(result.cells.count, 81); XCTAssertEqual(result.puzzleString.count, 81)
        XCTAssertFalse(result.hasPlayerProgress)
        let player = PuzzleOCRService(recognizeText: { _, _, _ in .init(text: "5", confidence: 1) }, detectRectangles: { _, _ in [self.fullRect] })
        let blue = ServiceFixtures.image { context in
            UIColor.blue.setFill()
            for row in 0..<9 { for col in 0..<9 {
                context.fill(CGRect(x: col * 20 + 7, y: row * 20 + 7, width: 6, height: 6))
            } }
        }
        let playerResult = try await player.recognizePuzzle(from: blue)
        XCTAssertTrue(playerResult.hasPlayerProgress)
        XCTAssertEqual(playerResult.puzzleString, String(repeating: "0", count: 81))
        let empty = PuzzleOCRService(recognizeText: { _, _, _ in nil }, detectRectangles: { _, _ in [self.fullRect] })
        let emptyResult = try await empty.recognizePuzzle(from: ServiceFixtures.image())
        XCTAssertEqual(emptyResult.cells.count, 81)
        XCTAssertFalse(emptyResult.hasPlayerProgress)
        XCTAssertEqual(emptyResult.puzzleString, String(repeating: "0", count: 81))
        do { _ = try await service.recognizePuzzle(from: UIImage()); XCTFail("Expected invalid image") }
        catch { XCTAssertEqual(error.localizedDescription, OCRError.invalidImage.localizedDescription) }
        let failed = PuzzleOCRService(detectRectangles: { _, _ in throw OCRError.noGridFound })
        do { _ = try await failed.recognizePuzzle(from: grid()); XCTFail("Expected Vision failure") }
        catch { XCTAssertEqual(error.localizedDescription, OCRError.noGridFound.localizedDescription) }
        XCTAssertTrue(OCRError.perspectiveCorrectionFailed.localizedDescription.contains("straighten"))
    }
}
