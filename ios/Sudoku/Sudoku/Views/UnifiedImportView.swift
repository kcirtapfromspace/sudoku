import SwiftUI
import AVFoundation
import UIKit
import Vision

/// Bridge object that allows SwiftUI to trigger a photo capture on the AVFoundation camera.
final class CameraBridge: ObservableObject {
    var captureAction: (() -> Void)?

    func capture() {
        captureAction?()
    }
}

/// Unified camera import view that simultaneously scans QR codes, detects Sudoku grids,
/// and captures photos for OCR. QR codes and grids are detected automatically on the live feed.
struct UnifiedImportView: View {
    let onPuzzleFound: (String) -> Void
    let onImageCaptured: (UIImage) -> Void
    @Environment(\.dismiss) var dismiss
    @StateObject private var bridge = CameraBridge()
    @State private var errorMessage: String?
    @State private var qrDetected = false
    @State private var gridTracking: GridTrackingState = .none
    @State private var showingPhotoLibrary = false

    enum GridTrackingState: Equatable {
        case none
        case detected
        case holdSteady(Int) // consecutive stable frames
        case capturing
    }

    var body: some View {
        ZStack {
            UnifiedCameraRepresentable(
                bridge: bridge,
                onQRCodeScanned: handleQRCode,
                onPhotoCaptured: { image in
                    onImageCaptured(image)
                    dismiss()
                },
                onError: { message in
                    errorMessage = message
                },
                onGridStateChanged: { stable, count in
                    withAnimation(.easeInOut(duration: 0.2)) {
                        if !stable {
                            gridTracking = .none
                        } else if count >= 3 {
                            gridTracking = .capturing
                        } else {
                            gridTracking = .holdSteady(count)
                        }
                    }
                }
            )
            .ignoresSafeArea()

            // Overlay controls
            VStack(spacing: 0) {
                // Top bar
                HStack {
                    Button {
                        dismiss()
                    } label: {
                        Image(systemName: "xmark")
                            .font(.title3.weight(.semibold))
                            .foregroundStyle(.white)
                            .frame(width: 36, height: 36)
                            .background(.black.opacity(0.5), in: Circle())
                    }
                    .padding(.leading, 16)

                    Spacer()

                    Button {
                        showingPhotoLibrary = true
                    } label: {
                        Image(systemName: "photo.on.rectangle")
                            .font(.title3)
                            .foregroundStyle(.white)
                            .frame(width: 36, height: 36)
                            .background(.black.opacity(0.5), in: Circle())
                    }
                    .padding(.trailing, 16)
                }
                .padding(.top, 8)

                Spacer()

                // Error banner
                if let error = errorMessage {
                    Text(error)
                        .font(.subheadline.weight(.medium))
                        .foregroundStyle(.white)
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .background(.red.opacity(0.85), in: RoundedRectangle(cornerRadius: 10))
                        .padding(.horizontal, 20)
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                }

                // QR detected banner
                if qrDetected {
                    HStack(spacing: 8) {
                        Image(systemName: "qrcode.viewfinder")
                        Text("QR code found!")
                    }
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(.green.opacity(0.85), in: RoundedRectangle(cornerRadius: 10))
                    .transition(.scale.combined(with: .opacity))
                }

                // Grid tracking banner
                if case .holdSteady = gridTracking {
                    HStack(spacing: 8) {
                        Image(systemName: "viewfinder")
                        Text("Grid detected — hold steady...")
                    }
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(.blue.opacity(0.85), in: RoundedRectangle(cornerRadius: 10))
                    .transition(.scale.combined(with: .opacity))
                } else if gridTracking == .capturing {
                    HStack(spacing: 8) {
                        ProgressView()
                            .tint(.white)
                        Text("Capturing puzzle...")
                    }
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(.green.opacity(0.85), in: RoundedRectangle(cornerRadius: 10))
                    .transition(.scale.combined(with: .opacity))
                }

                // Guidance
                guidanceText
                    .font(.subheadline)
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .background(.black.opacity(0.6), in: Capsule())
                    .padding(.top, 12)

                Text("QR codes and grids are detected automatically")
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.7))
                    .padding(.top, 4)

                // Shutter button (manual fallback)
                Button {
                    bridge.capture()
                } label: {
                    ZStack {
                        Circle()
                            .fill(.white)
                            .frame(width: 68, height: 68)
                        Circle()
                            .stroke(.white, lineWidth: 3)
                            .frame(width: 78, height: 78)
                    }
                }
                .padding(.top, 20)
                .padding(.bottom, 40)
            }
        }
        .animation(.easeInOut(duration: 0.3), value: errorMessage != nil)
        .animation(.easeInOut(duration: 0.3), value: qrDetected)
        .animation(.easeInOut(duration: 0.3), value: gridTracking)
        .sheet(isPresented: $showingPhotoLibrary) {
            CameraCaptureView(sourceType: .photoLibrary) { image in
                onImageCaptured(image)
                dismiss()
            }
        }
    }

    private var guidanceText: Text {
        switch gridTracking {
        case .none:
            return Text("Point at a QR code or Sudoku puzzle")
        case .detected, .holdSteady:
            return Text("Hold the camera steady...")
        case .capturing:
            return Text("Processing...")
        }
    }

    private func handleQRCode(_ code: String) {
        if let puzzle = extractPuzzle(from: code) {
            withAnimation { qrDetected = true }
            // Brief visual confirmation before processing
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) {
                onPuzzleFound(puzzle)
                dismiss()
            }
        } else {
            withAnimation { errorMessage = "Not a valid Sudoku QR code" }
            DispatchQueue.main.asyncAfter(deadline: .now() + 2.5) {
                withAnimation { errorMessage = nil }
            }
        }
    }

    /// Extract puzzle string from QR code content (URL, raw 81-char string, or 8-char short code)
    private func extractPuzzle(from code: String) -> String? {
        let trimmed = code.trimmingCharacters(in: .whitespacesAndNewlines)

        // Raw 81-char puzzle string
        if trimmed.count == 81 && trimmed.allSatisfy({ $0.isNumber || $0 == "." }) {
            return trimmed
        }

        // Raw 8-char short code (alphanumeric PuzzleId)
        if trimmed.count == 8 && trimmed.allSatisfy({ $0.isLetter || $0.isNumber }) {
            return trimmed
        }

        // URL format (backward compat)
        guard let url = URL(string: code),
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        if let shortCode = components.queryItems?.first(where: { $0.name == "s" })?.value,
           shortCode.count == 8 {
            return shortCode
        }

        if let puzzleParam = components.queryItems?.first(where: { $0.name == "p" })?.value,
           puzzleParam.count == 81 {
            return puzzleParam
        }

        return nil
    }
}

// MARK: - Camera UIViewControllerRepresentable

struct UnifiedCameraRepresentable: UIViewControllerRepresentable {
    let bridge: CameraBridge
    let onQRCodeScanned: (String) -> Void
    let onPhotoCaptured: (UIImage) -> Void
    let onError: (String) -> Void
    let onGridStateChanged: (_ stable: Bool, _ consecutiveCount: Int) -> Void

    func makeUIViewController(context: Context) -> UnifiedCameraController {
        let controller = UnifiedCameraController()
        controller.onQRCodeScanned = onQRCodeScanned
        controller.onPhotoCaptured = onPhotoCaptured
        controller.onError = onError
        controller.onGridStateChanged = onGridStateChanged
        bridge.captureAction = { [weak controller] in
            controller?.takePhoto()
        }
        return controller
    }

    func updateUIViewController(_ uiViewController: UnifiedCameraController, context: Context) {}
}

// MARK: - Camera Controller (AVFoundation + Vision grid detection)

final class UnifiedCameraController: UIViewController,
    AVCaptureMetadataOutputObjectsDelegate,
    AVCapturePhotoCaptureDelegate,
    AVCaptureVideoDataOutputSampleBufferDelegate {

    var onQRCodeScanned: ((String) -> Void)?
    var onPhotoCaptured: ((UIImage) -> Void)?
    var onError: ((String) -> Void)?
    var onGridStateChanged: ((_ stable: Bool, _ consecutiveCount: Int) -> Void)?

    private var captureSession: AVCaptureSession?
    private var photoOutput: AVCapturePhotoOutput?
    private var previewLayer: AVCaptureVideoPreviewLayer?
    private var hasProcessedQR = false

    // Grid detection state
    private var gridOverlayLayer = CAShapeLayer()
    private var consecutiveGridDetections = 0
    private var lastGridDetectionTime: CFAbsoluteTime = 0
    private var isProcessingGrid = false
    private var hasAutoCapture = false
    private let gridDetectionInterval: CFAbsoluteTime = 0.5 // seconds between detection attempts
    private let requiredStableFrames = 3 // consecutive detections before auto-capture
    private let gridDetectionQueue = DispatchQueue(label: "com.ukodus.gridDetection", qos: .userInitiated)
    private let ciContext = CIContext()

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black
        setupGridOverlay()
        requestCameraAccess()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        previewLayer?.frame = view.bounds
        gridOverlayLayer.frame = view.bounds
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            self?.captureSession?.stopRunning()
        }
    }

    private func setupGridOverlay() {
        gridOverlayLayer.fillColor = UIColor.clear.cgColor
        gridOverlayLayer.strokeColor = UIColor.systemGreen.cgColor
        gridOverlayLayer.lineWidth = 3
        gridOverlayLayer.opacity = 0
    }

    private func requestCameraAccess() {
        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized:
            setupCamera()
        case .notDetermined:
            AVCaptureDevice.requestAccess(for: .video) { [weak self] granted in
                DispatchQueue.main.async {
                    if granted {
                        self?.setupCamera()
                    } else {
                        self?.onError?("Camera access denied. Enable it in Settings > Privacy > Camera.")
                    }
                }
            }
        case .denied, .restricted:
            onError?("Camera access denied. Enable it in Settings > Privacy > Camera.")
        @unknown default:
            onError?("Camera is not available.")
        }
    }

    private func setupCamera() {
        let session = AVCaptureSession()
        session.sessionPreset = .photo

        guard let device = AVCaptureDevice.default(for: .video) else {
            onError?("No camera found on this device.")
            return
        }

        let input: AVCaptureDeviceInput
        do {
            input = try AVCaptureDeviceInput(device: device)
        } catch {
            onError?("Could not access camera: \(error.localizedDescription)")
            return
        }

        if session.canAddInput(input) {
            session.addInput(input)
        }

        // QR code detection
        let metadataOutput = AVCaptureMetadataOutput()
        if session.canAddOutput(metadataOutput) {
            session.addOutput(metadataOutput)
            metadataOutput.setMetadataObjectsDelegate(self, queue: .main)
            metadataOutput.metadataObjectTypes = [.qr]
        }

        // Photo capture
        let photo = AVCapturePhotoOutput()
        if session.canAddOutput(photo) {
            session.addOutput(photo)
            self.photoOutput = photo
        }

        // Video output for live grid detection
        let videoOutput = AVCaptureVideoDataOutput()
        videoOutput.setSampleBufferDelegate(self, queue: gridDetectionQueue)
        videoOutput.alwaysDiscardsLateVideoFrames = true
        if session.canAddOutput(videoOutput) {
            session.addOutput(videoOutput)
        }

        // Preview layer
        let preview = AVCaptureVideoPreviewLayer(session: session)
        preview.frame = view.bounds
        preview.videoGravity = .resizeAspectFill
        view.layer.addSublayer(preview)
        self.previewLayer = preview

        // Grid overlay on top of preview
        view.layer.addSublayer(gridOverlayLayer)

        self.captureSession = session

        DispatchQueue.global(qos: .userInitiated).async {
            session.startRunning()
        }
    }

    func takePhoto() {
        guard let photoOutput = photoOutput else { return }
        let settings = AVCapturePhotoSettings()
        photoOutput.capturePhoto(with: settings, delegate: self)
    }

    // MARK: - QR Code Detection

    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput metadataObjects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        guard !hasProcessedQR,
              let object = metadataObjects.first as? AVMetadataMachineReadableCodeObject,
              let stringValue = object.stringValue else {
            return
        }

        hasProcessedQR = true

        // Haptic feedback
        let generator = UINotificationFeedbackGenerator()
        generator.notificationOccurred(.success)

        onQRCodeScanned?(stringValue)

        // Allow re-scanning after a delay (in case the QR was invalid)
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) { [weak self] in
            self?.hasProcessedQR = false
        }
    }

    // MARK: - Photo Capture

    func photoOutput(
        _ output: AVCapturePhotoOutput,
        didFinishProcessingPhoto photo: AVCapturePhoto,
        error: Error?
    ) {
        guard let data = photo.fileDataRepresentation(),
              let image = UIImage(data: data) else {
            return
        }

        // Haptic feedback
        let generator = UIImpactFeedbackGenerator(style: .medium)
        generator.impactOccurred()

        onPhotoCaptured?(image)
    }

    // MARK: - Live Grid Detection

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        let now = CFAbsoluteTimeGetCurrent()
        guard now - lastGridDetectionTime >= gridDetectionInterval else { return }
        guard !isProcessingGrid, !hasAutoCapture else { return }

        lastGridDetectionTime = now
        isProcessingGrid = true

        guard let pixelBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else {
            isProcessingGrid = false
            return
        }

        let ciImage = CIImage(cvPixelBuffer: pixelBuffer)

        let request = VNDetectRectanglesRequest { [weak self] request, error in
            guard let self = self else { return }
            defer { self.isProcessingGrid = false }

            if let error = error {
                self.resetGridTracking()
                return
            }

            guard let results = request.results as? [VNRectangleObservation],
                  !results.isEmpty else {
                self.resetGridTracking()
                return
            }

            // Find the best candidate that passes grid structure verification
            var bestRect: VNRectangleObservation?
            var bestScore: Float = 0

            for candidate in results {
                guard candidate.confidence >= 0.5 else { continue }
                let w = hypot(candidate.topRight.x - candidate.topLeft.x,
                              candidate.topRight.y - candidate.topLeft.y)
                let h = hypot(candidate.bottomLeft.x - candidate.topLeft.x,
                              candidate.bottomLeft.y - candidate.topLeft.y)
                let aspect = w / h
                guard aspect >= 0.75 && aspect <= 1.33 else { continue }

                let score = PuzzleOCRService.gridStructureScore(
                    image: ciImage, rect: candidate, context: self.ciContext
                )
                if score > bestScore {
                    bestScore = score
                    bestRect = candidate
                }
            }

            // Require minimum grid structure score — reject random rectangles
            guard let best = bestRect, bestScore >= 0.25 else {
                self.resetGridTracking()
                return
            }

            self.consecutiveGridDetections += 1

            DispatchQueue.main.async {
                self.showGridOverlay(for: best)
                self.onGridStateChanged?(true, self.consecutiveGridDetections)

                if self.consecutiveGridDetections >= self.requiredStableFrames && !self.hasAutoCapture {
                    self.hasAutoCapture = true
                    let generator = UINotificationFeedbackGenerator()
                    generator.notificationOccurred(.success)
                    self.takePhoto()
                }
            }
        }

        request.minimumAspectRatio = 0.7
        request.maximumAspectRatio = 1.3
        request.minimumSize = 0.2
        request.maximumObservations = 5
        request.minimumConfidence = 0.4

        let handler = VNImageRequestHandler(cvPixelBuffer: pixelBuffer, options: [:])
        try? handler.perform([request])
    }

    private func rectArea(_ rect: VNRectangleObservation) -> CGFloat {
        let w = hypot(rect.topRight.x - rect.topLeft.x, rect.topRight.y - rect.topLeft.y)
        let h = hypot(rect.bottomLeft.x - rect.topLeft.x, rect.bottomLeft.y - rect.topLeft.y)
        return w * h
    }

    private func resetGridTracking() {
        if consecutiveGridDetections > 0 {
            consecutiveGridDetections = 0
            DispatchQueue.main.async { [weak self] in
                self?.hideGridOverlay()
                self?.onGridStateChanged?(false, 0)
            }
        }
    }

    private func showGridOverlay(for rect: VNRectangleObservation) {
        guard let previewLayer = previewLayer else { return }
        let bounds = previewLayer.bounds

        // Convert normalized Vision coordinates (origin bottom-left) to view coordinates
        func toViewPoint(_ p: CGPoint) -> CGPoint {
            CGPoint(x: p.x * bounds.width, y: (1 - p.y) * bounds.height)
        }

        let path = UIBezierPath()
        path.move(to: toViewPoint(rect.topLeft))
        path.addLine(to: toViewPoint(rect.topRight))
        path.addLine(to: toViewPoint(rect.bottomRight))
        path.addLine(to: toViewPoint(rect.bottomLeft))
        path.close()

        gridOverlayLayer.path = path.cgPath
        gridOverlayLayer.frame = bounds

        // Animate in if not visible
        if gridOverlayLayer.opacity == 0 {
            let animation = CABasicAnimation(keyPath: "opacity")
            animation.fromValue = 0
            animation.toValue = 1
            animation.duration = 0.2
            gridOverlayLayer.add(animation, forKey: "fadeIn")
            gridOverlayLayer.opacity = 1
        }

        // Pulse color based on stability
        let progress = min(CGFloat(consecutiveGridDetections) / CGFloat(requiredStableFrames), 1.0)
        let color = UIColor(
            red: 0.2 * (1 - progress),
            green: 0.8,
            blue: 0.2 + 0.6 * (1 - progress),
            alpha: 1.0
        )
        gridOverlayLayer.strokeColor = color.cgColor
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
