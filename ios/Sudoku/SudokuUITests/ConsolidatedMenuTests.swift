import XCTest

/// Test the consolidated menu flows: New Game, Progress, Import
final class ConsolidatedMenuTests: XCTestCase {

    var app: XCUIApplication!

    override func setUpWithError() throws {
        continueAfterFailure = true
        app = XCUIApplication()
        app.launch()
    }

    override func tearDownWithError() throws {
        app = nil
    }

    // MARK: - Test 1: New Game (Difficulty list with SE sliders)

    func testNewGamePicker() throws {
        let newGameButton = app.buttons["New Game"]
        XCTAssertTrue(newGameButton.waitForExistence(timeout: 5), "New Game button should exist")
        newGameButton.tap()
        sleep(1)

        // Should see unified difficulty list
        let screenshot1 = XCUIScreen.main.screenshot()
        let attach1 = XCTAttachment(screenshot: screenshot1)
        attach1.name = "01_NewGame_DifficultyList"
        attach1.lifetime = .keepAlways
        add(attach1)

        // Tap a difficulty to expand SE slider
        let mediumButton = app.staticTexts["Medium"]
        if mediumButton.waitForExistence(timeout: 2) {
            mediumButton.tap()
            sleep(1)

            let screenshot2 = XCUIScreen.main.screenshot()
            let attach2 = XCTAttachment(screenshot: screenshot2)
            attach2.name = "02_NewGame_SESliderExpanded"
            attach2.lifetime = .keepAlways
            add(attach2)
        }

        // Dismiss
        let cancelButton = app.buttons["Cancel"]
        if cancelButton.exists {
            cancelButton.tap()
            sleep(1)
        }
    }

    // MARK: - Test 2: Progress (Stats + Library + Leaderboard tabs)

    func testProgressHub() throws {
        let progressButton = app.buttons["Progress"]
        XCTAssertTrue(progressButton.waitForExistence(timeout: 5), "Progress button should exist")
        progressButton.tap()
        sleep(1)

        // Stats tab (default)
        let screenshot1 = XCUIScreen.main.screenshot()
        let attach1 = XCTAttachment(screenshot: screenshot1)
        attach1.name = "03_Progress_Stats"
        attach1.lifetime = .keepAlways
        add(attach1)

        // Tap Library tab
        let libraryTab = app.buttons["Library"]
        if libraryTab.waitForExistence(timeout: 2) {
            libraryTab.tap()
            sleep(1)

            let screenshot2 = XCUIScreen.main.screenshot()
            let attach2 = XCTAttachment(screenshot: screenshot2)
            attach2.name = "04_Progress_Library"
            attach2.lifetime = .keepAlways
            add(attach2)
        }

        // Tap Leaderboard tab
        let leaderboardTab = app.buttons["Leaderboard"]
        if leaderboardTab.waitForExistence(timeout: 2) {
            leaderboardTab.tap()
            sleep(1)

            let screenshot3 = XCUIScreen.main.screenshot()
            let attach3 = XCTAttachment(screenshot: screenshot3)
            attach3.name = "05_Progress_Leaderboard"
            attach3.lifetime = .keepAlways
            add(attach3)
        }

        // Dismiss
        let doneButton = app.buttons["Done"]
        if doneButton.exists {
            doneButton.tap()
            sleep(1)
        }
    }

    // MARK: - Test 3: Import (Unified Camera)

    func testImportCamera() throws {
        let importButton = app.buttons["Import"]
        XCTAssertTrue(importButton.waitForExistence(timeout: 5), "Import button should exist")
        importButton.tap()
        sleep(3)

        // On simulator, should see the "No Camera Available" fallback
        let noCameraText = app.staticTexts["No Camera Available"]
        let photoLibButton = app.buttons["Choose from Photo Library"]

        if noCameraText.waitForExistence(timeout: 5) {
            // No-camera fallback is showing — take screenshot
            let screenshot1 = XCUIScreen.main.screenshot()
            let attach1 = XCTAttachment(screenshot: screenshot1)
            attach1.name = "06_Import_NoCameraFallback"
            attach1.lifetime = .keepAlways
            add(attach1)

            XCTAssertTrue(photoLibButton.exists, "Photo library button should exist")

            #if DEBUG
            let testPuzzleButton = app.buttons["Use Test Puzzle Image"]
            XCTAssertTrue(testPuzzleButton.exists, "Test puzzle button should exist in DEBUG")
            #endif
        } else {
            // Real camera view (on device)
            let screenshot1 = XCUIScreen.main.screenshot()
            let attach1 = XCTAttachment(screenshot: screenshot1)
            attach1.name = "06_Import_UnifiedCamera"
            attach1.lifetime = .keepAlways
            add(attach1)
        }

        // Dismiss
        let cancelButton = app.buttons["Cancel"]
        let closeButton = app.buttons["xmark"]
        if cancelButton.waitForExistence(timeout: 2) {
            cancelButton.tap()
        } else if closeButton.exists {
            closeButton.tap()
        }
        sleep(1)
    }

    // MARK: - Test 4: Import Test Puzzle Image (end-to-end OCR)

    func testImportTestPuzzle() throws {
        let importButton = app.buttons["Import"]
        XCTAssertTrue(importButton.waitForExistence(timeout: 5), "Import button should exist")
        importButton.tap()
        sleep(2)

        // On simulator, tap "Use Test Puzzle Image"
        let testPuzzleButton = app.buttons["Use Test Puzzle Image"]
        guard testPuzzleButton.waitForExistence(timeout: 5) else {
            // Not on simulator or not a DEBUG build — skip
            return
        }

        let ss1 = XCUIScreen.main.screenshot()
        let a1 = XCTAttachment(screenshot: ss1)
        a1.name = "07_TestPuzzle_BeforeTap"
        a1.lifetime = .keepAlways
        add(a1)

        testPuzzleButton.tap()

        // Wait for OCR to process — the confirmation view should appear
        // Look for typical confirmation UI elements
        sleep(8)

        let ss2 = XCUIScreen.main.screenshot()
        let a2 = XCTAttachment(screenshot: ss2)
        a2.name = "08_TestPuzzle_AfterOCR"
        a2.lifetime = .keepAlways
        add(a2)

        // Wait a bit more in case OCR is still running
        sleep(5)

        let ss3 = XCUIScreen.main.screenshot()
        let a3 = XCTAttachment(screenshot: ss3)
        a3.name = "09_TestPuzzle_Final"
        a3.lifetime = .keepAlways
        add(a3)
    }
}
