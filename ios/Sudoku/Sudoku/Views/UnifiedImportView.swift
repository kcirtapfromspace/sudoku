import SwiftUI
import AVFoundation
import UIKit

/// Bridge object that allows SwiftUI to trigger a photo capture on the AVFoundation camera.
final class CameraBridge: ObservableObject {
    var captureAction: (() -> Void)?

    func capture() {
        captureAction?()
    }
}

/// Unified camera import view that simultaneously scans QR codes and captures photos for OCR.
/// QR codes are detected automatically on the live feed. Users tap the shutter for printed puzzles.
struct UnifiedImportView: View {
    let onPuzzleFound: (String) -> Void
    let onImageCaptured: (UIImage) -> Void
    @Environment(\.dismiss) var dismiss
    @StateObject private var bridge = CameraBridge()
    @State private var errorMessage: String?
    @State private var qrDetected = false
    @State private var showingPhotoLibrary = false

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

                // Guidance
                Text("Point at a QR code or Sudoku puzzle")
                    .font(.subheadline)
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .background(.black.opacity(0.6), in: Capsule())
                    .padding(.top, 12)

                Text("QR codes are detected automatically")
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.7))
                    .padding(.top, 4)

                // Shutter button
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
        .sheet(isPresented: $showingPhotoLibrary) {
            CameraCaptureView(sourceType: .photoLibrary) { image in
                onImageCaptured(image)
                dismiss()
            }
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

    func makeUIViewController(context: Context) -> UnifiedCameraController {
        let controller = UnifiedCameraController()
        controller.onQRCodeScanned = onQRCodeScanned
        controller.onPhotoCaptured = onPhotoCaptured
        controller.onError = onError
        bridge.captureAction = { [weak controller] in
            controller?.takePhoto()
        }
        return controller
    }

    func updateUIViewController(_ uiViewController: UnifiedCameraController, context: Context) {}
}

// MARK: - Camera Controller (AVFoundation)

final class UnifiedCameraController: UIViewController,
    AVCaptureMetadataOutputObjectsDelegate,
    AVCapturePhotoCaptureDelegate {

    var onQRCodeScanned: ((String) -> Void)?
    var onPhotoCaptured: ((UIImage) -> Void)?
    var onError: ((String) -> Void)?

    private var captureSession: AVCaptureSession?
    private var photoOutput: AVCapturePhotoOutput?
    private var previewLayer: AVCaptureVideoPreviewLayer?
    private var hasProcessedQR = false

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black
        requestCameraAccess()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        previewLayer?.frame = view.bounds
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            self?.captureSession?.stopRunning()
        }
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

        // Preview layer
        let preview = AVCaptureVideoPreviewLayer(session: session)
        preview.frame = view.bounds
        preview.videoGravity = .resizeAspectFill
        view.layer.addSublayer(preview)
        self.previewLayer = preview

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
}
