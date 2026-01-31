import XCTest

/// App Store screenshot automation tests
/// Run with: ./scripts/capture_screenshots.sh
final class ScreenshotTests: XCTestCase {

    var app: XCUIApplication!
    var screenshotDir: URL!

    override func setUpWithError() throws {
        continueAfterFailure = false
        app = XCUIApplication()

        // Create screenshots directory
        let documentsPath = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        screenshotDir = documentsPath.appendingPathComponent("Screenshots")
        try? FileManager.default.createDirectory(at: screenshotDir, withIntermediateDirectories: true)

        // Launch with clean state
        app.launchArguments = ["--reset-state", "--screenshot-mode"]
        app.launch()
    }

    override func tearDownWithError() throws {
        app = nil
    }

    // MARK: - Screenshot Capture Tests

    /// Capture all screenshots for App Store submission
    func testCaptureAllScreenshots() throws {
        // 1. Main Menu
        captureScreenshot(named: "01_MainMenu")

        // 2. Difficulty Picker
        let newGameButton = app.buttons["New Game"]
        XCTAssertTrue(newGameButton.waitForExistence(timeout: 5))
        newGameButton.tap()
        sleep(1)
        captureScreenshot(named: "02_DifficultyPicker")

        // Select Medium difficulty
        let mediumButton = app.buttons["Medium"]
        if mediumButton.exists {
            mediumButton.tap()
        } else {
            // Fallback: tap first available difficulty
            app.cells.element(boundBy: 2).tap()
        }

        // Wait for game to load
        sleep(2)

        // 3. Gameplay - Light Mode
        captureScreenshot(named: "03_Gameplay_Light")

        // 4. Tap a cell to show highlighting
        tapCell(row: 4, col: 4)
        sleep(1)
        captureScreenshot(named: "04_Gameplay_Highlighted")

        // 5. Toggle to Notes mode
        let notesButton = app.buttons["Notes Mode"]
        if notesButton.exists {
            notesButton.tap()
        } else {
            // Find button with pencil icon
            let pencilButton = app.buttons.matching(NSPredicate(format: "label CONTAINS 'Notes' OR label CONTAINS 'Candidates'")).firstMatch
            if pencilButton.exists {
                pencilButton.tap()
            }
        }
        sleep(1)
        captureScreenshot(named: "05_NotesMode")

        // 6. Pause the game
        let pauseButton = app.buttons["pause"]
        if pauseButton.exists {
            pauseButton.tap()
            sleep(1)
            captureScreenshot(named: "06_Paused")

            // Resume
            let resumeButton = app.buttons["Resume"]
            if resumeButton.exists {
                resumeButton.tap()
            }
        }

        // 7. Open Settings via pause menu
        pauseButton.tap()
        sleep(1)
        let saveExitButton = app.buttons["Save & Exit"]
        if saveExitButton.exists {
            saveExitButton.tap()
            sleep(1)
        }

        // Open Settings from menu
        let settingsButton = app.buttons["Settings"]
        if settingsButton.waitForExistence(timeout: 3) {
            settingsButton.tap()
            sleep(1)
            captureScreenshot(named: "07_Settings")

            // Enable Dark Mode for next screenshots
            let themePicker = app.buttons["Theme"]
            if themePicker.exists {
                themePicker.tap()
                let darkOption = app.buttons["Dark"]
                if darkOption.exists {
                    darkOption.tap()
                }
            }

            // Close settings
            let doneButton = app.buttons["Done"]
            if doneButton.exists {
                doneButton.tap()
            }
        }

        // 8. Stats
        let statsButton = app.buttons["Stats"]
        if statsButton.waitForExistence(timeout: 3) {
            statsButton.tap()
            sleep(1)
            captureScreenshot(named: "08_Statistics")

            // Close stats
            let closeButton = app.buttons["Done"]
            if closeButton.exists {
                closeButton.tap()
            } else {
                // Swipe down to dismiss
                app.swipeDown()
            }
        }

        // 9. Start new game in Dark Mode
        sleep(1)
        if newGameButton.waitForExistence(timeout: 3) {
            newGameButton.tap()
            sleep(1)
            app.cells.element(boundBy: 3).tap() // Intermediate
            sleep(2)
            captureScreenshot(named: "09_Gameplay_Dark")
        }

        print("Screenshots saved to: \(screenshotDir.path)")
    }

    /// Test to capture win screen - requires debug mode to fill puzzle
    func testCaptureWinScreen() throws {
        // Start a new game
        let newGameButton = app.buttons["New Game"]
        XCTAssertTrue(newGameButton.waitForExistence(timeout: 5))
        newGameButton.tap()

        // Select Beginner for faster testing
        sleep(1)
        let beginnerButton = app.buttons["Beginner"]
        if beginnerButton.exists {
            beginnerButton.tap()
        } else {
            app.cells.element(boundBy: 0).tap()
        }

        sleep(2)

        // In debug mode, long press for 2 seconds to open debug menu
        #if DEBUG
        let grid = app.otherElements["SudokuGrid"]
        if grid.exists {
            grid.press(forDuration: 2.5)
            sleep(1)

            // Select "Fill All (leave 1 cell)"
            let fillButton = app.buttons["Fill All (leave 1 cell) - Win Test"]
            if fillButton.exists {
                fillButton.tap()
                sleep(1)

                // Find and tap the remaining empty cell
                // Then enter the correct number
                captureScreenshot(named: "10_AlmostComplete")
            }
        }
        #endif

        // Note: For actual win screen, you'll need to manually complete a puzzle
        // or use the debug menu
    }

    /// Capture celebration toast by completing a row
    func testCaptureCelebration() throws {
        // This requires debug mode or manual setup
        // The celebration overlay appears briefly when completing a row/column/box

        let newGameButton = app.buttons["New Game"]
        XCTAssertTrue(newGameButton.waitForExistence(timeout: 5))
        newGameButton.tap()

        sleep(1)
        app.cells.element(boundBy: 0).tap() // Beginner
        sleep(2)

        captureScreenshot(named: "11_Gameplay_Ready")

        // Note: To capture celebration:
        // 1. Run app manually
        // 2. Use debug menu (long press 2s) to fill row except 1 cell
        // 3. Complete the row
        // 4. Screenshot the celebration toast quickly
    }

    // MARK: - Helpers

    private func captureScreenshot(named name: String) {
        let screenshot = app.screenshot()
        let attachment = XCTAttachment(screenshot: screenshot)
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)

        // Also save to disk
        let fileURL = screenshotDir.appendingPathComponent("\(name).png")
        try? screenshot.pngRepresentation.write(to: fileURL)

        print("Captured: \(name)")
    }

    private func tapCell(row: Int, col: Int) {
        // Try to find the grid and tap within it
        let grid = app.otherElements["SudokuGrid"]
        if grid.exists {
            let frame = grid.frame
            let cellWidth = frame.width / 9
            let cellHeight = frame.height / 9

            let x = frame.origin.x + (CGFloat(col) + 0.5) * cellWidth
            let y = frame.origin.y + (CGFloat(row) + 0.5) * cellHeight

            let coordinate = app.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
                .withOffset(CGVector(dx: x, dy: y))
            coordinate.tap()
        }
    }
}

// MARK: - Manual Screenshot Mode

extension ScreenshotTests {

    /// Run this test in the simulator, then manually navigate to capture screenshots
    /// Press Cmd+S in simulator to save screenshots
    func testManualScreenshotMode() throws {
        // Just launch the app and wait
        // Use Cmd+S in simulator to capture manually

        print("""

        ========================================
        MANUAL SCREENSHOT MODE
        ========================================

        The app is now running. Navigate to each screen
        and press Cmd+S in the Simulator to capture.

        Recommended screenshots:
        1. Main Menu
        2. Difficulty Selection
        3. Gameplay (light mode, with highlighting)
        4. Gameplay (dark mode)
        5. Notes/Candidates mode
        6. Win Screen (complete a puzzle)
        7. Settings
        8. Statistics

        Screenshots are saved to Desktop by default.

        Press any key in the test console to end...
        ========================================

        """)

        // Keep the app running for manual screenshots
        sleep(300) // 5 minutes
    }
}
