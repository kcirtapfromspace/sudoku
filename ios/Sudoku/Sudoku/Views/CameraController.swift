import SwiftUI
import AVFoundation
import UIKit
import Vision

// The controller owns camera UX and detection state. The session adapter is the
// only part that needs physical capture hardware; both remain production code.
protocol CameraSessionDriving: AnyObject {
    var previewLayer: AVCaptureVideoPreviewLayer { get }
    func configure(metadataDelegate: AVCaptureMetadataOutputObjectsDelegate,
                   videoDelegate: AVCaptureVideoDataOutputSampleBufferDelegate,
                   queue: DispatchQueue) throws
    func start()
    func stop()
    func capture(delegate: AVCapturePhotoCaptureDelegate)
}

struct CameraSetupError: LocalizedError {
    let message: String
    var errorDescription: String? { message }
}

final class AVCameraSession: CameraSessionDriving {
    private let session: AVCaptureSession
    private let photo: AVCapturePhotoOutput
    private let makeInput: () throws -> AVCaptureInput
    private let makeMetadata: () -> AVCaptureMetadataOutput
    lazy var previewLayer = AVCaptureVideoPreviewLayer(session: session)

    init(session: AVCaptureSession = AVCaptureSession(), photo: AVCapturePhotoOutput = AVCapturePhotoOutput(),
         makeInput: @escaping () throws -> AVCaptureInput = AVCameraSession.deviceInput,
         makeMetadata: @escaping () -> AVCaptureMetadataOutput = { AVCaptureMetadataOutput() }) {
        self.session = session
        self.photo = photo
        self.makeInput = makeInput
        self.makeMetadata = makeMetadata
    }

    static func deviceInput() throws -> AVCaptureInput {
        guard let device = AVCaptureDevice.default(for: .video) else {
            throw CameraSetupError(message: "No camera found on this device.")
        }
        return try AVCaptureDeviceInput(device: device)
    }

    func configure(metadataDelegate: AVCaptureMetadataOutputObjectsDelegate,
                   videoDelegate: AVCaptureVideoDataOutputSampleBufferDelegate,
                   queue: DispatchQueue) throws {
        session.sessionPreset = .photo
        let input = try makeInput()
        guard session.canAddInput(input) else {
            throw CameraSetupError(message: "Could not connect the camera.")
        }
        session.addInput(input)

        let metadata = makeMetadata()
        if session.canAddOutput(metadata) {
            session.addOutput(metadata)
            metadata.setMetadataObjectsDelegate(metadataDelegate, queue: .main)
            if metadata.availableMetadataObjectTypes.contains(.qr) {
                metadata.metadataObjectTypes = [.qr]
            }
        }
        guard session.canAddOutput(photo) else {
            throw CameraSetupError(message: "Photo capture is not available.")
        }
        session.addOutput(photo)

        let video = AVCaptureVideoDataOutput()
        video.setSampleBufferDelegate(videoDelegate, queue: queue)
        video.alwaysDiscardsLateVideoFrames = true
        if session.canAddOutput(video) { session.addOutput(video) }
    }

    func start() { session.startRunning() }
    func stop() { session.stopRunning() }
    func capture(delegate: AVCapturePhotoCaptureDelegate) {
        photo.capturePhoto(with: AVCapturePhotoSettings(), delegate: delegate)
    }
}

struct CameraDependencies {
    private static let workQueue = DispatchQueue(label: "com.ukodus.cameraWork", qos: .userInitiated)
    var authorizationStatus: () -> AVAuthorizationStatus = { AVCaptureDevice.authorizationStatus(for: .video) }
    var requestAccess: (@escaping (Bool) -> Void) -> Void = { AVCaptureDevice.requestAccess(for: .video, completionHandler: $0) }
    var makeSession: () -> CameraSessionDriving = { AVCameraSession() }
    var now: () -> CFAbsoluteTime = CFAbsoluteTimeGetCurrent
    // Session start/stop and Vision jobs share one serial order. A late start
    // cannot overtake the stop submitted when the controller disappears.
    var background: (@escaping () -> Void) -> Void = { CameraDependencies.workQueue.async(execute: $0) }
    var main: (@escaping () -> Void) -> Void = { DispatchQueue.main.async(execute: $0) }
    var after: (TimeInterval, @escaping () -> Void) -> Void = { delay, action in
        DispatchQueue.main.asyncAfter(deadline: .now() + delay, execute: action)
    }
    var detectRectangles: (CVPixelBuffer) throws -> [VNRectangleObservation] = CameraDependencies.detectRectangles(in:)
    var gridScore: (CIImage, VNRectangleObservation, CIContext) -> Float = {
        PuzzleOCRService.gridStructureScore(image: $0, rect: $1, context: $2)
    }
    var convertPoint: (AVCaptureVideoPreviewLayer, CGPoint) -> CGPoint = {
        $0.layerPointConverted(fromCaptureDevicePoint: $1)
    }

    static func detectRectangles(in buffer: CVPixelBuffer) throws -> [VNRectangleObservation] {
        let request = VNDetectRectanglesRequest()
        request.minimumAspectRatio = 0.7
        request.maximumAspectRatio = 1.3
        request.minimumSize = 0.1
        request.maximumObservations = 5
        request.minimumConfidence = 0.3
        try VNImageRequestHandler(cvPixelBuffer: buffer, options: [:]).perform([request])
        return request.results ?? []
    }
}

struct UnifiedCameraRepresentable: UIViewControllerRepresentable {
    let bridge: CameraBridge
    let onQRCodeScanned: (String) -> Void
    let onPhotoCaptured: (UIImage) -> Void
    let onError: (String) -> Void
    let onGridStateChanged: (_ stable: Bool, _ consecutiveCount: Int) -> Void
    var dependencies = CameraDependencies()

    func makeUIViewController(context: Context) -> UnifiedCameraController { makeController(dependencies: dependencies) }

    func makeController(dependencies: CameraDependencies = CameraDependencies()) -> UnifiedCameraController {
        let controller = UnifiedCameraController(dependencies: dependencies)
        controller.onQRCodeScanned = onQRCodeScanned
        controller.onPhotoCaptured = onPhotoCaptured
        controller.onError = onError
        controller.onGridStateChanged = onGridStateChanged
        bridge.captureAction = { [weak controller] in controller?.takePhoto() }
        return controller
    }

    func updateUIViewController(_ uiViewController: UnifiedCameraController, context: Context) {}
}

final class UnifiedCameraController: UIViewController,
    AVCaptureMetadataOutputObjectsDelegate,
    AVCapturePhotoCaptureDelegate,
    AVCaptureVideoDataOutputSampleBufferDelegate {

    var onQRCodeScanned: ((String) -> Void)?
    var onPhotoCaptured: ((UIImage) -> Void)?
    var onError: ((String) -> Void)?
    var onGridStateChanged: ((_ stable: Bool, _ consecutiveCount: Int) -> Void)?

    private let dependencies: CameraDependencies
    private var session: CameraSessionDriving?
    private var isVisible = true
    private var lifecycleGeneration = 0
    private var hasProcessedQR = false
    private(set) var gridOverlayLayer = CAShapeLayer()
    private var consecutiveGridDetections = 0
    private var lastGridDetectionTime: CFAbsoluteTime = -.infinity
    private var isProcessingGrid = false
    private var hasAutoCapture = false
    private let gridDetectionInterval: CFAbsoluteTime = 0.3
    private let requiredStableFrames = 3
    private let gridDetectionQueue = DispatchQueue(label: "com.ukodus.gridDetection", qos: .userInitiated)
    private let ciContext = CIContext()

    init(dependencies: CameraDependencies = CameraDependencies()) {
        self.dependencies = dependencies
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        dependencies = CameraDependencies()
        super.init(coder: coder)
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black
        gridOverlayLayer.fillColor = UIColor.clear.cgColor
        gridOverlayLayer.strokeColor = UIColor.systemGreen.cgColor
        gridOverlayLayer.lineWidth = 3
        gridOverlayLayer.opacity = 0
        requestCameraAccess()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        session?.previewLayer.frame = view.bounds
        gridOverlayLayer.frame = view.bounds
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        guard !isVisible else { return }
        isVisible = true
        if let session = session { dependencies.background { session.start() } }
        else { requestCameraAccess() }
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        isVisible = false
        lifecycleGeneration += 1
        isProcessingGrid = false
        hasAutoCapture = false
        hasProcessedQR = false
        lastGridDetectionTime = -.infinity
        if consecutiveGridDetections > 0 {
            consecutiveGridDetections = 0
            hideGridOverlay()
            onGridStateChanged?(false, 0)
        }
        if let session = session { dependencies.background { session.stop() } }
    }

    private func requestCameraAccess() {
        switch dependencies.authorizationStatus() {
        case .authorized:
            setupCamera()
        case .notDetermined:
            let generation = lifecycleGeneration
            dependencies.requestAccess { [weak self] granted in
                guard let self = self else { return }
                self.dependencies.main {
                    guard self.isVisible, self.lifecycleGeneration == generation else { return }
                    if granted { self.setupCamera() }
                    else { self.onError?(Self.permissionMessage) }
                }
            }
        case .denied, .restricted:
            onError?(Self.permissionMessage)
        @unknown default:
            onError?("Camera is not available.")
        }
    }

    private static let permissionMessage = "Camera access denied. Enable it in Settings > Privacy > Camera."

    private func setupCamera() {
        let candidate = dependencies.makeSession()
        do {
            try candidate.configure(metadataDelegate: self, videoDelegate: self, queue: gridDetectionQueue)
        } catch {
            onError?("Could not access camera: \(error.localizedDescription)")
            return
        }
        let preview = candidate.previewLayer
        preview.frame = view.bounds
        preview.videoGravity = .resizeAspectFill
        view.layer.addSublayer(preview)
        view.layer.addSublayer(gridOverlayLayer)
        session = candidate
        dependencies.background { candidate.start() }
    }

    func takePhoto() {
        guard isVisible else { return }
        session?.capture(delegate: self)
    }

    func metadataOutput(_ output: AVCaptureMetadataOutput,
                        didOutput metadataObjects: [AVMetadataObject],
                        from connection: AVCaptureConnection) {
        handleQRCode((metadataObjects.first as? AVMetadataMachineReadableCodeObject)?.stringValue)
    }

    func handleQRCode(_ value: String?) {
        guard isVisible, !hasProcessedQR, let value = value else { return }
        hasProcessedQR = true
        UINotificationFeedbackGenerator().notificationOccurred(.success)
        onQRCodeScanned?(value)
        let generation = lifecycleGeneration
        dependencies.after(3) { [weak self] in
            guard let self = self, self.lifecycleGeneration == generation else { return }
            self.hasProcessedQR = false
        }
    }

    func photoOutput(_ output: AVCapturePhotoOutput,
                     didFinishProcessingPhoto photo: AVCapturePhoto, error: Error?) {
        handlePhoto(data: photo.fileDataRepresentation(), error: error)
    }

    func handlePhoto(data: Data?, error: Error?) {
        dependencies.main { [weak self] in
            guard let self = self, self.isVisible else { return }
            guard error == nil, let data = data, let image = UIImage(data: data) else {
                self.hasAutoCapture = false
                self.consecutiveGridDetections = 0
                self.hideGridOverlay()
                self.onGridStateChanged?(false, 0)
                self.onError?(error?.localizedDescription ?? "Could not read the captured photo. Please try again.")
                return
            }
            UIImpactFeedbackGenerator(style: .medium).impactOccurred()
            self.onPhotoCaptured?(image)
        }
    }

    func captureOutput(_ output: AVCaptureOutput, didOutput sampleBuffer: CMSampleBuffer,
                       from connection: AVCaptureConnection) {
        let buffer = CMSampleBufferGetImageBuffer(sampleBuffer)
        dependencies.main { [weak self] in self?.processFrame(buffer) }
    }

    // All state transitions run on the main queue. Expensive Vision/scoring work
    // runs in the background, and a thrown Vision request always releases the gate.
    func processFrame(_ buffer: CVPixelBuffer?) {
        guard isVisible else { return }
        let now = dependencies.now()
        guard now - lastGridDetectionTime >= gridDetectionInterval,
              !isProcessingGrid, !hasAutoCapture else { return }
        lastGridDetectionTime = now
        guard let buffer = buffer else { return }
        isProcessingGrid = true
        let generation = lifecycleGeneration
        dependencies.background { [weak self] in
            guard let self = self else { return }
            let best: VNRectangleObservation?
            do {
                best = self.bestGrid(in: try self.dependencies.detectRectangles(buffer),
                                     image: CIImage(cvPixelBuffer: buffer))
            } catch {
                best = nil
            }
            self.dependencies.main { [weak self] in
                guard let self = self, self.isVisible, self.lifecycleGeneration == generation else { return }
                self.isProcessingGrid = false
                guard let best = best else { self.resetGridTracking(); return }
                self.consecutiveGridDetections += 1
                self.showGridOverlay(for: best)
                self.onGridStateChanged?(true, self.consecutiveGridDetections)
                if self.consecutiveGridDetections >= self.requiredStableFrames && !self.hasAutoCapture {
                    self.hasAutoCapture = true
                    UINotificationFeedbackGenerator().notificationOccurred(.success)
                    self.takePhoto()
                }
            }
        }
    }

    private func bestGrid(in candidates: [VNRectangleObservation], image: CIImage) -> VNRectangleObservation? {
        var best: VNRectangleObservation?
        var bestScore: Float = 0
        for candidate in candidates {
            guard candidate.confidence >= 0.3 else { continue }
            let width = hypot(candidate.topRight.x - candidate.topLeft.x, candidate.topRight.y - candidate.topLeft.y)
            let height = hypot(candidate.bottomLeft.x - candidate.topLeft.x, candidate.bottomLeft.y - candidate.topLeft.y)
            let aspect = width / height
            guard aspect >= 0.75 && aspect <= 1.33 else { continue }
            let score = dependencies.gridScore(image, candidate, ciContext)
            if score > bestScore { best = candidate; bestScore = score }
        }
        return bestScore >= 0.15 ? best : nil
    }

    private func resetGridTracking() {
        if consecutiveGridDetections > 0 {
            consecutiveGridDetections -= 1
            if consecutiveGridDetections == 0 {
                hideGridOverlay()
                onGridStateChanged?(false, 0)
            }
        }
    }

    private func showGridOverlay(for rect: VNRectangleObservation) {
        guard let preview = session?.previewLayer else { return }
        // Vision is bottom-left/y-up; preview conversion expects top-left/y-down.
        func convert(_ point: CGPoint) -> CGPoint {
            dependencies.convertPoint(preview, CGPoint(x: point.x, y: 1 - point.y))
        }
        let path = UIBezierPath()
        path.move(to: convert(rect.topLeft))
        path.addLine(to: convert(rect.topRight))
        path.addLine(to: convert(rect.bottomRight))
        path.addLine(to: convert(rect.bottomLeft))
        path.close()
        gridOverlayLayer.path = path.cgPath
        gridOverlayLayer.frame = preview.bounds
        if gridOverlayLayer.opacity == 0 {
            let animation = CABasicAnimation(keyPath: "opacity")
            animation.fromValue = 0
            animation.toValue = 1
            animation.duration = 0.2
            gridOverlayLayer.add(animation, forKey: "fadeIn")
            gridOverlayLayer.opacity = 1
        }
        let progress = min(CGFloat(consecutiveGridDetections) / CGFloat(requiredStableFrames), 1)
        gridOverlayLayer.strokeColor = UIColor(red: 0.2 * (1 - progress), green: 0.8,
                                               blue: 0.2 + 0.6 * (1 - progress), alpha: 1).cgColor
        gridOverlayLayer.lineWidth = 3 + progress * 2
    }

    private func hideGridOverlay() {
        let animation = CABasicAnimation(keyPath: "opacity")
        animation.fromValue = gridOverlayLayer.opacity
        animation.toValue = 0
        animation.duration = 0.3
        gridOverlayLayer.add(animation, forKey: "fadeOut")
        gridOverlayLayer.opacity = 0
    }
}
