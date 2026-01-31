import SwiftUI

struct GameHistoryView: View {
    @EnvironmentObject var gameManager: GameManager
    @StateObject private var historyManager = GameHistoryManager.shared
    @State private var selectedDifficulty: Difficulty?
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                // Stats section
                Section {
                    HStack {
                        VStack(alignment: .leading) {
                            Text("\(historyManager.stats.totalPuzzles)")
                                .font(.title2.bold())
                            Text("Puzzles")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        VStack(alignment: .center) {
                            Text("\(historyManager.stats.solvedPuzzles)")
                                .font(.title2.bold())
                            Text("Solved")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        VStack(alignment: .trailing) {
                            Text("\(Int(historyManager.stats.completionRate * 100))%")
                                .font(.title2.bold())
                            Text("Completion")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(.vertical, 4)
                } header: {
                    Text("Library Stats")
                }

                // Difficulty filter
                Section {
                    Picker("Filter", selection: $selectedDifficulty) {
                        Text("All").tag(nil as Difficulty?)
                        ForEach(Difficulty.allCases) { difficulty in
                            Text(difficulty.displayName).tag(difficulty as Difficulty?)
                        }
                    }
                    .pickerStyle(.segmented)
                }

                // Puzzle list
                Section {
                    let puzzles = filteredPuzzles
                    if puzzles.isEmpty {
                        Text("No puzzles yet")
                            .foregroundStyle(.secondary)
                            .italic()
                    } else {
                        ForEach(puzzles) { puzzle in
                            PuzzleRowView(puzzle: puzzle) {
                                replayPuzzle(puzzle)
                            }
                        }
                    }
                } header: {
                    Text("Recent Puzzles")
                }
            }
            .navigationTitle("Puzzle Library")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }

    private var filteredPuzzles: [PuzzleRecord] {
        if let difficulty = selectedDifficulty {
            return historyManager.puzzles(for: difficulty)
        }
        return historyManager.recentPuzzles
    }

    private func replayPuzzle(_ puzzle: PuzzleRecord) {
        // Load the puzzle from its string
        if let game = gameFromString(puzzle: puzzle.puzzleString) {
            let viewModel = GameViewModel(cachedGame: game, difficulty: puzzle.difficulty)
            gameManager.currentGame = viewModel
            gameManager.gameState = .playing
            dismiss()
        }
    }
}

// MARK: - Puzzle Row View

struct PuzzleRowView: View {
    let puzzle: PuzzleRecord
    let onReplay: () -> Void

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    // Difficulty badge
                    Text(puzzle.difficulty.displayName)
                        .font(.caption.bold())
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(difficultyColor.opacity(0.2))
                        .foregroundStyle(difficultyColor)
                        .clipShape(Capsule())

                    // Solved indicator
                    if puzzle.hasBeenSolved {
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                            .font(.caption)
                    }
                }

                HStack {
                    Text("Played \(puzzle.playCount)x")
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    if let bestTime = puzzle.bestTime {
                        Text("Best: \(formatTime(bestTime))")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                Text(relativeDate(puzzle.lastPlayedAt))
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            Spacer()

            Button {
                onReplay()
            } label: {
                Image(systemName: "play.circle.fill")
                    .font(.title2)
                    .foregroundStyle(Color.accentColor)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 4)
    }

    private var difficultyColor: Color {
        switch puzzle.difficulty {
        case .beginner: return .green
        case .easy: return .mint
        case .medium: return .blue
        case .intermediate: return .orange
        case .hard: return .red
        case .expert: return .purple
        case .master: return .indigo
        case .extreme: return .pink
        }
    }

    private func formatTime(_ time: TimeInterval) -> String {
        let seconds = Int(time)
        let mins = seconds / 60
        let secs = seconds % 60
        return String(format: "%d:%02d", mins, secs)
    }

    private func relativeDate(_ date: Date) -> String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}

#Preview {
    GameHistoryView()
        .environmentObject(GameManager())
}
