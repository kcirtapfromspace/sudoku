import XCTest
import GameKit
import UIKit
@testable import Sudoku

@MainActor
final class GameCenterServiceTests: XCTestCase {
    func testAuthenticationStates() async {
        for state in 0..<4 {
            var dependencies = GameCenterManager.Dependencies()
            dependencies.authenticate = { completion in
                completion(state == 1 ? UIViewController() : nil, state == 0 ? URLError(.notConnectedToInternet) : nil, state == 2, state == 2 ? GKLocalPlayer.local : nil)
            }
            let manager = GameCenterManager(defaults: ServiceFixtures.defaults(), dependencies: dependencies)
            manager.authenticate()
            for _ in 0..<10 { await Task.yield() }
            XCTAssertEqual(manager.isAuthenticated, state == 2)
            XCTAssertEqual(manager.localPlayer != nil, state == 2)
        }
    }
    func testScoreAndAchievementSubmissionGuardsSuccessAndFailure() async {
        var scores: [(Int, String)] = [], achievements: [(String, Double)] = []
        var shouldFail = false
        var dependencies = GameCenterManager.Dependencies()
        dependencies.submitScore = { value, board in
            if shouldFail { throw URLError(.timedOut) }; scores.append((value, board))
        }
        dependencies.reportAchievement = { achievement in
            XCTAssertTrue(achievement.showsCompletionBanner)
            if shouldFail { throw URLError(.timedOut) }
            achievements.append((achievement.identifier, achievement.percentComplete))
        }
        let manager = GameCenterManager(defaults: ServiceFixtures.defaults(), dependencies: dependencies)
        XCTAssertNil(manager.submitScore(time: 1, difficulty: .easy)); XCTAssertNil(manager.submitWinStreak(1)); XCTAssertNil(manager.unlockAchievement("none"))
        manager.isAuthenticated = true
        for invalid in [-1.0, Double.infinity, Double.nan, Double.greatestFiniteMagnitude] { XCTAssertNil(manager.submitScore(time: invalid, difficulty: .easy)) }
        XCTAssertNil(manager.submitWinStreak(-1)); XCTAssertNil(manager.unlockAchievement("none", percentComplete: .nan))
        for difficulty in Difficulty.allCases {
            await manager.submitScore(time: 12.34, difficulty: difficulty)?.value
            XCTAssertEqual(scores.last?.0, 1234)
            XCTAssertEqual(scores.last?.1, GameCenterManager.LeaderboardID.forDifficulty(difficulty))
        }
        await manager.submitWinStreak(8)?.value
        XCTAssertEqual(scores.last?.1, "win_streak"); XCTAssertEqual(scores.last?.0, 8)
        await manager.unlockAchievement("custom", percentComplete: 150)?.value
        XCTAssertEqual(achievements.last?.0, "custom"); XCTAssertEqual(achievements.last?.1, 100)
        await manager.unlockAchievement("negative", percentComplete: -10)?.value
        XCTAssertEqual(achievements.last?.1, 0)
        shouldFail = true
        await manager.submitScore(time: 1, difficulty: .hard)?.value
        await manager.submitWinStreak(1)?.value
        await manager.unlockAchievement("failure")?.value
        XCTAssertEqual(scores.count, 9); XCTAssertEqual(achievements.count, 2)
    }
    func testAchievementThresholdsPersistenceAndKonami() async throws {
        let defaults = ServiceFixtures.defaults()
        defaults.set(try JSONEncoder().encode(["Beginner": 9, "not-a-difficulty": 3]), forKey: "gc_achievement_progress")
        var reported: [String] = []
        var dependencies = GameCenterManager.Dependencies()
        dependencies.reportAchievement = { reported.append($0.identifier) }
        let manager = GameCenterManager(defaults: defaults, dependencies: dependencies)
        manager.isAuthenticated = true
        manager.checkAchievements(difficulty: .beginner, time: 300, mistakes: 1, currentStreak: 0, totalWins: 2)
        for _ in 0..<20 { await Task.yield() }
        XCTAssertEqual(reported, [GameCenterManager.AchievementID.beginnerMaster])
        manager.checkAchievements(difficulty: .expert, time: 299, mistakes: 0, currentStreak: 10, totalWins: 1)
        manager.unlockKonamiAchievement()
        for _ in 0..<20 { await Task.yield() }
        XCTAssertEqual(Set(reported), Set(["beginner_master", "first_win", "no_mistakes", "speed_demon", "expert_solver", "streak_5", "streak_10", "konami"]))
        let saved = try JSONDecoder().decode([String: Int].self, from: defaults.data(forKey: "gc_achievement_progress")!)
        XCTAssertEqual(saved["Beginner"], 10); XCTAssertEqual(saved["Expert"], 1); XCTAssertNil(saved["not-a-difficulty"])
        defaults.set(Data("invalid".utf8), forKey: "gc_achievement_progress")
        let corrupt = GameCenterManager(defaults: defaults, dependencies: dependencies)
        corrupt.checkAchievements(difficulty: .easy, time: 600, mistakes: 3, currentStreak: 0, totalWins: 2)
    }
    func testPresentationAndDismissalUseInjectedBoundary() async {
        var dependencies = GameCenterManager.Dependencies()
        let root = UIViewController()
        var presented: [UIViewController] = [], dismissed = false
        dependencies.rootController = { root }
        dependencies.present = { parent, child in XCTAssertTrue(parent === root); presented.append(child) }
        dependencies.dismiss = { _ in dismissed = true }
        let manager = GameCenterManager(defaults: ServiceFixtures.defaults(), dependencies: dependencies)
        manager.showLeaderboards(); manager.showAchievements(); manager.showGameCenter()
        XCTAssertEqual(presented.count, 3)
        let controller = presented[0] as! GKGameCenterViewController
        XCTAssertTrue(controller.gameCenterDelegate === manager)
        manager.gameCenterViewControllerDidFinish(controller)
        for _ in 0..<10 { await Task.yield() }
        XCTAssertTrue(dismissed)
        dependencies.rootController = { nil }
        GameCenterManager(defaults: ServiceFixtures.defaults(), dependencies: dependencies).showGameCenter()
    }
    func testPresentationTraversesExistingModalAndUIKitAdapters() {
        final class Controller: UIViewController {
            var child: UIViewController?
            var didPresent = false
            var didDismiss = false
            override var presentedViewController: UIViewController? { child }
            override func present(_ viewControllerToPresent: UIViewController, animated: Bool, completion: (() -> Void)? = nil) {
                didPresent = true; completion?()
            }
            override func dismiss(animated: Bool, completion: (() -> Void)? = nil) {
                didDismiss = true; completion?()
            }
        }
        let root = Controller(), child = Controller()
        root.child = child
        var dependencies = GameCenterManager.Dependencies()
        dependencies.rootController = { root }
        dependencies.present = { parent, _ in XCTAssertTrue(parent === child) }
        GameCenterManager(defaults: ServiceFixtures.defaults(), dependencies: dependencies).showGameCenter()
        let defaults = GameCenterManager.Dependencies()
        defaults.present(root, child); XCTAssertTrue(root.didPresent)
        defaults.dismiss(root); XCTAssertTrue(root.didDismiss)
        _ = defaults.rootController()
    }

}
