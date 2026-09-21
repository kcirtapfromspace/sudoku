import Foundation

// MARK: - Difficulty

enum Difficulty: String, CaseIterable, Identifiable, Codable {
    case beginner = "Beginner"
    case easy = "Easy"
    case medium = "Medium"
    case intermediate = "Intermediate"
    case hard = "Hard"
    case expert = "Expert"
    case master = "Master"
    case extreme = "Extreme"

    var id: String { rawValue }

    var displayName: String { rawValue }

    /// Difficulties that are always available
    static var alwaysUnlocked: [Difficulty] {
        [.beginner, .easy, .medium, .intermediate, .hard, .expert]
    }

    /// Check if this difficulty requires unlocking
    var requiresUnlock: Bool {
        self == .master || self == .extreme
    }

    /// Required wins to unlock this difficulty
    var unlockRequirement: (difficulty: Difficulty, wins: Int)? {
        switch self {
        case .master: return (.expert, 50)
        case .extreme: return (.master, 50)
        default: return nil
        }
    }

    /// SE rating range matching the Rust engine's Difficulty::se_range()
    var seRange: ClosedRange<Float> {
        switch self {
        case .beginner:     return 1.5...2.0
        case .easy:         return 2.0...2.5
        case .medium:       return 2.5...3.4
        case .intermediate: return 3.4...3.8
        case .hard:         return 3.8...4.5
        case .expert:       return 4.5...5.5
        case .master:       return 5.5...7.0
        case .extreme:      return 7.0...11.0
        }
    }

    /// Midpoint SE rating for quick-play
    var defaultSE: Float {
        let r = seRange
        return (r.lowerBound + r.upperBound) / 2.0
    }

    /// Short description of techniques at this difficulty
    var seDescription: String {
        switch self {
        case .beginner:     return "Naked singles"
        case .easy:         return "Hidden singles"
        case .medium:       return "Pairs & pointing"
        case .intermediate: return "Triples & box/line"
        case .hard:         return "Quads & intersections"
        case .expert:       return "Fish & uniqueness"
        case .master:       return "Wings & chains"
        case .extreme:      return "ALS, forcing chains & beyond"
        }
    }
}

// MARK: - Imported Puzzle Data

struct ImportedPuzzleData {
    let givensString: String                       // 81-char string with only given digits
    let playerMoves: [(index: Int, digit: Int)]    // Player-filled cells to replay
    let playerNotes: [(index: Int, notes: Set<Int>)] // Pencil marks to restore
    let isContinuing: Bool                         // true = "Continue Puzzle", false = "Start Fresh"
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
    let seRating: Float
    let involvedCells: [(row: Int, col: Int)]
}

// MARK: - Hint Detail Level

enum HintDetailLevel: Int {
    case none = -1
    case summary = 0
    case proofDetail = 1
}

// Note: HintCellRole is provided by the Rust engine (SudokuEngine.swift via UniFFI)

// Note: MoveResult is provided by the Rust engine (SudokuEngine.swift)
// Use the Rust-generated MoveResult type directly

// MARK: - Game Statistics

struct GameStatistics: Codable {
    var gamesPlayed: Int = 0
    var gamesWon: Int = 0
    var totalPlayTime: TimeInterval = 0
    var bestTimes: [Difficulty: TimeInterval] = [:]
    var winsPerDifficulty: [Difficulty: Int] = [:]  // Track wins per difficulty for unlocks
    var currentStreak: Int = 0
    var bestStreak: Int = 0
    var sequentialCompletions: Int = 0  // Count of rows/cols/boxes filled in order 1-9
    var easterEggUnlocked: Bool = false  // Secret unlock for testing

    var winRate: Double {
        guard gamesPlayed > 0 else { return 0 }
        return Double(gamesWon) / Double(gamesPlayed)
    }

    /// Get wins for a specific difficulty
    func wins(for difficulty: Difficulty) -> Int {
        winsPerDifficulty[difficulty] ?? 0
    }

    /// Check if a difficulty is unlocked
    func isUnlocked(_ difficulty: Difficulty) -> Bool {
        // Easter egg unlocks everything
        if easterEggUnlocked { return true }

        // A difficulty without an unlock requirement is available immediately.
        guard let requirement = difficulty.unlockRequirement else { return true }
        return wins(for: requirement.difficulty) >= requirement.wins
    }

    /// Get all currently available difficulties
    var availableDifficulties: [Difficulty] {
        Difficulty.allCases.filter { isUnlocked($0) }
    }

    /// Get progress toward unlocking a difficulty (0.0 to 1.0)
    func unlockProgress(for difficulty: Difficulty) -> Double {
        guard let requirement = difficulty.unlockRequirement else { return 1.0 }
        let currentWins = wins(for: requirement.difficulty)
        return min(1.0, Double(currentWins) / Double(requirement.wins))
    }

    mutating func recordWin(difficulty: Difficulty, time: TimeInterval) {
        gamesPlayed += 1
        gamesWon += 1
        totalPlayTime += time
        currentStreak += 1
        bestStreak = max(bestStreak, currentStreak)

        // Track wins per difficulty
        winsPerDifficulty[difficulty, default: 0] += 1

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

    mutating func recordSequentialCompletion() {
        sequentialCompletions += 1
    }

    mutating func activateEasterEgg() {
        easterEggUnlocked = true
    }
}

// MARK: - Input Mode

enum InputMode {
    case normal
    case candidate
    case temporaryCandidate  // Long-press mode: reverts after one note entry

    var displayName: String {
        switch self {
        case .normal: return "Normal"
        case .candidate: return "Notes"
        case .temporaryCandidate: return "Note (1x)"
        }
    }

    mutating func toggle() {
        self = self == .normal ? .candidate : .normal
    }

    var isNotesMode: Bool {
        self == .candidate || self == .temporaryCandidate
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
    var autoClearNotes: Bool = true  // Automatically remove notes from row/column/box when entering a number
    var celebrationsEnabled: Bool = true  // Show celebrations for completions
    var showErrorsImmediately: Bool = true  // Show wrong answers immediately vs check on submit
    var cameraImportEnabled: Bool = false

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
    case rowComplete(row: Int, isSequential: Bool = false)
    case columnComplete(col: Int, isSequential: Bool = false)
    case boxComplete(boxIndex: Int, isSequential: Bool = false)
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
