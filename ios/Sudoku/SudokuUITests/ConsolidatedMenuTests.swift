import XCTest

/// Assert complete menu/import paths; missing controls fail instead of silently skipping.
final class ConsolidatedMenuTests: XCTestCase {
    private var app: XCUIApplication!

    override func setUpWithError() throws {
        continueAfterFailure = false
        XCUIDevice.shared.orientation = .portrait
        app = XCUIApplication()
        app.launchArguments = ["--ui-testing", "--reset-state"]
        app.launch()
        XCTAssertTrue(app.buttons["New Game"].waitForExistence(timeout: 10))
    }

    override func tearDownWithError() throws { app.terminate() }

    func testNewGamePicker() {
        app.buttons["New Game"].tap()
        let medium = app.buttons.matching(NSPredicate(format: "label BEGINSWITH 'Medium'")).firstMatch
        XCTAssertTrue(medium.waitForExistence(timeout: 5))
        medium.tap()
        XCTAssertTrue(app.sliders.firstMatch.exists)
        let play = app.buttons.matching(NSPredicate(format: "label BEGINSWITH 'Play (SE'")).firstMatch
        XCTAssertTrue(play.exists)
        play.tap()
        XCTAssertTrue(app.buttons["Digit1"].waitForExistence(timeout: 10))
        XCTAssertTrue(app.buttons["Cell_0_2"].exists)
    }

    func testProgressHub() {
        app.buttons["Progress"].tap()
        XCTAssertTrue(app.navigationBars["Statistics"].waitForExistence(timeout: 5))
        let library = app.tabBars.buttons["Library"]
        XCTAssertTrue(library.exists)
        library.tap()
        XCTAssertTrue(app.navigationBars["Puzzle Library"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.staticTexts["No puzzles yet"].exists)
        app.tabBars.buttons["Leaderboard"].tap()
        XCTAssertTrue(app.buttons["Open Game Center"].waitForExistence(timeout: 5))
        app.buttons["Done"].tap()
        XCTAssertTrue(app.buttons["New Game"].waitForExistence(timeout: 5))
    }

    func testImportCamera() {
        app.buttons["Import"].tap()
        XCTAssertTrue(app.staticTexts["No Camera Available"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.buttons["Choose from Photo Library"].exists)
        XCTAssertTrue(app.buttons["Use Test Puzzle Image"].exists)
        app.buttons["Cancel"].tap()
        XCTAssertTrue(app.buttons["Import"].waitForExistence(timeout: 5))
    }

    func testImportTestPuzzle() {
        app.buttons["Import"].tap()
        let fixture = app.buttons["Use Test Puzzle Image"]
        XCTAssertTrue(fixture.waitForExistence(timeout: 5))
        fixture.tap()
        let validate = app.buttons["Validate"]
        XCTAssertTrue(validate.waitForExistence(timeout: 30), "OCR must yield a reviewable grid")
        XCTAssertTrue(app.navigationBars["Import Puzzle"].exists)
        validate.tap()
        XCTAssertTrue(app.staticTexts["Valid puzzle with a unique solution"].waitForExistence(timeout: 10))
        let play = app.buttons.matching(NSPredicate(format: "label == %@ AND selected == NO", "Continue Puzzle")).firstMatch
        XCTAssertTrue(play.isEnabled)
        play.tap()
        XCTAssertTrue(app.buttons["Digit1"].waitForExistence(timeout: 10))
    }
}
