import SwiftUI

struct NewGameSelection: View {
    let choices: [DifficultyChoice]
    @Binding var preferredRating: Float
    let onPlay: (Float) -> Void

    // A saved preference may belong to a level that is currently locked.
    private var selectedIndex: Int? {
        choices.indices.min { abs(choices[$0].rating - preferredRating) < abs(choices[$1].rating - preferredRating) }
    }

    var body: some View {
        if let selectedIndex {
            let choice = choices[selectedIndex]
            VStack(alignment: .leading, spacing: 16) {
                HStack(alignment: .firstTextBaseline) {
                    Text(choice.difficulty.displayName)
                        .font(.title2.weight(.semibold))
                    Spacer()
                    Text(choice.ratingLabel)
                        .font(.subheadline.monospacedDigit())
                        .foregroundStyle(.secondary)
                }
                .accessibilityHidden(true)

                if choices.count > 1 {
                    VStack(spacing: 2) {
                        Slider(value: Binding(
                            get: { Double(selectedIndex) },
                            set: { preferredRating = choices[Int($0.rounded())].rating }
                        ), in: 0...Double(choices.count - 1), step: 1)
                        .accessibilityLabel("Difficulty")
                        .accessibilityValue("\(choice.difficulty.displayName), \(choice.ratingLabel)")

                        HStack {
                            Text(choices[0].difficulty.displayName)
                            Spacer()
                            Text(choices[choices.count - 1].difficulty.displayName)
                        }
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .accessibilityHidden(true)
                    }
                }

                Button {
                    onPlay(choice.rating)
                } label: {
                    Label("New Game", systemImage: "plus")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .accessibilityIdentifier("New Game")
            }
        }
    }
}
