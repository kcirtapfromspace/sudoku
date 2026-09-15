import SwiftUI
import SpriteKit
import UIKit

// MARK: - Win Messages (matching TUI/WASM)

private let winMessages = [
    "SUDOKU SOLVED!",
    "BRILLIANT!",
    "AMAZING!",
    "CHAMPION!",
    "PERFECT!",
    "EXCELLENT!",
    "CONGRATULATIONS!",
    "WELL DONE!",
    "ON FIRE!",
    "INCREDIBLE!",
    "SUPERSTAR!",
    "LEGENDARY!",
    "FLAWLESS!",
    "MAGNIFICENT!"
]

// MARK: - Particle Effect Types

enum ParticleEffectType: CaseIterable {
    case confetti
    case fireworks
    case sparkles
    case rainbow
}

/// Owns the repeating message timer so dismissing a result stops all future updates.
final class WinMessageCycle: ObservableObject {
    @Published private(set) var message: String
    private(set) var timer: Timer?
    private let chooseMessage: () -> String
    private let schedule: (@escaping () -> Void) -> Timer

    init(chooseMessage: @escaping () -> String = { winMessages.randomElement()! },
         schedule: @escaping (@escaping () -> Void) -> Timer = { callback in
             Timer.scheduledTimer(withTimeInterval: 3, repeats: true) { _ in callback() }
         }) {
        self.chooseMessage = chooseMessage
        self.schedule = schedule
        message = chooseMessage()
    }

    func start() {
        stop()
        timer = schedule { [weak self] in
            guard let self else { return }
            withAnimation(.easeInOut(duration: 0.3)) { self.message = self.chooseMessage() }
        }
    }

    func stop() {
        timer?.invalidate()
        timer = nil
    }

    deinit { timer?.invalidate() }
}

/// Keeps SpriteKit nodes scoped to the result screen's visible lifetime.
final class CompletionScenePresentation: ObservableObject {
    @Published private(set) var scene: SKScene?
    @Published private(set) var isVisible = false
    private let makeScene: () -> SKScene

    init(makeScene: @escaping () -> SKScene) { self.makeScene = makeScene }

    func start(animation: Animation) {
        stop()
        let scene = makeScene()
        scene.scaleMode = .resizeFill
        scene.backgroundColor = .clear
        self.scene = scene
        withAnimation(animation) { isVisible = true }
    }

    func stop() {
        scene?.removeAllActions()
        scene?.removeAllChildren()
        scene = nil
        isVisible = false
    }
}

/// One bounded random source keeps effect selection and particle geometry reproducible.
struct ParticleRandomness {
    var unit: () -> Double = { Double.random(in: 0...1) }

    func scalar(_ range: ClosedRange<CGFloat>) -> CGFloat {
        range.lowerBound + (range.upperBound - range.lowerBound) * CGFloat(min(1, max(0, unit())))
    }

    func duration(_ range: ClosedRange<Double>) -> Double {
        range.lowerBound + (range.upperBound - range.lowerBound) * min(1, max(0, unit()))
    }

    func integer(_ range: ClosedRange<Int>) -> Int {
        min(range.upperBound, range.lowerBound + Int(Double(range.count) * min(1, max(0, unit()))))
    }
}

// MARK: - Win Screen View

struct WinScreenView: View {
    let time: TimeInterval
    let difficulty: Difficulty
    let hintsUsed: Int
    let mistakes: Int
    let seRating: Float
    let onDismiss: () -> Void
    let onLeaderboard: @MainActor () -> Void

    @StateObject private var messageCycle: WinMessageCycle
    @StateObject private var presentation: CompletionScenePresentation

    init(time: TimeInterval, difficulty: Difficulty, hintsUsed: Int, mistakes: Int, seRating: Float,
         onDismiss: @escaping () -> Void,
         onLeaderboard: @escaping @MainActor () -> Void = { GameCenterManager.shared.showLeaderboards() },
         messageCycle: WinMessageCycle = WinMessageCycle(),
         presentation: CompletionScenePresentation = CompletionScenePresentation(makeScene: { WinParticleScene() })) {
        self.time = time
        self.difficulty = difficulty
        self.hintsUsed = hintsUsed
        self.mistakes = mistakes
        self.seRating = seRating
        self.onDismiss = onDismiss
        self.onLeaderboard = onLeaderboard
        _messageCycle = StateObject(wrappedValue: messageCycle)
        _presentation = StateObject(wrappedValue: presentation)
    }

    var body: some View {
        ZStack {
            // Animated gradient background
            AnimatedGradientBackground()

            // SpriteKit particle layer
            if let scene = presentation.scene {
                SpriteView(scene: scene, options: [.allowsTransparency])
                    .ignoresSafeArea()
            }

            // Content
            VStack(spacing: 30) {
                Spacer()

                // Trophy icon
                Text("🏆")
                    .font(.system(size: 80))
                    .shadow(color: .yellow.opacity(0.5), radius: 20)

                // Win message
                Text(messageCycle.message)
                    .font(.system(size: 36, weight: .black, design: .rounded))
                    .foregroundStyle(
                        LinearGradient(
                            colors: [.yellow, .orange, .pink],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .shadow(color: .black.opacity(0.8), radius: 2, x: 0, y: 2)
                    .shadow(color: .black.opacity(0.5), radius: 8, x: 0, y: 4)
                    .multilineTextAlignment(.center)

                // Stats card
                if presentation.isVisible {
                    StatsCard(time: time, difficulty: difficulty, hintsUsed: hintsUsed, mistakes: mistakes, seRating: seRating)
                        .transition(.scale.combined(with: .opacity))
                }

                Spacer()

                // Leaderboard button
                Button {
                    onLeaderboard()
                } label: {
                    Label("View Leaderboard", systemImage: "trophy.fill")
                        .font(.headline)
                        .foregroundStyle(.yellow)
                        .padding(.horizontal, 24)
                        .padding(.vertical, 12)
                        .background(
                            Capsule()
                                .fill(.black.opacity(0.4))
                                .overlay(Capsule().strokeBorder(.yellow.opacity(0.5), lineWidth: 1))
                        )
                }

                // Tap to continue
                Text("Tap anywhere to continue")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.9))
                    .padding(.bottom, 40)
            }
            .padding()
        }
        .onTapGesture {
            onDismiss()
        }
        .onAppear {
            presentation.start(animation: .spring(response: 0.6, dampingFraction: 0.8).delay(0.5))
            messageCycle.start()
        }
        .onDisappear {
            messageCycle.stop()
            presentation.stop()
        }
    }
}

// MARK: - Stats Card

private struct StatsCard: View {
    let time: TimeInterval
    let difficulty: Difficulty
    let hintsUsed: Int
    let mistakes: Int
    let seRating: Float

    var body: some View {
        VStack(spacing: 16) {
            Text("Game Stats")
                .font(.headline)
                .foregroundStyle(.white)

            HStack(spacing: 30) {
                WinStatItem(icon: "clock", label: "Time", value: formatTime(time))
                WinStatItem(icon: "chart.bar", label: "Difficulty", value: difficulty.displayName)
            }

            HStack(spacing: 30) {
                WinStatItem(icon: "lightbulb", label: "Hints", value: "\(hintsUsed)")
                WinStatItem(icon: "xmark.circle", label: "Mistakes", value: "\(mistakes)")
            }

            HStack(spacing: 30) {
                WinStatItem(icon: "gauge.medium", label: "SE Rating", value: String(format: "%.1f", seRating))
            }
        }
        .padding(24)
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(.black.opacity(0.5))
                .overlay(
                    RoundedRectangle(cornerRadius: 20)
                        .strokeBorder(.white.opacity(0.15), lineWidth: 1)
                )
                .shadow(color: .black.opacity(0.3), radius: 10)
        )
    }

    private func formatTime(_ interval: TimeInterval) -> String {
        let mins = Int(interval) / 60
        let secs = Int(interval) % 60
        return String(format: "%d:%02d", mins, secs)
    }
}

private struct WinStatItem: View {
    let icon: String
    let label: String
    let value: String

    var body: some View {
        VStack(spacing: 4) {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(.yellow)
            Text(value)
                .font(.title3.bold())
                .foregroundStyle(.white)
            Text(label)
                .font(.caption)
                .foregroundStyle(.white.opacity(0.9))
        }
        .frame(minWidth: 80)
    }
}

// MARK: - Animated Gradient Background

private struct AnimatedGradientBackground: View {
    @State private var animateGradient = false

    var body: some View {
        LinearGradient(
            colors: [
                Color(hue: animateGradient ? 0.7 : 0.8, saturation: 0.8, brightness: 0.22),
                Color(hue: animateGradient ? 0.85 : 0.75, saturation: 0.7, brightness: 0.14),
                Color(hue: animateGradient ? 0.6 : 0.9, saturation: 0.9, brightness: 0.08)
            ],
            startPoint: animateGradient ? .topLeading : .bottomTrailing,
            endPoint: animateGradient ? .bottomTrailing : .topLeading
        )
        .ignoresSafeArea()
        .onAppear {
            withAnimation(.easeInOut(duration: 4.0).repeatForever(autoreverses: true)) {
                animateGradient.toggle()
            }
        }
    }
}

// MARK: - SpriteKit Particle Scene

class WinParticleScene: SKScene {
    private(set) var effectType: ParticleEffectType = .confetti
    var randomness = ParticleRandomness()
    var chooseEffect: () -> ParticleEffectType = { ParticleEffectType.allCases.randomElement()! }
    private var frameCount: Int = 0
    private var fireworkCooldown: Int = 0

    override func didMove(to view: SKView) {
        backgroundColor = .clear
        effectType = chooseEffect()
    }

    override func update(_ currentTime: TimeInterval) {
        guard size.width.isFinite, size.height.isFinite, size.width > 0, size.height > 0 else { return }
        frameCount += 1

        // Switch effects every 5 seconds
        if frameCount % 300 == 0 {
            effectType = chooseEffect()
        }

        // Spawn particles based on effect type
        switch effectType {
        case .confetti:
            spawnConfetti()
        case .fireworks:
            spawnFireworks()
        case .sparkles:
            spawnSparkles()
        case .rainbow:
            spawnRainbow()
        }

        // Clean up old particles
        children.filter { $0.position.y < -50 || (!$0.hasActions() && $0.alpha < 0.01) }.forEach { $0.removeFromParent() }
    }

    private func spawnConfetti() {
        guard frameCount % 3 == 0 else { return }

        let confettiChars = ["✦", "✧", "◆", "◇", "○", "●", "■", "□", "▲", "▽", "★"]
        let colors: [UIColor] = [.systemRed, .systemOrange, .systemYellow, .systemGreen, .systemBlue, .systemPurple, .systemPink]

        for _ in 0..<2 {
            let label = SKLabelNode(text: confettiChars[randomness.integer(0...(confettiChars.count - 1))])
            label.fontSize = randomness.scalar(14...24)
            label.fontColor = colors[randomness.integer(0...(colors.count - 1))]
            label.position = CGPoint(x: randomness.scalar(0...size.width), y: size.height + 20)

            let fall = SKAction.moveBy(x: randomness.scalar(-30...30), y: -size.height - 50, duration: randomness.duration(4...7))
            let rotate = SKAction.rotate(byAngle: randomness.scalar(-4...4), duration: randomness.duration(2...4))
            let fade = SKAction.sequence([
                SKAction.wait(forDuration: randomness.duration(3...5)),
                SKAction.fadeOut(withDuration: 1.0)
            ])

            label.run(SKAction.sequence([.group([fall, rotate, fade]), .removeFromParent()]))

            addChild(label)
        }
    }

    private func spawnFireworks() {
        if fireworkCooldown > 0 {
            fireworkCooldown -= 1
            return
        }

        guard randomness.integer(0...100) < 5 else { return }

        let margin = min(100, size.width / 2)
        let x = randomness.scalar(margin...(size.width - margin))
        let y = randomness.scalar(size.height * 0.4...size.height * 0.8)
        let color = UIColor(
            hue: randomness.scalar(0...1),
            saturation: 1.0,
            brightness: 1.0,
            alpha: 1.0
        )

        for _ in 0..<20 {
            let particle = SKShapeNode(circleOfRadius: randomness.scalar(3...6))
            particle.fillColor = color
            particle.strokeColor = .clear
            particle.position = CGPoint(x: x, y: y)
            particle.alpha = 1.0

            let angle = randomness.scalar(0...(2 * .pi))
            let speed = randomness.scalar(50...150)
            let dx = cos(angle) * speed
            let dy = sin(angle) * speed

            let move = SKAction.moveBy(x: dx, y: dy - 100, duration: randomness.duration(1...2))
            move.timingMode = .easeOut
            let fade = SKAction.fadeOut(withDuration: randomness.duration(0.8...1.5))
            let scale = SKAction.scale(to: 0.2, duration: 1.5)

            particle.run(SKAction.sequence([.group([move, fade, scale]), .removeFromParent()]))

            addChild(particle)
        }

        fireworkCooldown = 20
    }

    private func spawnSparkles() {
        guard frameCount % 2 == 0 else { return }

        let sparkleChars = ["✨", "⭐", "✦", "★", "☆", "✫"]

        for _ in 0..<3 {
            let label = SKLabelNode(text: sparkleChars[randomness.integer(0...(sparkleChars.count - 1))])
            label.fontSize = randomness.scalar(16...28)
            label.fontColor = UIColor(white: 1.0, alpha: randomness.scalar(0.7...1.0))
            label.position = CGPoint(
                x: randomness.scalar(0...size.width),
                y: randomness.scalar(0...size.height)
            )
            label.alpha = 0

            let fadeIn = SKAction.fadeIn(withDuration: 0.2)
            let wait = SKAction.wait(forDuration: randomness.duration(0.3...0.8))
            let fadeOut = SKAction.fadeOut(withDuration: 0.3)
            let scale = SKAction.sequence([
                SKAction.scale(to: 1.3, duration: 0.2),
                SKAction.scale(to: 1.0, duration: 0.3)
            ])

            label.run(SKAction.sequence([
                SKAction.group([fadeIn, scale]),
                wait,
                fadeOut,
                .removeFromParent()
            ]))

            addChild(label)
        }
    }

    private func spawnRainbow() {
        guard frameCount % 2 == 0 else { return }

        let hue = CGFloat(frameCount % 360) / 360.0

        for _ in 0..<2 {
            let particle = SKShapeNode(rectOf: CGSize(width: randomness.scalar(8...16), height: randomness.scalar(15...25)))
            particle.fillColor = UIColor(hue: (hue + randomness.scalar(0...0.2)).truncatingRemainder(dividingBy: 1.0), saturation: 1.0, brightness: 1.0, alpha: 1.0)
            particle.strokeColor = .clear
            particle.position = CGPoint(x: randomness.scalar(0...size.width), y: size.height + 20)

            let fall = SKAction.moveBy(x: randomness.scalar(-20...20), y: -size.height - 50, duration: randomness.duration(3...5))
            let fade = SKAction.sequence([
                SKAction.wait(forDuration: randomness.duration(2...4)),
                SKAction.fadeOut(withDuration: 1.0)
            ])

            particle.run(SKAction.sequence([.group([fall, fade]), .removeFromParent()]))

            addChild(particle)
        }
    }
}

// MARK: - Preview

#Preview {
    WinScreenView(
        time: 185,
        difficulty: .medium,
        hintsUsed: 2,
        mistakes: 1,
        seRating: 3.4,
        onDismiss: {}
    )
}
