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
    let onImportComplete: (ImportedPuzzleData) -> Void
    @Environment(\.dismiss) var dismiss
    @StateObject private var bridge: CameraBridge
    @StateObject private var presentation: ImportPresentation

    init(presentation: ImportPresentation? = nil,
         bridge: CameraBridge? = nil,
         onPuzzleFound: @escaping (String) -> Void,
         onImportComplete: @escaping (ImportedPuzzleData) -> Void) {
        _presentation = StateObject(wrappedValue: presentation ?? ImportPresentation())
        _bridge = StateObject(wrappedValue: bridge ?? CameraBridge())
        self.onPuzzleFound = onPuzzleFound
        self.onImportComplete = onImportComplete
    }

    var body: some View {
        ZStack {
            if let image = presentation.capturedImage {
                PuzzleConfirmationView(image: image) { importData in
                    dismiss()
                    onImportComplete(importData)
                }
            } else if presentation.cameraUnavailable {
                noCameraFallback
            } else {
                cameraView
            }
        }
        .animation(.easeInOut(duration: 0.3), value: presentation.errorMessage != nil)
        .animation(.easeInOut(duration: 0.3), value: presentation.qrDetected)
        .animation(.easeInOut(duration: 0.3), value: presentation.gridTracking)
        .onAppear { presentation.activate() }
        .onDisappear { presentation.deactivate() }
        .sheet(isPresented: $presentation.showingPhotoLibrary) {
            CameraCaptureView(sourceType: .photoLibrary) { image in
                presentation.capturedImage = image
            }
        }
    }

    // MARK: - No Camera Fallback (Simulator)

    private var noCameraFallback: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            VStack(spacing: 24) {
                Spacer()

                Image(systemName: "camera.fill")
                    .font(.system(size: 48))
                    .foregroundStyle(.secondary)

                Text("No Camera Available")
                    .font(.title2.weight(.semibold))
                    .foregroundStyle(.white)

                Text("Select a photo of a Sudoku puzzle\nfrom your photo library.")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.7))
                    .multilineTextAlignment(.center)

                Button {
                    presentation.showingPhotoLibrary = true
                } label: {
                    Label("Choose from Photo Library", systemImage: "photo.on.rectangle")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .padding(.horizontal, 40)

                #if DEBUG
                Button {
                    presentation.capturedImage = Self.generateTestPuzzleImage()
                } label: {
                    Label("Use Test Puzzle Image", systemImage: "testtube.2")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .tint(.orange)
                .controlSize(.large)
                .padding(.horizontal, 40)
                #endif

                Spacer()

                Button {
                    dismiss()
                } label: {
                    Text("Cancel")
                        .foregroundStyle(.white.opacity(0.7))
                }
                .padding(.bottom, 40)
            }
        }
    }

    // MARK: - Live Camera View

    private var cameraView: some View {
        ZStack {
            UnifiedCameraRepresentable(
                bridge: bridge,
                onQRCodeScanned: handleQRCode,
                onPhotoCaptured: { image in
                    presentation.capturedImage = image
                },
                onError: { presentation.handleCameraError($0) },
                onGridStateChanged: { stable, count in
                    withAnimation(.easeInOut(duration: 0.2)) {
                        presentation.updateTracking(stable: stable, count: count)
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
                        presentation.showingPhotoLibrary = true
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
                if let error = presentation.errorMessage {
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
                if presentation.qrDetected {
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
                if case .holdSteady = presentation.gridTracking {
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
                } else if presentation.gridTracking == .capturing {
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
    }

    private var guidanceText: Text {
        switch presentation.gridTracking {
        case .none:
            return Text("Point at a QR code or Sudoku puzzle")
        case .detected, .holdSteady:
            return Text("Hold the camera steady...")
        case .capturing:
            return Text("Processing...")
        }
    }

    private func handleQRCode(_ code: String) {
        presentation.scanQRCode(code) { puzzle in
            onPuzzleFound(puzzle)
            dismiss()
        }
    }

    #if DEBUG
    /// Generate a synthetic Sudoku puzzle image for simulator testing.
    /// Draws a 9x9 grid with digits from a known valid puzzle.
    static func generateTestPuzzleImage() -> UIImage {
        let gridSize: CGFloat = 900
        let padding: CGFloat = 80  // White margin so grid detector can find the rectangle
        let totalSize = gridSize + padding * 2
        let cellSize = gridSize / 9.0

        // A known valid Sudoku puzzle (0 = empty)
        let puzzle: [[Int]] = [
            [5,3,0, 0,7,0, 0,0,0],
            [6,0,0, 1,9,5, 0,0,0],
            [0,9,8, 0,0,0, 0,6,0],

            [8,0,0, 0,6,0, 0,0,3],
            [4,0,0, 8,0,3, 0,0,1],
            [7,0,0, 0,2,0, 0,0,6],

            [0,6,0, 0,0,0, 2,8,0],
            [0,0,0, 4,1,9, 0,0,5],
            [0,0,0, 0,8,0, 0,7,9],
        ]

        // Player-filled digits (blue) — simulates a game in progress
        let playerMoves: [(row: Int, col: Int, digit: Int)] = [
            (0, 3, 6),  // row 0, col 3 → 6
            (1, 1, 7),  // row 1, col 1 → 7
        ]

        // Pencil marks in empty cells — 3x3 sub-grid layout:
        // [1][2][3]
        // [4][5][6]
        // [7][8][9]
        let pencilMarks: [(row: Int, col: Int, notes: [Int])] = [
            (0, 2, [1, 2, 4]),      // row 0, col 2
            (0, 5, [4, 8]),          // row 0, col 5
            (2, 0, [1, 2]),          // row 2, col 0
            (2, 3, [2, 3, 4]),       // row 2, col 3
        ]

        let renderer = UIGraphicsImageRenderer(size: CGSize(width: totalSize, height: totalSize))
        return renderer.image { ctx in
            // White background (including padding)
            UIColor.white.setFill()
            ctx.fill(CGRect(x: 0, y: 0, width: totalSize, height: totalSize))

            // Translate to draw grid inside padding
            ctx.cgContext.translateBy(x: padding, y: padding)

            // Draw thin cell lines
            UIColor.gray.setStroke()
            let thinPath = UIBezierPath()
            thinPath.lineWidth = 2
            for i in 1..<9 {
                let pos = CGFloat(i) * cellSize
                thinPath.move(to: CGPoint(x: pos, y: 0))
                thinPath.addLine(to: CGPoint(x: pos, y: gridSize))
                thinPath.move(to: CGPoint(x: 0, y: pos))
                thinPath.addLine(to: CGPoint(x: gridSize, y: pos))
            }
            thinPath.stroke()

            // Draw thick box lines
            UIColor.black.setStroke()
            let thickPath = UIBezierPath()
            thickPath.lineWidth = 6
            for i in 0...3 {
                let pos = CGFloat(i) * cellSize * 3
                thickPath.move(to: CGPoint(x: pos, y: 0))
                thickPath.addLine(to: CGPoint(x: pos, y: gridSize))
                thickPath.move(to: CGPoint(x: 0, y: pos))
                thickPath.addLine(to: CGPoint(x: gridSize, y: pos))
            }
            thickPath.stroke()

            // Draw given digits (black, bold)
            let givenFont = UIFont.systemFont(ofSize: cellSize * 0.65, weight: .bold)
            let givenAttrs: [NSAttributedString.Key: Any] = [
                .font: givenFont,
                .foregroundColor: UIColor.black,
            ]
            for row in 0..<9 {
                for col in 0..<9 {
                    let digit = puzzle[row][col]
                    guard digit != 0 else { continue }
                    let text = "\(digit)" as NSString
                    let textSize = text.size(withAttributes: givenAttrs)
                    let x = CGFloat(col) * cellSize + (cellSize - textSize.width) / 2
                    let y = CGFloat(row) * cellSize + (cellSize - textSize.height) / 2
                    text.draw(at: CGPoint(x: x, y: y), withAttributes: givenAttrs)
                }
            }

            // Draw player-filled digits (blue)
            let playerFont = UIFont.systemFont(ofSize: cellSize * 0.65, weight: .medium)
            let playerAttrs: [NSAttributedString.Key: Any] = [
                .font: playerFont,
                .foregroundColor: UIColor.systemBlue,
            ]
            for move in playerMoves {
                let text = "\(move.digit)" as NSString
                let textSize = text.size(withAttributes: playerAttrs)
                let x = CGFloat(move.col) * cellSize + (cellSize - textSize.width) / 2
                let y = CGFloat(move.row) * cellSize + (cellSize - textSize.height) / 2
                text.draw(at: CGPoint(x: x, y: y), withAttributes: playerAttrs)
            }

            // Draw pencil marks (small gray digits in 3x3 sub-grid)
            let noteFont = UIFont.systemFont(ofSize: cellSize * 0.22, weight: .regular)
            let noteAttrs: [NSAttributedString.Key: Any] = [
                .font: noteFont,
                .foregroundColor: UIColor.darkGray,
            ]
            let subCellSize = cellSize / 3.0
            for mark in pencilMarks {
                let cellX = CGFloat(mark.col) * cellSize
                let cellY = CGFloat(mark.row) * cellSize
                for note in mark.notes {
                    let subRow = (note - 1) / 3
                    let subCol = (note - 1) % 3
                    let text = "\(note)" as NSString
                    let textSize = text.size(withAttributes: noteAttrs)
                    let x = cellX + CGFloat(subCol) * subCellSize + (subCellSize - textSize.width) / 2
                    let y = cellY + CGFloat(subRow) * subCellSize + (subCellSize - textSize.height) / 2
                    text.draw(at: CGPoint(x: x, y: y), withAttributes: noteAttrs)
                }
            }
        }
    }
    #endif
}
