import Foundation

/// Fetches pre-mined puzzles from the ukodus API for instant loading.
/// Used for Hard+ difficulties where local generation is expensive.
actor PuzzleAPIService {
    static let shared = PuzzleAPIService()

    private let session: URLSession
    private let defaults: UserDefaults
    private let bundleAPIKey: String?

    /// Base URL for the API. Override via UserDefaults key "api_base_url" for testing.
    private var endpointURL: URL? {
        let base = defaults.string(forKey: "api_base_url")
            ?? "https://ukodus.now"
        return URL(string: "\(base)/api/v1/internal/puzzles/undiscovered")
    }

    /// API key for the mining endpoints. Set via MINING_API_KEY in the build environment
    /// or override at runtime via UserDefaults key "mining_api_key".
    private var apiKey: String? {
        defaults.string(forKey: "mining_api_key")
            ?? bundleAPIKey
    }

    init(session: URLSession? = nil, defaults: UserDefaults = .standard,
         bundleAPIKey: String? = Bundle.main.infoDictionary?["MINING_API_KEY"] as? String) {
        self.defaults = defaults
        self.bundleAPIKey = bundleAPIKey
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 8
        config.waitsForConnectivity = false
        self.session = session ?? URLSession(configuration: config)
    }

    /// Difficulties eligible for API-fetched puzzles
    static let eligibleDifficulties: Set<Difficulty> = [.hard, .expert, .master, .extreme]

    /// Fetch a pre-mined puzzle for the given difficulty.
    /// Returns nil on any failure (network, auth, no puzzles available).
    func fetchPuzzle(difficulty: Difficulty) async -> SudokuGame? {
        guard let key = apiKey, !key.isEmpty else {
            #if DEBUG
            print("PuzzleAPI: no API key configured")
            #endif
            return nil
        }

        guard let endpointURL,
              var components = URLComponents(url: endpointURL, resolvingAgainstBaseURL: false),
              ["https", "http"].contains(components.scheme), components.host != nil else { return nil }
        components.queryItems = [URLQueryItem(name: "difficulty", value: difficulty.rawValue)]

        guard let url = components.url else { return nil }

        var request = URLRequest(url: url)
        request.setValue(key, forHTTPHeaderField: "X-Api-Key")

        do {
            let (data, response) = try await session.data(for: request)
            guard let http = response as? HTTPURLResponse, http.statusCode == 200 else {
                #if DEBUG
                let status = (response as? HTTPURLResponse)?.statusCode ?? 0
                print("PuzzleAPI: \(status) for \(difficulty.rawValue)")
                #endif
                return nil
            }

            guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let puzzleString = json["puzzle_string"] as? String,
                  let solutionString = json["solution_string"] as? String,
                  let difficultyStr = json["difficulty"] as? String,
                  let seRating = (json["se_rating"] as? NSNumber)?.floatValue else {
                #if DEBUG
                print("PuzzleAPI: invalid response JSON")
                #endif
                return nil
            }

            let game = gameFromPregenerated(
                puzzleString: puzzleString,
                solutionString: solutionString,
                difficulty: difficultyStr,
                seRating: seRating
            )

            #if DEBUG
            if game != nil {
                print("PuzzleAPI: fetched \(difficultyStr) puzzle (SE \(seRating))")
            }
            #endif

            return game
        } catch {
            #if DEBUG
            print("PuzzleAPI: \(error.localizedDescription)")
            #endif
            return nil
        }
    }
}
