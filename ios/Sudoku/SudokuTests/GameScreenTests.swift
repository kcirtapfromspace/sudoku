import XCTest
import SwiftUI
import ViewInspector
@testable import Sudoku

@MainActor
final class GameScreenTests: XCTestCase {
    private func state() -> GamePresentation {
        var effects = GamePresentation.Effects()
        effects.sleep = { _ in }
        effects.impact = { _ in }
        effects.notification = { _ in }
        effects.unlockKonami = {}
        effects.chooseMessage = { $0[0] }
        return GamePresentation(effects: effects)
    }
    private func button<V: View>(_ id: String, in view: V) throws -> InspectableView<ViewType.Button> {
        try view.inspect().find(ViewType.Button.self, where: { try $0.accessibilityIdentifier() == id })
    }
    func testPortraitAndLandscapeKeepAllDigitsActionsAndHintDetails() throws {
        let manager = ViewTestFixture.manager(), game = try ViewTestFixture.game(), state = state()
        manager.currentGame = game
        let view = GameView(game: game, presentation: state).environmentObject(manager)
        let actual = try view.inspect().find(GameView.self).actualView()
        for size in [CGSize(width: 390, height: 844), CGSize(width: 844, height: 390), CGSize(width: 568, height: 320)] {
            let layout = actual.boardLayout(size: size).environmentObject(manager)
            XCTAssertEqual(try layout.inspect().findAll(GridView.self).count, 1)
            XCTAssertEqual(try layout.inspect().findAll(NumberPadView.self).count, 1)
            for digit in 1...9 { XCTAssertNoThrow(try button("Digit\(digit)", in: layout)) }
            XCTAssertNoThrow(try button("Pause", in: layout))
            game.getHint()
            let hinted = actual.boardLayout(size: size).environmentObject(manager)
            let hint = try hinted.inspect().find(HintPanelView.self).actualView()
            XCTAssertNotNil(game.currentHint)
            hint.onUpgrade(); XCTAssertEqual(game.hintDetailLevel, .proofDetail)
            hint.onDismiss(); XCTAssertNil(game.currentHint)
            try ViewTestFixture.render(layout, size: size)
        }
        manager.settings.timerVisible = false
        let labels = try view.inspect().findAll(ViewType.Label.self)
        XCTAssertFalse(labels.contains { (try? $0.title().text().string()) == game.elapsedTimeString })
    }
    func testUndoRedoEraseNotesHintCheckAndPauseCallbacks() async throws {
        let manager = ViewTestFixture.manager(), game = try ViewTestFixture.game(), state = state()
        manager.currentGame = game; manager.gameState = .playing
        game.selectCell(row: 0, col: 2)
        let view = GameView(game: game, presentation: state).environmentObject(manager)
        XCTAssertTrue(try button("Undo", in: view).isDisabled())
        try button("Digit4", in: view).tap(); XCTAssertEqual(game.cells[0][2].value, 4)
        try button("Undo", in: view).tap(); XCTAssertEqual(game.cells[0][2].value, 0)
        try button("Redo", in: view).tap(); XCTAssertEqual(game.cells[0][2].value, 4)
        try button("EraseCell", in: view).tap(); XCTAssertEqual(game.cells[0][2].value, 0)
        try view.inspect().find(button: "Fill All Notes").tap(); XCTAssertTrue(game.showCandidates)
        try view.inspect().find(button: "Check Notes").tap()
        try view.inspect().find(button: "Clear All Notes").tap(); XCTAssertFalse(game.showCandidates)
        try button("NotesMode", in: view).tap(); XCTAssertEqual(game.inputMode, .candidate)
        try button("NotesMode", in: view).tap(); XCTAssertEqual(game.inputMode, .normal)
        try button("Hint", in: view).tap(); XCTAssertNotNil(game.currentHint)
        manager.settings.showErrorsImmediately = false
        try button("CheckSolution", in: view).tap(); XCTAssertTrue(state.showingCheckResult)
        XCTAssertTrue(try view.inspect().find(GridView.self).actualView().forceShowErrors)
        await state.checkSolution(game: game, manager: manager).value; XCTAssertFalse(state.showingCheckResult)
        try button("Pause", in: view).tap(); XCTAssertEqual(manager.gameState, .paused)
        manager.resumeGame()
        try button("PauseGame", in: view).tap(); XCTAssertEqual(manager.gameState, .paused)
    }
    func testShareCompletionAndDebugControls() throws {
        let manager = ViewTestFixture.manager(), game = try ViewTestFixture.game(), state = state()
        manager.currentGame = game; manager.gameState = .playing
        let view = GameView(game: game, presentation: state).environmentObject(manager)
        try button("SharePuzzle", in: view).tap(); XCTAssertTrue(state.showingShareSheet)
        try view.inspect().find(ViewType.ZStack.self).callOnLongPressGesture()
        XCTAssertTrue(state.showingDebugMenu)
        let dialog = try view.inspect().find(ViewType.ZStack.self).confirmationDialog()
        XCTAssertTrue(try dialog.message().text().string().contains("test scenario"))
        for label in ["Fill Row 1 (except 1 cell)", "Fill Column 1 (except 1 cell)", "Fill Box 1 (except 1 cell)", "Fill All (leave 3 cells)", "Fill All (leave 1 cell) - Win Test", "Cancel"] {
            try dialog.actions().find(button: label).tap()
        }
        XCTAssertEqual(game.cells.flatMap { $0 }.filter { $0.value == 0 }.count, 1)
        let solved = GameViewModel(cachedGame: gameFromString(puzzle: ViewTestFixture.solution)!, difficulty: .medium)
        manager.currentGame = solved
        state.showCompletionOverlay = true; state.showCelebration = true; state.celebrationText = "Solved"
        let finished = GameView(game: solved, presentation: state).environmentObject(manager)
        XCTAssertTrue(try button("PauseGame", in: finished).isDisabled())
        XCTAssertNoThrow(try finished.inspect().find(CelebrationOverlay.self))
        try finished.inspect().find(button: "Continue").tap(); XCTAssertEqual(manager.gameState, .won)
        state.showingDebugMenu = true
        let completedDialog = try finished.inspect().find(ViewType.ZStack.self).confirmationDialog()
        for label in ["Fill Row 1 (except 1 cell)", "Fill Column 1 (except 1 cell)", "Fill Box 1 (except 1 cell)"] {
            try completedDialog.actions().find(button: label).tap()
        }
    }
    func testAlreadyCompletedImportedBoardShowsContinueWhenItAppears() async throws {
        let completed = expectation(description: "completion delay scheduled")
        var effects = GamePresentation.Effects()
        effects.sleep = { delay in XCTAssertEqual(delay, 1.5); completed.fulfill() }
        let presentation = GamePresentation(effects: effects)
        let game = GameViewModel(cachedGame: gameFromString(puzzle: ViewTestFixture.solution)!, difficulty: .medium)
        let manager = ViewTestFixture.manager()
        manager.currentGame = game
        let view = GameView(game: game, presentation: presentation).environmentObject(manager)
        try view.inspect().find(ViewType.ZStack.self).callOnAppear()
        await fulfillment(of: [completed], timeout: 1)
        XCTAssertTrue(presentation.showCompletionOverlay)
        let ordinary = GameView(game: try ViewTestFixture.game(), presentation: state()).environmentObject(manager)
        try ordinary.inspect().find(ViewType.ZStack.self).callOnAppear()
    }
    func testSwipeSequenceNumberInputsKonamiAlertAndObservedEvents() async throws {
        let manager = ViewTestFixture.manager(), game = try ViewTestFixture.game(), state = state()
        manager.currentGame = game; manager.gameState = .playing
        let view = GameView(game: game, presentation: state).environmentObject(manager)
        let root = try view.inspect().find(ViewType.ZStack.self)
        let gesture = try root.simultaneousGesture(DragGesture.self)
        for point in [CGPoint(x: 0, y: -60), CGPoint(x: 0, y: -60), CGPoint(x: 0, y: 60), CGPoint(x: 0, y: 60), CGPoint(x: -60, y: 0), CGPoint(x: 60, y: 0), CGPoint(x: -60, y: 0), CGPoint(x: 60, y: 0)] {
            try gesture.callOnEnded(value: DragGesture.Value(time: Date(), location: point, startLocation: .zero, velocity: .zero))
        }
        let actual = try view.inspect().find(GameView.self).actualView()
        actual.handleKonamiNumberPad(9)
        actual.handleKonamiNumberPad(2); actual.handleKonamiNumberPad(1)
        XCTAssertTrue(state.konamiDetector.isActivated)
        for index in 0..<3 { try root.callOnChange(newValue: false, index: index) }
        game.selectCell(row: 0, col: 2)
        for wrong in 1...3 { game.enterNumber(wrong) }
        XCTAssertTrue(game.isGameOver)
        for index in 0..<3 { try root.callOnChange(newValue: true, index: index) }
        XCTAssertTrue(manager.statistics.easterEggUnlocked)
        XCTAssertEqual(manager.gameState, .lost)
        try root.callOnChange(newValue: Optional<CelebrationEvent>.none)
        try root.callOnChange(newValue: Optional.some(CelebrationEvent.cellComplete(row: 0, col: 0)))
        XCTAssertTrue(state.showingKonamiAlert)
        let alert = try view.inspect().find(ViewType.ZStack.self).alert()
        XCTAssertTrue(try alert.message().text().string().contains("SECRET UNLOCKED"))
        try alert.actions().find(button: "Awesome!").tap(); XCTAssertFalse(state.konamiDetector.isActivated)
        await state.revealCompletion().value; XCTAssertTrue(state.showCompletionOverlay)
        try root.callOnDisappear(); XCTAssertFalse(state.showCompletionOverlay)
        XCTAssertTrue(game.celebratingCells.isEmpty)
    }
    func testCelebrationEventsMistakeFeedbackAndCheckDurations() async throws {
        let game = try ViewTestFixture.game(), manager = ViewTestFixture.manager()
        manager.currentGame = game
        var delays: [TimeInterval] = [], impacts: [UIImpactFeedbackGenerator.FeedbackStyle] = [], notifications: [UINotificationFeedbackGenerator.FeedbackType] = []
        var effects = GamePresentation.Effects()
        effects.sleep = { delays.append($0) }
        effects.impact = { impacts.append($0) }; effects.notification = { notifications.append($0) }; effects.unlockKonami = {}
        let state = GamePresentation(effects: effects)
        XCTAssertNil(state.handleCelebration(.gameComplete, game: game, manager: manager))
        manager.settings.celebrationsEnabled = true; manager.settings.hapticsEnabled = true
        for sequential in [false, true] {
            for event in [CelebrationEvent.rowComplete(row: 0, isSequential: sequential), .columnComplete(col: 0, isSequential: sequential), .boxComplete(boxIndex: 0, isSequential: sequential)] {
                XCTAssertNil(state.handleCelebration(event, game: game, manager: manager))
                XCTAssertFalse(game.celebratingCells.isEmpty)
            }
        }
        XCTAssertEqual(manager.statistics.sequentialCompletions, 3)
        XCTAssertEqual(notifications.filter { $0 == .success }.count, 6)
        XCTAssertNil(state.handleCelebration(.cellComplete(row: 0, col: 0), game: game, manager: manager))
        let celebration = state.handleCelebration(.gameComplete, game: game, manager: manager)
        XCTAssertTrue(state.showCelebration); XCTAssertTrue(state.celebrationText.contains("PUZZLE SOLVED"))
        await celebration?.value; XCTAssertFalse(state.showCelebration)
        await state.checkSolution(game: game, manager: manager).value; XCTAssertFalse(state.showingCheckResult)
        game.selectCell(row: 0, col: 2); game.enterNumber(1); XCTAssertEqual(game.mistakes, 1)
        let view = GameView(game: game, presentation: state).environmentObject(manager)
        let header = try view.inspect().find(ViewType.HStack.self, where: { (try? $0.callOnAppear()) != nil })
        XCTAssertEqual(state.lastMistakeCount, 1)
        try header.callOnChange(newValue: 2); XCTAssertTrue(state.heartShake)
        try header.callOnChange(newValue: 1)
        manager.settings.showErrorsImmediately = false
        state.updateMistakes(3, manager: manager); XCTAssertEqual(state.lastMistakeCount, 3)
        await state.checkSolution(game: game, manager: manager).value
        await state.triggerMistakeFeedback(enabled: true).value; XCTAssertFalse(state.heartShake)
        XCTAssertTrue(notifications.contains(.error)); XCTAssertTrue(impacts.contains(.medium))
        XCTAssertTrue(delays.contains(1.2)); XCTAssertTrue(delays.contains(2)); XCTAssertTrue(delays.contains(0.5))
        state.successHaptic(enabled: false); state.hapticFeedback(.light, enabled: false)
        await state.triggerMistakeFeedback(enabled: false).value
    }
    func testEffectsCancelWhenLeavingOrReplacingAndDoNotRetainScreen() async {
        var effects = GamePresentation.Effects()
        effects.sleep = { _ in try await Task.sleep(nanoseconds: 10_000_000_000) }
        let state = GamePresentation(effects: effects)
        let first = state.revealCompletion(), second = state.revealCompletion()
        XCTAssertTrue(first.isCancelled)
        state.cancelPendingEffects()
        await first.value; await second.value
        XCTAssertFalse(state.showCompletionOverlay)
        var temporary: GamePresentation? = GamePresentation(effects: effects)
        weak var reference = temporary
        let task = temporary!.revealCompletion()
        temporary = nil
        await task.value
        XCTAssertNil(reference)
        reference = nil
        effects.sleep = { _ in throw CancellationError() }
        let cancelled = GamePresentation(effects: effects)
        await cancelled.revealCompletion().value; XCTAssertFalse(cancelled.showCompletionOverlay)
    }
    func testKonamiUnlockAndRepeatedMessageAndLiveEffectAdapters() async {
        let manager = ViewTestFixture.manager()
        var unlocks = 0
        var effects = GamePresentation.Effects()
        effects.unlockKonami = { unlocks += 1 }; effects.chooseMessage = { $0[2] }
        let state = GamePresentation(effects: effects)
        state.triggerKonamiEasterEgg(manager: manager)
        XCTAssertTrue(manager.statistics.easterEggUnlocked); XCTAssertTrue(state.konamiMessage.contains("Master & Extreme"))
        state.triggerKonamiEasterEgg(manager: manager)
        XCTAssertTrue(state.konamiMessage.contains("9000")); XCTAssertEqual(unlocks, 2)
        let live = GamePresentation()
        live.hapticFeedback(.light, enabled: true); live.successHaptic(enabled: true)
        live.triggerKonamiEasterEgg(manager: manager)
        await live.revealCompletion().value; XCTAssertTrue(live.showCompletionOverlay)
    }
    func testCelebrationAppearanceAndShakeInterpolation() throws {
        let celebration = CelebrationOverlay(text: "Solved")
        try celebration.inspect().find(ViewType.Text.self).callOnAppear()
        try ViewTestFixture.render(celebration, size: CGSize(width: 320, height: 80))
        var shake = ShakeEffect(shakes: 0)
        shake.animatableData = 0.25; XCTAssertEqual(shake.animatableData, 0.25)
        XCTAssertEqual(shake.effectValue(size: .zero), ProjectionTransform(CGAffineTransform(translationX: 6, y: 0)))
    }
}
