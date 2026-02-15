import Foundation
#if canImport(UIKit)
import UIKit
#endif

/// Manages auth tokens for the telemetry API.
private actor TokenManager {
    private let tokenEndpoint = URL(string: "https://ukodus.now/api/v1/token")!
    private let session: URLSession

    private var cachedToken: String?
    private var expiresAt: Date = .distantPast

    init(session: URLSession) {
        self.session = session
    }

    /// Get a valid token, fetching a new one if the cached one is expired.
    /// Returns nil during migration period (server doesn't support tokens yet).
    func getToken(playerId: String) async -> String? {
        // Check if cached token is still valid (with 60s buffer)
        if let token = cachedToken, Date() < expiresAt.addingTimeInterval(-60) {
            return token
        }

        // Fetch new token
        return await fetchToken(playerId: playerId)
    }

    /// Clear the cached token (called on 401 response).
    func clearToken() {
        cachedToken = nil
        expiresAt = .distantPast
    }

    private func fetchToken(playerId: String) async -> String? {
        let body: [String: Any] = ["player_id": playerId]
        guard let jsonData = try? JSONSerialization.data(withJSONObject: body) else { return nil }

        var request = URLRequest(url: tokenEndpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = jsonData

        do {
            let (data, response) = try await session.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse,
                  httpResponse.statusCode == 200 else { return nil }

            guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let token = json["token"] as? String,
                  let expiresAtUnix = json["expires_at"] as? TimeInterval else { return nil }

            self.cachedToken = token
            self.expiresAt = Date(timeIntervalSince1970: expiresAtUnix)
            return token
        } catch {
            #if DEBUG
            print("Token fetch error: \(error.localizedDescription)")
            #endif
            return nil
        }
    }
}

/// Fire-and-forget telemetry that submits game results to the ukodus API.
/// Results populate the Galaxy visualization and leaderboards alongside web games.
final class TelemetryService: Sendable {
    static let shared = TelemetryService()

    private let endpoint = URL(string: "https://ukodus.now/api/v1/results")!
    private let session: URLSession
    private let tokenManager: TokenManager

    private init() {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 10
        config.waitsForConnectivity = false
        self.session = URLSession(configuration: config)
        self.tokenManager = TokenManager(session: self.session)
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

        Task.detached(priority: .utility) { [endpoint, session, tokenManager] in
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

            // Add auth token if available (nil during migration)
            if let token = await tokenManager.getToken(playerId: pid) {
                request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            }

            do {
                let (_, response) = try await session.data(for: request)
                let status = (response as? HTTPURLResponse)?.statusCode ?? 0

                switch status {
                case 200...299:
                    #if DEBUG
                    print("Telemetry: \(status) for \(puzzleHash)")
                    #endif
                case 401:
                    // Token expired — clear cache for next submission
                    await tokenManager.clearToken()
                    #if DEBUG
                    print("Telemetry: 401 — token expired, cleared cache")
                    #endif
                case 429:
                    #if DEBUG
                    let retryAfter = (response as? HTTPURLResponse)?.value(forHTTPHeaderField: "Retry-After") ?? "?"
                    print("Telemetry: 429 — rate limited, retry after \(retryAfter)s")
                    #endif
                default:
                    #if DEBUG
                    print("Telemetry: unexpected status \(status)")
                    #endif
                }
            } catch {
                #if DEBUG
                print("Telemetry error: \(error.localizedDescription)")
                #endif
            }
        }
    }

    // MARK: - Puzzle Hash (SHA-256)

    /// SHA-256 hash via FFI bridge (matches TUI and web canonical form).
    func hashPuzzle(_ puzzleString: String) -> String {
        return canonicalPuzzleHash(puzzleString: puzzleString)
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
