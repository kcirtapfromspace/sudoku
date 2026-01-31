import SwiftUI

struct StatsView: View {
    @EnvironmentObject var gameManager: GameManager
    @Environment(\.dismiss) var dismiss

    var stats: GameStatistics { gameManager.statistics }

    // Estimated total puzzles in the universe (conservative estimate)
    // Based on: ~5.47 billion unique grids × clue combinations × uniqueness probability
    private let totalPuzzleUniverse: Double = 1e30  // ~10^30 puzzles across all difficulties

    var body: some View {
        NavigationStack {
            List {
                // Overview
                Section("Overview") {
                    StatItem(title: "Games Played", value: "\(stats.gamesPlayed)")
                    StatItem(title: "Games Won", value: "\(stats.gamesWon)")
                    StatItem(title: "Win Rate", value: String(format: "%.1f%%", stats.winRate * 100))
                    StatItem(title: "Total Play Time", value: formatTime(stats.totalPlayTime))
                }

                // Streaks
                Section("Streaks") {
                    StatItem(title: "Current Streak", value: "\(stats.currentStreak)")
                    StatItem(title: "Best Streak", value: "\(stats.bestStreak)")
                }

                // Best Times
                Section("Best Times") {
                    ForEach(Difficulty.allCases) { difficulty in
                        if let bestTime = stats.bestTimes[difficulty] {
                            StatItem(title: difficulty.displayName, value: formatTime(bestTime))
                        } else {
                            StatItem(title: difficulty.displayName, value: "—")
                        }
                    }
                }

                // Puzzle Universe - Fun stats!
                Section {
                    VStack(alignment: .leading, spacing: 12) {
                        // Universe coverage
                        HStack {
                            Image(systemName: "globe.americas.fill")
                                .foregroundStyle(.blue)
                            Text("Universe Explored")
                                .font(.headline)
                        }

                        Text(universeExploredText)
                            .font(.system(.body, design: .monospaced))
                            .foregroundStyle(.secondary)

                        // Progress visualization
                        GeometryReader { geo in
                            ZStack(alignment: .leading) {
                                RoundedRectangle(cornerRadius: 4)
                                    .fill(Color.secondary.opacity(0.2))
                                    .frame(height: 8)

                                // The bar would be invisibly small, so show a tiny sliver
                                RoundedRectangle(cornerRadius: 4)
                                    .fill(Color.blue)
                                    .frame(width: stats.gamesWon > 0 ? max(2, geo.size.width * 0.001) : 0, height: 8)
                            }
                        }
                        .frame(height: 8)

                        Text(universeProgressNote)
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .italic()
                    }
                    .padding(.vertical, 8)

                    // Time to complete all puzzles
                    VStack(alignment: .leading, spacing: 12) {
                        HStack {
                            Image(systemName: "clock.fill")
                                .foregroundStyle(.orange)
                            Text("Time to Complete All")
                                .font(.headline)
                        }

                        if stats.gamesWon > 0 {
                            Text(timeToCompleteAllText)
                                .font(.system(.body, design: .monospaced))
                                .foregroundStyle(.secondary)

                            Text(cheekyTimeNote)
                                .font(.caption)
                                .foregroundStyle(.tertiary)
                                .italic()
                        } else {
                            Text("Complete a puzzle to see this stat!")
                                .font(.body)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(.vertical, 8)
                } header: {
                    Label("Puzzle Universe", systemImage: "sparkles")
                }
            }
            .navigationTitle("Statistics")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }

    // MARK: - Universe Stats

    private var universeExploredText: String {
        guard stats.gamesWon > 0 else {
            return "0 / 10³⁰ puzzles (0%)"
        }

        // Calculate the exponent for display
        let percentage = Double(stats.gamesWon) / totalPuzzleUniverse * 100
        let exponent = Int(log10(percentage).rounded(.down))

        return "\(stats.gamesWon) / 10³⁰ puzzles (10^\(exponent)%)"
    }

    private var universeProgressNote: String {
        if stats.gamesWon == 0 {
            return "The puzzle universe awaits your first victory!"
        } else if stats.gamesWon == 1 {
            return "One small step for you, one giant... well, still tiny step for puzzlekind."
        } else if stats.gamesWon < 10 {
            return "You've made a dent! A very, very, very small dent."
        } else if stats.gamesWon < 100 {
            return "At this rate, you'll finish in approximately... never."
        } else if stats.gamesWon < 1000 {
            return "Impressive dedication! The universe remains unimpressed."
        } else {
            return "A true puzzle warrior! The universe trembles (microscopically)."
        }
    }

    private var averageSolveTime: TimeInterval {
        guard stats.gamesWon > 0 else { return 300 } // Default 5 min
        return stats.totalPlayTime / Double(stats.gamesWon)
    }

    private var timeToCompleteAllText: String {
        let totalSeconds = averageSolveTime * totalPuzzleUniverse
        let years = totalSeconds / (365.25 * 24 * 3600)

        // Express in scientific notation
        let exponent = Int(log10(years).rounded(.down))
        let mantissa = years / pow(10, Double(exponent))

        return String(format: "≈ %.1f × 10^%d years", mantissa, exponent)
    }

    private var cheekyTimeNote: String {
        let avgMinutes = averageSolveTime / 60

        if avgMinutes < 2 {
            return "Speed demon! But even at light speed, you'd need multiple universe lifetimes."
        } else if avgMinutes < 5 {
            return "Quick solver! The heat death of the universe called - it'll wait."
        } else if avgMinutes < 10 {
            return "Solid pace! Only 10²² generations of your descendants needed to help."
        } else if avgMinutes < 20 {
            return "Taking your time? Good strategy. You'll still need immortality though."
        } else if avgMinutes < 30 {
            return "Thoughtful approach! The Sun will burn out first, but hey, no pressure."
        } else {
            return "Savoring each puzzle! At this pace, new universes will form and die. Repeatedly."
        }
    }

    private func formatTime(_ interval: TimeInterval) -> String {
        let totalSeconds = Int(interval)
        let hours = totalSeconds / 3600
        let minutes = (totalSeconds % 3600) / 60
        let seconds = totalSeconds % 60

        if hours > 0 {
            return String(format: "%d:%02d:%02d", hours, minutes, seconds)
        } else {
            return String(format: "%d:%02d", minutes, seconds)
        }
    }
}

struct StatItem: View {
    let title: String
    let value: String

    var body: some View {
        HStack {
            Text(title)
            Spacer()
            Text(value)
                .foregroundStyle(.secondary)
                .fontWeight(.medium)
        }
    }
}

#Preview {
    StatsView()
        .environmentObject(GameManager())
}
