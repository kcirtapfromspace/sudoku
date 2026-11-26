import XCTest
import SwiftUI
import Combine
import ViewInspector
@testable import Sudoku

@MainActor
final class ContentNavigationTests: XCTestCase {
    func testOneSliderStartsTheExactDisplayedRatingAtEveryUnlockedStop() throws {
        let manager = ViewTestFixture.manager()
        manager.statistics.easterEggUnlocked = true
        let choices = manager.statistics.availableDifficultyChoices
        var played: [Float] = []
        let view = NewGameSelection(choices: choices, preferredRating: Binding(
            get: { manager.preferredNewGameRating }, set: { manager.preferredNewGameRating = $0 }
        )) { played.append($0) }

        XCTAssertEqual(try view.inspect().findAll(ViewType.Slider.self).count, 1)
        for (index, choice) in choices.enumerated() {
            let slider = try view.inspect().find(ViewType.Slider.self)
            try slider.setValue(Double(index) / Double(choices.count - 1))
            XCTAssertEqual(manager.preferredNewGameRating, choice.rating)
            XCTAssertNoThrow(try view.inspect().find(text: choice.ratingLabel))
            XCTAssertEqual(played.count, index)
            try view.inspect().find(button: "New Game").tap()
            XCTAssertEqual(played.last, choice.rating)
        }
        XCTAssertEqual(played.last, 11)
    }

    func testLockedRememberedScoreStartsAtHighestAvailableStop() throws {
        let manager = ViewTestFixture.manager()
        var played: Float?
        let view = NewGameSelection(choices: manager.statistics.availableDifficultyChoices,
                                    preferredRating: .constant(11)) { played = $0 }
        try view.inspect().find(button: "New Game").tap()
        XCTAssertEqual(played, 5.4)
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
