import SwiftUI
import SpriteKit
import UIKit

// MARK: - Lose Messages

private let loseMessages = [
    "GAME OVER",
    "BETTER LUCK NEXT TIME",
    "DON'T GIVE UP!",
    "TRY AGAIN",
    "SO CLOSE!",
    "KEEP PRACTICING"
]

// MARK: - Lose Screen View

struct LoseScreenView: View {
    let time: TimeInterval
    let difficulty: Difficulty
    let mistakes: Int
    let onDismiss: () -> Void
    let onRetry: () -> Void

    @State private var message: String = loseMessages.randomElement()!
    @State private var showContent = false
    @State private var particleScene: LoseParticleScene?

    var body: some View {
        ZStack {
            // Dark gradient background
            LinearGradient(
                colors: [
                    Color(red: 0.1, green: 0.05, blue: 0.15),
                    Color(red: 0.15, green: 0.08, blue: 0.1),
                    Color.black
                ],
                startPoint: .top,
                endPoint: .bottom
            )
            .ignoresSafeArea()

            // Rain particle layer
            if let scene = particleScene {
                SpriteView(scene: scene, options: [.allowsTransparency])
                    .ignoresSafeArea()
            }

            // Content
            if showContent {
                VStack(spacing: 30) {
                    Spacer()

                    // Broken heart icon
                    Text("💔")
                        .font(.system(size: 70))
                        .opacity(0.8)

                    // Lose message
                    Text(message)
                        .font(.system(size: 32, weight: .bold, design: .rounded))
                        .foregroundStyle(
                            LinearGradient(
                                colors: [.gray, .white.opacity(0.6)],
                                startPoint: .top,
                                endPoint: .bottom
                            )
                        )
                        .multilineTextAlignment(.center)

                    // Stats
                    VStack(spacing: 12) {
                        HStack(spacing: 20) {
                            LoseStatItem(icon: "clock", value: formatTime(time))
                            LoseStatItem(icon: "xmark.circle.fill", value: "\(mistakes) mistakes")
                        }
                        Text(difficulty.displayName)
                            .font(.subheadline)
                            .foregroundStyle(.gray)
                    }
                    .padding(.top, 10)

                    Spacer()

                    // Buttons
                    VStack(spacing: 16) {
                        Button {
                            onRetry()
                        } label: {
                            Label("Try Again", systemImage: "arrow.clockwise")
                                .font(.headline)
                                .frame(maxWidth: .infinity)
                                .padding()
                                .background(
                                    RoundedRectangle(cornerRadius: 12)
                                        .fill(Color.blue.opacity(0.8))
                                )
                                .foregroundStyle(.white)
                        }

                        Button {
                            onDismiss()
                        } label: {
                            Text("Main Menu")
                                .font(.subheadline)
                                .foregroundStyle(.gray)
                        }
                    }
                    .padding(.horizontal, 40)
                    .padding(.bottom, 40)
                }
                .transition(.opacity.combined(with: .scale(scale: 0.9)))
            }
        }
        .onAppear {
            // Create particle scene
            let scene = LoseParticleScene()
            scene.scaleMode = .resizeFill
            scene.backgroundColor = .clear
            particleScene = scene

            // Show content with animation
            withAnimation(.easeOut(duration: 0.5).delay(0.3)) {
                showContent = true
            }
        }
    }

    private func formatTime(_ interval: TimeInterval) -> String {
        let mins = Int(interval) / 60
        let secs = Int(interval) % 60
        return String(format: "%d:%02d", mins, secs)
    }
}

// MARK: - Lose Stat Item

private struct LoseStatItem: View {
    let icon: String
    let value: String

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .foregroundStyle(.red.opacity(0.7))
            Text(value)
                .foregroundStyle(.white.opacity(0.8))
        }
        .font(.subheadline)
    }
}

// MARK: - SpriteKit Rain/Debris Scene

class LoseParticleScene: SKScene {
    private var frameCount: Int = 0

    private let rainChars = ["│", "╎", "┊", "┆", "|", "."]
    private let debrisChars = ["×", "✕", "✖", "▼", "▾", "◾"]

    override func didMove(to view: SKView) {
        backgroundColor = .clear
    }

    override func update(_ currentTime: TimeInterval) {
        frameCount += 1
        spawnRain()

        // Clean up old particles
        children.filter { $0.position.y < -20 }.forEach { $0.removeFromParent() }
    }

    private func spawnRain() {
        guard frameCount % 2 == 0 else { return }

        for _ in 0..<3 {
            let useRain = Int.random(in: 0...10) < 7
            let chars = useRain ? rainChars : debrisChars

            let label = SKLabelNode(text: chars.randomElement()!)
            label.fontSize = CGFloat.random(in: 10...16)

            let gray = CGFloat.random(in: 0.25...0.5)
            let redTint = CGFloat.random(in: 0...0.15)
            label.fontColor = UIColor(red: gray + redTint, green: gray, blue: gray, alpha: 0.8)

            label.position = CGPoint(
                x: CGFloat.random(in: 0...size.width),
                y: size.height + 10
            )

            let fall = SKAction.moveBy(
                x: CGFloat.random(in: -10...10),
                y: -size.height - 30,
                duration: Double.random(in: 2...4)
            )

            label.run(fall) {
                label.removeFromParent()
            }

            addChild(label)
        }
    }
}

// MARK: - Preview

#Preview {
    LoseScreenView(
        time: 245,
        difficulty: .hard,
        mistakes: 3,
        onDismiss: {},
        onRetry: {}
    )
}
