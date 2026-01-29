import Foundation

// MARK: - Difficulty

enum Difficulty: String, CaseIterable, Identifiable, Codable {
    case beginner = "Beginner"
    case easy = "Easy"
    case medium = "Medium"
    case intermediate = "Intermediate"
    case hard = "Hard"
    case expert = "Expert"

    var id: String { rawValue }

    var displayName: String { rawValue }
}

// MARK: - Cell Model

struct CellModel: Identifiable, Equatable {
    let row: Int
    let col: Int
    var value: Int // 0 if empty
    var isGiven: Bool
    var candidates: Set<Int>
    var hasConflict: Bool

    var id: String { "\(row)-\(col)" }
    var isEmpty: Bool { value == 0 }
    var position: (row: Int, col: Int) { (row, col) }

    // Box index (0-8)
    var boxIndex: Int {
        (row / 3) * 3 + (col / 3)
    }

    static func empty(row: Int, col: Int) -> CellModel {
        CellModel(row: row, col: col, value: 0, isGiven: false, candidates: [], hasConflict: false)
    }
}

// MARK: - Hint Model

struct HintModel {
    let row: Int
    let col: Int
    let value: Int? // Value to set, if applicable
    let eliminate: [Int] // Candidates to eliminate
    let explanation: String
    let technique: String
}

// Note: MoveResult is provided by the Rust engine (SudokuEngine.swift)
// Use the Rust-generated MoveResult type directly

// MARK: - Game Statistics

struct GameStatistics: Codable {
    var gamesPlayed: Int = 0
    var gamesWon: Int = 0
    var totalPlayTime: TimeInterval = 0
    var bestTimes: [Difficulty: TimeInterval] = [:]
    var currentStreak: Int = 0
    var bestStreak: Int = 0

    var winRate: Double {
        guard gamesPlayed > 0 else { return 0 }
        return Double(gamesWon) / Double(gamesPlayed)
    }

    mutating func recordWin(difficulty: Difficulty, time: TimeInterval) {
        gamesPlayed += 1
        gamesWon += 1
        totalPlayTime += time
        currentStreak += 1
        bestStreak = max(bestStreak, currentStreak)

        if let best = bestTimes[difficulty] {
            bestTimes[difficulty] = min(best, time)
        } else {
            bestTimes[difficulty] = time
        }
    }

    mutating func recordLoss(time: TimeInterval) {
        gamesPlayed += 1
        totalPlayTime += time
        currentStreak = 0
    }
}

// MARK: - Input Mode

enum InputMode {
    case normal
    case candidate

    var displayName: String {
        switch self {
        case .normal: return "Normal"
        case .candidate: return "Notes"
        }
    }

    mutating func toggle() {
        self = self == .normal ? .candidate : .normal
    }
}

// MARK: - Game State

enum GameState {
    case playing
    case paused
    case won
    case lost
    case menu
    case loading
}

// MARK: - Settings

struct GameSettings: Codable {
    var theme: ThemeSetting = .system
    var hapticsEnabled: Bool = true
    var timerVisible: Bool = true
    var mistakeLimitEnabled: Bool = true
    var mistakeLimit: Int = 3
    var ghostHintsEnabled: Bool = false
    var highlightValidCells: Bool = false
    var highlightRelatedCells: Bool = true
    var highlightSameNumbers: Bool = true
    var autoFillCandidates: Bool = false  // Start games with notes pre-filled
    var celebrationsEnabled: Bool = true  // Show celebrations for completions

    enum ThemeSetting: String, Codable, CaseIterable {
        case system = "System"
        case light = "Light"
        case dark = "Dark"
        case highContrast = "High Contrast"
    }
}

// MARK: - Celebration Events

enum CelebrationEvent: Equatable {
    case cellComplete(row: Int, col: Int)
    case rowComplete(row: Int)
    case columnComplete(col: Int)
    case boxComplete(boxIndex: Int)
    case gameComplete
}

// MARK: - Konami Code

enum KonamiInput: Equatable {
    case up, down, left, right, a, b
}

class KonamiCodeDetector: ObservableObject {
    static let sequence: [KonamiInput] = [.up, .up, .down, .down, .left, .right, .left, .right, .b, .a]

    @Published var isActivated = false
    @Published var progress: Int = 0

    private var inputBuffer: [KonamiInput] = []

    func input(_ input: KonamiInput) {
        inputBuffer.append(input)

        // Keep buffer at sequence length
        if inputBuffer.count > Self.sequence.count {
            inputBuffer.removeFirst()
        }

        // Check for match
        if inputBuffer == Self.sequence {
            isActivated = true
            inputBuffer.removeAll()
            progress = 0
        } else {
            // Update progress (how many consecutive correct inputs from start)
            progress = 0
            for (index, expected) in Self.sequence.enumerated() {
                if index < inputBuffer.count && inputBuffer[index] == expected {
                    progress = index + 1
                } else {
                    break
                }
            }
        }
    }

    func reset() {
        inputBuffer.removeAll()
        isActivated = false
        progress = 0
    }
}
