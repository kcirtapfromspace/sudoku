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

// MARK: - Win Screen View

struct WinScreenView: View {
    let time: TimeInterval
    let difficulty: Difficulty
    let hintsUsed: Int
    let mistakes: Int
    let onDismiss: () -> Void

    @State private var message: String = winMessages.randomElement()!
    @State private var showStats = false
    @State private var particleScene: WinParticleScene?

    var body: some View {
        ZStack {
            // Animated gradient background
            AnimatedGradientBackground()

            // SpriteKit particle layer
            if let scene = particleScene {
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
                Text(message)
                    .font(.system(size: 36, weight: .black, design: .rounded))
                    .foregroundStyle(
                        LinearGradient(
                            colors: [.yellow, .orange, .pink],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .shadow(color: .black.opacity(0.3), radius: 2, x: 0, y: 2)
                    .multilineTextAlignment(.center)

                // Stats card
                if showStats {
                    StatsCard(time: time, difficulty: difficulty, hintsUsed: hintsUsed, mistakes: mistakes)
                        .transition(.scale.combined(with: .opacity))
                }

                Spacer()

                // Tap to continue
                Text("Tap anywhere to continue")
                    .font(.subheadline)
                    .foregroundStyle(.white.opacity(0.7))
                    .padding(.bottom, 40)
            }
            .padding()
        }
        .onTapGesture {
            onDismiss()
        }
        .onAppear {
            // Create particle scene
            let scene = WinParticleScene()
            scene.scaleMode = .resizeFill
            scene.backgroundColor = .clear
            particleScene = scene

            // Show stats with delay
            withAnimation(.spring(response: 0.6, dampingFraction: 0.8).delay(0.5)) {
                showStats = true
            }

            // Cycle messages
            startMessageCycle()
        }
    }

    private func startMessageCycle() {
        Timer.scheduledTimer(withTimeInterval: 3.0, repeats: true) { _ in
            withAnimation(.easeInOut(duration: 0.3)) {
                message = winMessages.randomElement()!
            }
        }
    }
}

// MARK: - Stats Card

private struct StatsCard: View {
    let time: TimeInterval
    let difficulty: Difficulty
    let hintsUsed: Int
    let mistakes: Int

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
        }
        .padding(24)
        .background(
            RoundedRectangle(cornerRadius: 20)
                .fill(.ultraThinMaterial)
                .shadow(color: .black.opacity(0.2), radius: 10)
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
                .foregroundStyle(.white.opacity(0.7))
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
                Color(hue: animateGradient ? 0.7 : 0.8, saturation: 0.8, brightness: 0.3),
                Color(hue: animateGradient ? 0.85 : 0.75, saturation: 0.7, brightness: 0.2),
                Color(hue: animateGradient ? 0.6 : 0.9, saturation: 0.9, brightness: 0.15)
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
    private var effectType: ParticleEffectType = .confetti
    private var frameCount: Int = 0
    private var fireworkCooldown: Int = 0

    override func didMove(to view: SKView) {
        backgroundColor = .clear
        effectType = ParticleEffectType.allCases.randomElement()!
    }

    override func update(_ currentTime: TimeInterval) {
        frameCount += 1

        // Switch effects every 5 seconds
        if frameCount % 300 == 0 {
            effectType = ParticleEffectType.allCases.randomElement()!
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
        children.filter { $0.position.y < -50 || $0.alpha < 0.01 }.forEach { $0.removeFromParent() }
    }

    private func spawnConfetti() {
        guard frameCount % 3 == 0 else { return }

        let confettiChars = ["✦", "✧", "◆", "◇", "○", "●", "■", "□", "▲", "▽", "★"]
        let colors: [UIColor] = [.systemRed, .systemOrange, .systemYellow, .systemGreen, .systemBlue, .systemPurple, .systemPink]

        for _ in 0..<2 {
            let label = SKLabelNode(text: confettiChars.randomElement()!)
            label.fontSize = CGFloat.random(in: 14...24)
            label.fontColor = colors.randomElement()!
            label.position = CGPoint(x: CGFloat.random(in: 0...size.width), y: size.height + 20)

            let fall = SKAction.moveBy(x: CGFloat.random(in: -30...30), y: -size.height - 50, duration: Double.random(in: 4...7))
            let rotate = SKAction.rotate(byAngle: CGFloat.random(in: -4...4), duration: Double.random(in: 2...4))
            let fade = SKAction.sequence([
                SKAction.wait(forDuration: Double.random(in: 3...5)),
                SKAction.fadeOut(withDuration: 1.0)
            ])

            label.run(SKAction.group([fall, rotate, fade])) {
                label.removeFromParent()
            }

            addChild(label)
        }
    }

    private func spawnFireworks() {
        if fireworkCooldown > 0 {
            fireworkCooldown -= 1
            return
        }

        guard Int.random(in: 0...100) < 5 else { return }

        let x = CGFloat.random(in: 100...(size.width - 100))
        let y = CGFloat.random(in: size.height * 0.4...size.height * 0.8)
        let color = UIColor(
            hue: CGFloat.random(in: 0...1),
            saturation: 1.0,
            brightness: 1.0,
            alpha: 1.0
        )

        for _ in 0..<20 {
            let particle = SKShapeNode(circleOfRadius: CGFloat.random(in: 3...6))
            particle.fillColor = color
            particle.strokeColor = .clear
            particle.position = CGPoint(x: x, y: y)
            particle.alpha = 1.0

            let angle = CGFloat.random(in: 0...(2 * .pi))
            let speed = CGFloat.random(in: 50...150)
            let dx = cos(angle) * speed
            let dy = sin(angle) * speed

            let move = SKAction.moveBy(x: dx, y: dy - 100, duration: Double.random(in: 1...2))
            move.timingMode = .easeOut
            let fade = SKAction.fadeOut(withDuration: Double.random(in: 0.8...1.5))
            let scale = SKAction.scale(to: 0.2, duration: 1.5)

            particle.run(SKAction.group([move, fade, scale])) {
                particle.removeFromParent()
            }

            addChild(particle)
        }

        fireworkCooldown = 20
    }

    private func spawnSparkles() {
        guard frameCount % 2 == 0 else { return }

        let sparkleChars = ["✨", "⭐", "✦", "★", "☆", "✫"]

        for _ in 0..<3 {
            let label = SKLabelNode(text: sparkleChars.randomElement()!)
            label.fontSize = CGFloat.random(in: 16...28)
            label.fontColor = UIColor(white: 1.0, alpha: CGFloat.random(in: 0.7...1.0))
            label.position = CGPoint(
                x: CGFloat.random(in: 0...size.width),
                y: CGFloat.random(in: 0...size.height)
            )
            label.alpha = 0

            let fadeIn = SKAction.fadeIn(withDuration: 0.2)
            let wait = SKAction.wait(forDuration: Double.random(in: 0.3...0.8))
            let fadeOut = SKAction.fadeOut(withDuration: 0.3)
            let scale = SKAction.sequence([
                SKAction.scale(to: 1.3, duration: 0.2),
                SKAction.scale(to: 1.0, duration: 0.3)
            ])

            label.run(SKAction.sequence([
                SKAction.group([fadeIn, scale]),
                wait,
                fadeOut
            ])) {
                label.removeFromParent()
            }

            addChild(label)
        }
    }

    private func spawnRainbow() {
        guard frameCount % 2 == 0 else { return }

        let hue = CGFloat(frameCount % 360) / 360.0

        for _ in 0..<2 {
            let particle = SKShapeNode(rectOf: CGSize(width: CGFloat.random(in: 8...16), height: CGFloat.random(in: 15...25)))
            particle.fillColor = UIColor(hue: (hue + CGFloat.random(in: 0...0.2)).truncatingRemainder(dividingBy: 1.0), saturation: 1.0, brightness: 1.0, alpha: 1.0)
            particle.strokeColor = .clear
            particle.position = CGPoint(x: CGFloat.random(in: 0...size.width), y: size.height + 20)

            let fall = SKAction.moveBy(x: CGFloat.random(in: -20...20), y: -size.height - 50, duration: Double.random(in: 3...5))
            let fade = SKAction.sequence([
                SKAction.wait(forDuration: Double.random(in: 2...4)),
                SKAction.fadeOut(withDuration: 1.0)
            ])

            particle.run(SKAction.group([fall, fade])) {
                particle.removeFromParent()
            }

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
        onDismiss: {}
    )
}
