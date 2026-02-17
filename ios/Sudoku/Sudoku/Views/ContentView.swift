import SwiftUI

struct ContentView: View {
    @EnvironmentObject var gameManager: GameManager

    var body: some View {
        ZStack {
            switch gameManager.gameState {
            case .menu:
                MenuView()
            case .loading:
                LoadingView()
            case .playing:
                if let game = gameManager.currentGame {
                    GameView(game: game)
                }
            case .paused:
                if let game = gameManager.currentGame {
                    GameView(game: game)
                        .blur(radius: 10)
                        .overlay {
                            PauseOverlay()
                        }
                }
            case .won:
                if let game = gameManager.currentGame {
                    WinScreenView(
                        time: game.elapsedTime,
                        difficulty: game.difficulty,
                        hintsUsed: game.hintsUsed,
                        mistakes: game.mistakes,
                        seRating: game.seRating,
                        onDismiss: {
                            gameManager.quitGame()
                        }
                    )
                    .transition(.opacity)
                }
            case .lost:
                if let game = gameManager.currentGame {
                    LoseScreenView(
                        time: game.elapsedTime,
                        difficulty: game.difficulty,
                        mistakes: game.mistakes,
                        onDismiss: {
                            gameManager.quitGame()
                        },
                        onRetry: {
                            gameManager.newGame(difficulty: game.difficulty)
                        }
                    )
                    .transition(.opacity)
                }
            }
        }
        .preferredColorScheme(colorScheme)
    }

    private var colorScheme: ColorScheme? {
        switch gameManager.settings.theme {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        case .highContrast: return .dark
        }
    }
}

// MARK: - Menu View

struct MenuView: View {
    @EnvironmentObject var gameManager: GameManager
    @State private var showingNewGame = false
    @State private var showingSettings = false
    @State private var showingProgress = false
    @State private var showingImport = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 30) {
                Spacer()

                // Title
                VStack(spacing: 8) {
                    Text("SUDOKU")
                        .font(.system(size: 48, weight: .bold, design: .rounded))
                        .foregroundStyle(.primary)

                    Text("Challenge your mind")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                Spacer()

                // Main buttons
                VStack(spacing: 16) {
                    if gameManager.hasSavedGame {
                        Button {
                            gameManager.resumeGame()
                        } label: {
                            Label("Continue", systemImage: "play.fill")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.large)
                    }

                    Button {
                        showingNewGame = true
                    } label: {
                        Label("New Game", systemImage: "plus")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                    .accessibilityIdentifier("New Game")
                }
                .padding(.horizontal, 40)

                Spacer()

                // Bottom buttons
                HStack(spacing: 40) {
                    Button {
                        showingProgress = true
                    } label: {
                        VStack {
                            Image(systemName: "chart.bar.fill")
                                .font(.title2)
                            Text("Progress")
                                .font(.caption)
                        }
                    }
                    .accessibilityIdentifier("Progress")

                    Button {
                        showingImport = true
                    } label: {
                        VStack {
                            Image(systemName: "camera.fill")
                                .font(.title2)
                            Text("Import")
                                .font(.caption)
                        }
                    }
                    .accessibilityIdentifier("Import")

                    Button {
                        showingSettings = true
                    } label: {
                        VStack {
                            Image(systemName: "gearshape.fill")
                                .font(.title2)
                            Text("Settings")
                                .font(.caption)
                        }
                    }
                    .accessibilityIdentifier("Settings")
                }
                .foregroundStyle(.secondary)
                .padding(.bottom, 40)
            }
            .sheet(isPresented: $showingNewGame) {
                NewGamePickerView { targetSE in
                    showingNewGame = false
                    gameManager.newGameWithSE(targetSE: targetSE)
                }
            }
            .sheet(isPresented: $showingSettings) {
                SettingsView()
            }
            .sheet(isPresented: $showingProgress) {
                ProgressHubView()
            }
            .fullScreenCover(isPresented: $showingImport) {
                UnifiedImportView(
                    onPuzzleFound: { puzzleString in
                        showingImport = false
                        gameManager.loadSharedPuzzle(puzzleString)
                    },
                    onImportComplete: { importData in
                        showingImport = false
                        gameManager.loadImportedPuzzle(importData)
                    }
                )
            }
        }
    }
}

// MARK: - New Game Picker (unified difficulty + SE slider)

struct NewGamePickerView: View {
    let onPlay: (Float) -> Void
    @Environment(\.dismiss) var dismiss
    @EnvironmentObject var gameManager: GameManager
    @State private var expanded: Difficulty?
    @State private var targetSE: Float = 2.0

    var body: some View {
        NavigationStack {
            List {
                ForEach(gameManager.statistics.availableDifficulties) { difficulty in
                    difficultyRow(difficulty)

                    if expanded == difficulty {
                        seSlider(for: difficulty)
                    }
                }
            }
            .navigationTitle("New Game")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
        }
    }

    @ViewBuilder
    private func difficultyRow(_ diff: Difficulty) -> some View {
        Button {
            if expanded == diff {
                // Already expanded — play with current SE
                onPlay(targetSE)
            } else {
                withAnimation(.easeInOut(duration: 0.25)) {
                    expanded = diff
                    targetSE = diff.defaultSE
                }
            }
        } label: {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(diff.displayName)
                        .font(.body.weight(expanded == diff ? .semibold : .regular))
                        .foregroundStyle(.primary)
                    Text(diff.seDescription)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                difficultyIndicator(diff)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private func seSlider(for diff: Difficulty) -> some View {
        let range = diff.seRange
        VStack(spacing: 10) {
            HStack {
                Text(String(format: "%.1f", range.lowerBound))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Slider(value: $targetSE, in: range, step: 0.1)
                Text(String(format: "%.1f", range.upperBound))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Button {
                onPlay(targetSE)
            } label: {
                Text("Play (SE \(String(format: "%.1f", targetSE)))")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.regular)
        }
        .padding(.vertical, 4)
        .transition(.opacity.combined(with: .move(edge: .top)))
    }

    private func difficultyIndicator(_ diff: Difficulty) -> some View {
        let maxDots = gameManager.statistics.availableDifficulties.count
        let filled: Int = {
            switch diff {
            case .beginner: return 1
            case .easy: return 2
            case .medium: return 3
            case .intermediate: return 4
            case .hard: return 5
            case .expert: return 6
            case .master: return 7
            case .extreme: return 8
            }
        }()

        return HStack(spacing: 2) {
            ForEach(0..<maxDots, id: \.self) { i in
                Circle()
                    .fill(i < filled ? Color.accentColor : Color.secondary.opacity(0.3))
                    .frame(width: 6, height: 6)
            }
        }
    }
}

// MARK: - Progress Hub (Stats + Library + Leaderboard)

struct ProgressHubView: View {
    @State private var tab: ProgressTab = .stats

    enum ProgressTab: String, CaseIterable {
        case stats = "Stats"
        case library = "Library"
        case leaderboard = "Leaderboard"
    }

    var body: some View {
        TabView(selection: $tab) {
            StatsView()
                .tag(ProgressTab.stats)
                .tabItem {
                    Label("Stats", systemImage: "chart.bar.fill")
                }

            GameHistoryView()
                .tag(ProgressTab.library)
                .tabItem {
                    Label("Library", systemImage: "book.fill")
                }

            LeaderboardTabView()
                .tag(ProgressTab.leaderboard)
                .tabItem {
                    Label("Leaderboard", systemImage: "trophy.fill")
                }
        }
    }
}

/// Leaderboard tab — launches Game Center
struct LeaderboardTabView: View {
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Spacer()
                Image(systemName: "trophy.fill")
                    .font(.system(size: 48))
                    .foregroundStyle(.secondary)
                Text("View your rankings and compete\nwith players worldwide.")
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
                Button {
                    GameCenterManager.shared.showGameCenter()
                } label: {
                    Label("Open Game Center", systemImage: "trophy")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .padding(.horizontal, 40)
                Spacer()
            }
            .navigationTitle("Leaderboard")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }
}


// MARK: - Loading View

struct LoadingView: View {
    var body: some View {
        VStack(spacing: 24) {
            ProgressView()
                .scaleEffect(1.5)

            Text("Generating puzzle...")
                .font(.headline)
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Pause Overlay

struct PauseOverlay: View {
    @EnvironmentObject var gameManager: GameManager
    @State private var showingQuitConfirmation = false

    var body: some View {
        VStack(spacing: 24) {
            Text("PAUSED")
                .font(.largeTitle.bold())

            Button {
                gameManager.resumeGame()
            } label: {
                Label("Resume", systemImage: "play.fill")
                    .frame(width: 200)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)

            Button {
                gameManager.returnToMenu()
            } label: {
                Text("Save & Exit")
            }
            .foregroundStyle(.secondary)

            Button(role: .destructive) {
                showingQuitConfirmation = true
            } label: {
                Text("Quit Game")
            }
            .foregroundStyle(.red)
        }
        .padding(40)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
        .confirmationDialog("Quit Game?", isPresented: $showingQuitConfirmation) {
            Button("Quit", role: .destructive) {
                gameManager.quitGame()
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Your progress will be lost.")
        }
    }
}

// MARK: - Win Overlay

struct WinOverlay: View {
    @EnvironmentObject var gameManager: GameManager
    let game: GameViewModel

    var body: some View {
        VStack(spacing: 24) {
            Text("🎉")
                .font(.system(size: 60))

            Text("PUZZLE COMPLETE!")
                .font(.title.bold())

            VStack(spacing: 8) {
                StatRow(label: "Time", value: game.elapsedTimeString)
                StatRow(label: "Difficulty", value: game.difficulty.displayName)
                StatRow(label: "Hints Used", value: "\(game.hintsUsed)")
                StatRow(label: "Mistakes", value: "\(game.mistakes)")
            }
            .padding()
            .background(Color.secondary.opacity(0.1), in: RoundedRectangle(cornerRadius: 12))

            VStack(spacing: 12) {
                Button {
                    gameManager.newGame(difficulty: game.difficulty)
                } label: {
                    Label("New Game", systemImage: "plus")
                        .frame(width: 200)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)

                Button {
                    gameManager.returnToMenu()
                } label: {
                    Text("Main Menu")
                }
                .foregroundStyle(.secondary)
            }
        }
        .padding(40)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
    }
}

struct StatRow: View {
    let label: String
    let value: String

    var body: some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .fontWeight(.medium)
        }
    }
}

// MARK: - Lose Overlay

struct LoseOverlay: View {
    @EnvironmentObject var gameManager: GameManager
    let game: GameViewModel

    var body: some View {
        VStack(spacing: 24) {
            Text("😔")
                .font(.system(size: 60))

            Text("GAME OVER")
                .font(.title.bold())

            Text("Too many mistakes")
                .foregroundStyle(.secondary)

            VStack(spacing: 12) {
                Button {
                    gameManager.newGame(difficulty: game.difficulty)
                } label: {
                    Label("Try Again", systemImage: "arrow.counterclockwise")
                        .frame(width: 200)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)

                Button {
                    gameManager.returnToMenu()
                } label: {
                    Text("Main Menu")
                }
                .foregroundStyle(.secondary)
            }
        }
        .padding(40)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
    }
}

#Preview {
    ContentView()
        .environmentObject(GameManager())
}
