import XCTest
import SwiftUI
import ViewInspector
@testable import Sudoku

@MainActor
final class UnifiedImportViewTests: XCTestCase {
    func testQRCodeFeedbackDelaysDeliveryAndCancelsStaleResults() {
        var pending: [(TimeInterval, () -> Void)] = []
        let state = ImportPresentation(cameraAvailable: true, schedule: { pending.append(($0, $1)) })
        var delivered: [String] = []
        state.scanQRCode("invalid") { delivered.append($0) }
        XCTAssertEqual(state.errorMessage, "Not a valid Sudoku QR code")
        XCTAssertFalse(state.qrDetected)
        XCTAssertEqual(pending[0].0, 2.5)
        pending.removeFirst().1()
        XCTAssertNil(state.errorMessage)
        state.scanQRCode("invalid") { delivered.append($0) }
        state.scanQRCode(ViewTestFixture.puzzle) { delivered.append($0) }
        XCTAssertNil(state.errorMessage)
        XCTAssertTrue(state.qrDetected)
        XCTAssertEqual(pending[1].0, 0.4)
        pending.removeFirst().1()
        XCTAssertTrue(state.qrDetected)
        XCTAssertTrue(delivered.isEmpty)
        pending.removeFirst().1()
        XCTAssertEqual(delivered, [ViewTestFixture.puzzle])
        state.scanQRCode("abcd1234") { delivered.append($0) }
        state.cancelPendingScan()
        pending.removeFirst().1()
        XCTAssertEqual(delivered.count, 1)
        state.scanQRCode("invalid") { delivered.append($0) }
        state.cancelPendingScan()
        pending.removeFirst().1()
        XCTAssertNil(state.errorMessage)
        var ephemeral: ImportPresentation? = ImportPresentation(cameraAvailable: true, schedule: { pending.append(($0, $1)) })
        ephemeral?.scanQRCode("abcd1234") { delivered.append($0) }
        ephemeral = nil
        pending.removeFirst().1()
        XCTAssertEqual(delivered.count, 1)
    }

    func testSwitchingToPhotoImportCancelsQRCodeAndIgnoresFurtherScans() {
        var pending: [() -> Void] = []
        var delivered = 0
        let state = ImportPresentation(cameraAvailable: true, schedule: { _, action in pending.append(action) })
        state.scanQRCode("abcd1234") { _ in delivered += 1 }
        state.showingPhotoLibrary = true
        pending.removeFirst()()
        XCTAssertEqual(delivered, 0)
        XCTAssertFalse(state.qrDetected)
        state.scanQRCode("abcd1234") { _ in delivered += 1 }
        XCTAssertTrue(pending.isEmpty)
        state.showingPhotoLibrary = false
        state.scanQRCode("abcd1234") { _ in delivered += 1 }
        state.capturedImage = UIImage()
        pending.removeFirst()()
        XCTAssertEqual(delivered, 0)
        state.scanQRCode("abcd1234") { _ in delivered += 1 }
        XCTAssertTrue(pending.isEmpty)
        state.capturedImage = nil
        state.handleCameraError("Permission denied")
        state.cancelPendingScan()
        XCTAssertEqual(state.errorMessage, "Permission denied")
        state.deactivate()
        state.scanQRCode("abcd1234") { _ in delivered += 1 }
        XCTAssertTrue(pending.isEmpty)
        state.activate()
        state.scanQRCode("abcd1234") { _ in delivered += 1 }
        pending.removeFirst()()
        XCTAssertEqual(delivered, 1)
    }

    func testDefaultSchedulerAndCameraAvailability() async {
        let state = ImportPresentation()
        XCTAssertEqual(state.cameraUnavailable, !UIImagePickerController.isSourceTypeAvailable(.camera))
        let delivered = expectation(description: "QR feedback completes")
        state.scanQRCode("abcd1234") { code in
            XCTAssertEqual(code, "abcd1234")
            delivered.fulfill()
        }
        await fulfillment(of: [delivered], timeout: 2)
    }

    func testCameraFeedbackAndTrackingStates() {
        let state = ImportPresentation(cameraAvailable: true)
        state.handleCameraError("Permission denied")
        XCTAssertFalse(state.cameraUnavailable)
        XCTAssertEqual(state.errorMessage, "Permission denied")
        state.handleCameraError("No camera found")
        XCTAssertTrue(state.cameraUnavailable)
        state.cameraUnavailable = false
        state.handleCameraError("Capture not available")
        XCTAssertTrue(state.cameraUnavailable)
        state.updateTracking(stable: true, count: 1)
        XCTAssertEqual(state.gridTracking, .holdSteady(1))
        state.updateTracking(stable: true, count: 3)
        XCTAssertEqual(state.gridTracking, .capturing)
        state.updateTracking(stable: false, count: 3)
        XCTAssertEqual(state.gridTracking, .none)
    }

    func testFallbackPhotoLibraryAndDebugFixture() throws {
        let state = ImportPresentation(cameraAvailable: false)
        let view = UnifiedImportView(presentation: state, onPuzzleFound: { _ in }, onImportComplete: { _ in })
        XCTAssertNoThrow(try view.inspect().find(text: "No Camera Available"))
        try view.inspect().find(button: "Choose from Photo Library").tap()
        XCTAssertTrue(state.showingPhotoLibrary)
        state.showingPhotoLibrary = false
        try ViewTestFixture.render(view)
        try view.inspect().find(button: "Cancel").tap()
        #if DEBUG
        try view.inspect().find(button: "Use Test Puzzle Image").tap()
        XCTAssertEqual(state.capturedImage?.size.width, 1060)
        XCTAssertNoThrow(try view.inspect().find(PuzzleConfirmationView.self))
        #endif
    }

    func testCameraCallbacksDriveImportAndVisibleFeedback() throws {
        var pending: [() -> Void] = []
        let state = ImportPresentation(cameraAvailable: true, schedule: { _, action in pending.append(action) })
        var delivered: String?
        var imported: ImportedPuzzleData?
        let bridge = CameraBridge()
        let view = UnifiedImportView(presentation: state, bridge: bridge, onPuzzleFound: { delivered = $0 }, onImportComplete: { imported = $0 })
        let camera = try view.inspect().find(UnifiedCameraRepresentable.self).actualView()
        var captures = 0
        camera.bridge.captureAction = { captures += 1 }
        let buttons = try view.inspect().findAll(ViewType.Button.self)
        try buttons[0].tap()
        try buttons[1].tap()
        XCTAssertTrue(state.showingPhotoLibrary)
        try buttons[2].tap()
        XCTAssertEqual(captures, 1)
        state.showingPhotoLibrary = false
        camera.onError("Permission denied")
        XCTAssertNoThrow(try view.inspect().find(text: "Permission denied"))
        for (tracking, guidance) in [(ImportPresentation.GridTrackingState.none, "Point at a QR code or Sudoku puzzle"), (.detected, "Hold the camera steady..."), (.holdSteady(2), "Hold the camera steady..."), (.capturing, "Processing...")] {
            state.gridTracking = tracking
            XCTAssertNoThrow(try view.inspect().find(text: guidance))
        }
        camera.onGridStateChanged(true, 1)
        XCTAssertNoThrow(try view.inspect().find(text: "Grid detected — hold steady..."))
        camera.onGridStateChanged(true, 3)
        XCTAssertNoThrow(try view.inspect().find(text: "Capturing puzzle..."))
        camera.onQRCodeScanned("abcd1234")
        XCTAssertNoThrow(try view.inspect().find(text: "QR code found!"))
        pending.removeFirst()()
        XCTAssertEqual(delivered, "abcd1234")
        camera.onPhotoCaptured(UIImage())
        let confirmation = try view.inspect().find(PuzzleConfirmationView.self).actualView()
        let data = ImportedPuzzleData(givensString: ViewTestFixture.puzzle, playerMoves: [], playerNotes: [], isContinuing: false)
        confirmation.onPlay(data)
        XCTAssertEqual(imported?.givensString, ViewTestFixture.puzzle)
    }
}
