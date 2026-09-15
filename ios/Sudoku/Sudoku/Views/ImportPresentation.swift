import SwiftUI
import UIKit

/// Presentation state survives SwiftUI refreshes and invalidates delayed QR results on dismissal.
@MainActor
final class ImportPresentation: ObservableObject {
    enum GridTrackingState: Equatable {
        case none
        case detected
        case holdSteady(Int)
        case capturing
    }

    @Published var errorMessage: String?
    @Published var qrDetected = false
    @Published var gridTracking: GridTrackingState = .none
    @Published var showingPhotoLibrary = false {
        didSet { if showingPhotoLibrary { cancelPendingScan() } }
    }
    @Published var cameraUnavailable: Bool
    @Published var capturedImage: UIImage? {
        didSet { if capturedImage != nil { cancelPendingScan() } }
    }
    private var scanGeneration = 0
    private var isActive = true
    private let schedule: (TimeInterval, @escaping () -> Void) -> Void

    init(cameraAvailable: Bool? = nil,
         schedule: @escaping (TimeInterval, @escaping () -> Void) -> Void = { delay, action in
             DispatchQueue.main.asyncAfter(deadline: .now() + delay) { action() }
         }) {
        cameraUnavailable = !(cameraAvailable ?? UIImagePickerController.isSourceTypeAvailable(.camera))
        self.schedule = schedule
    }

    func handleCameraError(_ message: String) {
        if message.contains("No camera") || message.contains("not available") {
            cameraUnavailable = true
        } else {
            errorMessage = message
        }
    }

    func updateTracking(stable: Bool, count: Int) {
        if !stable { gridTracking = .none }
        else if count >= 3 { gridTracking = .capturing }
        else { gridTracking = .holdSteady(count) }
    }

    func scanQRCode(_ code: String, onPuzzle: @escaping (String) -> Void) {
        guard isActive, !showingPhotoLibrary, capturedImage == nil else { return }
        scanGeneration += 1
        let generation = scanGeneration
        if let puzzle = PuzzleLink.extract(from: code) {
            errorMessage = nil
            qrDetected = true
            schedule(0.4) { [weak self] in
                guard self?.scanGeneration == generation else { return }
                onPuzzle(puzzle)
            }
        } else {
            qrDetected = false
            errorMessage = "Not a valid Sudoku QR code"
            schedule(2.5) { [weak self] in
                guard self?.scanGeneration == generation else { return }
                self?.errorMessage = nil
            }
        }
    }

    func activate() { isActive = true }

    func deactivate() {
        isActive = false
        cancelPendingScan()
    }

    func cancelPendingScan() {
        scanGeneration += 1
        qrDetected = false
        if errorMessage == "Not a valid Sudoku QR code" { errorMessage = nil }
    }
}
