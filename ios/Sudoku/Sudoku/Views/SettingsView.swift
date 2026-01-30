import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var gameManager: GameManager
    @Environment(\.dismiss) var dismiss
    @State private var showingResetConfirmation = false

    var body: some View {
        NavigationStack {
            List {
                // Appearance
                Section("Appearance") {
                    Picker("Theme", selection: $gameManager.settings.theme) {
                        ForEach(GameSettings.ThemeSetting.allCases, id: \.self) { theme in
                            Text(theme.rawValue).tag(theme)
                        }
                    }
                }

                // Gameplay
                Section("Gameplay") {
                    Toggle("Show Timer", isOn: $gameManager.settings.timerVisible)

                    Toggle("Mistake Limit", isOn: $gameManager.settings.mistakeLimitEnabled)

                    if gameManager.settings.mistakeLimitEnabled {
                        Stepper("Max Mistakes: \(gameManager.settings.mistakeLimit)",
                                value: $gameManager.settings.mistakeLimit,
                                in: 1...10)
                    }

                    Toggle("Show Errors Immediately", isOn: $gameManager.settings.showErrorsImmediately)
                }

                // Helpers
                Section("Helpers") {
                    Toggle("Highlight Related Cells", isOn: $gameManager.settings.highlightRelatedCells)

                    Toggle("Highlight Same Numbers", isOn: $gameManager.settings.highlightSameNumbers)

                    Toggle("Ghost Hints", isOn: $gameManager.settings.ghostHintsEnabled)

                    Toggle("Highlight Valid Cells", isOn: $gameManager.settings.highlightValidCells)

                    Toggle("Auto-Fill Notes on Start", isOn: $gameManager.settings.autoFillCandidates)
                }

                // Feedback
                Section("Feedback") {
                    Toggle("Haptic Feedback", isOn: $gameManager.settings.hapticsEnabled)
                    Toggle("Celebrations", isOn: $gameManager.settings.celebrationsEnabled)
                }

                // Data
                Section {
                    Button(role: .destructive) {
                        showingResetConfirmation = true
                    } label: {
                        Label("Reset Statistics", systemImage: "trash")
                    }
                }

                // About
                Section("About") {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text("1.0.0")
                            .foregroundStyle(.secondary)
                    }

                    Link(destination: URL(string: "https://github.com")!) {
                        HStack {
                            Text("Source Code")
                            Spacer()
                            Image(systemName: "arrow.up.right")
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .navigationTitle("Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        gameManager.saveSettings()
                        dismiss()
                    }
                }
            }
            .confirmationDialog("Reset Statistics",
                                isPresented: $showingResetConfirmation,
                                titleVisibility: .visible) {
                Button("Reset", role: .destructive) {
                    gameManager.resetStatistics()
                }
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("This will permanently delete all your game statistics and leaderboard entries.")
            }
        }
    }
}

#Preview {
    SettingsView()
        .environmentObject(GameManager())
}
