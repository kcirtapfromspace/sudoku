import XCTest
import SwiftUI
import ViewInspector
@testable import Sudoku

@MainActor
enum ViewTestFixture {
    static let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
    static let solution = "534678912672195348198342567859761423426853791713924856961537284287419635345286179"

    static func game() throws -> GameViewModel {
        let engine = try XCTUnwrap(gameFromString(puzzle: puzzle))
        return GameViewModel(cachedGame: engine, difficulty: .medium)
    }

    static func manager() -> GameManager {
        let defaults = UserDefaults(suiteName: "ViewTests.\(UUID().uuidString)")!
        let manager = GameManager(defaults: defaults, dependencies: .isolated)
        manager.settings.hapticsEnabled = false
        manager.settings.celebrationsEnabled = false
        return manager
    }

    @discardableResult
    static func render<V: View>(_ view: V, size: CGSize = CGSize(width: 390, height: 844),
                                file: StaticString = #filePath, line: UInt = #line) throws -> UIImage {
        let renderer = ImageRenderer(content: view.frame(width: size.width, height: size.height))
        renderer.scale = 1
        let image = try XCTUnwrap(renderer.uiImage, file: file, line: line)
        XCTAssertEqual(image.size, size, file: file, line: line)
        XCTAssertGreaterThan(try XCTUnwrap(image.pngData()).count, 100, file: file, line: line)
        return image
    }
}

@MainActor
final class CellViewTests: XCTestCase {
    func testCellAnnouncementsIncludeCoordinatesContextAndRespectHiddenErrors() {
        XCTAssertEqual(CellModel.empty(row: 0, col: 2).accessibilityValue(), "Empty")
        let note = CellModel(row: 0, col: 2, value: 0, isGiven: false, candidates: [7, 2], hasConflict: false)
        XCTAssertEqual(note.accessibilityValue(), "Empty. Notes 2, 7")
        let given = CellModel(row: 0, col: 0, value: 5, isGiven: true, candidates: [], hasConflict: true)
        XCTAssertEqual(given.accessibilityValue(), "5. Given. Conflict")
        XCTAssertEqual(given.accessibilityValue(showErrors: false), "5. Given")
    }

    func testGivenAndEnteredDigitsUseDifferentWeightAndErrorColor() throws {
        for given in [true, false] {
            for conflict in [true, false] {
                for errors in [true, false] {
                    let cell = CellModel(row: 1, col: 2, value: 5, isGiven: given, candidates: [], hasConflict: conflict)
                    let view = makeCell(cell, errors: errors)
                    let text = try view.inspect().find(text: "5")
                    let font = try XCTUnwrap(text.attributes().font())
                    XCTAssertEqual(try font.size(), 27.5, accuracy: 0.001)
                    XCTAssertEqual(try font.weight(), given ? .bold : .medium)
                    XCTAssertEqual(try font.design(), .rounded)
                    try ViewTestFixture.render(view, size: CGSize(width: 50, height: 50))
                }
            }
        }
    }

    func testManualNotesTakePriorityOverGhostsAndGhostsCanBeDisabled() throws {
        var cell = CellModel.empty(row: 0, col: 0)
        cell.candidates = [1, 7]
        let manual = makeCell(cell, ghosts: [2, 9], showGhosts: true, highlighted: 7)
        let labels = try manual.inspect().findAll(ViewType.Text.self).map { try $0.string() }
        XCTAssertTrue(labels.contains("1"))
        XCTAssertTrue(labels.contains("7"))
        XCTAssertFalse(labels.contains("2"))
        cell.candidates = []
        let ghost = makeCell(cell, ghosts: [2, 9], showGhosts: true)
        XCTAssertEqual(try ghost.inspect().findAll(ViewType.Text.self).map { try $0.string() }.filter { $0 != " " }, ["2", "9"])
        XCTAssertTrue(try makeCell(cell, ghosts: [2], showGhosts: false).inspect().findAll(ViewType.Text.self).isEmpty)
    }

    func testSelectionRelatedCellsAndEveryHintRoleRenderInBothThemes() throws {
        let roles: [HintCellRole] = [.none, .target, .involved, .chainOn, .chainOff, .fishBase, .fishCover, .fishFin, .urFloor, .urRoof, .alsGroup]
        XCTAssertNil(HintColors.color(for: .none))
        for role in roles.dropFirst() { XCTAssertNotNil(HintColors.color(for: role)) }
        for role in roles {
            for scheme in [ColorScheme.light, .dark] {
                let view = makeCell(CellModel(row: 0, col: 0, value: 4, isGiven: false, candidates: [], hasConflict: false),
                                    selected: role == .none, related: role == .involved, same: role == .chainOn,
                                    single: role == .target, role: role)
                XCTAssertEqual(try view.inspect().find(text: "4").string(), "4")
                try ViewTestFixture.render(view.environment(\.colorScheme, scheme), size: CGSize(width: 50, height: 50))
            }
        }
        try ViewTestFixture.render(makeCell(.empty(row: 0, col: 0), related: true), size: CGSize(width: 50, height: 50))
    }

    private func makeCell(_ cell: CellModel, selected: Bool = false, related: Bool = false,
                          same: Bool = false, single: Bool = false, errors: Bool = true,
                          role: HintCellRole = .none, ghosts: Set<Int> = [], showGhosts: Bool = false,
                          highlighted: Int = 0) -> CellView {
        CellView(cell: cell, isSelected: selected, isRelated: related, hasSameValue: same,
                 isNakedSingle: single, ghostCandidates: ghosts, showGhosts: showGhosts,
                 showErrors: errors, hintRole: role, highlightedNumber: highlighted, size: 50)
    }
}

@MainActor
final class GameplayViewTests: XCTestCase {
    func testKeypadEntersErasesAndTogglesNotesOnTheSelectedCell() throws {
        let game = try ViewTestFixture.game()
        game.clearAllCandidates()
        game.selectCell(row: 0, col: 2)
        let manager = ViewTestFixture.manager()
        var tapped: [Int] = []
        let view = NumberPadView(game: game, onNumberTap: { tapped.append($0) }).environmentObject(manager)
        try view.inspect().find(button: "4").tap()
        XCTAssertEqual(game.cells[0][2].value, 4)
        XCTAssertEqual(tapped, [4])
        let erase = try view.inspect().find(ViewType.Button.self, where: { try $0.accessibilityIdentifier() == "KeypadErase" })
        XCTAssertEqual(try erase.accessibilityLabel().string(), "Erase cell")
        try erase.tap()
        XCTAssertEqual(game.cells[0][2].value, 0)
        game.inputMode = .candidate
        try view.inspect().find(button: "2").tap()
        XCTAssertEqual(game.cells[0][2].candidates, [2])
        try erase.tap()
        XCTAssertTrue(game.cells[0][2].candidates.isEmpty)
    }

    func testCompletedDigitsDisableNormalEntryButRemainAvailableForNotes() throws {
        let engine = try XCTUnwrap(gameFromString(puzzle: ViewTestFixture.solution))
        let game = GameViewModel(cachedGame: engine, difficulty: .easy)
        let view = NumberPadView(game: game).environmentObject(ViewTestFixture.manager())
        XCTAssertTrue(try view.inspect().find(button: "1").isDisabled())
        game.inputMode = .candidate
        XCTAssertFalse(try view.inspect().find(button: "1").isDisabled())
        try ViewTestFixture.render(view, size: CGSize(width: 360, height: 120))
    }

    func testGridIncludesAll81AddressableCellsAndLabels() throws {
        let game = try ViewTestFixture.game()
        let view = GridView(game: game, size: 360).environmentObject(ViewTestFixture.manager())
        let cells = try view.inspect().findAll(CellView.self)
        XCTAssertEqual(cells.count, 81)
        XCTAssertEqual(try cells[2].accessibilityIdentifier(), "Cell_0_2")
        XCTAssertEqual(try cells[2].accessibilityLabel().string(), "Row 1, column 3")
        try cells[2].callOnTapGesture()
        XCTAssertEqual(game.selectedCell?.row, 0)
        XCTAssertEqual(game.selectedCell?.col, 2)
        try ViewTestFixture.render(view, size: CGSize(width: 360, height: 360))
    }

    func testGridRenderingForRelatedGhostAndHighContrastStates() throws {
        let game = try ViewTestFixture.game()
        let manager = ViewTestFixture.manager()
        game.selectCell(row: 0, col: 0)
        manager.settings.ghostHintsEnabled = true
        manager.settings.highlightValidCells = true
        manager.settings.theme = .highContrast
        let view = GridView(game: game, size: 360, forceShowErrors: true).environmentObject(manager)
        XCTAssertEqual(try view.inspect().findAll(CellView.self).count, 81)
        try ViewTestFixture.render(view, size: CGSize(width: 360, height: 360))
        XCTAssertEqual(ShakeEffect(shakes: 0).effectValue(size: .zero), ProjectionTransform(.identity))
        XCTAssertEqual(WiggleEffect(progress: 0).effectValue(size: CGSize(width: 40, height: 40)), ProjectionTransform(.identity))
    }

    func testHintSummaryOffersDetailsAndDismissCallbacks() throws {
        let hint = HintModel(row: 0, col: 2, value: 4, eliminate: [], explanation: "Only 4 fits this cell.", technique: "Naked Single", seRating: 1, involvedCells: [(0, 2)])
        var upgrades = 0
        var dismissals = 0
        let view = HintPanelView(hint: hint, detailLevel: .summary, onUpgrade: { upgrades += 1 }, onDismiss: { dismissals += 1 })
        XCTAssertEqual(try view.inspect().find(text: hint.explanation).string(), hint.explanation)
        try view.inspect().find(button: "Details").tap()
        XCTAssertEqual(upgrades, 1)
        let buttons = try view.inspect().findAll(ViewType.Button.self)
        try buttons[1].tap()
        XCTAssertEqual(dismissals, 1)
        let detailed = HintPanelView(hint: hint, detailLevel: .proofDetail, onUpgrade: {}, onDismiss: {})
        XCTAssertEqual(try detailed.inspect().find(text: "Proof shown").string(), "Proof shown")
    }
}

@MainActor
final class NavigationViewTests: XCTestCase {
    func testSharedPuzzleParsingRejectsInvalidDigitAndCodePayloads() {
        XCTAssertEqual(PuzzleLink.extract(from: " \n" + ViewTestFixture.puzzle + "\n"), ViewTestFixture.puzzle)
        XCTAssertEqual(PuzzleLink.extract(from: "abcd1234"), "abcd1234")
        XCTAssertEqual(PuzzleLink.extract(from: "https://ukodus.now/play/?s=Abcd1234"), "Abcd1234")
        XCTAssertEqual(PuzzleLink.extract(from: "https://ukodus.now/play/?p=" + ViewTestFixture.puzzle), ViewTestFixture.puzzle)
        XCTAssertEqual(PuzzleLink.extract(from: "https://ukodus.now/play/?s=invalid&p=" + ViewTestFixture.puzzle), ViewTestFixture.puzzle)
        for bad in ["", "https://[bad", String(repeating: "a", count: 81), "1234567!", String(repeating: "٣", count: 81), "https://ukodus.now/play/?s=1234567!", "https://ukodus.now/play/?p=" + String(repeating: "x", count: 81)] {
            XCTAssertNil(PuzzleLink.extract(from: bad), bad)
        }
    }

    func testLaunchFlagsKeepAutomationDataIsolatedAndSupportKnownPuzzle() {
        let ordinary = AppLaunchConfiguration(arguments: ["app", "--reset-state"], environment: [:])
        XCTAssertFalse(ordinary.isTesting)
        XCTAssertFalse(ordinary.resetState)
        XCTAssertNil(ordinary.initialPuzzle)
        let automated = AppLaunchConfiguration(arguments: ["--ui-testing", "--reset-state", "--puzzle", ViewTestFixture.puzzle], environment: [:])
        XCTAssertTrue(automated.isTesting)
        XCTAssertTrue(automated.resetState)
        let manager = automated.makeManager()
        XCTAssertEqual(manager.gameState, .playing)
        XCTAssertEqual(manager.currentGame?.cells[0][0].value, 5)
        XCTAssertFalse(manager.settings.hapticsEnabled)
        XCTAssertTrue(manager.settings.cameraImportEnabled)
        XCTAssertTrue(AppLaunchConfiguration(arguments: [], environment: ["SUDOKU_TESTING": "1"]).isTesting)
        XCTAssertNil(AppLaunchConfiguration(arguments: ["--ui-testing", "--puzzle"], environment: [:]).initialPuzzle)
        XCTAssertNil(AppLaunchConfiguration(arguments: ["--ui-testing", "--puzzle", "bad"], environment: [:]).initialPuzzle)
    }

    func testMenuPrimaryActionsReflectSavedGameAndCameraSetting() throws {
        let manager = ViewTestFixture.manager()
        let view = MenuView().environmentObject(manager)
        XCTAssertNoThrow(try view.inspect().find(button: "New Game"))
        XCTAssertThrowsError(try view.inspect().find(button: "Continue"))
        XCTAssertThrowsError(try view.inspect().find(button: "Import"))
        manager.settings.cameraImportEnabled = true
        manager.currentGame = try ViewTestFixture.game()
        XCTAssertNoThrow(try view.inspect().find(button: "Import"))
        try view.inspect().find(button: "Continue").tap()
        XCTAssertEqual(manager.gameState, .playing)
    }

    func testPauseOverlayPreservesGameOnSaveExitAndResumes() throws {
        let manager = ViewTestFixture.manager()
        manager.currentGame = try ViewTestFixture.game()
        manager.gameState = .playing
        manager.pauseGame()
        let view = PauseOverlay().environmentObject(manager)
        XCTAssertNoThrow(try view.inspect().find(text: "PAUSED"))
        try view.inspect().find(button: "Resume").tap()
        XCTAssertEqual(manager.gameState, .playing)
        manager.pauseGame()
        try view.inspect().find(button: "Save & Exit").tap()
        XCTAssertEqual(manager.gameState, .menu)
        XCTAssertTrue(manager.hasSavedGame)
    }

    func testContentChoosesEachGameScreenAndTheme() throws {
        let manager = ViewTestFixture.manager()
        manager.currentGame = try ViewTestFixture.game()
        for theme in GameSettings.ThemeSetting.allCases {
            manager.settings.theme = theme
            manager.gameState = .menu
            XCTAssertNoThrow(try ContentView().environmentObject(manager).inspect().find(MenuView.self))
        }
        manager.gameState = .loading
        XCTAssertNoThrow(try ContentView().environmentObject(manager).inspect().find(LoadingView.self))
        manager.gameState = .playing
        XCTAssertNoThrow(try ContentView().environmentObject(manager).inspect().find(GameView.self))
        manager.gameState = .paused
        XCTAssertNoThrow(try ContentView().environmentObject(manager).inspect().find(PauseOverlay.self))
        manager.gameState = .won
        XCTAssertNoThrow(try ContentView().environmentObject(manager).inspect().find(WinScreenView.self))
        manager.gameState = .lost
        let loss = try ContentView().environmentObject(manager).inspect().find(LoseScreenView.self).actualView()
        loss.onDismiss()
        XCTAssertEqual(manager.gameState, .menu)
    }

    func testStatisticsRenderWinCountsBestTimesAndElapsedFormats() throws {
        for (wins, seconds) in [(0, 0.0), (1, 60), (5, 200), (50, 400), (500, 900), (1000, 1500), (2, 2000)] {
            let manager = ViewTestFixture.manager()
            manager.statistics.gamesWon = wins
            manager.statistics.gamesPlayed = wins + 1
            manager.statistics.totalPlayTime = Double(wins) * seconds
            manager.statistics.bestTimes[.easy] = 120
            let view = StatsView().environmentObject(manager)
            let items = try view.inspect().findAll(StatItem.self).map { try $0.actualView() }
            XCTAssertEqual(items.first(where: { $0.title == "Games Won" })?.value, "\(wins)")
            XCTAssertEqual(items.first(where: { $0.title == "Easy" })?.value, "2:00")
            XCTAssertNotNil(items.first(where: { $0.title == "Beginner" && $0.value == "—" }))
            let labels = try view.inspect().findAll(ViewType.Text.self).map { try $0.string() }
            XCTAssertTrue(labels.contains("Universe Explored"))
            if wins == 0 { XCTAssertTrue(labels.contains("Complete a puzzle to see this stat!")) }
            else { XCTAssertTrue(labels.contains(where: { $0.contains("years") })) }
            try ViewTestFixture.render(view)
        }
    }

    func testQRCodeCopyUsesShareableURLAndRendersQRCode() throws {
        let original = UIPasteboard.general.string
        defer { UIPasteboard.general.string = original }
        for code in [nil, "abcd1234"] as [String?] {
            let view = QRCodeView(puzzleString: ViewTestFixture.puzzle, shortCode: code)
            try view.inspect().find(button: "Copy Code").tap()
            XCTAssertEqual(UIPasteboard.general.string, "https://ukodus.now/play/?" + (code.map { "s=\($0)" } ?? "p=\(ViewTestFixture.puzzle)"))
            XCTAssertNoThrow(try view.inspect().find(text: "Share This Puzzle"))
            try ViewTestFixture.render(view)
        }
    }
}

@MainActor
final class GridInteractionTests: XCTestCase {
    func testTapAndLongPressRespectCellAndTemporaryNotesState() throws {
        let game = try ViewTestFixture.game()
        let manager = ViewTestFixture.manager()
        manager.settings.hapticsEnabled = true
        let view = GridView(game: game, size: 360).environmentObject(manager)
        let cells = try view.inspect().findAll(CellView.self)
        try cells[2].callOnTapGesture()
        XCTAssertEqual(game.selectedCell?.row, 0)
        XCTAssertEqual(game.selectedCell?.col, 2)
        try cells[3].callOnLongPressGesture()
        XCTAssertEqual(game.selectedCell?.col, 3)
        XCTAssertEqual(game.inputMode, .temporaryCandidate)
        let keypad = NumberPadView(game: game).environmentObject(manager)
        try keypad.inspect().find(button: "6").tap()
        XCTAssertEqual(game.cells[0][3].candidates, [6])
        let erase = try keypad.inspect().findAll(ViewType.Button.self).last!
        try erase.tap()
        XCTAssertTrue(game.cells[0][3].candidates.isEmpty)
        try cells[2].callOnTapGesture()
        XCTAssertEqual(game.selectedCell?.col, 2)
    }

    func testAnimationTransformsReturnToIdentityAndUpdateInterpolatedValues() {
        var shake = ShakeEffect(shakes: 0)
        XCTAssertEqual(shake.animatableData, 0)
        shake.animatableData = 0.25
        XCTAssertEqual(shake.shakes, 0.25)
        XCTAssertEqual(shake.effectValue(size: CGSize(width: 40, height: 40)), ProjectionTransform(CGAffineTransform(translationX: 6, y: 0)))
        var wiggle = WiggleEffect(progress: 0)
        wiggle.animatableData = 0.5
        XCTAssertEqual(wiggle.animatableData, 0.5)
        XCTAssertNotEqual(wiggle.effectValue(size: CGSize(width: 40, height: 40)), ProjectionTransform(.identity))
        wiggle.animatableData = 1
        XCTAssertEqual(wiggle.effectValue(size: CGSize(width: 40, height: 40)), ProjectionTransform(.identity))
    }
}
