import SwiftUI

struct NumberPadView: View {
    @EnvironmentObject var gameManager: GameManager
    @ObservedObject var game: GameViewModel
    var onNumberTap: ((Int) -> Void)? = nil  // Optional callback for Konami code

    var body: some View {
        VStack(spacing: 8) {
            HStack(spacing: 8) {
                ForEach(1...5, id: \.self) { num in
                    numberButton(num)
                }
            }
            HStack(spacing: 8) {
                ForEach(6...9, id: \.self) { num in
                    numberButton(num)
                }
                // Clear button in number pad
                Button {
                    game.clearSelectedCell()
                    hapticFeedback(.light)
                } label: {
                    Image(systemName: "xmark")
                        .font(.title2.weight(.medium))
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
                .buttonStyle(NumberPadButtonStyle(isCompleted: false))
            }
        }
    }

    @ViewBuilder
    private func numberButton(_ number: Int) -> some View {
        let isCompleted = game.completedNumbers.contains(number)

        Button {
            game.enterNumber(number)
            onNumberTap?(number)  // Notify for Konami code detection
            hapticFeedback(isCompleted ? .light : .medium)
        } label: {
            Text("\(number)")
                .font(.title.weight(.semibold))
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .buttonStyle(NumberPadButtonStyle(isCompleted: isCompleted))
        .disabled(isCompleted && game.inputMode == .normal)
    }

    private func hapticFeedback(_ style: UIImpactFeedbackGenerator.FeedbackStyle) {
        guard gameManager.settings.hapticsEnabled else { return }
        UIImpactFeedbackGenerator(style: style).impactOccurred()
    }
}

// MARK: - Number Pad Button Style

struct NumberPadButtonStyle: ButtonStyle {
    let isCompleted: Bool

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .foregroundStyle(isCompleted ? Color.secondary : Color.primary)
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(configuration.isPressed ?
                          Color.accentColor.opacity(0.2) :
                          Color.secondary.opacity(isCompleted ? 0.1 : 0.15))
            )
            .frame(height: 56)
            .scaleEffect(configuration.isPressed ? 0.95 : 1.0)
            .animation(.easeInOut(duration: 0.1), value: configuration.isPressed)
    }
}

#Preview {
    NumberPadView(game: GameViewModel(difficulty: .medium))
        .environmentObject(GameManager())
        .padding()
}
