import XCTest
import SwiftUI
import ViewInspector
@testable import Sudoku

@MainActor
final class GridPresentationTests: XCTestCase {
    func testEachCelebrationCycleReturnsToIdentityAndHasTheSameMidpoint() {
        let size = CGSize(width: 40, height: 40)
        for progress: CGFloat in [0, 1, 2] {
            XCTAssertEqual(WiggleEffect(progress: progress).effectValue(size: size), ProjectionTransform(.identity))
        }
        XCTAssertEqual(WiggleEffect(progress: 0.5).effectValue(size: size),
                       WiggleEffect(progress: 1.5).effectValue(size: size))
    }

    func testCellDefaultHintAndNumberHighlightsRenderWithoutExtraSelectionState() throws {
        let cell = CellView(cell: .empty(row: 0, col: 0), isSelected: false,
                            isRelated: false, hasSameValue: false, isNakedSingle: false,
                            ghostCandidates: [], showGhosts: false, showErrors: true, size: 40)
        XCTAssertEqual(cell.hintRole, .none)
        XCTAssertEqual(cell.highlightedNumber, 0)
        XCTAssertTrue(try cell.inspect().findAll(ViewType.Text.self).isEmpty)
        try ViewTestFixture.render(cell, size: CGSize(width: 40, height: 40))
    }

    func testCelebratingCellsAnimateAndSettleWithoutChangingPuzzleOrSelection() async throws {
        let game = try ViewTestFixture.game()
        let manager = ViewTestFixture.manager()
        let initialGrid = game.cells
        let host = UIHostingController(rootView: GridView(game: game, size: 360)
            .environmentObject(manager).frame(width: 360, height: 360).ignoresSafeArea())
        let window = UIWindow(frame: CGRect(x: 0, y: 0, width: 360, height: 360))
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true; window.rootViewController = nil }
        host.view.layoutIfNeeded()
        try await Task.sleep(nanoseconds: 100_000_000)
        func snapshot() throws -> Data {
            let renderer = UIGraphicsImageRenderer(bounds: host.view.bounds)
            return try XCTUnwrap(renderer.image { _ in
                host.view.drawHierarchy(in: host.view.bounds, afterScreenUpdates: true)
            }.pngData())
        }
        let baseline = try snapshot()
        game.celebratingCells = Set((0..<9).map { "0-\($0)" })
        try await Task.sleep(nanoseconds: 100_000_000)
        XCTAssertNotEqual(try snapshot(), baseline, "The hosted grid should visibly animate celebrating cells")
        try await Task.sleep(nanoseconds: 550_000_000)
        XCTAssertEqual(try snapshot(), baseline, "The wiggle must settle back to the unchanged board")
        XCTAssertEqual(game.cells, initialGrid)
        XCTAssertNil(game.selectedCell)
        game.celebratingCells = []
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        XCTAssertEqual(try snapshot(), baseline, "Clearing celebrations must render before the next event")
        game.celebratingCells = Set((0..<9).map { "0-\($0)" })
        try await Task.sleep(nanoseconds: 100_000_000)
        XCTAssertNotEqual(try snapshot(), baseline, "A later celebration of the same row must animate again")
    }
}
