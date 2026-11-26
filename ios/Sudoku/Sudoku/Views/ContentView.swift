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
                        .allowsHitTesting(false)
                        .accessibilityRepresentation { EmptyView() }
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
                            gameManager.newGame(matching: game)
                        }
                    )
                    .transition(.opacity)
                }
            }
        }
        .preferredColorScheme(colorScheme)
        .alert("Rating unavailable", isPresented: $gameManager.showRatingUnavailable) {
            Button("OK") {}
        } message: {
            Text("Couldn't start a puzzle with the selected SE rating. Please choose a rating again.")
        }
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
    @State private var showingSettings = false
    @State private var showingProgress = false
    @State private var showingImport = false

    var body: some View {
        NavigationStack {
            GeometryReader { geometry in
                ScrollView {
                    VStack(spacing: 24) {
                        Spacer(minLength: 0)
                        Text("SUDOKU")
                            .font(.system(size: 48, weight: .bold, design: .rounded))
                            .accessibilityAddTraits(.isHeader)
                        Spacer(minLength: 0)

                        VStack(spacing: 24) {
                            if gameManager.hasSavedGame {
                                Button {
                                    gameManager.resumeGame()
                                } label: {
                                    Label("Continue", systemImage: "play.fill")
                                        .frame(maxWidth: .infinity)
                                }
                                .buttonStyle(.bordered)
                                .controlSize(.large)
                            }
                            NewGameSelection(choices: gameManager.statistics.availableDifficultyChoices,
                                             preferredRating: $gameManager.preferredNewGameRating) { rating in
                                gameManager.newGameWithSE(targetSE: rating)
                            }
                        }
                        .frame(maxWidth: 400)
                        .padding(.horizontal, 32)

                        Spacer(minLength: 0)
                        menuLinks
                    }
                    .padding(.vertical, 24)
                    .frame(maxWidth: .infinity, minHeight: geometry.size.height)
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

    private var menuLinks: some View {
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

            if gameManager.settings.cameraImportEnabled {
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
            }

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
    }
}

// MARK: - Progress Hub (Stats + Library + Leaderboard)

struct ProgressHubView: View {
    @EnvironmentObject var gameManager: GameManager
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

            GameHistoryView(historyManager: gameManager.historyManager)
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
                    gameManager.newGame(matching: game)
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
                    gameManager.newGame(matching: game)
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
