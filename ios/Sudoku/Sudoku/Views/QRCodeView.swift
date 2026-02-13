import SwiftUI
import UIKit
import CoreImage.CIFilterBuiltins

struct QRCodeView: View {
    let puzzleString: String
    let shortCode: String?
    @Environment(\.dismiss) var dismiss

    init(puzzleString: String, shortCode: String? = nil) {
        self.puzzleString = puzzleString
        self.shortCode = shortCode
    }

    private var shareCode: String {
        if let code = shortCode {
            return code
        }
        return puzzleString
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Text("Share This Puzzle")
                    .font(.title2.bold())

                if let image = generateQRCode(from: shareCode) {
                    Image(uiImage: image)
                        .interpolation(.none)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 250, height: 250)
                        .padding()
                        .background(Color.white)
                        .cornerRadius(12)
                }

                Text("Scan this QR code to play the same puzzle")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)

                ShareLink(item: shareCode) {
                    Label("Share Code", systemImage: "square.and.arrow.up")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .padding(.horizontal, 40)

                Button {
                    UIPasteboard.general.string = shareCode
                } label: {
                    Label("Copy Code", systemImage: "doc.on.doc")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .controlSize(.large)
                .padding(.horizontal, 40)
            }
            .padding()
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }

    private func generateQRCode(from string: String) -> UIImage? {
        let context = CIContext()
        let filter = CIFilter.qrCodeGenerator()
        filter.message = Data(string.utf8)
        filter.correctionLevel = "M"

        guard let outputImage = filter.outputImage else { return nil }

        // Scale up for crisp rendering
        let scale = 10.0
        let transform = CGAffineTransform(scaleX: scale, y: scale)
        let scaledImage = outputImage.transformed(by: transform)

        guard let cgImage = context.createCGImage(scaledImage, from: scaledImage.extent) else {
            return nil
        }

        return UIImage(cgImage: cgImage)
    }
}

#Preview {
    QRCodeView(puzzleString: "530070000600195000098000060800060003400803001700020006060000280000419005000080079")
}
