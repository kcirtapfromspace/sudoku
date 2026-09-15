import XCTest
import AVFoundation
import Vision
import UIKit
import SwiftUI
@testable import Sudoku

private final class StubCameraSession: CameraSessionDriving {
    let previewLayer = AVCaptureVideoPreviewLayer()
    var setupError: Error?
    var configured = 0
    var starts = 0
    var stops = 0
    var photos = 0
    var onConfigured: (() -> Void)?
    weak var photoDelegate: AVCapturePhotoCaptureDelegate?

    func configure(metadataDelegate: AVCaptureMetadataOutputObjectsDelegate,
                   videoDelegate: AVCaptureVideoDataOutputSampleBufferDelegate,
                   queue: DispatchQueue) throws {
        configured += 1
        onConfigured?()
        if let setupError = setupError { throw setupError }
    }
    func start() { starts += 1 }
    func stop() { stops += 1 }
    func capture(delegate: AVCapturePhotoCaptureDelegate) { photos += 1; photoDelegate = delegate }
}

private final class RecordingCaptureSession: AVCaptureSession {
    var acceptsInput = true
    var acceptsPhoto = true
    var acceptsOptionalOutputs = true
    var connectedInputs: [AVCaptureInput] = []
    var connectedOutputs: [AVCaptureOutput] = []
    var starts = 0
    var stops = 0
    override func canAddInput(_ input: AVCaptureInput) -> Bool { acceptsInput }
    override func addInput(_ input: AVCaptureInput) { connectedInputs.append(input) }
    override func canAddOutput(_ output: AVCaptureOutput) -> Bool {
        output is AVCapturePhotoOutput ? acceptsPhoto : acceptsOptionalOutputs
    }
    override func addOutput(_ output: AVCaptureOutput) { connectedOutputs.append(output) }
    override func startRunning() { starts += 1 }
    override func stopRunning() { stops += 1 }
}

private final class RecordingPhotoOutput: AVCapturePhotoOutput {
    var settings: AVCapturePhotoSettings?
    weak var requestedDelegate: AVCapturePhotoCaptureDelegate?
    override func capturePhoto(with settings: AVCapturePhotoSettings, delegate: AVCapturePhotoCaptureDelegate) {
        self.settings = settings; requestedDelegate = delegate
    }
}

private final class RecordingMetadataOutput: AVCaptureMetadataOutput {
    var requestedTypes: [AVMetadataObject.ObjectType] = []
    override var availableMetadataObjectTypes: [AVMetadataObject.ObjectType] { [.qr] }
    override var metadataObjectTypes: [AVMetadataObject.ObjectType]? {
        get { requestedTypes }
        set { requestedTypes = newValue ?? [] }
    }
}

private final class LowConfidenceRectangle: VNRectangleObservation {
    override var confidence: VNConfidence { 0.2 }
}

@MainActor
private final class CameraHarness {
    let session = StubCameraSession()
    var status: AVAuthorizationStatus = .authorized
    var permission: ((Bool) -> Void)?
    var time: CFAbsoluteTime = 0
    var delayed: [(TimeInterval, () -> Void)] = []
    var pendingWork: [() -> Void] = []
    var runImmediately = true
    var rectangles: [VNRectangleObservation] = []
    var visionError: Error?
    var detections = 0
    var scores: [UUID: Float] = [:]
    var errors: [String] = []
    var qrCodes: [String] = []
    var images: [UIImage] = []
    var gridStates: [(Bool, Int)] = []

    func dependencies() -> CameraDependencies {
        var value = CameraDependencies()
        value.authorizationStatus = { self.status }
        value.requestAccess = { self.permission = $0 }
        value.makeSession = { self.session }
        value.now = { self.time }
        value.main = { $0() }
        value.background = { action in
            if self.runImmediately { action() } else { self.pendingWork.append(action) }
        }
        value.after = { self.delayed.append(($0, $1)) }
        value.detectRectangles = { _ in
            self.detections += 1
            if let error = self.visionError { throw error }
            return self.rectangles
        }
        value.gridScore = { _, rect, _ in self.scores[rect.uuid] ?? 0.8 }
        value.convertPoint = { layer, point in
            CGPoint(x: point.x * layer.bounds.width, y: point.y * layer.bounds.height)
        }
        return value
    }

    func controller() -> UnifiedCameraController {
        let controller = UnifiedCameraController(dependencies: dependencies())
        controller.onError = { self.errors.append($0) }
        controller.onQRCodeScanned = { self.qrCodes.append($0) }
        controller.onPhotoCaptured = { self.images.append($0) }
        controller.onGridStateChanged = { self.gridStates.append(($0, $1)) }
        controller.loadViewIfNeeded()
        controller.view.frame = CGRect(x: 0, y: 0, width: 200, height: 400)
        controller.viewDidLayoutSubviews()
        return controller
    }

    func frame(_ controller: UnifiedCameraController, buffer: CVPixelBuffer) {
        time += 0.31
        controller.processFrame(buffer)
    }
}

@MainActor
final class CameraControllerTests: XCTestCase {
    private func captureInput() throws -> AVCaptureInput {
        // A real non-device input exercises AV session wiring on Simulator.
        var format: CMMetadataFormatDescription?
        let specification: [String: Any] = [
            kCMMetadataFormatDescriptionMetadataSpecificationKey_Identifier as String:
                AVMetadataIdentifier.quickTimeMetadataLocationISO6709.rawValue,
            kCMMetadataFormatDescriptionMetadataSpecificationKey_DataType as String:
                kCMMetadataBaseDataType_UTF8 as String
        ]
        let result = CMMetadataFormatDescriptionCreateWithMetadataSpecifications(
            allocator: kCFAllocatorDefault, metadataType: kCMMetadataFormatType_Boxed,
            metadataSpecifications: [specification] as CFArray, formatDescriptionOut: &format)
        XCTAssertEqual(result, noErr)
        return AVCaptureMetadataInput(formatDescription: try XCTUnwrap(format), clock: CMClockGetHostTimeClock())
    }

    private func pixelBuffer() throws -> CVPixelBuffer {
        var buffer: CVPixelBuffer?
        let status = CVPixelBufferCreate(kCFAllocatorDefault, 100, 100,
                                        kCVPixelFormatType_32BGRA, nil, &buffer)
        XCTAssertEqual(status, kCVReturnSuccess)
        let result = try XCTUnwrap(buffer)
        CVPixelBufferLockBaseAddress(result, [])
        if let base = CVPixelBufferGetBaseAddress(result) {
            memset(base, 255, CVPixelBufferGetDataSize(result))
        }
        CVPixelBufferUnlockBaseAddress(result, [])
        return result
    }

    private func rectangle(_ box: CGRect = CGRect(x: 0.2, y: 0.2, width: 0.6, height: 0.6)) -> VNRectangleObservation {
        VNRectangleObservation(boundingBox: box)
    }

    func testAuthorizedCameraConnectsPreviewCaptureAndStopsOnDismissal() {
        let harness = CameraHarness()
        let controller = harness.controller()
        XCTAssertEqual(harness.session.configured, 1)
        XCTAssertEqual(harness.session.starts, 1)
        XCTAssertEqual(harness.session.previewLayer.videoGravity, .resizeAspectFill)
        XCTAssertEqual(harness.session.previewLayer.frame, controller.view.bounds)
        XCTAssertEqual(controller.gridOverlayLayer.frame, controller.view.bounds)
        XCTAssertTrue(controller.view.layer.sublayers?.last === controller.gridOverlayLayer)
        controller.takePhoto()
        XCTAssertEqual(harness.session.photos, 1)
        XCTAssertTrue(harness.session.photoDelegate === controller)
        controller.viewWillDisappear(false)
        XCTAssertEqual(harness.session.stops, 1)
    }

    func testDeniedRestrictedAndUnknownAuthorizationReportErrorsWithoutStartingSession() {
        for status in [AVAuthorizationStatus.denied, .restricted, AVAuthorizationStatus(rawValue: 99)!] {
            let harness = CameraHarness(); harness.status = status
            let controller = harness.controller()
            XCTAssertEqual(harness.errors.count, 1)
            XCTAssertEqual(harness.session.configured, 0)
            controller.takePhoto(); controller.viewWillDisappear(false)
            XCTAssertEqual(harness.session.photos, 0)
            XCTAssertEqual(harness.session.stops, 0)
        }
    }

    func testPermissionPromptHonorsAllowAndDenyAndDoesNotRetainDismissedController() {
        for granted in [true, false] {
            let harness = CameraHarness(); harness.status = .notDetermined
            let controller = harness.controller()
            XCTAssertEqual(harness.session.starts, 0)
            harness.permission?(granted)
            XCTAssertEqual(harness.session.starts, granted ? 1 : 0)
            XCTAssertEqual(harness.errors.count, granted ? 0 : 1)
            withExtendedLifetime(controller) {}
        }
        let harness = CameraHarness(); harness.status = .notDetermined
        var controller: UnifiedCameraController? = harness.controller()
        weak var retained = controller
        controller = nil
        XCTAssertNil(retained)
        harness.permission?(true)
        XCTAssertEqual(harness.session.starts, 0)
    }

    func testSessionConfigurationFailureIsVisibleAndDoesNotStartOrCapture() {
        let harness = CameraHarness()
        harness.session.setupError = CameraSetupError(message: "No lens available")
        let controller = harness.controller()
        XCTAssertEqual(harness.errors, ["Could not access camera: No lens available"])
        XCTAssertEqual(harness.session.starts, 0)
        controller.takePhoto()
        XCTAssertEqual(harness.session.photos, 0)
    }

    func testQRDebounceAllowsRetryAfterDelayAndIgnoresMissingValues() {
        let harness = CameraHarness(); let controller = harness.controller()
        controller.handleQRCode(nil)
        controller.handleQRCode("bad-code")
        controller.handleQRCode("second-code")
        XCTAssertEqual(harness.qrCodes, ["bad-code"])
        XCTAssertEqual(harness.delayed.first?.0, 3)
        harness.delayed.removeFirst().1()
        controller.handleQRCode("valid-code")
        XCTAssertEqual(harness.qrCodes, ["bad-code", "valid-code"])
    }

    func testPhotoDecodingDeliversImageAndFailuresAllowAnotherCapture() throws {
        let harness = CameraHarness(); let controller = harness.controller()
        let image = UIGraphicsImageRenderer(size: CGSize(width: 10, height: 10)).image { context in
            UIColor.white.setFill(); context.fill(CGRect(x: 0, y: 0, width: 10, height: 10))
        }
        controller.handlePhoto(data: image.pngData(), error: nil)
        XCTAssertEqual(harness.images.count, 1)
        XCTAssertEqual(harness.images[0].cgImage?.width, image.cgImage?.width)
        XCTAssertEqual(harness.images[0].cgImage?.height, image.cgImage?.height)
        controller.handlePhoto(data: nil, error: nil)
        controller.handlePhoto(data: Data([1, 2]), error: nil)
        controller.handlePhoto(data: image.pngData(), error: CameraSetupError(message: "Capture failed"))
        XCTAssertEqual(harness.errors.count, 3)
        XCTAssertEqual(harness.errors.last, "Capture failed")
        XCTAssertEqual(harness.gridStates.map { $0.1 }, [0, 0, 0])
        XCTAssertEqual(controller.gridOverlayLayer.opacity, 0)
        controller.takePhoto()
        XCTAssertEqual(harness.session.photos, 1)
    }

    func testThreeStableGridFramesAutocaptureOnceAndDrawConvertedOverlay() throws {
        let harness = CameraHarness(); harness.rectangles = [rectangle()]
        let controller = harness.controller(); let buffer = try pixelBuffer()
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(controller.gridOverlayLayer.opacity, 1)
        let bounds = try XCTUnwrap(controller.gridOverlayLayer.path).boundingBox
        XCTAssertEqual(bounds.minX, 40, accuracy: 0.001)
        XCTAssertEqual(bounds.minY, 80, accuracy: 0.001)
        XCTAssertEqual(bounds.width, 120, accuracy: 0.001)
        XCTAssertEqual(bounds.height, 240, accuracy: 0.001)
        harness.frame(controller, buffer: buffer)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.gridStates.map { $0.1 }, [1, 2, 3])
        XCTAssertEqual(harness.session.photos, 1)
        XCTAssertEqual(controller.gridOverlayLayer.lineWidth, 5)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.detections, 3)
        XCTAssertEqual(harness.session.photos, 1)
        controller.handlePhoto(data: nil, error: nil)
        for _ in 0..<3 { harness.frame(controller, buffer: buffer) }
        XCTAssertEqual(harness.session.photos, 2)
    }

    func testThrottlingMissingBuffersAndPendingDetectionDoNotStartExtraRequests() throws {
        let harness = CameraHarness(); let controller = harness.controller(); let buffer = try pixelBuffer()
        controller.processFrame(nil)
        controller.processFrame(buffer)
        XCTAssertEqual(harness.detections, 0)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.detections, 1)
        harness.runImmediately = false
        harness.frame(controller, buffer: buffer)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.pendingWork.count, 1)
        harness.pendingWork.removeFirst()()
        XCTAssertEqual(harness.detections, 2)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.pendingWork.count, 1)
    }

    func testVisionFailureReleasesProcessingGateAndGridTrackingDecaysToHidden() throws {
        let harness = CameraHarness(); harness.rectangles = [rectangle()]
        let controller = harness.controller(); let buffer = try pixelBuffer()
        harness.frame(controller, buffer: buffer)
        harness.frame(controller, buffer: buffer)
        harness.visionError = CameraSetupError(message: "Vision interrupted")
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(controller.gridOverlayLayer.opacity, 1)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(controller.gridOverlayLayer.opacity, 0)
        XCTAssertEqual(harness.gridStates.last?.0, false)
        XCTAssertEqual(harness.gridStates.last?.1, 0)
        harness.visionError = nil
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.gridStates.last?.1, 1)
        XCTAssertEqual(harness.detections, 5)
    }

    func testGridCandidatesRejectConfidenceAspectAndWeakStructureThenChooseBest() throws {
        let harness = CameraHarness(); let controller = harness.controller(); let buffer = try pixelBuffer()
        let weak = rectangle(); harness.scores[weak.uuid] = 0.1
        let low = LowConfidenceRectangle(boundingBox: CGRect(x: 0, y: 0, width: 1, height: 1))
        let wide = rectangle(CGRect(x: 0, y: 0, width: 0.9, height: 0.2))
        let flat = rectangle(CGRect(x: 0, y: 0, width: 0.5, height: 0))
        harness.rectangles = [low, wide, flat, weak]
        harness.frame(controller, buffer: buffer)
        XCTAssertTrue(harness.gridStates.isEmpty)
        let winner = rectangle(CGRect(x: 0.1, y: 0.1, width: 0.5, height: 0.5))
        harness.scores[winner.uuid] = 0.9
        harness.rectangles = [weak, winner, rectangle()]
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.gridStates.last?.1, 1)
        XCTAssertEqual(try XCTUnwrap(controller.gridOverlayLayer.path).boundingBox.width, 100, accuracy: 0.001)
    }

    func testGridDetectionWithoutSessionAndDismissedPendingWorkAreSafe() throws {
        let harness = CameraHarness(); harness.status = .denied; harness.rectangles = [rectangle()]
        var controller: UnifiedCameraController? = harness.controller()
        let buffer = try pixelBuffer()
        harness.frame(try XCTUnwrap(controller), buffer: buffer)
        XCTAssertEqual(harness.gridStates.last?.1, 1)
        XCTAssertEqual(controller?.gridOverlayLayer.opacity, 0)
        harness.runImmediately = false
        harness.frame(try XCTUnwrap(controller), buffer: buffer)
        weak var retained = controller
        controller = nil
        XCTAssertNil(retained)
        harness.pendingWork.removeFirst()()
        XCTAssertEqual(harness.detections, 1)
    }

    func testRepresentableBridgeForwardsCallbacksAndDoesNotRetainController() {
        let harness = CameraHarness(); let bridge = CameraBridge()
        let representable = UnifiedCameraRepresentable(bridge: bridge,
            onQRCodeScanned: { harness.qrCodes.append($0) },
            onPhotoCaptured: { harness.images.append($0) },
            onError: { harness.errors.append($0) },
            onGridStateChanged: { harness.gridStates.append(($0, $1)) })
        var controller: UnifiedCameraController? = representable.makeController(dependencies: harness.dependencies())
        controller?.loadViewIfNeeded()
        bridge.capture()
        XCTAssertEqual(harness.session.photos, 1)
        controller?.handleQRCode("shared")
        XCTAssertEqual(harness.qrCodes, ["shared"])
        weak var retained = controller
        controller = nil
        XCTAssertNil(retained)
        bridge.capture()
        XCTAssertEqual(harness.session.photos, 1)
    }

    func testLiveVisionAndNoHardwareSessionBoundaries() throws {
        let buffer = try pixelBuffer()
        XCTAssertTrue(try CameraDependencies.detectRectangles(in: buffer).isEmpty)
        #if targetEnvironment(simulator)
        let live = AVCameraSession()
        let controller = UnifiedCameraController(dependencies: CameraHarness().dependencies())
        XCTAssertThrowsError(try live.configure(metadataDelegate: controller, videoDelegate: controller, queue: .main))
        XCTAssertNotNil(live.previewLayer)
        live.stop()
        #endif
    }

    func testAVSessionAdapterConnectsOutputsAndForwardsCaptureSettings() throws {
        let session = RecordingCaptureSession()
        let photo = RecordingPhotoOutput()
        let metadata = RecordingMetadataOutput()
        let input = try captureInput()
        let adapter = AVCameraSession(session: session, photo: photo, makeInput: { input }, makeMetadata: { metadata })
        let controller = UnifiedCameraController(dependencies: CameraHarness().dependencies())
        try adapter.configure(metadataDelegate: controller, videoDelegate: controller, queue: .main)
        XCTAssertTrue(session.connectedInputs.first === input)
        XCTAssertEqual(session.connectedOutputs.count, 3)
        XCTAssertTrue(session.connectedOutputs[0] === metadata)
        XCTAssertTrue(session.connectedOutputs[1] === photo)
        XCTAssertEqual(metadata.requestedTypes, [.qr])
        XCTAssertTrue(try XCTUnwrap(session.connectedOutputs[2] as? AVCaptureVideoDataOutput).alwaysDiscardsLateVideoFrames)
        adapter.start(); adapter.stop(); adapter.capture(delegate: controller)
        XCTAssertEqual(session.starts, 1); XCTAssertEqual(session.stops, 1)
        XCTAssertNotNil(photo.settings)
        XCTAssertTrue(photo.requestedDelegate === controller)
        XCTAssertTrue(adapter.previewLayer.session === session)
    }

    func testAVSessionAdapterRejectsRequiredConnectionsAndAllowsOptionalOutputFallback() throws {
        let session = RecordingCaptureSession()
        let controller = UnifiedCameraController(dependencies: CameraHarness().dependencies())
        let adapter = AVCameraSession(session: session, makeInput: { try self.captureInput() })
        session.acceptsInput = false
        XCTAssertThrowsError(try adapter.configure(metadataDelegate: controller, videoDelegate: controller, queue: .main)) {
            XCTAssertEqual($0.localizedDescription, "Could not connect the camera.")
        }
        session.acceptsInput = true; session.acceptsPhoto = false
        XCTAssertThrowsError(try adapter.configure(metadataDelegate: controller, videoDelegate: controller, queue: .main)) {
            XCTAssertEqual($0.localizedDescription, "Photo capture is not available.")
        }
        session.connectedOutputs = []
        session.acceptsPhoto = true; session.acceptsOptionalOutputs = false
        try adapter.configure(metadataDelegate: controller, videoDelegate: controller, queue: .main)
        XCTAssertEqual(session.connectedOutputs.count, 1)
        XCTAssertTrue(session.connectedOutputs[0] is AVCapturePhotoOutput)
        let failedInput = AVCameraSession(session: session, makeInput: { throw CameraSetupError(message: "Lens busy") })
        XCTAssertThrowsError(try failedInput.configure(metadataDelegate: controller, videoDelegate: controller, queue: .main)) {
            XCTAssertEqual($0.localizedDescription, "Lens busy")
        }
    }

    func testRepresentableCreatesAndUpdatesControllerWhenHostedBySwiftUI() async {
        let harness = CameraHarness()
        let configured = expectation(description: "SwiftUI created camera controller")
        harness.session.onConfigured = { configured.fulfill() }
        let representable = UnifiedCameraRepresentable(bridge: CameraBridge(),
            onQRCodeScanned: { _ in }, onPhotoCaptured: { _ in }, onError: { _ in },
            onGridStateChanged: { _, _ in }, dependencies: harness.dependencies())
        let host = UIHostingController(rootView: representable.ignoresSafeArea())
        let window = UIWindow(frame: CGRect(x: 0, y: 0, width: 200, height: 400))
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true; window.rootViewController = nil }
        host.view.layoutIfNeeded()
        await fulfillment(of: [configured], timeout: 3)
        XCTAssertEqual(harness.session.starts, 1)
        XCTAssertEqual(harness.session.previewLayer.frame.size, host.view.bounds.size)
    }

    func testControllerSupportsUIKitCodingWithoutLoadingCamera() throws {
        let controller = UnifiedCameraController()
        let data = try NSKeyedArchiver.archivedData(withRootObject: controller, requiringSecureCoding: false)
        let decoded = try XCTUnwrap(NSKeyedUnarchiver.unarchiveTopLevelObjectWithData(data) as? UnifiedCameraController)
        XCTAssertFalse(decoded.isViewLoaded)
    }

    func testReturningFromPhotoLibraryResumesExistingSessionWithoutReconfiguration() throws {
        let harness = CameraHarness(); harness.rectangles = [rectangle()]
        let controller = harness.controller(); let buffer = try pixelBuffer()
        controller.viewWillAppear(false)
        XCTAssertEqual(harness.session.starts, 1)
        harness.frame(controller, buffer: buffer)
        controller.viewWillDisappear(false)
        XCTAssertEqual(harness.session.stops, 1)
        XCTAssertEqual(controller.gridOverlayLayer.opacity, 0)
        XCTAssertEqual(harness.gridStates.last?.1, 0)
        controller.takePhoto(); controller.handleQRCode("hidden")
        controller.handlePhoto(data: nil, error: CameraSetupError(message: "hidden"))
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.session.photos, 0)
        XCTAssertTrue(harness.qrCodes.isEmpty)
        XCTAssertTrue(harness.errors.isEmpty)
        XCTAssertEqual(harness.detections, 1)
        controller.viewWillAppear(false)
        XCTAssertEqual(harness.session.starts, 2)
        XCTAssertEqual(harness.session.configured, 1)
        for _ in 0..<3 { harness.frame(controller, buffer: buffer) }
        XCTAssertEqual(harness.session.photos, 1)
    }

    func testPermissionResponseFromHiddenGenerationCannotStartSession() {
        let harness = CameraHarness(); harness.status = .notDetermined
        let controller = harness.controller()
        let oldPermission = harness.permission
        controller.viewWillDisappear(false)
        oldPermission?(true)
        XCTAssertEqual(harness.session.starts, 0)
        harness.status = .authorized
        controller.viewWillAppear(false)
        XCTAssertEqual(harness.session.starts, 1)
        oldPermission?(false)
        XCTAssertTrue(harness.errors.isEmpty)
        XCTAssertEqual(harness.session.configured, 1)
    }

    func testStaleVisionCompletionDoesNotCaptureOrUnlockNewGenerationRequest() throws {
        let harness = CameraHarness(); harness.rectangles = [rectangle()]
        let controller = harness.controller(); let buffer = try pixelBuffer()
        harness.frame(controller, buffer: buffer)
        harness.frame(controller, buffer: buffer)
        harness.runImmediately = false
        harness.frame(controller, buffer: buffer)
        let oldVision = harness.pendingWork.removeFirst()
        controller.viewWillDisappear(false)
        controller.viewWillAppear(false)
        // Drain ordered stop/start operations; the earlier Vision result is
        // deliberately delivered late to exercise the generation check.
        while !harness.pendingWork.isEmpty { harness.pendingWork.removeFirst()() }
        harness.frame(controller, buffer: buffer)
        oldVision()
        XCTAssertEqual(harness.session.photos, 0)
        harness.frame(controller, buffer: buffer)
        XCTAssertEqual(harness.pendingWork.count, 1)
        harness.pendingWork.removeFirst()()
        XCTAssertEqual(harness.gridStates.last?.1, 1)
        XCTAssertEqual(harness.session.photos, 0)
    }

    func testOldQRDebounceCannotClearAResumedSessionsDebounce() {
        let harness = CameraHarness(); let controller = harness.controller()
        controller.handleQRCode("before")
        let oldReset = harness.delayed.removeFirst().1
        controller.viewWillDisappear(false); controller.viewWillAppear(false)
        controller.handleQRCode("after")
        oldReset()
        controller.handleQRCode("duplicate")
        XCTAssertEqual(harness.qrCodes, ["before", "after"])
        harness.delayed.removeFirst().1()
        controller.handleQRCode("retry")
        XCTAssertEqual(harness.qrCodes, ["before", "after", "retry"])
    }

    func testProductionCameraWorkQueuePreservesStartStopOrder() async {
        let dependencies = CameraDependencies()
        let finished = expectation(description: "Serial camera operations")
        let events = NSMutableArray()
        dependencies.background { events.add("start") }
        dependencies.background { events.add("stop") }
        dependencies.background { finished.fulfill() }
        await fulfillment(of: [finished], timeout: 3)
        XCTAssertEqual(events as? [String], ["start", "stop"])
    }
}
