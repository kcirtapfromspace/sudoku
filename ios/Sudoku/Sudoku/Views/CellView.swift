import SwiftUI

// MARK: - Hint Colors

struct HintColors {
    static func color(for role: HintCellRole) -> Color? {
        switch role {
        case .none: return nil
        case .target: return Color(red: 0.39, green: 0.24, blue: 0.12).opacity(0.6)
        case .involved: return Color(red: 0.24, green: 0.27, blue: 0.20).opacity(0.5)
        case .chainOn: return Color.green.opacity(0.3)
        case .chainOff: return Color.red.opacity(0.3)
        case .fishBase: return Color.blue.opacity(0.3)
        case .fishCover: return Color.orange.opacity(0.3)
        case .fishFin: return Color.yellow.opacity(0.3)
        case .urFloor: return Color(red: 0.20, green: 0.31, blue: 0.47).opacity(0.4)
        case .urRoof: return Color(red: 0.47, green: 0.20, blue: 0.35).opacity(0.4)
        case .alsGroup: return Color.purple.opacity(0.3)
        }
    }
}

struct CellView: View {
    let cell: CellModel
    let isSelected: Bool
    let isRelated: Bool
    let hasSameValue: Bool
    let isNakedSingle: Bool
    let ghostCandidates: Set<Int>
    let showGhosts: Bool
    let showErrors: Bool  // Whether to show error indication
    var hintRole: HintCellRole = .none
    var highlightedNumber: Int = 0
    let size: CGFloat

    private var backgroundColor: Color {
        if isSelected {
            return Color.accentColor.opacity(0.3)
        } else if hintRole == .target {
            return HintColors.color(for: .target)!
        } else if let hintColor = HintColors.color(for: hintRole) {
            return hintColor
        } else if hasSameValue && !cell.isEmpty {
            return Color.accentColor.opacity(0.15)
        } else if isRelated {
            // Use a system fill color (instead of `secondary` + low opacity) so the
            // row/col/box helper shading is visible across light/dark and bright screens.
            return Color(uiColor: .secondarySystemFill)
        } else {
            return .clear
        }
    }

    private var textColor: Color {
        // Only show red for conflicts when showErrors is enabled
        if cell.hasConflict && showErrors {
            return .red
        } else if cell.isGiven {
            return .primary
        } else {
            return .accentColor
        }
    }

    var body: some View {
        ZStack {
            // Background
            Rectangle()
                .fill(backgroundColor)

            // Naked single highlight
            if isNakedSingle {
                Rectangle()
                    .strokeBorder(Color.green.opacity(0.5), lineWidth: 2)
            }

            // Content
            if cell.value > 0 {
                // Main value
                Text("\(cell.value)")
                    .font(.system(size: size * 0.55, weight: cell.isGiven ? .bold : .medium, design: .rounded))
                    .foregroundStyle(textColor)
            } else if !cell.candidates.isEmpty {
                // User-entered candidates
                candidatesGrid(cell.candidates, opacity: 1.0)
            } else if showGhosts && !ghostCandidates.isEmpty {
                // Ghost candidates
                candidatesGrid(ghostCandidates, opacity: 0.40)
            }

            // Selected indicator
            if isSelected {
                Rectangle()
                    .strokeBorder(Color.accentColor, lineWidth: 2)
            }
        }
        .frame(width: size, height: size)
    }

    @ViewBuilder
    private func candidatesGrid(_ candidates: Set<Int>, opacity: Double) -> some View {
        let fontSize = size * 0.25

        VStack(spacing: 0) {
            ForEach(0..<3, id: \.self) { row in
                HStack(spacing: 0) {
                    ForEach(0..<3, id: \.self) { col in
                        let num = row * 3 + col + 1
                        let isHighlighted = candidates.contains(num) && highlightedNumber > 0 && num == highlightedNumber
                        Text(candidates.contains(num) ? "\(num)" : " ")
                            .font(.system(size: fontSize, weight: isHighlighted ? .bold : .medium, design: .monospaced))
                            .foregroundStyle(isHighlighted ? Color.accentColor : Color.secondary.opacity(opacity))
                            .frame(width: size / 3, height: size / 3)
                    }
                }
            }
        }
    }
}

#Preview {
    HStack {
        CellView(
            cell: CellModel(row: 0, col: 0, value: 5, isGiven: true, candidates: [], hasConflict: false),
            isSelected: false,
            isRelated: false,
            hasSameValue: false,
            isNakedSingle: false,
            ghostCandidates: [],
            showGhosts: false,
            showErrors: true,
            size: 50
        )

        CellView(
            cell: CellModel(row: 0, col: 1, value: 3, isGiven: false, candidates: [], hasConflict: true),
            isSelected: true,
            isRelated: false,
            hasSameValue: false,
            isNakedSingle: false,
            ghostCandidates: [],
            showGhosts: false,
            showErrors: true,
            size: 50
        )

        CellView(
            cell: CellModel(row: 0, col: 2, value: 0, isGiven: false, candidates: [1, 3, 7, 9], hasConflict: false),
            isSelected: false,
            isRelated: true,
            hasSameValue: false,
            isNakedSingle: false,
            ghostCandidates: [],
            showGhosts: false,
            showErrors: true,
            size: 50
        )

        CellView(
            cell: CellModel(row: 0, col: 3, value: 0, isGiven: false, candidates: [], hasConflict: false),
            isSelected: false,
            isRelated: false,
            hasSameValue: false,
            isNakedSingle: true,
            ghostCandidates: [4],
            showGhosts: true,
            showErrors: true,
            size: 50
        )
    }
    .padding()
}
