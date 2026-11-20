import Foundation

/// Loads playable boards from the same public catalog as the web app.
actor PuzzleAPIService {
    static let shared = PuzzleAPIService()
    static let eligibleDifficulties: Set<Difficulty> = [.hard, .expert, .master, .extreme]

    private struct CatalogPuzzle: Decodable {
        let puzzle_string: String
        let difficulty: String
    }

    private let session: URLSession
    private let defaults: UserDefaults

    init(session: URLSession? = nil, defaults: UserDefaults = .standard) {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForResource = 5
        config.waitsForConnectivity = false
        self.session = session ?? URLSession(configuration: config)
        self.defaults = defaults
    }

    func fetchPuzzle(difficulty: Difficulty) async -> SudokuGame? {
        let base = defaults.string(forKey: "api_base_url") ?? "https://ukodus.now"
        guard var url = URLComponents(string: base), ["http", "https"].contains(url.scheme),
              url.host?.isEmpty == false else { return nil }
        url.path = "/api/v1/puzzles/random"
        url.queryItems = [URLQueryItem(name: "difficulty", value: difficulty.rawValue)]
        guard let endpoint = url.url else { return nil }
        do {
            let (data, response) = try await session.data(from: endpoint)
            guard (response as? HTTPURLResponse)?.statusCode == 200,
                  let record = try? JSONDecoder().decode(CatalogPuzzle.self, from: data),
                  record.difficulty == difficulty.rawValue,
                  case .valid = validatePuzzleString(puzzle: record.puzzle_string),
                  let game = gameFromString(puzzle: record.puzzle_string),
                  Difficulty.from(game.getRatedDifficulty()) == difficulty else { return nil }
            return game
        } catch {
            return nil
        }
    }
}
