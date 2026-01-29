import SwiftUI

struct GridView: View {
    @EnvironmentObject var gameManager: GameManager
    @ObservedObject var game: GameViewModel
    let size: CGFloat

    private var cellSize: CGFloat { size / 9 }
    private var thickLineWidth: CGFloat { 2 }
    private var thinLineWidth: CGFloat { 0.5 }

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
                                size: cellSize
                            )
                            .onTapGesture {
                                game.selectCell(row: row, col: col)
                                hapticFeedback(.light)
                            }
                        }
                    }
                }
            }

            // Grid lines (allow touches to pass through)
            GridLines(size: size, cellSize: cellSize)
                .allowsHitTesting(false)
        }
        .frame(width: size, height: size)
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .shadow(color: .black.opacity(0.1), radius: 8, x: 0, y: 4)
    }

    private func hapticFeedback(_ style: UIImpactFeedbackGenerator.FeedbackStyle) {
        guard gameManager.settings.hapticsEnabled else { return }
        UIImpactFeedbackGenerator(style: style).impactOccurred()
    }
}

// MARK: - Grid Lines

struct GridLines: View {
    let size: CGFloat
    let cellSize: CGFloat

    var body: some View {
        Canvas { context, size in
            let lineColor = Color.secondary.opacity(0.3)
            let thickColor = Color.primary.opacity(0.6)

            // Thin lines
            for i in 1..<9 {
                if i % 3 != 0 {
                    let pos = CGFloat(i) * cellSize

                    // Vertical
                    var vPath = Path()
                    vPath.move(to: CGPoint(x: pos, y: 0))
                    vPath.addLine(to: CGPoint(x: pos, y: size.height))
                    context.stroke(vPath, with: .color(lineColor), lineWidth: 0.5)

                    // Horizontal
                    var hPath = Path()
                    hPath.move(to: CGPoint(x: 0, y: pos))
                    hPath.addLine(to: CGPoint(x: size.width, y: pos))
                    context.stroke(hPath, with: .color(lineColor), lineWidth: 0.5)
                }
            }

            // Thick lines (3x3 boxes)
            for i in 0...3 {
                let pos = CGFloat(i) * cellSize * 3

                // Vertical
                var vPath = Path()
                vPath.move(to: CGPoint(x: pos, y: 0))
                vPath.addLine(to: CGPoint(x: pos, y: size.height))
                context.stroke(vPath, with: .color(thickColor), lineWidth: 2)

                // Horizontal
                var hPath = Path()
                hPath.move(to: CGPoint(x: 0, y: pos))
                hPath.addLine(to: CGPoint(x: size.width, y: pos))
                context.stroke(hPath, with: .color(thickColor), lineWidth: 2)
            }
        }
    }
}

#Preview {
    GridView(game: GameViewModel(difficulty: .medium), size: 350)
        .environmentObject(GameManager())
        .padding()
}
