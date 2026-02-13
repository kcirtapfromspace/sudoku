import SwiftUI
import AVFoundation

struct QRScannerView: View {
    let onPuzzleFound: (String) -> Void
    @Environment(\.dismiss) var dismiss
    @State private var errorMessage: String?

    var body: some View {
        NavigationStack {
            ZStack {
                QRCameraView(onCodeScanned: handleScannedCode)
                    .ignoresSafeArea()

                VStack {
                    Spacer()

                    if let error = errorMessage {
                        Text(error)
                            .font(.subheadline)
                            .foregroundStyle(.white)
                            .padding()
                            .background(.red.opacity(0.8), in: RoundedRectangle(cornerRadius: 8))
                            .padding()
                    }

                    Text("Point the camera at a Sudoku QR code")
                        .font(.subheadline)
                        .foregroundStyle(.white)
                        .padding()
                        .background(.black.opacity(0.6), in: Capsule())
                        .padding(.bottom, 40)
                }
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                        .foregroundStyle(.white)
                }
            }
        }
    }

    private func handleScannedCode(_ code: String) {
        // Try to extract puzzle from URL
        if let puzzleString = extractPuzzle(from: code) {
            onPuzzleFound(puzzleString)
            dismiss()
        } else {
            errorMessage = "Not a valid Sudoku puzzle"
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                errorMessage = nil
            }
        }
    }

    /// Extract the puzzle string from a URL, raw 81-char string, or 8-char short code
    private func extractPuzzle(from code: String) -> String? {
        let trimmed = code.trimmingCharacters(in: .whitespacesAndNewlines)

        // Check if it's a raw 81-char puzzle string
        if trimmed.count == 81 && trimmed.allSatisfy({ $0.isNumber || $0 == "." }) {
            return trimmed
        }

        // Check if it's a raw 8-char short code (alphanumeric)
        if trimmed.count == 8 && trimmed.allSatisfy({ $0.isLetter || $0.isNumber }) {
            return trimmed
        }

        // Try to parse as URL (backward compat)
        guard let url = URL(string: code),
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }

        // Check for short code (?s= parameter, 8-char PuzzleId)
        if let shortCode = components.queryItems?.first(where: { $0.name == "s" })?.value,
           shortCode.count == 8 {
            return shortCode
        }

        // Check for full puzzle string (?p= parameter)
        if let puzzleParam = components.queryItems?.first(where: { $0.name == "p" })?.value,
           puzzleParam.count == 81 {
            return puzzleParam
        }

        return nil
    }
}

// MARK: - Camera View

struct QRCameraView: UIViewControllerRepresentable {
    let onCodeScanned: (String) -> Void

    func makeUIViewController(context: Context) -> QRScannerController {
        let controller = QRScannerController()
        controller.onCodeScanned = onCodeScanned
        return controller
    }

    func updateUIViewController(_ uiViewController: QRScannerController, context: Context) {}
}

class QRScannerController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
    var onCodeScanned: ((String) -> Void)?
    private var captureSession: AVCaptureSession?
    private var hasScanned = false

    override func viewDidLoad() {
        super.viewDidLoad()

        let session = AVCaptureSession()

        guard let device = AVCaptureDevice.default(for: .video),
              let input = try? AVCaptureDeviceInput(device: device) else {
            return
        }

        if session.canAddInput(input) {
            session.addInput(input)
        }

        let output = AVCaptureMetadataOutput()
        if session.canAddOutput(output) {
            session.addOutput(output)
            output.setMetadataObjectsDelegate(self, queue: .main)
            output.metadataObjectTypes = [.qr]
        }

        let previewLayer = AVCaptureVideoPreviewLayer(session: session)
        previewLayer.frame = view.bounds
        previewLayer.videoGravity = .resizeAspectFill
        view.layer.addSublayer(previewLayer)

        captureSession = session

        DispatchQueue.global(qos: .userInitiated).async {
            session.startRunning()
        }
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        captureSession?.stopRunning()
    }

    func metadataOutput(_ output: AVCaptureMetadataOutput,
                        didOutput metadataObjects: [AVMetadataObject],
                        from connection: AVCaptureConnection) {
        guard !hasScanned,
              let object = metadataObjects.first as? AVMetadataMachineReadableCodeObject,
              let stringValue = object.stringValue else {
            return
        }

        hasScanned = true
        captureSession?.stopRunning()
        onCodeScanned?(stringValue)
    }
}

#Preview {
    QRScannerView { puzzle in
        print("Found puzzle: \(puzzle)")
    }
}
