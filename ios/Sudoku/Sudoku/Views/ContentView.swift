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
    @State private var showingDifficultyPicker = false
    @State private var showingSettings = false
    @State private var showingStats = false

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
                        showingDifficultyPicker = true
                    } label: {
                        Label("New Game", systemImage: "plus")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                }
                .padding(.horizontal, 40)

                Spacer()

                // Bottom buttons
                HStack(spacing: 40) {
                    Button {
                        showingStats = true
                    } label: {
                        VStack {
                            Image(systemName: "chart.bar.fill")
                                .font(.title2)
                            Text("Stats")
                                .font(.caption)
                        }
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
                }
                .foregroundStyle(.secondary)
                .padding(.bottom, 40)
            }
            .sheet(isPresented: $showingDifficultyPicker) {
                DifficultyPickerView { difficulty in
                    gameManager.newGame(difficulty: difficulty)
                    showingDifficultyPicker = false
                }
                .presentationDetents([.medium])
            }
            .sheet(isPresented: $showingSettings) {
                SettingsView()
            }
            .sheet(isPresented: $showingStats) {
                StatsView()
            }
        }
    }
}

// MARK: - Difficulty Picker

struct DifficultyPickerView: View {
    let onSelect: (Difficulty) -> Void
    @Environment(\.dismiss) var dismiss

    var body: some View {
        NavigationStack {
            List {
                ForEach(Difficulty.allCases) { difficulty in
                    Button {
                        onSelect(difficulty)
                    } label: {
                        HStack {
                            Text(difficulty.displayName)
                                .foregroundStyle(.primary)
                            Spacer()
                            difficultyIndicator(difficulty)
                        }
                    }
                }
            }
            .navigationTitle("Select Difficulty")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
        }
    }

    private func difficultyIndicator(_ diff: Difficulty) -> some View {
        let filled: Int = {
            switch diff {
            case .beginner: return 1
            case .easy: return 2
            case .medium: return 3
            case .intermediate: return 4
            case .hard: return 5
            case .expert: return 6
            }
        }()

        return HStack(spacing: 2) {
            ForEach(0..<6) { i in
                Circle()
                    .fill(i < filled ? Color.accentColor : Color.secondary.opacity(0.3))
                    .frame(width: 8, height: 8)
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
