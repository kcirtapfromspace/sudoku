import SwiftUI

struct GameView: View {
    @EnvironmentObject var gameManager: GameManager
    @ObservedObject var game: GameViewModel
    @StateObject private var konamiDetector = KonamiCodeDetector()
    @State private var showingKonamiAlert = false
    @State private var konamiMessage = ""
    @State private var celebrationText = ""
    @State private var showCelebration = false
    @State private var heartShake = false
    @State private var lastMistakeCount = 0
    @State private var showingCheckResult = false  // Temporarily reveal errors on "Check"
    #if DEBUG
    @State private var showingDebugMenu = false
    #endif

    var body: some View {
        ZStack {
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

            // Celebration overlay
            if showCelebration {
                CelebrationOverlay(text: celebrationText)
                    .transition(.scale.combined(with: .opacity))
                    .zIndex(100)
            }
        }
        .simultaneousGesture(konamiGesture)
        .onChange(of: konamiDetector.isActivated) { activated in
            if activated {
                triggerKonamiEasterEgg()
            }
        }
        .onChange(of: game.lastCelebration) { celebration in
            if let celebration = celebration {
                handleCelebration(celebration)
            }
        }
        .onChange(of: game.isComplete) { complete in
            if complete {
                // Delay slightly so celebration shows first
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    gameManager.endGame(won: true)
                }
            }
        }
        .onChange(of: game.isGameOver) { gameOver in
            if gameOver {
                gameManager.endGame(won: false)
            }
        }
        .alert("🎮 KONAMI CODE!", isPresented: $showingKonamiAlert) {
            Button("Awesome!") {
                konamiDetector.reset()
            }
        } message: {
            Text(konamiMessage)
        }
        #if DEBUG
        .onLongPressGesture(minimumDuration: 2.0) {
            showingDebugMenu = true
        }
        .confirmationDialog("🔧 Debug Menu", isPresented: $showingDebugMenu, titleVisibility: .visible) {
            Button("Fill Row 1 (except 1 cell)") {
                if let col = game.findEmptyCellInRow(0) {
                    game.fillRowExcept(row: 0, exceptCol: col)
                }
            }
            Button("Fill Column 1 (except 1 cell)") {
                if let row = game.findEmptyCellInColumn(0) {
                    game.fillColumnExcept(col: 0, exceptRow: row)
                }
            }
            Button("Fill Box 1 (except 1 cell)") {
                if let pos = game.findEmptyCellInBox(0) {
                    game.fillBoxExcept(boxIndex: 0, exceptRow: pos.row, exceptCol: pos.col)
                }
            }
            Button("Fill All (leave 3 cells)") {
                game.fillAllExcept(count: 3)
            }
            Button("Fill All (leave 1 cell) - Win Test") {
                game.fillAllExcept(count: 1)
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Long-press for 2s to open.\nSelect a test scenario:")
        }
        #endif
    }

    private func handleCelebration(_ event: CelebrationEvent) {
        guard gameManager.settings.celebrationsEnabled else {
            game.clearCelebration()
            return
        }

        switch event {
        case .rowComplete(let row, let isSequential):
            // Use subtle wiggle instead of overlay
            game.triggerRowCelebration(row)
            // Stronger haptic for sequential completion
            hapticFeedback(isSequential ? .medium : .light)
            if isSequential {
                gameManager.statistics.recordSequentialCompletion()
            }
            game.clearCelebration()
            return
        case .columnComplete(let col, let isSequential):
            // Use subtle wiggle instead of overlay
            game.triggerColumnCelebration(col)
            hapticFeedback(isSequential ? .medium : .light)
            if isSequential {
                gameManager.statistics.recordSequentialCompletion()
            }
            game.clearCelebration()
            return
        case .boxComplete(let box, let isSequential):
            // Use subtle wiggle instead of overlay
            game.triggerBoxCelebration(box)
            hapticFeedback(isSequential ? .medium : .light)
            if isSequential {
                gameManager.statistics.recordSequentialCompletion()
            }
            game.clearCelebration()
            return
        case .gameComplete:
            // Keep the overlay only for game completion
            celebrationText = "🏆 PUZZLE SOLVED! 🏆"
        case .cellComplete:
            // Don't show celebration for individual cells
            game.clearCelebration()
            return
        }

        hapticFeedback(.medium)
        withAnimation(.spring(response: 0.3, dampingFraction: 0.6)) {
            showCelebration = true
        }

        // Auto-dismiss after delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
            withAnimation(.easeOut(duration: 0.3)) {
                showCelebration = false
            }
            game.clearCelebration()
        }
    }

    // MARK: - Konami Code

    private var konamiGesture: some Gesture {
        // Detect swipe directions (minimumDistance set high to reduce tap interference)
        DragGesture(minimumDistance: 50)
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
        // Check if already unlocked
        let alreadyUnlocked = gameManager.statistics.easterEggUnlocked

        if alreadyUnlocked {
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
        } else {
            // First time - unlock Master and Extreme difficulties!
            gameManager.unlockEasterEgg()
            konamiMessage = "🔓 SECRET UNLOCKED!\n\nMaster & Extreme difficulties are now available!\n\n⬆️⬆️⬇️⬇️⬅️➡️⬅️➡️🅱️🅰️"
        }

        showingKonamiAlert = true
        hapticFeedback(.heavy)

        // Unlock Game Center achievement
        GameCenterManager.shared.unlockKonamiAchievement()
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

            // Difficulty + SE rating
            VStack(spacing: 2) {
                Text(game.difficulty.displayName)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Text(String(format: "SE %.1f", game.seRating))
                    .font(.caption2.monospacedDigit())
                    .foregroundStyle(.tertiary)
            }

            Spacer()

            // Mistakes - hearts with animation
            HStack(spacing: 4) {
                ForEach(0..<game.maxMistakes, id: \.self) { i in
                    Image(systemName: i < game.mistakes ? "heart.slash.fill" : "heart.fill")
                        .foregroundStyle(i < game.mistakes ? .red : .pink)
                        .scaleEffect(heartShake && i == game.mistakes - 1 ? 1.3 : 1.0)
                        .opacity(heartShake && i == game.mistakes - 1 ? 0.7 : 1.0)
                }
            }
            .modifier(ShakeEffect(shakes: heartShake ? 4 : 0))
            .animation(.easeInOut(duration: 0.4), value: heartShake)
        }
        .onChange(of: game.mistakes) { newMistakes in
            if newMistakes > lastMistakeCount && gameManager.settings.showErrorsImmediately {
                // Mistake was made - trigger animation and haptic (only if showing errors immediately)
                triggerMistakeFeedback()
            }
            lastMistakeCount = newMistakes
        }
        .onAppear {
            lastMistakeCount = game.mistakes
        }
    }

    private func checkSolution() {
        showingCheckResult = true

        // Check if there are any mistakes
        if game.mistakes > 0 {
            triggerMistakeFeedback()
        } else {
            // No mistakes - provide positive feedback
            hapticFeedback(.medium)
        }

        // Auto-hide check results after a delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
            showingCheckResult = false
        }
    }

    private func triggerMistakeFeedback() {
        // Haptic feedback
        if gameManager.settings.hapticsEnabled {
            UINotificationFeedbackGenerator().notificationOccurred(.error)
        }

        // Visual shake animation
        withAnimation(.easeInOut(duration: 0.1)) {
            heartShake = true
        }

        // Reset after animation
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            withAnimation {
                heartShake = false
            }
        }
    }

    // MARK: - Grid

    private func gridSection(size: CGFloat) -> some View {
        GridView(game: game, size: size, forceShowErrors: showingCheckResult)
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

            // Fill/Clear Notes button
            Menu {
                Button {
                    game.fillAllCandidates()
                    hapticFeedback(.medium)
                } label: {
                    Label("Fill All Notes", systemImage: "square.grid.3x3.fill")
                }

                Button {
                    game.clearAllCandidates()
                    hapticFeedback(.medium)
                } label: {
                    Label("Clear All Notes", systemImage: "square.grid.3x3")
                }

                Divider()

                Button {
                    game.checkNotes()
                    hapticFeedback(.medium)
                } label: {
                    Label("Check Notes", systemImage: "checkmark.circle")
                }
            } label: {
                Image(systemName: "note.text")
            }

            Button {
                game.getHint()
                hapticFeedback(.medium)
            } label: {
                Image(systemName: "lightbulb")
            }

            // Check Solution button (only when not showing errors immediately)
            if !gameManager.settings.showErrorsImmediately {
                Button {
                    checkSolution()
                } label: {
                    Image(systemName: "checkmark.circle")
                }
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

// MARK: - Celebration Overlay

struct CelebrationOverlay: View {
    let text: String
    @State private var scale: CGFloat = 0.5
    @State private var opacity: Double = 0

    var body: some View {
        Text(text)
            .font(.title.bold())
            .foregroundStyle(.white)
            .padding(.horizontal, 24)
            .padding(.vertical, 16)
            .background(
                Capsule()
                    .fill(
                        LinearGradient(
                            colors: [.purple, .pink, .orange],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .shadow(color: .purple.opacity(0.5), radius: 10, x: 0, y: 5)
            )
            .scaleEffect(scale)
            .opacity(opacity)
            .onAppear {
                withAnimation(.spring(response: 0.4, dampingFraction: 0.6)) {
                    scale = 1.0
                    opacity = 1.0
                }
            }
    }
}

// MARK: - Shake Effect

struct ShakeEffect: GeometryEffect {
    var shakes: CGFloat

    var animatableData: CGFloat {
        get { shakes }
        set { shakes = newValue }
    }

    func effectValue(size: CGSize) -> ProjectionTransform {
        let translation = sin(shakes * .pi * 2) * 6
        return ProjectionTransform(CGAffineTransform(translationX: translation, y: 0))
    }
}


#Preview {
    GameView(game: GameViewModel(difficulty: .medium))
        .environmentObject(GameManager())
}

#Preview("Celebration") {
    CelebrationOverlay(text: "🎉 Row 5 Complete!")
}
