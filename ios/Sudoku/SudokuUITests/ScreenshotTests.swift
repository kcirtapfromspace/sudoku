import XCTest

/// Captures only after asserting each important screen/interaction.
final class ScreenshotTests: XCTestCase {
    private var app: XCUIApplication!
    private let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
    private let almostComplete = "034678912672195348198342567859761423426853791713924856961537284287419635345286179"

    override func setUpWithError() throws {
        continueAfterFailure = false
        XCUIDevice.shared.orientation = .portrait
        app = XCUIApplication()
    }

    override func tearDownWithError() throws {
        app.terminate()
        XCUIDevice.shared.orientation = .portrait
    }

    private func launch(puzzle: String? = nil, reset: Bool = true) {
        app.launchArguments = ["--ui-testing"]
        if reset { app.launchArguments.append("--reset-state") }
        if let puzzle { app.launchArguments += ["--puzzle", puzzle] }
        app.launch()
    }

    func testCaptureAllScreenshots() {
        launch(puzzle: puzzle)
        XCTAssertTrue(app.buttons["Cell_0_2"].waitForExistence(timeout: 10))
        capture("Gameplay Portrait")
        app.buttons["Cell_0_2"].tap()
        app.buttons["NotesMode"].tap()
        app.buttons["Digit2"].tap()
        XCTAssertTrue((app.buttons["Cell_0_2"].value as? String)?.contains("Notes 2") == true)
        capture("Manual Notes")
        app.buttons["SharePuzzle"].tap()
        XCTAssertTrue(app.staticTexts["Share This Puzzle"].waitForExistence(timeout: 5))
        app.buttons["Done"].tap()
        XCTAssertTrue((app.buttons["Cell_0_2"].value as? String)?.contains("Notes 2") == true)
        app.buttons["Pause"].tap()
        XCTAssertTrue(app.buttons["Resume"].waitForExistence(timeout: 5))
        let hiddenBoard = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == NO"), object: app.buttons["Cell_0_2"])
        XCTAssertEqual(XCTWaiter.wait(for: [hiddenBoard], timeout: 5), .completed, app.debugDescription)
        XCTAssertFalse(app.buttons["Digit2"].exists)
        capture("Paused")
        app.buttons["Save & Exit"].tap()
        app.terminate()
        launch(reset: false)
        XCTAssertTrue(app.buttons["Continue"].waitForExistence(timeout: 10))
        app.buttons["Continue"].tap()
        XCTAssertTrue((app.buttons["Cell_0_2"].value as? String)?.contains("Notes 2") == true)
        app.buttons["Cell_0_2"].tap()
        app.buttons["KeypadErase"].tap()
        XCTAssertEqual(app.buttons["Cell_0_2"].value as? String, "Empty")
        app.buttons["Undo"].tap()
        XCTAssertTrue((app.buttons["Cell_0_2"].value as? String)?.contains("Notes 2") == true)
    }

    func testCaptureWinScreen() {
        launch(puzzle: almostComplete)
        XCTAssertTrue(app.buttons["Cell_0_0"].waitForExistence(timeout: 10))
        app.buttons["Cell_0_0"].tap()
        app.buttons["Digit5"].tap()
        let continueButton = app.buttons["Continue"]
        XCTAssertTrue(continueButton.waitForExistence(timeout: 5))
        continueButton.tap()
        XCTAssertTrue(app.staticTexts["Game Stats"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.buttons["View Leaderboard"].exists)
        capture("Completed Game")
    }

    func testLandscapeKeepsNumberEntryAndHintsReachable() {
        launch(puzzle: puzzle)
        XCTAssertTrue(app.buttons["Digit1"].waitForExistence(timeout: 10))
        XCUIDevice.shared.orientation = .landscapeLeft
        for digit in 1...9 {
            let key = app.buttons["Digit\(digit)"]
            let reachable = XCTNSPredicateExpectation(predicate: NSPredicate(format: "hittable == YES"), object: key)
            XCTAssertEqual(XCTWaiter.wait(for: [reachable], timeout: 5), .completed, "Digit \(digit) must remain reachable in landscape")
        }
        app.buttons["Cell_0_2"].tap()
        app.buttons["Digit4"].tap()
        XCTAssertEqual(app.buttons["Cell_0_2"].value as? String, "4")
        app.buttons["Hint"].tap()
        XCTAssertTrue(app.buttons["Details"].waitForExistence(timeout: 5))
        app.buttons["Show more"].tap()
        XCTAssertTrue(app.buttons["Show less"].exists)
        app.buttons["Show less"].tap()
        XCTAssertTrue(app.buttons["Show more"].exists)
        app.staticTexts["HintExplanation"].tap()
        XCTAssertTrue(app.buttons["Show less"].exists)
        app.staticTexts["HintExplanation"].tap()
        XCTAssertTrue(app.buttons["Show more"].exists)
        capture("Gameplay Landscape")
        XCUIDevice.shared.orientation = .portrait
        let portraitReady = XCTNSPredicateExpectation(predicate: NSPredicate(format: "hittable == YES"), object: app.buttons["Digit4"])
        XCTAssertEqual(XCTWaiter.wait(for: [portraitReady], timeout: 5), .completed)
        capture("Gameplay Portrait With Hint")
        XCTAssertEqual(app.buttons["Cell_0_2"].value as? String, "4")
    }

    func testMenuSettingsAndStatisticsScreens() {
        launch()
        XCTAssertTrue(app.buttons["Settings"].waitForExistence(timeout: 10))
        app.buttons["Settings"].tap()
        XCTAssertTrue(app.switches["Mistake Limit"].waitForExistence(timeout: 5))
        capture("Settings")
        app.buttons["Done"].tap()
        app.buttons["Progress"].tap()
        XCTAssertTrue(app.navigationBars["Statistics"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.staticTexts["Games Played"].exists)
        capture("Statistics")
    }

    @available(iOS 16.4, *)
    func testSharedLinkOpensPuzzleAndQuitRequiresConfirmation() {
        launch()
        app.launchArguments = ["--ui-testing"]
        app.open(URL(string: "https://ukodus.now/play/?p=invalid")!)
        XCTAssertTrue(app.buttons["New Game"].waitForExistence(timeout: 5))
        app.open(URL(string: "https://ukodus.now/play/?p=" + puzzle)!)
        XCTAssertTrue(app.buttons["Cell_0_2"].waitForExistence(timeout: 10))
        app.buttons["Cell_0_2"].tap()
        app.buttons["Digit4"].tap()
        XCTAssertEqual(app.buttons["Cell_0_2"].value as? String, "4")
        app.buttons["Pause"].tap()
        app.buttons["Quit Game"].tap()
        XCTAssertTrue(app.buttons["Quit"].waitForExistence(timeout: 5))
        app.staticTexts["PAUSED"].tap()
        XCTAssertFalse(app.buttons["Quit"].exists)
        XCTAssertTrue(app.buttons["Resume"].exists)
        app.buttons["Resume"].tap()
        XCTAssertEqual(app.buttons["Cell_0_2"].value as? String, "4")
        app.buttons["Pause"].tap()
        app.buttons["Quit Game"].tap()
        app.buttons["Quit"].tap()
        XCTAssertTrue(app.buttons["New Game"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.buttons["Continue"].exists)
    }

    private func capture(_ name: String) {
        let attachment = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
