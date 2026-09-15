import XCTest
import SwiftUI
import Combine
import ViewInspector
@testable import Sudoku

@MainActor
final class ContentNavigationTests: XCTestCase {
    func testDifficultySelectionOffersItsRatingRangeAndStartsChosenRating() throws {
        let manager = ViewTestFixture.manager()
        manager.statistics.easterEggUnlocked = true
        let selection = NewGameSelection()
        var played: [Float] = []
        let view = NewGamePickerView(selection: selection, onPlay: { played.append($0) }).environmentObject(manager)
        XCTAssertEqual(try view.inspect().findAll(ViewType.Slider.self).count, 0)
        for difficulty in Difficulty.allCases {
            let row = try view.inspect().find(ViewType.Button.self, where: { (try? $0.labelView().find(text: difficulty.displayName)) != nil })
            try row.tap()
            XCTAssertEqual(selection.expanded, difficulty)
            XCTAssertEqual(selection.targetSE, difficulty.defaultSE)
            let slider = try view.inspect().find(ViewType.Slider.self)
            try slider.setValue(Double(difficulty.seRange.upperBound))
            XCTAssertEqual(selection.targetSE, difficulty.seRange.upperBound, accuracy: 0.0001)
            let play = try view.inspect().find(ViewType.Button.self, where: { (try? $0.labelView().find(ViewType.Text.self).string().hasPrefix("Play (SE")) == true })
            try play.tap()
            XCTAssertEqual(try XCTUnwrap(played.last), difficulty.seRange.upperBound, accuracy: 0.0001)
            try row.tap()
            XCTAssertEqual(try XCTUnwrap(played.last), difficulty.seRange.upperBound, accuracy: 0.0001)
        }
        XCTAssertEqual(played.count, Difficulty.allCases.count * 2)
        try view.inspect().find(button: "Cancel").tap()
        try ViewTestFixture.render(view)
    }

    func testResultScreensDismissAndRetryThroughTheManager() async throws {
        let manager = ViewTestFixture.manager()
        manager.currentGame = try ViewTestFixture.game()
        manager.gameState = .won
        let view = ContentView().environmentObject(manager)
        let win = try view.inspect().find(WinScreenView.self).actualView()
        win.onDismiss()
        XCTAssertEqual(manager.gameState, .menu)
        XCTAssertNil(manager.currentGame)
        manager.currentGame = try ViewTestFixture.game()
        manager.gameState = .lost
        let loss = try view.inspect().find(LoseScreenView.self).actualView()
        loss.onRetry()
        await waitForPlaying(manager)
        XCTAssertEqual(manager.gameState, .playing)
        XCTAssertNotNil(manager.currentGame)
        manager.gameState = .lost
        try view.inspect().find(LoseScreenView.self).actualView().onDismiss()
        XCTAssertEqual(manager.gameState, .menu)
    }

    func testLegacyResultOverlayActionsPreserveTheirGameplayBehavior() async throws {
        let manager = ViewTestFixture.manager()
        let game = try ViewTestFixture.game()
        manager.currentGame = game
        let win = WinOverlay(game: game).environmentObject(manager)
        XCTAssertNoThrow(try win.inspect().find(text: "PUZZLE COMPLETE!"))
        try win.inspect().find(button: "New Game").tap()
        await waitForPlaying(manager)
        XCTAssertEqual(manager.gameState, .playing)
        try win.inspect().find(button: "Main Menu").tap()
        XCTAssertEqual(manager.gameState, .menu)
        let loss = LoseOverlay(game: game).environmentObject(manager)
        XCTAssertNoThrow(try loss.inspect().find(text: "GAME OVER"))
        try loss.inspect().find(button: "Try Again").tap()
        await waitForPlaying(manager)
        XCTAssertEqual(manager.gameState, .playing)
        try loss.inspect().find(button: "Main Menu").tap()
        XCTAssertEqual(manager.gameState, .menu)
        try ViewTestFixture.render(win)
        try ViewTestFixture.render(loss)
    }

    func testContentHandlesMissingGameAcrossPlayableAndResultStates() throws {
        let manager = ViewTestFixture.manager()
        for state in [GameState.playing, .paused, .won, .lost] {
            manager.gameState = state
            let view = ContentView().environmentObject(manager)
            XCTAssertTrue(try view.inspect().findAll(GameView.self).isEmpty)
            XCTAssertTrue(try view.inspect().findAll(WinScreenView.self).isEmpty)
            XCTAssertTrue(try view.inspect().findAll(LoseScreenView.self).isEmpty)
        }
    }
    private func waitForPlaying(_ manager: GameManager) async {
        let ready = expectation(description: "Generated puzzle enters playing state")
        let subscription = manager.$gameState.filter { $0 == .playing }.first().sink { _ in ready.fulfill() }
        await fulfillment(of: [ready], timeout: 2)
        withExtendedLifetime(subscription) {}
    }

}
