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
    @Published private(set) var hintDetailLevel: HintDetailLevel = .none
    @Published private(set) var hintCellRoles: [HintCellRole] = Array(repeating: .none, count: 81)
    @Published private(set) var canUndo: Bool = false
    @Published private(set) var canRedo: Bool = false
    @Published private(set) var seRating: Float = 0.0
    @Published private(set) var lastCelebration: CelebrationEvent?

    /// Cells currently celebrating (for wiggle animation)
    @Published var celebratingCells: Set<String> = []

    /// Controls whether auto-calculated candidates are displayed
    /// When false, only user-entered candidates (via Notes mode) are shown
    @Published private(set) var showCandidates: Bool = false

    let difficulty: Difficulty
    @Published private(set) var maxMistakes = 3
    @Published private(set) var mistakeLimitEnabled = true

    // MARK: - Private Properties

    private var game: SudokuGame
    private let now: () -> Date
    private var startTime: Date?
    private var accumulatedTime: TimeInterval = 0

    private struct BoardSnapshot: Codable {
        let engine: String
        let candidates: [[Set<Int>]]
        let showCandidates: Bool
        let usingAutoFill: Bool
        var rowFillOrder: [[Int]] = Array(repeating: [], count: 9)
        var colFillOrder: [[Int]] = Array(repeating: [], count: 9)
        var boxFillOrder: [[Int]] = Array(repeating: [], count: 9)
        var completedRows: Set<Int> = []
        var completedCols: Set<Int> = []
        var completedBoxes: Set<Int> = []
    }
    private var undoHistory: [BoardSnapshot] = []
    private var redoHistory: [BoardSnapshot] = []

    // Track which rows/cols/boxes were already complete (to detect new completions)
    private var completedRows: Set<Int> = []
    private var completedCols: Set<Int> = []
    private var completedBoxes: Set<Int> = []

    // Track user-entered candidates separately from engine-calculated ones
    private var userCandidates: [[Set<Int>]] = Array(repeating: Array(repeating: [], count: 9), count: 9)
    private var usingAutoFill: Bool = false

    // Track fill order for sequential completion detection
    // For each row/col/box, track the order of values filled (excluding givens)
    private var rowFillOrder: [[Int]] = Array(repeating: [], count: 9)
    private var colFillOrder: [[Int]] = Array(repeating: [], count: 9)
    private var boxFillOrder: [[Int]] = Array(repeating: [], count: 9)

    // MARK: - Computed Properties

    var elapsedTime: TimeInterval {
        accumulatedTime + (startTime.map { max(0, now().timeIntervalSince($0)) } ?? 0)
    }

    var elapsedTimeString: String {
        let seconds = Int(elapsedTime)
        let mins = seconds / 60
        let secs = seconds % 60
        return String(format: "%02d:%02d", mins, secs)
    }

    var isGameOver: Bool {
        mistakeLimitEnabled && mistakes >= maxMistakes
    }

    var numberCounts: [Int] {
        let data = game.getNumberCounts()
        return data.map { Int($0) }
    }

    var completedNumbers: Set<Int> {
        Set(numberCounts.enumerated().compactMap { $0.element >= 9 ? $0.offset + 1 : nil })
    }

    /// Value of the currently selected cell (0 if no selection or cell is empty)
    var selectedValue: Int {
        guard let selected = selectedCell else { return 0 }
        return cells[selected.row][selected.col].value
    }

    // MARK: - Initialization

    convenience init(difficulty: Difficulty, now: @escaping () -> Date = Date.init) {
        self.init(cachedGame: SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty()), difficulty: difficulty, now: now)
    }

    /// Generate off the main thread, then publish the game on the main actor.
    static func createAsync(difficulty: Difficulty) async -> GameViewModel {
        let game = await Task.detached(priority: .userInitiated) {
            SudokuGame.newClassic(difficulty: difficulty.toGameDifficulty())
        }.value
        return GameViewModel(cachedGame: game, difficulty: difficulty)
    }

    init(cachedGame: SudokuGame, difficulty: Difficulty, now: @escaping () -> Date = Date.init) {
        self.game = cachedGame
        self.difficulty = difficulty
        self.now = now
        self.startTime = now()
        syncFromEngine()
    }

    func configureMistakeLimit(enabled: Bool, limit: Int) {
        mistakeLimitEnabled = enabled
        maxMistakes = min(10, max(1, limit))
        syncFromEngine()
    }

    // MARK: - Engine Sync

    /// Sync local state from the Rust engine
    private func syncFromEngine() {
        let cellStates = game.getAllCells()

        // Convert flat array to 2D grid
        var newCells = (0..<9).map { row in
            (0..<9).map { CellModel.empty(row: row, col: $0) }
        }
        for state in cellStates {
            let row = Int(state.row)
            let col = Int(state.col)

            // Use the correct candidate source based on whether we're using auto-fill
            let candidates: Set<Int>
            if usingAutoFill {
                // When using auto-fill, show engine-calculated candidates
                candidates = showCandidates ? Self.dataToSet(state.candidates) : []
            } else {
                // When not using auto-fill, show only user-entered candidates
                candidates = userCandidates[row][col]
            }

            let cell = CellModel(
                row: row,
                col: col,
                value: Int(state.value),
                isGiven: state.isGiven,
                candidates: candidates,
                hasConflict: state.hasConflict
            )

            newCells[row][col] = cell
        }

        cells = newCells
        mistakes = Int(game.getMistakes())
        hintsUsed = Int(game.getHintsUsed())
        isComplete = game.isComplete()
        seRating = game.getSeRating()
        canUndo = !undoHistory.isEmpty && !isComplete && !isGameOver
        canRedo = !redoHistory.isEmpty && !isComplete && !isGameOver
        if isComplete || isGameOver { pause() }
    }

    /// Convert Data (byte array) to Set<Int>
    private static func dataToSet(_ data: Data) -> Set<Int> {
        Set(data.map { Int($0) })
    }

    // MARK: - Cell Selection

    func selectCell(row: Int, col: Int) {
        guard (0..<9).contains(row), (0..<9).contains(col) else { return }
        selectedCell = (row, col)
        clearHint()
    }

    func clearSelection() {
        selectedCell = nil
    }

    // MARK: - Input

    /// Store mode before temporary candidate mode was activated
    private var modeBeforeTemporary: InputMode = .normal

    func enterNumber(_ number: Int) {
        guard !isComplete, !isGameOver, (0...255).contains(number), let selected = selectedCell else { return }
        let cell = cells[selected.row][selected.col]
        if cell.isGiven { return }

        if inputMode.isNotesMode {
            guard (1...9).contains(number) else { return }
            toggleCandidate(number, at: selected.row, col: selected.col)
            // Revert from temporary mode after entering one note
            if inputMode == .temporaryCandidate {
                inputMode = modeBeforeTemporary
            }
        } else {
            setValue(number, at: selected.row, col: selected.col)
        }
    }

    /// Enter temporary candidate mode (for long-press)
    func enterTemporaryNoteMode() {
        if inputMode != .temporaryCandidate {
            modeBeforeTemporary = inputMode
            inputMode = .temporaryCandidate
        }
    }

    private func setValue(_ value: Int, at row: Int, col: Int) {
        let previous = snapshot()
        let result = game.makeMove(row: UInt8(row), col: UInt8(col), value: UInt8(value))
        guard result != .cannotModifyGiven, result != .invalidValue else { return }
        undoHistory.append(previous)
        redoHistory.removeAll()
        userCandidates[row][col] = []
        recordFillOrder(value: value, row: row, col: col)
        syncFromEngine()
        if result == .complete {
            lastCelebration = .gameComplete
        } else if result == .success {
            checkForCompletions(afterPlacingAt: row, col: col, value: value)
        }
    }

    /// Record the fill order for sequential completion detection
    private func recordFillOrder(value: Int, row: Int, col: Int) {
        rowFillOrder[row].append(value)
        colFillOrder[col].append(value)
        boxFillOrder[(row / 3) * 3 + (col / 3)].append(value)
    }

    /// Check if a fill order represents sequential filling (1,2,3... or ...7,8,9)
    private func isSequentialFill(_ fillOrder: [Int]) -> Bool {
        guard fillOrder.count >= 2 else { return false }

        // Check ascending (1,2,3,...)
        var isAscending = true
        for i in 1..<fillOrder.count {
            if fillOrder[i] != fillOrder[i-1] + 1 {
                isAscending = false
                break
            }
        }

        // Check descending (...,3,2,1)
        var isDescending = true
        for i in 1..<fillOrder.count {
            if fillOrder[i] != fillOrder[i-1] - 1 {
                isDescending = false
                break
            }
        }

        return isAscending || isDescending
    }

    /// Check if placing a value completed any row, column, or box
    private func checkForCompletions(afterPlacingAt row: Int, col: Int, value: Int) {
        // Check row completion
        if !completedRows.contains(row) && isRowComplete(row) {
            completedRows.insert(row)
            let isSequential = isSequentialFill(rowFillOrder[row])
            lastCelebration = .rowComplete(row: row, isSequential: isSequential)
            return
        }

        // Check column completion
        if !completedCols.contains(col) && isColumnComplete(col) {
            completedCols.insert(col)
            let isSequential = isSequentialFill(colFillOrder[col])
            lastCelebration = .columnComplete(col: col, isSequential: isSequential)
            return
        }

        // Check box completion
        let boxIndex = (row / 3) * 3 + (col / 3)
        if !completedBoxes.contains(boxIndex) && isBoxComplete(boxIndex) {
            completedBoxes.insert(boxIndex)
            let isSequential = isSequentialFill(boxFillOrder[boxIndex])
            lastCelebration = .boxComplete(boxIndex: boxIndex, isSequential: isSequential)
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

    // MARK: - Celebration Helpers

    func triggerRowCelebration(_ row: Int) {
        for col in 0..<9 {
            celebratingCells.insert("\(row)-\(col)")
        }
        autoClearCelebration(after: 0.6)
    }

    func triggerColumnCelebration(_ col: Int) {
        for row in 0..<9 {
            celebratingCells.insert("\(row)-\(col)")
        }
        autoClearCelebration(after: 0.6)
    }

    func triggerBoxCelebration(_ boxIndex: Int) {
        let startRow = (boxIndex / 3) * 3
        let startCol = (boxIndex % 3) * 3
        for row in startRow..<startRow+3 {
            for col in startCol..<startCol+3 {
                celebratingCells.insert("\(row)-\(col)")
            }
        }
        autoClearCelebration(after: 0.6)
    }

    private func autoClearCelebration(after delay: TimeInterval) {
        DispatchQueue.main.asyncAfter(deadline: .now() + delay) { [weak self] in
            self?.celebratingCells.removeAll()
        }
    }

    func clearSelectedCell() {
        guard !isComplete, !isGameOver, let selected = selectedCell else { return }
        let cell = cells[selected.row][selected.col]
        if cell.isGiven { return }
        recordUndo()
        userCandidates[selected.row][selected.col] = []

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

        recordUndo()
        // Preserve the currently displayed notes when switching from automatic notes.
        if usingAutoFill {
            userCandidates = cells.map { $0.map(\.candidates) }
            usingAutoFill = false
        }
        showCandidates = true

        // Toggle in local user tracking
        if userCandidates[row][col].contains(value) {
            userCandidates[row][col].remove(value)
        } else {
            userCandidates[row][col].insert(value)
        }

        _ = game.toggleCandidate(row: UInt8(row), col: UInt8(col), value: UInt8(value))
        syncFromEngine()
    }

    // MARK: - Candidates

    func fillCandidatesForSelected() {
        guard !isComplete, !isGameOver, let selected = selectedCell,
              cells[selected.row][selected.col].isEmpty else { return }
        recordUndo()
        _ = game.fillCellCandidates(row: UInt8(selected.row), col: UInt8(selected.col))
        userCandidates[selected.row][selected.col] = getValidCandidates(row: selected.row, col: selected.col)
        showCandidates = true
        syncFromEngine()
    }

    func fillAllCandidates() {
        guard !isComplete, !isGameOver else { return }
        recordUndo()
        showCandidates = true
        usingAutoFill = true
        game.fillAllCandidates()
        syncFromEngine()
    }

    func clearCandidatesForSelected() {
        guard !isComplete, !isGameOver, let selected = selectedCell,
              cells[selected.row][selected.col].isEmpty else { return }
        recordUndo()
        userCandidates[selected.row][selected.col] = []
        _ = game.clearCellCandidates(row: UInt8(selected.row), col: UInt8(selected.col))
        syncFromEngine()
    }

    func clearAllCandidates() {
        guard !isComplete, !isGameOver else { return }
        recordUndo()
        showCandidates = false
        usingAutoFill = false
        // Clear user-entered candidates as well
        userCandidates = Array(repeating: Array(repeating: [], count: 9), count: 9)
        game.clearAllCandidates()
        syncFromEngine()
    }

    func getValidCandidates(row: Int, col: Int) -> Set<Int> {
        let data = game.getValidCandidates(row: UInt8(row), col: UInt8(col))
        return Self.dataToSet(data)
    }

    /// Remove invalid candidates (Check Notes feature)
    /// Keeps only candidates that match the solution
    func checkNotes() {
        guard !isComplete, !isGameOver else { return }
        recordUndo()
        game.removeInvalidCandidates()
        // Also update local user candidates to match
        for row in 0..<9 {
            for col in 0..<9 {
                let validCandidates = Set(game.getCandidates(row: UInt8(row), col: UInt8(col)).map { Int($0) })
                userCandidates[row][col] = userCandidates[row][col].intersection(validCandidates)
            }
        }
        syncFromEngine()
    }

    // MARK: - Imported Puzzle Support

    func applyImportedMove(row: Int, col: Int, value: Int) {
        guard (0..<9).contains(row), (0..<9).contains(col), (1...9).contains(value) else { return }
        _ = game.makeMove(row: UInt8(row), col: UInt8(col), value: UInt8(value))
        syncFromEngine()
    }

    func applyImportedNotes(row: Int, col: Int, notes: Set<Int>) {
        guard (0..<9).contains(row), (0..<9).contains(col), cells[row][col].isEmpty else { return }
        userCandidates[row][col] = notes.filter { (1...9).contains($0) }
        _ = game.clearCellCandidates(row: UInt8(row), col: UInt8(col))
        for note in userCandidates[row][col] {
            _ = game.toggleCandidate(row: UInt8(row), col: UInt8(col), value: UInt8(note))
        }
        showCandidates = true
        refreshCells()
    }

    /// Refresh cell display without a full engine sync (used for notes-only updates)
    private func refreshCells() {
        syncFromEngine()
    }

    // MARK: - Undo/Redo

    private func snapshot() -> BoardSnapshot {
        BoardSnapshot(engine: serializedBoard(), candidates: usingAutoFill ? cells.map { $0.map(\.candidates) } : userCandidates,
                      showCandidates: showCandidates, usingAutoFill: usingAutoFill,
                      rowFillOrder: rowFillOrder, colFillOrder: colFillOrder, boxFillOrder: boxFillOrder,
                      completedRows: completedRows, completedCols: completedCols, completedBoxes: completedBoxes)
    }

    private func recordUndo() {
        undoHistory.append(snapshot())
        redoHistory.removeAll()
    }

    /// Initial helper setup should not appear as a player move in Undo.
    func resetUndoHistory() {
        undoHistory.removeAll()
        redoHistory.removeAll()
        syncFromEngine()
    }

    private func restore(_ snapshot: BoardSnapshot) {
        guard let restored = Self.restoreEngine(snapshot.engine, mistakes: mistakes, hintsUsed: hintsUsed) else { return }
        game = restored
        userCandidates = snapshot.candidates
        usingAutoFill = snapshot.usingAutoFill
        showCandidates = snapshot.showCandidates
        rowFillOrder = snapshot.rowFillOrder
        colFillOrder = snapshot.colFillOrder
        boxFillOrder = snapshot.boxFillOrder
        completedRows = snapshot.completedRows
        completedCols = snapshot.completedCols
        completedBoxes = snapshot.completedBoxes
        game.clearAllCandidates()
        for row in 0..<9 {
            for col in 0..<9 {
                for candidate in userCandidates[row][col] {
                    _ = game.toggleCandidate(row: UInt8(row), col: UInt8(col), value: UInt8(candidate))
                }
            }
        }
        clearHint()
        syncFromEngine()
    }

    func undo() {
        guard !isComplete, !isGameOver, let previous = undoHistory.popLast() else { return }
        redoHistory.append(snapshot())
        restore(previous)
    }

    func redo() {
        guard !isComplete, !isGameOver, let next = redoHistory.popLast() else { return }
        undoHistory.append(snapshot())
        restore(next)
    }

    // MARK: - Hints

    func getHint() {
        guard !isComplete, !isGameOver else { return }
        if currentHint != nil && hintDetailLevel == .summary {
            // Second tap: upgrade to proof detail
            hintDetailLevel = .proofDetail
            updateHintCellRoles()
            return
        }

        // First tap: get a new hint
        guard let engineHint = game.getHint() else { return }

        currentHint = HintModel(
            row: Int(engineHint.row),
            col: Int(engineHint.col),
            value: engineHint.value.map { Int($0) },
            eliminate: engineHint.eliminate.map { Int($0) },
            explanation: engineHint.explanation,
            technique: engineHint.technique,
            seRating: engineHint.seRating,
            involvedCells: engineHint.involvedCells.map { (row: Int($0.row), col: Int($0.col)) }
        )
        selectedCell = (Int(engineHint.row), Int(engineHint.col))
        hintDetailLevel = .summary
        updateHintCellRoles()
        syncFromEngine()
    }

    func applyHint() {
        guard !isComplete, !isGameOver, currentHint != nil else { return }
        recordUndo()
        _ = game.applyHint()
        clearHint()
        syncFromEngine()
    }

    func clearHint() {
        currentHint = nil
        hintDetailLevel = .none
        updateHintCellRoles()
        game.clearHint()
    }

    func hintCellRole(row: Int, col: Int) -> HintCellRole {
        hintCellRoles[row * 9 + col]
    }

    private func updateHintCellRoles() {
        guard hintDetailLevel != .none else {
            hintCellRoles = Array(repeating: .none, count: 81)
            return
        }
        let rawRoles = game.getHintCellRoles(detailLevel: UInt8(hintDetailLevel.rawValue))
        hintCellRoles = rawRoles.map { Self.hintCellRole(from: $0) }
    }

    static func hintCellRole(from raw: UInt8) -> HintCellRole {
        switch raw {
        case 1: return .target
        case 2: return .involved
        case 3: return .chainOn
        case 4: return .chainOff
        case 5: return .fishBase
        case 6: return .fishCover
        case 7: return .fishFin
        case 8: return .urFloor
        case 9: return .urRoof
        case 10: return .alsGroup
        default: return .none
        }
    }

    // MARK: - Pause/Resume

    func pause() {
        guard let started = startTime else { return }
        accumulatedTime += max(0, now().timeIntervalSince(started))
        startTime = nil
    }

    func resume() {
        guard startTime == nil, !isComplete, !isGameOver else { return }
        startTime = now()
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

    // MARK: - Puzzle Fingerprint

    /// Get the puzzle string via FFI (givens as digits, empty as "0")
    /// Used for telemetry — same format the API expects
    func getPuzzleString() -> String {
        return game.getPuzzleString()
    }

    /// Get the puzzle fingerprint (81-char string with givens as digits, empty as ".")
    /// Used for identifying unique puzzles in the history
    func getPuzzleFingerprint() -> String {
        var result = ""
        for row in 0..<9 {
            for col in 0..<9 {
                let cell = cells[row][col]
                if cell.isGiven {
                    result += "\(cell.value)"
                } else {
                    result += "."
                }
            }
        }
        return result
    }

    /// Get the short code for this puzzle (8-char PuzzleId), if available
    func getShortCode() -> String? {
        return game.getShortCode()
    }

    /// Get the puzzle hash for history tracking
    var puzzleHash: String {
        PuzzleRecord.generateHash(from: getPuzzleFingerprint())
    }

    // MARK: - Serialization

    /// Store original givens separately: the FFI serializer otherwise promotes player moves to givens.
    private func serializedBoard() -> String {
        // The engine serializer guarantees a JSON object; silently saving an empty object would lose the puzzle.
        var dictionary = (try! JSONSerialization.jsonObject(with: Data(game.serialize().utf8))) as! [String: Any]
        dictionary["playerValues"] = cells.flatMap { $0.map(\.value) }
        dictionary["puzzle"] = getPuzzleFingerprint()
        let data = try! JSONSerialization.data(withJSONObject: dictionary, options: .sortedKeys)
        return String(decoding: data, as: UTF8.self)
    }

    private static func restoreEngine(_ json: String, mistakes: Int? = nil, hintsUsed: Int? = nil) -> SudokuGame? {
        guard var dictionary = (try? JSONSerialization.jsonObject(with: Data(json.utf8))) as? [String: Any] else { return nil }
        let values = dictionary["playerValues"] as? [Int]
        if let values {
            guard values.count == 81, values.allSatisfy({ (0...9).contains($0) }),
                  let puzzle = dictionary["puzzle"] as? String, puzzle.count == 81,
                  let solution = dictionary["solution"] as? String, solution.count == 81 else { return nil }
            let givens = Array(puzzle)
            let answers = Array(solution)
            let replayedMistakes = values.indices.filter {
                values[$0] > 0 && (givens[$0] == "." || givens[$0] == "0") && String(values[$0]) != String(answers[$0])
            }.count
            let savedMistakes = mistakes ?? dictionary["swiftMistakes"] as? Int ?? dictionary["mistakes"] as? Int ?? 0
            dictionary["mistakes"] = max(0, savedMistakes - replayedMistakes)
        }
        if let hintsUsed { dictionary["hints_used"] = hintsUsed }
        guard let data = try? JSONSerialization.data(withJSONObject: dictionary),
              let engine = gameDeserialize(json: String(decoding: data, as: UTF8.self)) else { return nil }
        if let values {
            let givenCells = engine.getAllCells()
            for (index, value) in values.enumerated() where value > 0 && !givenCells[index].isGiven {
                _ = engine.makeMove(row: UInt8(index / 9), col: UInt8(index % 9), value: UInt8(value))
            }
        }
        return engine
    }

    func serialize() -> String {
        var dictionary = (try! JSONSerialization.jsonObject(with: Data(serializedBoard().utf8))) as! [String: Any]
        dictionary["elapsedTime"] = elapsedTime
        dictionary["swiftDifficulty"] = difficulty.rawValue
        dictionary["swiftMistakes"] = mistakes
        dictionary["swiftHintsUsed"] = hintsUsed
        dictionary["notes"] = snapshot().candidates.map { $0.map { $0.sorted() } }
        dictionary["showCandidates"] = showCandidates
        dictionary["usingAutoFill"] = usingAutoFill
        let data = try! JSONSerialization.data(withJSONObject: dictionary, options: .sortedKeys)
        return String(decoding: data, as: UTF8.self)
    }

    static func deserialize(_ json: String, now: @escaping () -> Date = Date.init) -> GameViewModel? {
        guard let dictionary = (try? JSONSerialization.jsonObject(with: Data(json.utf8))) as? [String: Any],
              let engine = restoreEngine(json) else { return nil }
        let difficulty = Difficulty(rawValue: dictionary["swiftDifficulty"] as? String ?? "Medium") ?? .medium
        let viewModel = GameViewModel(cachedGame: engine, difficulty: difficulty, now: now)
        viewModel.accumulatedTime = max(0, dictionary["elapsedTime"] as? TimeInterval ?? 0)
        viewModel.startTime = nil
        if let notes = dictionary["notes"] as? [[[Int]]] {
            guard notes.count == 9, notes.allSatisfy({ $0.count == 9 }),
                  notes.flatMap({ $0 }).flatMap({ $0 }).allSatisfy({ (1...9).contains($0) }) else { return nil }
            viewModel.userCandidates = notes.map { $0.map(Set.init) }
        }
        viewModel.showCandidates = dictionary["showCandidates"] as? Bool ?? false
        viewModel.usingAutoFill = dictionary["usingAutoFill"] as? Bool ?? false
        viewModel.restore(BoardSnapshot(engine: json, candidates: viewModel.userCandidates,
                                       showCandidates: viewModel.showCandidates, usingAutoFill: viewModel.usingAutoFill))
        return viewModel
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
        case .master: return .master
        case .extreme: return .extreme
        }
    }

    static func from(_ gameDifficulty: GameDifficulty) -> Difficulty {
        switch gameDifficulty {
        case .beginner: return .beginner
        case .easy: return .easy
        case .medium: return .medium
        case .intermediate: return .intermediate
        case .hard: return .hard
        case .expert: return .expert
        case .master: return .master
        case .extreme: return .extreme
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
