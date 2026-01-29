import SwiftUI

struct GameView: View {
    @EnvironmentObject var gameManager: GameManager
    @ObservedObject var game: GameViewModel
    @StateObject private var konamiDetector = KonamiCodeDetector()
    @State private var showingKonamiAlert = false
    @State private var konamiMessage = ""

    var body: some View {
        GeometryReader { geometry in
            if geometry.size.width > geometry.size.height {
                // Landscape layout
                HStack(spacing: 20) {
                    gridSection(size: min(geometry.size.height - 40, geometry.size.width * 0.55))
                    controlsSection(compact: true)
                }
                .padding()
            } else {
                // Portrait layout
                VStack(spacing: 16) {
                    headerSection
                    gridSection(size: min(geometry.size.width - 32, geometry.size.height * 0.55))
                    Spacer(minLength: 8)
                    numberPadSection
                    controlsSection(compact: false)
                }
                .padding()
            }
        }
        .gesture(konamiGesture)
        .onChange(of: konamiDetector.isActivated) { activated in
            if activated {
                triggerKonamiEasterEgg()
            }
        }
        .alert("🎮 KONAMI CODE!", isPresented: $showingKonamiAlert) {
            Button("Awesome!") {
                konamiDetector.reset()
            }
        } message: {
            Text(konamiMessage)
        }
    }

    // MARK: - Konami Code

    private var konamiGesture: some Gesture {
        // Detect swipe directions
        DragGesture(minimumDistance: 30)
            .onEnded { gesture in
                let horizontal = gesture.translation.width
                let vertical = gesture.translation.height

                if abs(horizontal) > abs(vertical) {
                    // Horizontal swipe
                    if horizontal > 0 {
                        konamiDetector.input(.right)
                    } else {
                        konamiDetector.input(.left)
                    }
                } else {
                    // Vertical swipe
                    if vertical > 0 {
                        konamiDetector.input(.down)
                    } else {
                        konamiDetector.input(.up)
                    }
                }
                hapticFeedback(.light)
            }
    }

    /// Called when Konami code is entered on the number pad (2=B, 1=A after swipes)
    func handleKonamiNumberPad(_ number: Int) {
        if number == 2 {
            konamiDetector.input(.b)
        } else if number == 1 {
            konamiDetector.input(.a)
        }
    }

    private func triggerKonamiEasterEgg() {
        let easterEggs = [
            "🚀 +30 extra lives! (Just kidding, you only had 3)",
            "🎯 God mode activated! (Your mistakes still count though)",
            "🧠 IQ temporarily boosted to 9000!",
            "🎮 You found the secret! Here's a virtual high-five: 🖐️",
            "🔮 The puzzle whispers its secrets to you...",
            "⬆️⬆️⬇️⬇️⬅️➡️⬅️➡️🅱️🅰️ - A true gamer!",
            "🏆 Achievement Unlocked: Nostalgia Master",
            "🎪 Circus mode engaged! 🤹‍♂️ (Nothing changed, but imagine it did)"
        ]
        konamiMessage = easterEggs.randomElement() ?? "You did it!"
        showingKonamiAlert = true
        hapticFeedback(.heavy)
    }

    // MARK: - Header

    private var headerSection: some View {
        HStack {
            // Timer
            if gameManager.settings.timerVisible {
                Label(game.elapsedTimeString, systemImage: "clock")
                    .font(.headline.monospacedDigit())
            }

            Spacer()

            // Difficulty
            Text(game.difficulty.displayName)
                .font(.subheadline)
                .foregroundStyle(.secondary)

            Spacer()

            // Mistakes
            HStack(spacing: 4) {
                ForEach(0..<game.maxMistakes, id: \.self) { i in
                    Image(systemName: i < game.mistakes ? "heart.slash.fill" : "heart.fill")
                        .foregroundStyle(i < game.mistakes ? .red : .pink)
                }
            }
        }
    }

    // MARK: - Grid

    private func gridSection(size: CGFloat) -> some View {
        GridView(game: game, size: size)
            .frame(width: size, height: size)
    }

    // MARK: - Number Pad

    private var numberPadSection: some View {
        NumberPadView(game: game, onNumberTap: handleKonamiNumberPad)
    }

    // MARK: - Controls

    private func controlsSection(compact: Bool) -> some View {
        VStack(spacing: compact ? 8 : 12) {
            if compact {
                // Landscape: vertical controls
                VStack(spacing: 8) {
                    controlButtons
                    modeToggle
                }
            } else {
                // Portrait: horizontal controls
                HStack(spacing: 16) {
                    controlButtons
                }
                modeToggle
            }
        }
    }

    private var controlButtons: some View {
        Group {
            Button {
                game.undo()
                hapticFeedback(.light)
            } label: {
                Image(systemName: "arrow.uturn.backward")
            }
            .disabled(!game.canUndo)

            Button {
                game.redo()
                hapticFeedback(.light)
            } label: {
                Image(systemName: "arrow.uturn.forward")
            }
            .disabled(!game.canRedo)

            Button {
                game.clearSelectedCell()
                hapticFeedback(.light)
            } label: {
                Image(systemName: "delete.left")
            }

            Button {
                game.getHint()
                hapticFeedback(.medium)
            } label: {
                Image(systemName: "lightbulb")
            }

            Button {
                gameManager.pauseGame()
            } label: {
                Image(systemName: "pause")
            }
        }
        .buttonStyle(.bordered)
        .controlSize(.regular)
    }

    private var modeToggle: some View {
        Button {
            game.inputMode.toggle()
            hapticFeedback(.light)
        } label: {
            HStack {
                Image(systemName: game.inputMode == .normal ? "pencil" : "pencil.line")
                Text(game.inputMode.displayName)
            }
            .frame(maxWidth: .infinity)
        }
        .buttonStyle(.bordered)
        .tint(game.inputMode == .candidate ? .orange : nil)
    }

    // MARK: - Haptics

    private func hapticFeedback(_ style: UIImpactFeedbackGenerator.FeedbackStyle) {
        guard gameManager.settings.hapticsEnabled else { return }
        UIImpactFeedbackGenerator(style: style).impactOccurred()
    }
}

#Preview {
    GameView(game: GameViewModel(difficulty: .medium))
        .environmentObject(GameManager())
}
