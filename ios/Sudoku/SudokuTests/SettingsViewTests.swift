import XCTest
import SwiftUI
import ViewInspector
@testable import Sudoku

@MainActor
final class SettingsViewTests: XCTestCase {
    func testEverySettingsToggleBindsToTheLiveGameSettings() throws {
        let manager = ViewTestFixture.manager()
        let options: [(String, WritableKeyPath<GameSettings, Bool>)] = [
            ("Show Timer", \.timerVisible), ("Mistake Limit", \.mistakeLimitEnabled),
            ("Show Errors Immediately", \.showErrorsImmediately),
            ("Highlight Related Cells", \.highlightRelatedCells), ("Highlight Same Numbers", \.highlightSameNumbers),
            ("Ghost Hints", \.ghostHintsEnabled), ("Highlight Valid Cells", \.highlightValidCells),
            ("Auto-Fill Notes on Start", \.autoFillCandidates), ("Auto-Clear Notes", \.autoClearNotes),
            ("Haptic Feedback", \.hapticsEnabled),
            ("Celebrations", \.celebrationsEnabled), ("Camera Import", \.cameraImportEnabled)
        ]
        let view = SettingsView().environmentObject(manager)
        XCTAssertEqual(try view.inspect().findAll(ViewType.Toggle.self).count, options.count)
        for (name, keyPath) in options {
            let toggle = try view.inspect().find(ViewType.Toggle.self, where: { try $0.labelView().text().string() == name })
            let initial = manager.settings[keyPath: keyPath]
            XCTAssertEqual(try toggle.isOn(), initial, name)
            try toggle.tap(); XCTAssertEqual(manager.settings[keyPath: keyPath], !initial, name)
            try toggle.tap(); XCTAssertEqual(manager.settings[keyPath: keyPath], initial, name)
        }
    }
    func testThemePickerAndMistakeLimitBoundaries() throws {
        let manager = ViewTestFixture.manager()
        manager.currentGame = try ViewTestFixture.game()
        let view = SettingsView().environmentObject(manager)
        let picker = try view.inspect().find(ViewType.Picker.self)
        XCTAssertEqual(try picker.selectedValue(GameSettings.ThemeSetting.self), .system)
        XCTAssertEqual(try picker.findAll(ViewType.Text.self).map { try $0.string() }.filter { $0 != "Theme" }, GameSettings.ThemeSetting.allCases.map(\.rawValue))
        for theme in GameSettings.ThemeSetting.allCases {
            try picker.select(value: theme)
            XCTAssertEqual(manager.settings.theme, theme)
        }
        manager.settings.mistakeLimit = 1
        var stepper = try view.inspect().find(ViewType.Stepper.self)
        _ = try? stepper.decrement(); XCTAssertEqual(manager.settings.mistakeLimit, 1)
        try stepper.increment(); XCTAssertEqual(manager.settings.mistakeLimit, 2)
        XCTAssertEqual(manager.currentGame?.maxMistakes, 2)
        manager.settings.mistakeLimit = 10
        stepper = try view.inspect().find(ViewType.Stepper.self)
        _ = try? stepper.increment(); XCTAssertEqual(manager.settings.mistakeLimit, 10)
        try stepper.decrement(); XCTAssertEqual(manager.settings.mistakeLimit, 9)
        let limit = try view.inspect().find(ViewType.Toggle.self, where: { try $0.labelView().text().string() == "Mistake Limit" })
        try limit.tap()
        XCTAssertThrowsError(try view.inspect().find(ViewType.Stepper.self))
        XCTAssertEqual(manager.currentGame?.mistakeLimitEnabled, false)
    }
    func testDonePersistsSettingsAndAboutShowsCurrentAppVersionAndRepository() throws {
        let defaults = ServiceFixtures.defaults()
        let manager = GameManager(defaults: defaults, dependencies: .isolated)
        manager.settings.theme = .dark
        manager.settings.cameraImportEnabled = true
        let view = SettingsView().environmentObject(manager)
        try view.inspect().find(button: "Done").tap()
        let restored = GameManager(defaults: defaults, dependencies: .isolated)
        XCTAssertEqual(restored.settings.theme, .dark)
        XCTAssertTrue(restored.settings.cameraImportEnabled)
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "Unknown"
        XCTAssertNoThrow(try view.inspect().find(text: version))
        XCTAssertEqual(try view.inspect().find(ViewType.Link.self).url().absoluteString, "https://github.com/kcirtapfromspace/sudoku")
        try ViewTestFixture.render(view)
    }
    func testResetRequiresConfirmationAndCancelPreservesStatistics() throws {
        let defaults = ServiceFixtures.defaults()
        let manager = GameManager(defaults: defaults, dependencies: .isolated)
        manager.statistics.recordWin(difficulty: .easy, time: 60)
        defaults.set(try JSONEncoder().encode(manager.statistics), forKey: "sudoku_statistics")
        var isPresented = false, done = false
        let view = SettingsForm(showingResetConfirmation: Binding(get: { isPresented }, set: { isPresented = $0 }), onDone: { done = true }).environmentObject(manager)
        XCTAssertThrowsError(try view.inspect().find(ViewType.List.self).confirmationDialog())
        try view.inspect().find(button: "Reset Statistics").tap()
        XCTAssertTrue(isPresented)
        XCTAssertEqual(manager.statistics.gamesWon, 1)
        let dialog = try view.inspect().find(ViewType.List.self).confirmationDialog()
        XCTAssertEqual(try dialog.title().string(), "Reset Statistics")
        XCTAssertEqual(try dialog.titleVisibility(), .visible)
        XCTAssertTrue(try dialog.message().text().string().contains("Game Center leaderboard entries will remain"))
        try dialog.actions().find(button: "Cancel").tap()
        XCTAssertEqual(manager.statistics.gamesWon, 1)
        try dialog.dismiss(); XCTAssertFalse(isPresented)
        try view.inspect().find(button: "Reset Statistics").tap()
        try view.inspect().find(ViewType.List.self).confirmationDialog().actions().find(button: "Reset").tap()
        XCTAssertEqual(manager.statistics.gamesPlayed, 0)
        XCTAssertTrue(manager.statistics.bestTimes.isEmpty)
        XCTAssertEqual(GameManager(defaults: defaults, dependencies: .isolated).statistics.gamesPlayed, 0)
        try view.inspect().find(button: "Done").tap(); XCTAssertTrue(done)
    }
}
