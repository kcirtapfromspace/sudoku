import XCTest
import SwiftUI
import SpriteKit
import ViewInspector
@testable import Sudoku

@MainActor
final class CompletionScreenTests: XCTestCase {
    func testMessageCycleReplacesItsTimerAndStopsUpdatingAfterDismissal() {
        var next = 0
        var callbacks: [() -> Void] = []
        var cycle: WinMessageCycle? = WinMessageCycle(chooseMessage: {
            next += 1
            return "Message \(next)"
        }, schedule: { callback in
            callbacks.append(callback)
            return Timer(timeInterval: 3, repeats: true) { _ in callback() }
        })
        XCTAssertEqual(cycle?.message, "Message 1")
        cycle?.start()
        let first = cycle?.timer
        first?.fire()
        XCTAssertEqual(cycle?.message, "Message 2")
        cycle?.start()
        XCTAssertFalse(first!.isValid)
        let replacement = cycle?.timer
        replacement?.fire()
        XCTAssertEqual(cycle?.message, "Message 3")
        cycle?.stop()
        XCTAssertFalse(replacement!.isValid)
        XCTAssertNil(cycle?.timer)
        weak var released = cycle
        cycle = nil
        callbacks.forEach { $0() }
        XCTAssertNil(released)
        let defaultCycle = WinMessageCycle()
        defaultCycle.start()
        defaultCycle.timer?.fire()
        XCTAssertFalse(defaultCycle.message.isEmpty)
        defaultCycle.stop()
    }

    func testCompletionPresentationReleasesNodesAndActionsOnHideOrRestart() {
        let presentation = CompletionScenePresentation { WinParticleScene() }
        XCTAssertNil(presentation.scene)
        XCTAssertFalse(presentation.isVisible)
        presentation.start(animation: .linear(duration: 0))
        let oldScene = presentation.scene!
        oldScene.addChild(SKNode())
        oldScene.run(.repeatForever(.wait(forDuration: 1)))
        XCTAssertTrue(presentation.isVisible)
        XCTAssertEqual(oldScene.scaleMode, .resizeFill)
        presentation.start(animation: .linear(duration: 0))
        XCTAssertFalse(presentation.scene === oldScene)
        XCTAssertTrue(oldScene.children.isEmpty)
        XCTAssertFalse(oldScene.hasActions())
        presentation.stop()
        XCTAssertNil(presentation.scene)
        XCTAssertFalse(presentation.isVisible)
    }

    func testWinScreenShowsStatsAndCallsOnlyChosenAction() throws {
        var dismissed = 0
        var leaderboards = 0
        let cycle = WinMessageCycle(chooseMessage: { "SUDOKU SOLVED!" })
        let presentation = CompletionScenePresentation { WinParticleScene() }
        let view = WinScreenView(time: 125, difficulty: .expert, hintsUsed: 2, mistakes: 1, seRating: 4.5,
                                 onDismiss: { dismissed += 1 }, onLeaderboard: { leaderboards += 1 },
                                 messageCycle: cycle, presentation: presentation)
        XCTAssertThrowsError(try view.inspect().find(text: "Game Stats"))
        try view.inspect().find(ViewType.ZStack.self).callOnAppear()
        XCTAssertTrue(presentation.isVisible)
        XCTAssertNotNil(presentation.scene)
        XCTAssertNotNil(cycle.timer)
        XCTAssertEqual(try view.inspect().find(text: "2:05").string(), "2:05")
        XCTAssertEqual(try view.inspect().find(text: "Expert").string(), "Expert")
        XCTAssertEqual(try view.inspect().find(text: "4.5").string(), "4.5")
        try view.inspect().find(button: "View Leaderboard").tap()
        XCTAssertEqual(leaderboards, 1)
        XCTAssertEqual(dismissed, 0)
        try view.inspect().find(ViewType.ZStack.self).callOnTapGesture()
        XCTAssertEqual(dismissed, 1)
        try ViewTestFixture.render(view)
        try view.inspect().find(ViewType.ZStack.self).callOnDisappear()
        XCTAssertNil(presentation.scene)
        XCTAssertNil(cycle.timer)
    }

    func testLossScreenRendersStatsRetryAndMenuAndReleasesScene() throws {
        var retried = 0
        var dismissed = 0
        let presentation = CompletionScenePresentation { LoseParticleScene() }
        let view = LoseScreenView(time: 245, difficulty: .hard, mistakes: 3,
                                  onDismiss: { dismissed += 1 }, onRetry: { retried += 1 }, presentation: presentation)
        XCTAssertThrowsError(try view.inspect().find(button: "Try Again"))
        try view.inspect().find(ViewType.ZStack.self).callOnAppear()
        XCTAssertEqual(try view.inspect().find(text: "4:05").string(), "4:05")
        XCTAssertEqual(try view.inspect().find(text: "3 mistakes").string(), "3 mistakes")
        try view.inspect().find(button: "Try Again").tap()
        XCTAssertEqual(retried, 1)
        XCTAssertEqual(dismissed, 0)
        try view.inspect().find(button: "Main Menu").tap()
        XCTAssertEqual(dismissed, 1)
        try ViewTestFixture.render(view)
        try view.inspect().find(ViewType.ZStack.self).callOnDisappear()
        XCTAssertNil(presentation.scene)
        let defaultView = LoseScreenView(time: 0, difficulty: .easy, mistakes: 1, onDismiss: {}, onRetry: {})
        try defaultView.inspect().find(ViewType.ZStack.self).callOnAppear()
        try defaultView.inspect().find(ViewType.ZStack.self).callOnDisappear()
        let defaultWin = WinScreenView(time: 0, difficulty: .easy, hintsUsed: 0, mistakes: 0, seRating: 1.5, onDismiss: {})
        try defaultWin.inspect().find(ViewType.ZStack.self).callOnAppear()
        try defaultWin.inspect().find(ViewType.ZStack.self).callOnDisappear()
    }

    func testEveryWinEffectSpawnsVisibleAnimatedParticlesIncludingTinyFireworks() {
        for effect in ParticleEffectType.allCases {
            let scene = WinParticleScene(size: CGSize(width: 120, height: 80))
            scene.randomness = ParticleRandomness(unit: { 0 })
            scene.chooseEffect = { effect }
            scene.didMove(to: SKView())
            XCTAssertEqual(scene.effectType, effect)
            for frame in 1...3 { scene.update(Double(frame) / 60) }
            XCTAssertFalse(scene.children.isEmpty, "Missing \(effect) particles")
            XCTAssertTrue(scene.children.allSatisfy { $0.hasActions() })
            XCTAssertTrue(scene.children.allSatisfy { $0.position.x.isFinite && $0.position.y.isFinite })
            if effect == .fireworks { XCTAssertEqual(scene.children.count, 20) }
            if effect == .sparkles {
                XCTAssertTrue(scene.children.contains { $0.alpha == 0 }, "New sparkles must survive their fade-in frame")
            }
            let fallen = SKNode()
            fallen.position.y = -100
            scene.addChild(fallen)
            let transparent = SKNode()
            transparent.alpha = 0
            scene.addChild(transparent)
            scene.update(1)
            XCTAssertNil(fallen.parent)
            XCTAssertNil(transparent.parent)
        }
    }

    func testEffectsRotateAfter300FramesAndFireworksRespectProbabilityAndCooldown() {
        let scene = WinParticleScene(size: CGSize(width: 300, height: 600))
        var selections = 0
        scene.chooseEffect = {
            selections += 1
            return selections == 1 ? .confetti : .sparkles
        }
        scene.randomness = ParticleRandomness(unit: { 1 })
        scene.didMove(to: SKView())
        for frame in 1...300 { scene.update(Double(frame) / 60) }
        XCTAssertEqual(selections, 2)
        XCTAssertEqual(scene.effectType, .sparkles)
        let fireworks = WinParticleScene(size: CGSize(width: 1, height: 1))
        fireworks.chooseEffect = { .fireworks }
        fireworks.randomness = ParticleRandomness(unit: { 1 })
        fireworks.didMove(to: SKView())
        fireworks.update(0)
        XCTAssertTrue(fireworks.children.isEmpty)
        fireworks.randomness = ParticleRandomness(unit: { 0 })
        fireworks.update(1)
        XCTAssertEqual(fireworks.children.count, 20)
        fireworks.update(2)
        XCTAssertEqual(fireworks.children.count, 20)
    }

    func testRainAndDebrisBothAnimateAndOffscreenNodesAreRemoved() {
        for unit in [0.0, 1.0] {
            let scene = LoseParticleScene(size: CGSize(width: 1, height: 1))
            scene.randomness = ParticleRandomness(unit: { unit })
            scene.didMove(to: SKView())
            scene.update(0)
            XCTAssertTrue(scene.children.isEmpty)
            scene.update(1)
            XCTAssertEqual(scene.children.count, 3)
            XCTAssertTrue(scene.children.allSatisfy { $0.hasActions() })
            let text = (scene.children.first as? SKLabelNode)?.text
            XCTAssertEqual(text, unit == 0 ? "│" : "◾")
            let old = scene.children[0]
            old.position.y = -30
            scene.update(2)
            XCTAssertNil(old.parent)
        }
    }

    func testInvalidSceneSizesDoNotAttemptInvalidRandomRanges() {
        for size in [CGSize.zero, CGSize(width: 0, height: 1), CGSize(width: 1, height: 0)] {
            let win = WinParticleScene(size: size)
            let loss = LoseParticleScene(size: size)
            win.update(0)
            loss.update(0)
            XCTAssertTrue(win.children.isEmpty)
            XCTAssertTrue(loss.children.isEmpty)
        }
        let defaultScene = WinParticleScene(size: CGSize(width: 100, height: 200))
        defaultScene.didMove(to: SKView())
        for frame in 1...3 { defaultScene.update(Double(frame)) }
        let random = ParticleRandomness()
        XCTAssertTrue((0...1).contains(random.scalar(0...1)))
        XCTAssertTrue((0...1).contains(random.duration(0...1)))
        XCTAssertTrue((0...2).contains(random.integer(0...2)))
        XCTAssertEqual(ParticleRandomness(unit: { -1 }).integer(0...2), 0)
        XCTAssertEqual(ParticleRandomness(unit: { 2 }).integer(0...2), 2)
    }
}
