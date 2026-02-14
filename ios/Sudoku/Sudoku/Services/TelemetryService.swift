import Foundation
#if canImport(UIKit)
import UIKit
#endif

/// Fire-and-forget telemetry that submits game results to the ukodus API.
/// Results populate the Galaxy visualization and leaderboards alongside web games.
final class TelemetryService: Sendable {
    static let shared = TelemetryService()

    private let endpoint = URL(string: "https://ukodus.now/api/v1/results")!
    private let session: URLSession

    private init() {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 10
        config.waitsForConnectivity = false
        self.session = URLSession(configuration: config)
    }

    // MARK: - Player ID

    private static let playerIdKey = "ukodus_player_id"

    var playerId: String {
        if let existing = UserDefaults.standard.string(forKey: Self.playerIdKey) {
            return existing
        }
        let id = UUID().uuidString
        UserDefaults.standard.set(id, forKey: Self.playerIdKey)
        return id
    }

    // MARK: - Submit Result

    /// Collect data on the main thread, then fire a detached network task.
    @MainActor
    func submitResult(game: GameViewModel, won: Bool) {
        let puzzleString = game.getPuzzleString()
        let puzzleHash = hashPuzzle(puzzleString)
        let shortCode = game.getShortCode()
        let difficulty = game.difficulty.rawValue
        let seRating = game.seRating
        let timeSecs = Int(game.elapsedTime)
        let hintsUsed = game.hintsUsed
        let mistakes = game.mistakes
        let pid = playerId

        let deviceModel = Self.deviceModel()
        let osVersion = Self.osVersion()
        let appVersion = Self.appVersion()

        Task.detached(priority: .utility) { [endpoint, session] in
            var body: [String: Any] = [
                "puzzle_hash": puzzleHash,
                "puzzle_string": puzzleString,
                "difficulty": difficulty,
                "se_rating": seRating,
                "result": won ? "Win" : "Loss",
                "time_secs": timeSecs,
                "hints_used": hintsUsed,
                "mistakes": mistakes,
                "player_id": pid,
                "platform": "ios",
                "device_model": deviceModel,
                "os_version": osVersion,
                "app_version": appVersion,
            ]
            if let code = shortCode, !code.isEmpty {
                body["short_code"] = code
            }

            guard let jsonData = try? JSONSerialization.data(withJSONObject: body) else { return }

            var request = URLRequest(url: endpoint)
            request.httpMethod = "POST"
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = jsonData

            do {
                let (_, response) = try await session.data(for: request)
                let status = (response as? HTTPURLResponse)?.statusCode ?? 0
                #if DEBUG
                print("Telemetry: \(status) for \(puzzleHash)")
                #endif
            } catch {
                #if DEBUG
                print("Telemetry error: \(error.localizedDescription)")
                #endif
            }
        }
    }

    // MARK: - DJB2 Hash (web-compatible)

    /// Matches the JS `hashPuzzle()` function: DJB2 with Int32 wrapping arithmetic,
    /// output as 8-char zero-padded hex.
    func hashPuzzle(_ puzzleString: String) -> String {
        var hash: Int32 = 0
        for ch in puzzleString.utf8 {
            hash = (hash &<< 5) &- hash &+ Int32(ch)
        }
        return String(format: "%08x", UInt32(bitPattern: hash))
    }

    // MARK: - Device Info

    private static func deviceModel() -> String {
        var systemInfo = utsname()
        uname(&systemInfo)
        return withUnsafePointer(to: &systemInfo.machine) {
            $0.withMemoryRebound(to: CChar.self, capacity: 1) {
                String(validatingUTF8: $0) ?? "unknown"
            }
        }
    }

    private static func osVersion() -> String {
        #if canImport(UIKit)
        return "iOS \(UIDevice.current.systemVersion)"
        #else
        return "unknown"
        #endif
    }

    private static func appVersion() -> String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "unknown"
    }
}
