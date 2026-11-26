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

    static func next(rating: Float, defaults: UserDefaults = .standard) -> SudokuGame? {
        for difficulty in Difficulty.allCases {
            if let game = next(for: difficulty, rating: rating, defaults: defaults) { return game }
        }
        return nil
    }

    // PuzzleCache serializes draws. Saving the cursor avoids replaying the first
    // board every time the app opens without connectivity.
    static func next(for difficulty: Difficulty, rating: Float? = nil, defaults: UserDefaults = .standard) -> SudokuGame? {
        guard let tier = boards[difficulty.rawValue] else { return nil }
        let choices = rating.map { target in tier.filter { $0.rating == target } } ?? tier
        guard !choices.isEmpty else { return nil }
        let key = "starter_puzzle_cursor_" + difficulty.rawValue + (rating.map { "_\($0)" } ?? "")
        let index = defaults.object(forKey: key) == nil
            ? Int.random(in: choices.indices)
            : max(0, defaults.integer(forKey: key)) % choices.count
        let board = choices[index]
        defaults.set((index + 1) % choices.count, forKey: key)
        return gameFromPregenerated(puzzleString: board.puzzle, solutionString: board.solution,
                                    difficulty: difficulty.rawValue, seRating: board.rating)
    }
}
