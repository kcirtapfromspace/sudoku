import SwiftUI
import Combine

@MainActor
struct GameView: View {
    @EnvironmentObject var gameManager: GameManager
    @ObservedObject var game: GameViewModel
    @StateObject private var presentation: GamePresentation
    private var konamiDetector: KonamiCodeDetector { presentation.konamiDetector }

    init(game: GameViewModel, presentation: GamePresentation? = nil) {
        self.game = game
        let presentation = presentation ?? GamePresentation()
        _presentation = StateObject(wrappedValue: presentation)
    }

    var body: some View {
        ZStack {
            GeometryReader { geometry in
                boardLayout(size: geometry.size)
            }

            // Celebration overlay
            if presentation.showCelebration {
                CelebrationOverlay(text: presentation.celebrationText)
                    .transition(.scale.combined(with: .opacity))
                    .zIndex(100)
            }

            // Completion overlay — let the user admire the board before transitioning
            if presentation.showCompletionOverlay {
                VStack {
                    Spacer()
                    Button {
                        gameManager.endGame(won: true)
                    } label: {
                        Text("Continue")
                            .font(.headline)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 32)
                            .padding(.vertical, 14)
                            .background(
                                Capsule()
                                    .fill(
                                        LinearGradient(
                                            colors: [.purple, .pink],
                                            startPoint: .leading,
                                            endPoint: .trailing
                                        )
                                    )
                                    .shadow(color: .purple.opacity(0.4), radius: 8, x: 0, y: 4)
                            )
                    }
                    .padding(.bottom, 60)
                }
                .transition(.move(edge: .bottom).combined(with: .opacity))
                .zIndex(99)
            }
        }
        .onAppear {
            if game.isComplete { presentation.revealCompletion() }
        }
        .onDisappear {
            presentation.cancelPendingEffects()
            game.clearCelebration()
            game.celebratingCells.removeAll()
        }
        .simultaneousGesture(konamiGesture)
        .onChange(of: konamiDetector.isActivated) { activated in
            if activated {
                presentation.triggerKonamiEasterEgg(manager: gameManager)
            }
        }
        .onChange(of: game.lastCelebration) { celebration in
            if let celebration = celebration {
                presentation.handleCelebration(celebration, game: game, manager: gameManager)
            }
        }
        .onChange(of: game.isComplete) { complete in
            if complete {
                presentation.revealCompletion()
            }
        }
        .onChange(of: game.isGameOver) { gameOver in
            if gameOver {
                gameManager.endGame(won: false)
            }
        }
        .alert("🎮 KONAMI CODE!", isPresented: $presentation.showingKonamiAlert) {
            Button("Awesome!") {
                konamiDetector.reset()
            }
        } message: {
            Text(presentation.konamiMessage)
        }
        #if DEBUG
        .onLongPressGesture(minimumDuration: 2.0) {
            presentation.showingDebugMenu = true
        }
        .confirmationDialog("🔧 Debug Menu", isPresented: $presentation.showingDebugMenu, titleVisibility: .visible) {
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

    @ViewBuilder
    func boardLayout(size: CGSize) -> some View {
        if size.width > size.height {
            let contentWidth = max(0, size.width - 32)
            let boardSize = max(0, min(size.height - 32, (contentWidth - 20) * 0.48))
            let controlsWidth = max(0, contentWidth - boardSize - 20)
            HStack(spacing: 20) {
                gridSection(size: boardSize)
                ScrollView {
                    VStack(spacing: 12) {
                        headerSection
                        if let hint = game.currentHint {
                            HintPanelView(
                                hint: hint,
                                detailLevel: game.hintDetailLevel,
                                onUpgrade: { game.getHint() },
                                onDismiss: { game.clearHint() }
                            )
                        }
                        numberPadSection
                        controlsSection(compact: true)
                    }
                }
                .frame(width: controlsWidth)
            }
            .padding()
        } else {
            // Portrait layout
            let gridSize = max(0, min(size.width - 32, size.height * 0.55))
            VStack(spacing: 16) {
                headerSection
                gridSection(size: gridSize)
                if let hint = game.currentHint {
                    HintPanelView(
                        hint: hint,
                        detailLevel: game.hintDetailLevel,
                        onUpgrade: { game.getHint() },
                        onDismiss: { game.clearHint() }
                    )
                }
                Spacer(minLength: 8)
                numberPadSection
                controlsSection(compact: false)
            }
            .padding()
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

    // MARK: - Header

    private var headerSection: some View {
        HStack {
            // Close / menu button
            Button { gameManager.pauseGame() } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.title3)
                    .symbolRenderingMode(.hierarchical)
                    .foregroundStyle(.secondary)
            }
            .disabled(game.isComplete)
            .accessibilityLabel("Pause game")
            .accessibilityIdentifier("PauseGame")

            // Timer
            if gameManager.settings.timerVisible {
                Label(game.elapsedTimeString, systemImage: "clock")
                    .font(.headline.monospacedDigit())
            }

            Spacer()

            // Share button
            Button {
                presentation.showingShareSheet = true
            } label: {
                Image(systemName: "square.and.arrow.up")
                    .font(.body)
            }
            .accessibilityLabel("Share puzzle")
            .accessibilityIdentifier("SharePuzzle")
            .sheet(isPresented: $presentation.showingShareSheet) {
                QRCodeView(puzzleString: game.getPuzzleFingerprint(), shortCode: game.getShortCode())
                    .presentationDetents([.medium, .large])
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
                        .scaleEffect(presentation.heartShake && i == game.mistakes - 1 ? 1.3 : 1.0)
                        .opacity(presentation.heartShake && i == game.mistakes - 1 ? 0.7 : 1.0)
                }
            }
            .modifier(ShakeEffect(shakes: presentation.heartShake ? 4 : 0))
            .animation(.easeInOut(duration: 0.4), value: presentation.heartShake)
        }
        .onChange(of: game.mistakes) { newMistakes in
            presentation.updateMistakes(newMistakes, manager: gameManager)
        }
        .onAppear {
            presentation.lastMistakeCount = game.mistakes
        }
    }

    private func checkSolution() {
        presentation.checkSolution(game: game, manager: gameManager)
    }

    // MARK: - Grid

    private func gridSection(size: CGFloat) -> some View {
        GridView(game: game, size: size, forceShowErrors: presentation.showingCheckResult)
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
                // Keep actions beside the board without displacing digit entry.
                HStack(spacing: 8) {
                    controlButtons
                }
                modeToggle
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
            .accessibilityLabel("Undo")
            .accessibilityIdentifier("Undo")

            Button {
                game.redo()
                hapticFeedback(.light)
            } label: {
                Image(systemName: "arrow.uturn.forward")
            }
            .disabled(!game.canRedo)
            .accessibilityLabel("Redo")
            .accessibilityIdentifier("Redo")

            Button {
                game.clearSelectedCell()
                hapticFeedback(.light)
            } label: {
                Image(systemName: "delete.left")
            }
            .accessibilityLabel("Erase cell")
            .accessibilityIdentifier("EraseCell")

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
            .accessibilityLabel("Manage notes")

            Button {
                game.getHint()
                hapticFeedback(.medium)
            } label: {
                Image(systemName: game.currentHint != nil ? "lightbulb.fill" : "lightbulb")
            }
            .accessibilityLabel(game.currentHint == nil ? "Hint" : "Hint details")
            .accessibilityIdentifier("Hint")

            // Check Solution button (only when not showing errors immediately)
            if !gameManager.settings.showErrorsImmediately {
                Button {
                    checkSolution()
                } label: {
                    Image(systemName: "checkmark.circle")
                }
                .accessibilityLabel("Check solution")
                .accessibilityIdentifier("CheckSolution")
            }

            Button {
                gameManager.pauseGame()
            } label: {
                Image(systemName: "pause")
            }
            .accessibilityLabel("Pause")
            .accessibilityIdentifier("Pause")
        }
        .font(.body)
        .imageScale(.medium)
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
        .accessibilityLabel("Notes")
        .accessibilityValue(game.inputMode == .candidate ? "On" : "Off")
        .accessibilityIdentifier("NotesMode")
    }

    // MARK: - Haptics

    private func hapticFeedback(_ style: UIImpactFeedbackGenerator.FeedbackStyle) {
        presentation.hapticFeedback(style, enabled: gameManager.settings.hapticsEnabled)
    }


}

/// Transient gameplay effects have one owner so leaving a game cancels pending work.
@MainActor
final class GamePresentation: ObservableObject {
    struct Effects {
        var sleep: @MainActor (TimeInterval) async throws -> Void = { seconds in
            try await Task.sleep(nanoseconds: UInt64(max(0, seconds) * 1_000_000_000))
        }
        var impact: @MainActor (UIImpactFeedbackGenerator.FeedbackStyle) -> Void = {
            UIImpactFeedbackGenerator(style: $0).impactOccurred()
        }
        var notification: @MainActor (UINotificationFeedbackGenerator.FeedbackType) -> Void = {
            UINotificationFeedbackGenerator().notificationOccurred($0)
        }
        var unlockKonami: @MainActor () -> Void = { GameCenterManager.shared.unlockKonamiAchievement() }
        var chooseMessage: ([String]) -> String = { $0.randomElement() ?? "You did it!" }
    }

    let konamiDetector = KonamiCodeDetector()
    @Published var showingKonamiAlert = false
    @Published var konamiMessage = ""
    @Published var celebrationText = ""
    @Published var showCelebration = false
    @Published var heartShake = false
    @Published var showingCheckResult = false
    @Published var showingShareSheet = false
    @Published var showCompletionOverlay = false
    #if DEBUG
    @Published var showingDebugMenu = false
    #endif
    var lastMistakeCount = 0
    private let effects: Effects
    private var konamiObservation: AnyCancellable?
    private enum PendingEffect { case celebration, completion, mistake, check }
    private var pending: [PendingEffect: Task<Void, Never>] = [:]

    init(effects: Effects = Effects()) {
        self.effects = effects
        konamiObservation = konamiDetector.objectWillChange.sink { [weak self] in
            self?.objectWillChange.send()
        }
    }

    deinit { pending.values.forEach { $0.cancel() } }

    @discardableResult
    private func schedule(_ effect: PendingEffect, after seconds: TimeInterval,
                          action: @escaping @MainActor (GamePresentation) -> Void) -> Task<Void, Never> {
        pending[effect]?.cancel()
        let sleep = effects.sleep
        let task = Task { [weak self] in
            do { try await sleep(seconds) } catch { return }
            guard !Task.isCancelled, let self else { return }
            action(self)
            self.pending[effect] = nil
        }
        pending[effect] = task
        return task
    }

    func cancelPendingEffects() {
        pending.values.forEach { $0.cancel() }
        pending.removeAll()
        showCelebration = false
        showCompletionOverlay = false
        showingCheckResult = false
        heartShake = false
    }

    @discardableResult
    func revealCompletion() -> Task<Void, Never> {
        schedule(.completion, after: 1.5) { state in
            withAnimation(.spring(response: 0.4, dampingFraction: 0.8)) { state.showCompletionOverlay = true }
        }
    }

    @discardableResult
    func handleCelebration(_ event: CelebrationEvent, game: GameViewModel, manager: GameManager) -> Task<Void, Never>? {
        guard manager.settings.celebrationsEnabled else { game.clearCelebration(); return nil }
        switch event {
        case .rowComplete(let row, let sequential):
            game.triggerRowCelebration(row)
            successHaptic(enabled: manager.settings.hapticsEnabled)
            if sequential { manager.statistics.recordSequentialCompletion() }
            game.clearCelebration()
            return nil
        case .columnComplete(let col, let sequential):
            game.triggerColumnCelebration(col)
            successHaptic(enabled: manager.settings.hapticsEnabled)
            if sequential { manager.statistics.recordSequentialCompletion() }
            game.clearCelebration()
            return nil
        case .boxComplete(let box, let sequential):
            game.triggerBoxCelebration(box)
            successHaptic(enabled: manager.settings.hapticsEnabled)
            if sequential { manager.statistics.recordSequentialCompletion() }
            game.clearCelebration()
            return nil
        case .cellComplete:
            game.clearCelebration()
            return nil
        case .gameComplete:
            celebrationText = "🏆 PUZZLE SOLVED! 🏆"
        }
        hapticFeedback(.medium, enabled: manager.settings.hapticsEnabled)
        withAnimation(.spring(response: 0.3, dampingFraction: 0.6)) { showCelebration = true }
        return schedule(.celebration, after: 1.2) { state in
            withAnimation(.easeOut(duration: 0.3)) { state.showCelebration = false }
            game.clearCelebration()
        }
    }

    @discardableResult
    func checkSolution(game: GameViewModel, manager: GameManager) -> Task<Void, Never> {
        showingCheckResult = true
        if game.mistakes > 0 { triggerMistakeFeedback(enabled: manager.settings.hapticsEnabled) }
        else { hapticFeedback(.medium, enabled: manager.settings.hapticsEnabled) }
        return schedule(.check, after: 2) { $0.showingCheckResult = false }
    }

    func updateMistakes(_ count: Int, manager: GameManager) {
        if count > lastMistakeCount && manager.settings.showErrorsImmediately {
            triggerMistakeFeedback(enabled: manager.settings.hapticsEnabled)
        }
        lastMistakeCount = count
    }

    @discardableResult
    func triggerMistakeFeedback(enabled: Bool) -> Task<Void, Never> {
        if enabled { effects.notification(.error) }
        withAnimation(.easeInOut(duration: 0.1)) { heartShake = true }
        return schedule(.mistake, after: 0.5) { state in
            withAnimation { state.heartShake = false }
        }
    }

    func triggerKonamiEasterEgg(manager: GameManager) {
        if manager.statistics.easterEggUnlocked {
            konamiMessage = effects.chooseMessage([
                "🚀 +30 extra lives! (Just kidding, you only had 3)",
                "🎯 God mode activated! (Your mistakes still count though)",
                "🧠 IQ temporarily boosted to 9000!",
                "🎮 You found the secret! Here's a virtual high-five: 🖐️",
                "🔮 The puzzle whispers its secrets to you...",
                "⬆️⬆️⬇️⬇️⬅️➡️⬅️➡️🅱️🅰️ - A true gamer!",
                "🏆 Achievement Unlocked: Nostalgia Master",
                "🎪 Circus mode engaged! 🤹‍♂️ (Nothing changed, but imagine it did)"
            ])
        } else {
            manager.unlockEasterEgg()
            konamiMessage = "🔓 SECRET UNLOCKED!\n\nMaster & Extreme difficulties are now available!\n\n⬆️⬆️⬇️⬇️⬅️➡️⬅️➡️🅱️🅰️"
        }
        showingKonamiAlert = true
        hapticFeedback(.heavy, enabled: manager.settings.hapticsEnabled)
        effects.unlockKonami()
    }

    func hapticFeedback(_ style: UIImpactFeedbackGenerator.FeedbackStyle, enabled: Bool) {
        guard enabled else { return }
        effects.impact(style)
    }

    func successHaptic(enabled: Bool) {
        guard enabled else { return }
        effects.notification(.success)
    }
}

// MARK: - Hint Panel

struct HintPanelView: View {
    let hint: HintModel
    let detailLevel: HintDetailLevel
    let onUpgrade: () -> Void
    let onDismiss: () -> Void

    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(hint.technique)
                    .font(.headline)
                    .foregroundStyle(.green)
                Text(String(format: "SE %.1f", hint.seRating))
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.secondary)
                Spacer()
                if detailLevel == .summary {
                    Button("Details") { onUpgrade() }
                        .font(.caption)
                } else {
                    Text("Proof shown")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Button { onDismiss() } label: {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundStyle(.secondary)
                }
            }
            Text(hint.explanation)
                .accessibilityIdentifier("HintExplanation")
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(isExpanded ? nil : 3)
                .onTapGesture {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        isExpanded.toggle()
                    }
                }
            Button {
                withAnimation(.easeInOut(duration: 0.2)) {
                    isExpanded.toggle()
                }
            } label: {
                Text(isExpanded ? "Show less" : "Show more")
                    .font(.caption2)
                    .foregroundStyle(Color.accentColor)
            }
        }
        .padding(12)
        .background(RoundedRectangle(cornerRadius: 8).fill(.ultraThinMaterial))
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
