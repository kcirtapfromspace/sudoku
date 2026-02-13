import SwiftUI

struct GridView: View {
    @EnvironmentObject var gameManager: GameManager
    @ObservedObject var game: GameViewModel
    let size: CGFloat
    var forceShowErrors: Bool = false  // When true, forces errors to show even if setting is off

    private var cellSize: CGFloat { size / 9 }

    // Track wiggle animation state per cell
    @State private var wiggleProgress: [String: CGFloat] = [:]

    var body: some View {
        ZStack {
            // Background
            RoundedRectangle(cornerRadius: 4)
                .fill(Color(uiColor: .systemBackground))

            // Cells
            VStack(spacing: 0) {
                ForEach(0..<9, id: \.self) { row in
                    HStack(spacing: 0) {
                        ForEach(0..<9, id: \.self) { col in
                            CellView(
                                cell: game.cells[row][col],
                                isSelected: game.selectedCell?.row == row && game.selectedCell?.col == col,
                                isRelated: gameManager.settings.highlightRelatedCells && game.isRelated(to: (row, col)),
                                hasSameValue: gameManager.settings.highlightSameNumbers && game.hasSameValue(as: (row, col)),
                                isNakedSingle: gameManager.settings.highlightValidCells && game.isNakedSingle(row: row, col: col),
                                ghostCandidates: gameManager.settings.ghostHintsEnabled ? game.getValidCandidates(row: row, col: col) : [],
                                showGhosts: gameManager.settings.ghostHintsEnabled,
                                showErrors: forceShowErrors || gameManager.settings.showErrorsImmediately,
                                size: cellSize
                            )
                            .modifier(WiggleModifier(
                                isCelebrating: game.celebratingCells.contains("\(row)-\(col)"),
                                progress: wiggleProgress["\(row)-\(col)"] ?? 0
                            ))
                            .contentShape(Rectangle())
                            .onTapGesture {
                                game.selectCell(row: row, col: col)
                                hapticFeedback(.light)
                            }
                            .onLongPressGesture(minimumDuration: 0.5) {
                                // Long press enters temporary note mode
                                game.selectCell(row: row, col: col)
                                game.enterTemporaryNoteMode()
                                hapticFeedback(.medium)
                            }
                        }
                    }
                }
            }

            // Grid lines (allow touches to pass through)
            GridLines(
                size: size,
                cellSize: cellSize,
                highContrast: gameManager.settings.theme == .highContrast
            )
                .allowsHitTesting(false)
        }
        .frame(width: size, height: size)
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .shadow(color: .black.opacity(0.1), radius: 8, x: 0, y: 4)
        .accessibilityIdentifier("SudokuGrid")
        .onChange(of: game.celebratingCells) { newCells in
            // Start wiggle animation for newly celebrating cells
            for cellKey in newCells {
                wiggleProgress[cellKey] = 0
                withAnimation(.easeOut(duration: 0.5)) {
                    wiggleProgress[cellKey] = 1
                }
            }
        }
    }

    private func hapticFeedback(_ style: UIImpactFeedbackGenerator.FeedbackStyle) {
        guard gameManager.settings.hapticsEnabled else { return }
        UIImpactFeedbackGenerator(style: style).impactOccurred()
    }
}

// MARK: - Wiggle Modifier

struct WiggleModifier: ViewModifier {
    let isCelebrating: Bool
    let progress: CGFloat

    func body(content: Content) -> some View {
        content
            .modifier(WiggleEffect(progress: isCelebrating ? progress : 0))
    }
}

struct WiggleEffect: GeometryEffect {
    var progress: CGFloat

    var animatableData: CGFloat {
        get { progress }
        set { progress = newValue }
    }

    func effectValue(size: CGSize) -> ProjectionTransform {
        let shake = sin(progress * .pi * 6) * 4 * (1 - progress)
        return ProjectionTransform(CGAffineTransform(translationX: shake, y: 0))
    }
}

// MARK: - Grid Lines

struct GridLines: View {
    let size: CGFloat
    let cellSize: CGFloat
    let highContrast: Bool

    @Environment(\.displayScale) private var displayScale

    var body: some View {
        Canvas { context, size in
            // Use `primary` instead of `secondary` for grid lines: `secondary` + low opacity
            // becomes too faint on iOS, especially on bright screens.
            // Prefer a runtime accessibility signal over SwiftUI's `accessibilityContrast` env key
            // to keep compatibility with the project's current iOS deployment target.
            let isHighContrast = highContrast || UIAccessibility.isDarkerSystemColorsEnabled

            let thinLineWidth: CGFloat = isHighContrast ? 1.0 : 0.75
            let thickLineWidth: CGFloat = isHighContrast ? 3.0 : 2.0

            let thinColor = Color.primary.opacity(isHighContrast ? 0.32 : 0.22)
            let thickColor = Color.primary.opacity(isHighContrast ? 0.80 : 0.60)

            let thinStyle = StrokeStyle(lineWidth: thinLineWidth, lineCap: .square, lineJoin: .miter)
            let thickStyle = StrokeStyle(lineWidth: thickLineWidth, lineCap: .square, lineJoin: .miter)

            func alignToPixel(_ value: CGFloat) -> CGFloat {
                (value * displayScale).rounded() / displayScale
            }

            // Thin lines
            for i in 1..<9 {
                if i % 3 != 0 {
                    let pos = alignToPixel(CGFloat(i) * cellSize)

                    // Vertical
                    var vPath = Path()
                    vPath.move(to: CGPoint(x: pos, y: 0))
                    vPath.addLine(to: CGPoint(x: pos, y: size.height))
                    context.stroke(vPath, with: .color(thinColor), style: thinStyle)

                    // Horizontal
                    var hPath = Path()
                    hPath.move(to: CGPoint(x: 0, y: pos))
                    hPath.addLine(to: CGPoint(x: size.width, y: pos))
                    context.stroke(hPath, with: .color(thinColor), style: thinStyle)
                }
            }

            // Thick lines (3x3 boxes)
            for i in 0...3 {
                let pos = alignToPixel(CGFloat(i) * cellSize * 3)

                // Vertical
                var vPath = Path()
                vPath.move(to: CGPoint(x: pos, y: 0))
                vPath.addLine(to: CGPoint(x: pos, y: size.height))
                context.stroke(vPath, with: .color(thickColor), style: thickStyle)

                // Horizontal
                var hPath = Path()
                hPath.move(to: CGPoint(x: 0, y: pos))
                hPath.addLine(to: CGPoint(x: size.width, y: pos))
                context.stroke(hPath, with: .color(thickColor), style: thickStyle)
            }
        }
    }
}

#Preview {
    GridView(game: GameViewModel(difficulty: .medium), size: 350)
        .environmentObject(GameManager())
        .padding()
}
