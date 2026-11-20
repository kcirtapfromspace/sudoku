import Foundation

/// Small offline reserve, generated and rated with the same engine as the app.
enum StarterPuzzles {
    struct Board: Decodable {
        let puzzle: String
        let solution: String
        let rating: Float
    }

    static let boards: [String: [Board]] = {
        guard let url = Bundle.main.url(forResource: "StarterPuzzles", withExtension: "json"),
              let data = try? Data(contentsOf: url),
              let tiers = try? JSONDecoder().decode([String: [Board]].self, from: data) else { return [:] }
        return tiers
    }()

    // PuzzleCache serializes draws. Saving the cursor avoids replaying the first
    // board every time the app opens without connectivity.
    static func next(for difficulty: Difficulty, defaults: UserDefaults = .standard) -> SudokuGame? {
        guard let choices = boards[difficulty.rawValue], !choices.isEmpty else { return nil }
        let key = "starter_puzzle_cursor_" + difficulty.rawValue
        let index = defaults.object(forKey: key) == nil
            ? Int.random(in: choices.indices)
            : max(0, defaults.integer(forKey: key)) % choices.count
        let board = choices[index]
        defaults.set((index + 1) % choices.count, forKey: key)
        return gameFromPregenerated(puzzleString: board.puzzle, solutionString: board.solution,
                                    difficulty: difficulty.rawValue, seRating: board.rating)
    }
}
