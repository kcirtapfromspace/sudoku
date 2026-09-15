import Foundation
import GameKit
import SwiftUI

/// Manages Game Center authentication, leaderboards, and achievements
@MainActor
class GameCenterManager: NSObject, ObservableObject {
    static let shared = GameCenterManager()

    // MARK: - Published Properties

    @Published var isAuthenticated = false
    @Published var localPlayer: GKLocalPlayer?
    @Published var showingGameCenter = false

    // MARK: - Leaderboard IDs

    enum LeaderboardID {
        static let bestTimeBeginner = "best_time_beginner"
        static let bestTimeEasy = "best_time_easy"
        static let bestTimeMedium = "best_time_medium"
        static let bestTimeIntermediate = "best_time_intermediate"
        static let bestTimeHard = "best_time_hard"
        static let bestTimeExpert = "best_time_expert"
        static let bestTimeMaster = "best_time_master"
        static let bestTimeExtreme = "best_time_extreme"
        static let winStreak = "win_streak"

        static func forDifficulty(_ difficulty: Difficulty) -> String {
            switch difficulty {
            case .beginner: return bestTimeBeginner
            case .easy: return bestTimeEasy
            case .medium: return bestTimeMedium
            case .intermediate: return bestTimeIntermediate
            case .hard: return bestTimeHard
            case .expert: return bestTimeExpert
            case .master: return bestTimeMaster
            case .extreme: return bestTimeExtreme
            }
        }
    }

    // MARK: - Achievement IDs

    enum AchievementID {
        static let firstWin = "first_win"
        static let beginnerMaster = "beginner_master"
        static let noMistakes = "no_mistakes"
        static let speedDemon = "speed_demon"
        static let expertSolver = "expert_solver"
        static let streak5 = "streak_5"
        static let streak10 = "streak_10"
        static let konami = "konami"
    }

    // MARK: - Private Properties

    struct Dependencies {
        var authenticate: (@escaping (UIViewController?, Error?, Bool, GKLocalPlayer?) -> Void) -> Void = { completion in
            GKLocalPlayer.local.authenticateHandler = { controller, error in
                completion(controller, error, GKLocalPlayer.local.isAuthenticated, GKLocalPlayer.local)
            }
        }
        var submitScore: (Int, String) async throws -> Void = { value, leaderboard in
            try await GKLeaderboard.submitScore(value, context: 0, player: GKLocalPlayer.local, leaderboardIDs: [leaderboard])
        }
        var reportAchievement: (GKAchievement) async throws -> Void = { try await GKAchievement.report([$0]) }
        var rootController: @MainActor () -> UIViewController? = {
            (UIApplication.shared.connectedScenes.first as? UIWindowScene)?.windows.first?.rootViewController
        }
        var present: @MainActor (UIViewController, UIViewController) -> Void = { $0.present($1, animated: true) }
        var dismiss: @MainActor (UIViewController) -> Void = { $0.dismiss(animated: true) }
    }

    private let dependencies: Dependencies
    private let defaults: UserDefaults

    // Track achievement progress locally
    private let achievementProgressKey = "gc_achievement_progress"
    private var difficultyWinCounts: [Difficulty: Int] = [:]

    // MARK: - Initialization

    init(defaults: UserDefaults = .standard, dependencies: Dependencies = Dependencies()) {
        self.defaults = defaults
        self.dependencies = dependencies
        super.init()
        loadAchievementProgress()
    }

    // MARK: - Authentication

    func authenticate() {
        dependencies.authenticate { [weak self] viewController, error, authenticated, player in
            Task { @MainActor in
                if let error = error {
                    print("Game Center auth error: \(error.localizedDescription)")
                    self?.isAuthenticated = false
                    return
                }

                if viewController != nil {
                    // Player needs to log in - iOS will handle presenting this
                    self?.isAuthenticated = false
                } else if authenticated {
                    self?.isAuthenticated = true
                    self?.localPlayer = player
                    print("Game Center authenticated")
                } else {
                    self?.isAuthenticated = false
                }
            }
        }
    }

    // MARK: - Leaderboards

    @discardableResult
    func submitScore(time: TimeInterval, difficulty: Difficulty) -> Task<Void, Never>? {
        guard isAuthenticated, time.isFinite, time >= 0, time * 100 < Double(Int.max) else { return nil }

        let leaderboardID = LeaderboardID.forDifficulty(difficulty)
        // Convert time to centiseconds for precision (Game Center uses integers)
        let scoreValue = Int(time * 100)

        return Task {
            do {
                try await dependencies.submitScore(scoreValue, leaderboardID)
                print("Submitted score \(scoreValue) to \(leaderboardID)")
            } catch {
                print("Failed to submit score: \(error.localizedDescription)")
            }
        }
    }

    @discardableResult
    func submitWinStreak(_ streak: Int) -> Task<Void, Never>? {
        guard isAuthenticated, streak >= 0 else { return nil }

        return Task {
            do {
                try await dependencies.submitScore(streak, LeaderboardID.winStreak)
                print("Submitted win streak: \(streak)")
            } catch {
                print("Failed to submit win streak: \(error.localizedDescription)")
            }
        }
    }

    // MARK: - Achievements

    @discardableResult
    func unlockAchievement(_ achievementID: String, percentComplete: Double = 100.0) -> Task<Void, Never>? {
        guard isAuthenticated, percentComplete.isFinite else { return nil }

        return Task {
            let achievement = GKAchievement(identifier: achievementID)
            achievement.percentComplete = min(100, max(0, percentComplete))
            achievement.showsCompletionBanner = true

            do {
                try await dependencies.reportAchievement(achievement)
                print("Unlocked achievement: \(achievementID)")
            } catch {
                print("Failed to report achievement: \(error.localizedDescription)")
            }
        }
    }

    /// Call this when a game is won to check and unlock relevant achievements
    func checkAchievements(
        difficulty: Difficulty,
        time: TimeInterval,
        mistakes: Int,
        currentStreak: Int,
        totalWins: Int
    ) {
        // First win
        if totalWins == 1 {
            unlockAchievement(AchievementID.firstWin)
        }

        // No mistakes (Flawless)
        if mistakes == 0 {
            unlockAchievement(AchievementID.noMistakes)
        }

        // Speed demon (under 5 minutes)
        if time < 300 {
            unlockAchievement(AchievementID.speedDemon)
        }

        // Expert solver
        if difficulty == .expert {
            unlockAchievement(AchievementID.expertSolver)
        }

        // Win streaks
        if currentStreak >= 5 {
            unlockAchievement(AchievementID.streak5)
        }
        if currentStreak >= 10 {
            unlockAchievement(AchievementID.streak10)
        }

        // Track difficulty-specific wins
        difficultyWinCounts[difficulty, default: 0] += 1
        saveAchievementProgress()

        // Beginner master (10 beginner wins)
        if difficulty == .beginner && difficultyWinCounts[.beginner, default: 0] >= 10 {
            unlockAchievement(AchievementID.beginnerMaster)
        }
    }

    /// Unlock the Konami code easter egg achievement
    func unlockKonamiAchievement() {
        unlockAchievement(AchievementID.konami)
    }

    // MARK: - Game Center UI

    func showLeaderboards() {
        showGameCenterViewController(state: .leaderboards)
    }

    func showAchievements() {
        showGameCenterViewController(state: .achievements)
    }

    func showGameCenter() {
        showGameCenterViewController(state: .default)
    }

    private func showGameCenterViewController(state: GKGameCenterViewControllerState) {
        // Show Game Center UI - iOS will prompt for sign-in if needed
        let gcViewController = GKGameCenterViewController(state: state)
        gcViewController.gameCenterDelegate = self

        // Get the root view controller to present from
        if let rootViewController = dependencies.rootController() {
            var topController = rootViewController
            while let presented = topController.presentedViewController {
                topController = presented
            }
            dependencies.present(topController, gcViewController)
        } else {
            print("Could not find root view controller to present Game Center")
        }
    }

    // MARK: - Persistence

    private func loadAchievementProgress() {
        if let data = defaults.data(forKey: achievementProgressKey),
           let counts = try? JSONDecoder().decode([String: Int].self, from: data) {
            for (key, value) in counts {
                if let difficulty = Difficulty(rawValue: key) {
                    difficultyWinCounts[difficulty] = value
                }
            }
        }
    }

    private func saveAchievementProgress() {
        var stringCounts: [String: Int] = [:]
        for (difficulty, count) in difficultyWinCounts {
            stringCounts[difficulty.rawValue] = count
        }
        if let data = try? JSONEncoder().encode(stringCounts) {
            defaults.set(data, forKey: achievementProgressKey)
        }
    }
}

// MARK: - GKGameCenterControllerDelegate

extension GameCenterManager: GKGameCenterControllerDelegate {
    nonisolated func gameCenterViewControllerDidFinish(_ gameCenterViewController: GKGameCenterViewController) {
        Task { @MainActor in
            self.dependencies.dismiss(gameCenterViewController)
        }
    }
}
