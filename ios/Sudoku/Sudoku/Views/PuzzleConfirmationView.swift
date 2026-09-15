import SwiftUI

/// Screen for reviewing OCR results, editing mistakes, validating, and starting the game.
struct PuzzleConfirmationView: View {
    @StateObject private var viewModel: PuzzleConfirmationViewModel
    @Environment(\.dismiss) private var dismiss
    let image: UIImage
    let onPlay: (ImportedPuzzleData) -> Void

    init(image: UIImage, viewModel: PuzzleConfirmationViewModel? = nil,
         onPlay: @escaping (ImportedPuzzleData) -> Void) {
        self.image = image
        self.onPlay = onPlay
        _viewModel = StateObject(wrappedValue: viewModel ?? PuzzleConfirmationViewModel())
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                if viewModel.isProcessing {
                    Spacer()
                    ProgressView("Recognizing puzzle...")
                    Spacer()
                } else if let error = viewModel.errorMessage {
                    Spacer()
                    VStack(spacing: 12) {
                        Image(systemName: "exclamationmark.triangle")
                            .font(.largeTitle)
                            .foregroundStyle(.orange)
                        Text(error)
                            .multilineTextAlignment(.center)
                            .foregroundStyle(.secondary)
                        Button("Try Again") { dismiss() }
                            .buttonStyle(.bordered)
                    }
                    .padding()
                    Spacer()
                } else {
                    // Header info
                    HStack {
                        Text("\(viewModel.givenCount) digits found")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                        Spacer()
                        if viewModel.digits.enumerated().contains(where: { viewModel.isLowConfidence(at: $0.offset) }) {
                            Label("Uncertain cells highlighted", systemImage: "exclamationmark.circle")
                                .font(.caption)
                                .foregroundStyle(.orange)
                        }
                    }
                    .padding(.horizontal)

                    // Import mode picker (only shown when player progress detected)
                    if viewModel.hasPlayerProgress {
                        Picker("Import Mode", selection: $viewModel.importMode) {
                            ForEach(ImportMode.allCases, id: \.self) { mode in
                                Text(mode.rawValue).tag(mode)
                            }
                        }
                        .pickerStyle(.segmented)
                        .padding(.horizontal)
                    }

                    // Editable grid
                    ConfirmationGridView(viewModel: viewModel)
                        .padding(.horizontal)

                    // Number pad
                    ConfirmationNumberPad(viewModel: viewModel)
                        .padding(.horizontal)

                    // Validation status
                    validationBanner

                    Spacer()

                    // Action buttons
                    VStack(spacing: 12) {
                        Button {
                            viewModel.validate()
                        } label: {
                            Label("Validate", systemImage: "checkmark.shield")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                        .controlSize(.large)

                        Button {
                            let data = viewModel.buildImportData()
                            onPlay(data)
                            dismiss()
                        } label: {
                            Label(playButtonLabel, systemImage: "play.fill")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.large)
                        .disabled(!viewModel.canPlay)
                    }
                    .padding(.horizontal, 40)
                    .padding(.bottom)
                }
            }
            .navigationTitle("Import Puzzle")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
        }
        .onAppear {
            viewModel.processImage(image)
        }
    }

    private var playButtonLabel: String {
        if viewModel.hasPlayerProgress && viewModel.importMode == .continuePuzzle {
            return "Continue Puzzle"
        }
        return "Play"
    }

    @ViewBuilder
    private var validationBanner: some View {
        switch viewModel.validationResult {
        case .notValidated:
            EmptyView()
        case .valid:
            Label("Valid puzzle with a unique solution", systemImage: "checkmark.circle.fill")
                .font(.subheadline)
                .foregroundStyle(.green)
                .padding(.horizontal)
        case .invalid(let reason):
            Label(reason, systemImage: "xmark.circle.fill")
                .font(.subheadline)
                .foregroundStyle(.red)
                .padding(.horizontal)
        }
    }
}
