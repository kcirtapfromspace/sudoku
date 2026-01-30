import Foundation
import Combine
import SwiftUI

/// ViewModel wrapping the Rust Sudoku engine via UniFFI
@MainActor
class GameViewModel: ObservableObject {
    // MARK: - Published Properties

    @Published private(set) var cells: [[CellModel]] = []
    @Published private(set) var selectedCell: (row: Int, col: Int)?
    @Published var inputMode: InputMode = .normal
    @Published private(set) var mistakes: Int = 0
    @Published private(set) var hintsUsed: Int = 0
    @Published private(set) var isComplete: Bool = false
    @Published private(set) var currentHint: HintModel?
    @Published private(set) var canUndo: Bool = false
    @Published private(set) var canRedo: Bool = false
    @Published private(set) var lastCelebration: CelebrationEvent?

    /// Controls whether auto-calculated candidates are displayed
    /// When false, only user-entered candidates (via Notes mode) are shown
    @Published private(set) var showCandidates: Bool = false

    let difficulty: Difficulty
    let maxMistakes = 3

    // MARK: - Private Properties

    private var game: SudokuGame
    private var startTime: Date
    private var pausedTime: TimeInterval = 0
    private var lastPauseStart: Date?

    // Track which rows/cols/boxes were already complete (to detect new completions)
    private var completedRows: Set<Int> = []
    private var completedCols: Set<Int> = []
    private var completedBoxes: Set<Int> = []

    // MARK: - Computed Properties

    var elapsedTime: TimeInterval {
        if let pauseStart = lastPauseStart {
            return pausedTime + pauseStart.timeIntervalSinceNow * -1
        }
        return pausedTime + Date().timeIntervalSince(startTime)
    }

    var elapsedTimeString: String {
        let seconds = Int(elapsedTime)
        let mins = seconds / 60
        let secs = seconds % 60
        return String(format: "%02d:%02d", mins, secs)
    }

    var isGameOver: Bool {
        mistakes >= maxMistakes
    }

    var numberCounts: [Int] {
        let data = game.getNumberCounts()
        return data.map { Int($0) }
    }

    var completedNumbers: Set<Int> {
        Set(numberCounts.enumerated().compactMap { $0.element >= 9 ? $0.offset + 1 : nil })
    }

    // MARK: - Initialization

    init(difficulty: Difficulty) {
        self.difficulty = difficulty
        self.startTime = Date()
        self.game = SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        syncFromEngine()
    }

    /// Create a game asynchronously (puzzle generation happens off main thread)
    static func createAsync(difficulty: Difficulty) async -> GameViewModel {
        // Generate puzzle on background thread
        let game = await Task.detached(priority: .userInitiated) {
            SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        }.value

        // Create view model on main thread
        return await MainActor.run {
            GameViewModel(cachedGame: game, difficulty: difficulty)
        }
    }

    /// Init with pre-created game from cache
    init(cachedGame: SudokuGame, difficulty: Difficulty) {
        self.game = cachedGame
        self.difficulty = difficulty
        self.startTime = Date()
        self.pausedTime = 0
        syncFromEngine()
    }

    /// Internal init for deserialization with elapsed time
    private init(deserializedGame game: SudokuGame, difficulty: Difficulty, elapsedTime: TimeInterval) {
        self.game = game
        self.difficulty = difficulty
        self.pausedTime = elapsedTime
        self.startTime = Date()
        syncFromEngine()
    }

    // MARK: - Engine Sync

    /// Sync local state from the Rust engine
    private func syncFromEngine() {
        let cellStates = game.getAllCells()

        // Convert flat array to 2D grid
        var newCells: [[CellModel]] = Array(repeating: [], count: 9)
        for state in cellStates {
            let row = Int(state.row)
            let col = Int(state.col)
            // Only include candidates if showCandidates is enabled
            let candidates = showCandidates ? Self.dataToSet(state.candidates) : []

            let cell = CellModel(
                row: row,
                col: col,
                value: Int(state.value),
                isGiven: state.isGiven,
                candidates: candidates,
                hasConflict: state.hasConflict
            )

            if newCells[row].count <= col {
                newCells[row].append(cell)
            } else {
                newCells[row][col] = cell
            }
        }

        // Ensure all rows have 9 columns
        for row in 0..<9 {
            while newCells[row].count < 9 {
                newCells[row].append(CellModel.empty(row: row, col: newCells[row].count))
            }
        }

        cells = newCells
        mistakes = Int(game.getMistakes())
        hintsUsed = Int(game.getHintsUsed())
        isComplete = game.isComplete()
        canUndo = game.canUndo()
        canRedo = game.canRedo()
    }

    /// Convert Data (byte array) to Set<Int>
    private static func dataToSet(_ data: Data) -> Set<Int> {
        Set(data.map { Int($0) })
    }

    // MARK: - Cell Selection

    func selectCell(row: Int, col: Int) {
        selectedCell = (row, col)
        currentHint = nil
    }

    func clearSelection() {
        selectedCell = nil
    }

    // MARK: - Input

    func enterNumber(_ number: Int) {
        guard let selected = selectedCell else { return }
        let cell = cells[selected.row][selected.col]

        if cell.isGiven { return }

        if inputMode == .candidate {
            toggleCandidate(number, at: selected.row, col: selected.col)
        } else {
            setValue(number, at: selected.row, col: selected.col)
        }
    }

    private func setValue(_ value: Int, at row: Int, col: Int) {
        let result = game.makeMove(row: UInt8(row), col: UInt8(col), value: UInt8(value))

        switch result {
        case .success:
            syncFromEngine()
            checkForCompletions(afterPlacingAt: row, col: col)
        case .complete:
            syncFromEngine()
            lastCelebration = .gameComplete
        case .conflict:
            syncFromEngine()
        case .cannotModifyGiven, .invalidValue:
            break
        }
    }

    /// Check if placing a value completed any row, column, or box
    private func checkForCompletions(afterPlacingAt row: Int, col: Int) {
        // Check row completion
        if !completedRows.contains(row) && isRowComplete(row) {
            completedRows.insert(row)
            lastCelebration = .rowComplete(row: row)
            return
        }

        // Check column completion
        if !completedCols.contains(col) && isColumnComplete(col) {
            completedCols.insert(col)
            lastCelebration = .columnComplete(col: col)
            return
        }

        // Check box completion
        let boxIndex = (row / 3) * 3 + (col / 3)
        if !completedBoxes.contains(boxIndex) && isBoxComplete(boxIndex) {
            completedBoxes.insert(boxIndex)
            lastCelebration = .boxComplete(boxIndex: boxIndex)
            return
        }
    }

    private func isRowComplete(_ row: Int) -> Bool {
        for col in 0..<9 {
            if cells[row][col].value == 0 || cells[row][col].hasConflict {
                return false
            }
        }
        return true
    }

    private func isColumnComplete(_ col: Int) -> Bool {
        for row in 0..<9 {
            if cells[row][col].value == 0 || cells[row][col].hasConflict {
                return false
            }
        }
        return true
    }

    private func isBoxComplete(_ boxIndex: Int) -> Bool {
        let startRow = (boxIndex / 3) * 3
        let startCol = (boxIndex % 3) * 3
        for row in startRow..<startRow+3 {
            for col in startCol..<startCol+3 {
                if cells[row][col].value == 0 || cells[row][col].hasConflict {
                    return false
                }
            }
        }
        return true
    }

    func clearCelebration() {
        lastCelebration = nil
    }

    func clearSelectedCell() {
        guard let selected = selectedCell else { return }
        let cell = cells[selected.row][selected.col]

        if cell.isGiven { return }

        if cell.value != 0 {
            _ = game.clearCell(row: UInt8(selected.row), col: UInt8(selected.col))
            syncFromEngine()
        } else {
            _ = game.clearCellCandidates(row: UInt8(selected.row), col: UInt8(selected.col))
            syncFromEngine()
        }
    }

    private func toggleCandidate(_ value: Int, at row: Int, col: Int) {
        let cell = cells[row][col]
        if cell.isGiven || cell.value != 0 { return }

        // Enable candidate display when user manually enters candidates
        showCandidates = true

        _ = game.toggleCandidate(row: UInt8(row), col: UInt8(col), value: UInt8(value))
        syncFromEngine()
    }

    // MARK: - Candidates

    func fillCandidatesForSelected() {
        guard let selected = selectedCell else { return }
        _ = game.fillCellCandidates(row: UInt8(selected.row), col: UInt8(selected.col))
        syncFromEngine()
    }

    func fillAllCandidates() {
        showCandidates = true
        game.fillAllCandidates()
        syncFromEngine()
    }

    func clearCandidatesForSelected() {
        guard let selected = selectedCell else { return }
        _ = game.clearCellCandidates(row: UInt8(selected.row), col: UInt8(selected.col))
        syncFromEngine()
    }

    func clearAllCandidates() {
        showCandidates = false
        game.clearAllCandidates()
        syncFromEngine()
    }

    func getValidCandidates(row: Int, col: Int) -> Set<Int> {
        let data = game.getValidCandidates(row: UInt8(row), col: UInt8(col))
        return Self.dataToSet(data)
    }

    // MARK: - Undo/Redo

    func undo() {
        _ = game.undo()
        syncFromEngine()
    }

    func redo() {
        _ = game.redo()
        syncFromEngine()
    }

    // MARK: - Hints

    func getHint() {
        guard let engineHint = game.getHint() else { return }

        currentHint = HintModel(
            row: Int(engineHint.row),
            col: Int(engineHint.col),
            value: engineHint.value.map { Int($0) },
            eliminate: engineHint.eliminate.map { Int($0) },
            explanation: engineHint.explanation,
            technique: engineHint.technique
        )
        selectedCell = (Int(engineHint.row), Int(engineHint.col))
        syncFromEngine()
    }

    func applyHint() {
        guard currentHint != nil else { return }
        _ = game.applyHint()
        currentHint = nil
        syncFromEngine()
    }

    // MARK: - Pause/Resume

    func pause() {
        lastPauseStart = Date()
    }

    func resume() {
        if let pauseStart = lastPauseStart {
            pausedTime += Date().timeIntervalSince(pauseStart)
            lastPauseStart = nil
        }
    }

    // MARK: - Highlighting

    func isRelated(to position: (row: Int, col: Int)?) -> Bool {
        guard let pos = position, let selected = selectedCell else { return false }
        return pos.row == selected.row ||
               pos.col == selected.col ||
               (pos.row / 3 == selected.row / 3 && pos.col / 3 == selected.col / 3)
    }

    func hasSameValue(as position: (row: Int, col: Int)?) -> Bool {
        guard let pos = position, let selected = selectedCell else { return false }
        let selectedValue = cells[selected.row][selected.col].value
        return selectedValue > 0 && cells[pos.row][pos.col].value == selectedValue
    }

    func isNakedSingle(row: Int, col: Int) -> Bool {
        return game.isNakedSingle(row: UInt8(row), col: UInt8(col))
    }

    // MARK: - Serialization

    func serialize() -> String {
        // Include elapsed time in the JSON since it's not tracked by the Rust engine
        let engineJson = game.serialize()

        // Parse and add elapsed time
        guard let data = engineJson.data(using: .utf8),
              var dict = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return engineJson
        }

        dict["elapsedTime"] = elapsedTime
        dict["swiftDifficulty"] = difficulty.rawValue

        if let newData = try? JSONSerialization.data(withJSONObject: dict),
           let newJson = String(data: newData, encoding: .utf8) {
            return newJson
        }

        return engineJson
    }

    static func deserialize(_ json: String) -> GameViewModel? {
        // Extract elapsed time before passing to engine
        guard let data = json.data(using: .utf8),
              let dict = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return nil
        }

        let elapsedTime = dict["elapsedTime"] as? TimeInterval ?? 0
        let difficultyStr = dict["swiftDifficulty"] as? String ?? "Medium"
        let difficulty = Difficulty(rawValue: difficultyStr) ?? .medium

        guard let game = gameDeserialize(json: json) else {
            return nil
        }

        return GameViewModel(deserializedGame: game, difficulty: difficulty, elapsedTime: elapsedTime)
    }
}

// MARK: - Difficulty Conversion

extension Difficulty {
    func toGameDifficulty() -> GameDifficulty {
        switch self {
        case .beginner: return .beginner
        case .easy: return .easy
        case .medium: return .medium
        case .intermediate: return .intermediate
        case .hard: return .hard
        case .expert: return .expert
        }
    }

    static func from(_ gameDifficulty: GameDifficulty) -> Difficulty {
        switch gameDifficulty {
        case .beginner: return .beginner
        case .easy: return .easy
        case .medium: return .medium
        case .intermediate: return .intermediate
        case .hard: return .hard
        case .expert, .master, .extreme: return .expert
        }
    }
}

// MARK: - Test Helpers (DEBUG only)

#if DEBUG
extension GameViewModel {
    /// Fill all cells in a row except the specified column (for testing row completion celebration)
    func fillRowExcept(row: Int, exceptCol: Int) {
        for col in 0..<9 {
            if col == exceptCol { continue }
            let cell = cells[row][col]
            if !cell.isGiven && cell.value == 0 {
                let solution = game.getSolutionValue(row: UInt8(row), col: UInt8(col))
                _ = game.makeMove(row: UInt8(row), col: UInt8(col), value: solution)
            }
        }
        syncFromEngine()
    }

    /// Fill all cells in a column except the specified row (for testing column completion celebration)
    func fillColumnExcept(col: Int, exceptRow: Int) {
        for row in 0..<9 {
            if row == exceptRow { continue }
            let cell = cells[row][col]
            if !cell.isGiven && cell.value == 0 {
                let solution = game.getSolutionValue(row: UInt8(row), col: UInt8(col))
                _ = game.makeMove(row: UInt8(row), col: UInt8(col), value: solution)
            }
        }
        syncFromEngine()
    }

    /// Fill all cells in a box except the specified position (for testing box completion celebration)
    func fillBoxExcept(boxIndex: Int, exceptRow: Int, exceptCol: Int) {
        let startRow = (boxIndex / 3) * 3
        let startCol = (boxIndex % 3) * 3
        for row in startRow..<startRow+3 {
            for col in startCol..<startCol+3 {
                if row == exceptRow && col == exceptCol { continue }
                let cell = cells[row][col]
                if !cell.isGiven && cell.value == 0 {
                    let solution = game.getSolutionValue(row: UInt8(row), col: UInt8(col))
                    _ = game.makeMove(row: UInt8(row), col: UInt8(col), value: solution)
                }
            }
        }
        syncFromEngine()
    }

    /// Fill all cells except the last few (for testing win celebration)
    func fillAllExcept(count: Int) {
        var emptyCells: [(row: Int, col: Int)] = []

        // Collect all empty cells
        for row in 0..<9 {
            for col in 0..<9 {
                let cell = cells[row][col]
                if !cell.isGiven && cell.value == 0 {
                    emptyCells.append((row, col))
                }
            }
        }

        // Shuffle and keep only 'count' cells empty
        emptyCells.shuffle()
        let cellsToKeepEmpty = Set(emptyCells.prefix(count).map { "\($0.row)-\($0.col)" })

        // Fill all except the ones we want to keep empty
        for row in 0..<9 {
            for col in 0..<9 {
                let key = "\(row)-\(col)"
                if cellsToKeepEmpty.contains(key) { continue }
                let cell = cells[row][col]
                if !cell.isGiven && cell.value == 0 {
                    let solution = game.getSolutionValue(row: UInt8(row), col: UInt8(col))
                    _ = game.makeMove(row: UInt8(row), col: UInt8(col), value: solution)
                }
            }
        }
        syncFromEngine()
    }

    /// Get the solution value for a cell (exposed for testing)
    func getSolution(row: Int, col: Int) -> Int {
        return Int(game.getSolutionValue(row: UInt8(row), col: UInt8(col)))
    }

    /// Find first empty cell in a row
    func findEmptyCellInRow(_ row: Int) -> Int? {
        for col in 0..<9 {
            if cells[row][col].value == 0 && !cells[row][col].isGiven {
                return col
            }
        }
        return nil
    }

    /// Find first empty cell in a column
    func findEmptyCellInColumn(_ col: Int) -> Int? {
        for row in 0..<9 {
            if cells[row][col].value == 0 && !cells[row][col].isGiven {
                return row
            }
        }
        return nil
    }

    /// Find first empty cell in a box
    func findEmptyCellInBox(_ boxIndex: Int) -> (row: Int, col: Int)? {
        let startRow = (boxIndex / 3) * 3
        let startCol = (boxIndex % 3) * 3
        for row in startRow..<startRow+3 {
            for col in startCol..<startCol+3 {
                if cells[row][col].value == 0 && !cells[row][col].isGiven {
                    return (row, col)
                }
            }
        }
        return nil
    }
}
#endif
