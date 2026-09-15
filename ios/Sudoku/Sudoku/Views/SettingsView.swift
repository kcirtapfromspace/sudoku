import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var gameManager: GameManager
    @Environment(\.dismiss) var dismiss
    @State private var showingResetConfirmation = false

    var body: some View {
        NavigationStack {
            SettingsForm(showingResetConfirmation: $showingResetConfirmation) {
                gameManager.saveSettings()
                dismiss()
            }
        }
    }
}

/// Settings controls share the same state and persistence as the containing sheet.
struct SettingsForm: View {
    @EnvironmentObject var gameManager: GameManager
    @Binding var showingResetConfirmation: Bool
    var onDone: () -> Void

    var body: some View {
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

            // Experimental
            Section("Experimental") {
                Toggle("Camera Import", isOn: $gameManager.settings.cameraImportEnabled)
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
                    Text(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "Unknown")
                        .foregroundStyle(.secondary)
                }

                Link(destination: URL(string: "https://github.com/kcirtapfromspace/sudoku")!) {
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
                Button("Done", action: onDone)
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
            Text("This will permanently delete the statistics stored on this device. Game Center leaderboard entries will remain.")
        }
    }
}

#Preview {
    SettingsView()
        .environmentObject(GameManager())
}
