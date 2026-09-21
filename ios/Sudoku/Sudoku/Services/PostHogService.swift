import Foundation
#if canImport(UIKit)
import UIKit
#endif

/// Actor managing the event queue and network dispatch for PostHog metrics.
actor PostHogQueue {
    private let apiKey: String
    private let endpoint: URL
    private let session: URLSession
    private let defaults: UserDefaults
    private let queueKey: String
    private let isEnabled: Bool

    private var inFlight: Bool = false

    init(apiKey: String,
         host: URL,
         session: URLSession,
         defaults: UserDefaults,
         queueKey: String = "posthog_event_queue",
         isEnabled: Bool = true) {
        self.apiKey = apiKey
        // PostHog event ingestion endpoint
        self.endpoint = host.appendingPathComponent("batch")
        self.session = session
        self.defaults = defaults
        self.queueKey = queueKey
        self.isEnabled = isEnabled
    }

    /// Enqueue an event dictionary and automatically trigger a flush if needed.
    func enqueue(event: [String: Any]) {
        guard isEnabled, !apiKey.isEmpty else { return }
        var queue = loadQueue()
        queue.append(event)
        // Cap max queue length to avoid unbounded growth
        if queue.count > 500 {
            queue.removeFirst(queue.count - 500)
        }
        saveQueue(queue)
    }

    /// Read currently queued events.
    func getQueuedEvents() -> [[String: Any]] {
        loadQueue()
    }

    /// Clear all queued events (used in tests or resets).
    func clearQueue() {
        defaults.removeObject(forKey: queueKey)
    }

    /// Flush queued events to PostHog.
    @discardableResult
    func flush() async -> Bool {
        guard isEnabled, !apiKey.isEmpty, !inFlight else { return false }
        let queue = loadQueue()
        guard !queue.isEmpty else { return true }

        inFlight = true
        defer { inFlight = false }

        // Take up to 50 events in a batch
        let batchSize = min(queue.count, 50)
        let batchEvents = Array(queue.prefix(batchSize))

        let body: [String: Any] = [
            "api_key": apiKey,
            "batch": batchEvents
        ]

        guard let jsonData = try? JSONSerialization.data(withJSONObject: body) else {
            // Drop un-serializable payload
            let remaining = Array(queue.dropFirst(batchSize))
            saveQueue(remaining)
            return false
        }

        var request = URLRequest(url: endpoint)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = jsonData

        do {
            let (_, response) = try await session.data(for: request)
            let status = (response as? HTTPURLResponse)?.statusCode ?? 0

            switch status {
            case 200...299:
                // Success: remove sent batch from queue
                let currentQueue = loadQueue()
                let remaining = Array(currentQueue.dropFirst(min(currentQueue.count, batchSize)))
                saveQueue(remaining)
                #if DEBUG
                print("PostHog: Successfully sent batch of \(batchEvents.count) events")
                #endif
                return true
            case 400..<500 where status != 429:
                // Client error (e.g. invalid key or bad format): drop batch to prevent poison pill loop
                let currentQueue = loadQueue()
                let remaining = Array(currentQueue.dropFirst(min(currentQueue.count, batchSize)))
                saveQueue(remaining)
                #if DEBUG
                print("PostHog: Discarding batch due to HTTP \(status)")
                #endif
                return false
            default:
                // Rate limit (429) or server error (5xx) or other: keep in queue for retry
                #if DEBUG
                print("PostHog: Ingestion deferred with status \(status)")
                #endif
                return false
            }
        } catch {
            #if DEBUG
            print("PostHog network error: \(error.localizedDescription)")
            #endif
            return false
        }
    }

    private func loadQueue() -> [[String: Any]] {
        guard let data = defaults.data(forKey: queueKey),
              let items = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else {
            return []
        }
        return items
    }

    private func saveQueue(_ queue: [[String: Any]]) {
        if let data = try? JSONSerialization.data(withJSONObject: queue) {
            defaults.set(data, forKey: queueKey)
        }
    }
}

/// Service providing PostHog analytics and metrics collection for the Sudoku iOS app.
final class PostHogService: @unchecked Sendable {
    static let shared = PostHogService()

    static let defaultApiKey = "phc_fpR3Ys9gIknGVFxS0Wu0M7GIu0BSUZ4tyXAeCzq0ZOq"
    static let defaultHost = URL(string: "https://us.i.posthog.com")!
    private static let playerIdKey = "ukodus_player_id"

    let apiKey: String
    let host: URL
    let isEnabled: Bool
    private let defaults: UserDefaults
    private let queue: PostHogQueue
    private let isoFormatter: ISO8601DateFormatter

    init(apiKey: String? = nil,
         host: URL? = nil,
         session: URLSession? = nil,
         defaults: UserDefaults = .standard,
         isEnabled: Bool? = nil) {
        let env = ProcessInfo.processInfo.environment
        let resolvedKey = apiKey
            ?? env["POSTHOG_API_KEY"]
            ?? (Bundle.main.infoDictionary?["PostHogApiKey"] as? String)
            ?? Self.defaultApiKey

        let resolvedHostString = host?.absoluteString
            ?? env["POSTHOG_HOST"]
            ?? (Bundle.main.infoDictionary?["PostHogHost"] as? String)
            ?? Self.defaultHost.absoluteString

        let resolvedHost = URL(string: resolvedHostString) ?? Self.defaultHost

        let isTesting = ProcessInfo.processInfo.arguments.contains("--ui-testing") || env["SUDOKU_TESTING"] == "1"
        let resolvedEnabled = isEnabled ?? !isTesting

        self.apiKey = resolvedKey
        self.host = resolvedHost
        self.defaults = defaults
        self.isEnabled = resolvedEnabled

        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 10
        config.waitsForConnectivity = false
        let resolvedSession = session ?? URLSession(configuration: config)

        self.queue = PostHogQueue(apiKey: resolvedKey,
                                  host: resolvedHost,
                                  session: resolvedSession,
                                  defaults: defaults,
                                  isEnabled: resolvedEnabled)

        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        self.isoFormatter = formatter
    }

    // MARK: - Player Distinct ID

    var distinctId: String {
        if let existing = defaults.string(forKey: Self.playerIdKey) {
            return existing
        }
        let id = UUID().uuidString
        defaults.set(id, forKey: Self.playerIdKey)
        return id
    }

    // MARK: - Event Capture

    /// Capture an arbitrary event with properties and device metadata.
    @discardableResult
    func capture(event: String, properties: [String: Any] = [:]) -> Task<Void, Never> {
        let pid = distinctId
        var fullProps = defaultContext()
        for (k, v) in properties {
            fullProps[k] = v
        }

        let eventDict: [String: Any] = [
            "event": event,
            "distinct_id": pid,
            "properties": fullProps,
            "timestamp": isoFormatter.string(from: Date())
        ]

        return Task.detached(priority: .utility) { [queue] in
            await queue.enqueue(event: eventDict)
            await queue.flush()
        }
    }

    /// Identify the current player in PostHog.
    @discardableResult
    func identify(playerId: String? = nil, playerTag: String? = nil) -> Task<Void, Never> {
        let id = playerId ?? distinctId
        var setProps: [String: Any] = ["platform": "ios"]
        if let tag = playerTag, !tag.isEmpty {
            setProps["player_tag"] = tag
        }

        var properties: [String: Any] = [
            "$set": setProps
        ]
        if let tag = playerTag, !tag.isEmpty {
            properties["player_tag"] = tag
        }

        return capture(event: "$identify", properties: properties)
    }

    /// Track screen navigation.
    @discardableResult
    func screen(name: String, properties: [String: Any] = [:]) -> Task<Void, Never> {
        var props = properties
        props["$screen_name"] = name
        return capture(event: "$screen", properties: props)
    }

    /// Track game start.
    @discardableResult
    func captureGameStarted(puzzleHash: String,
                            difficulty: String,
                            seRating: Float? = nil,
                            isShared: Bool = false,
                            isImported: Bool = false) -> Task<Void, Never> {
        var props: [String: Any] = [
            "puzzle_hash": puzzleHash,
            "difficulty": difficulty,
            "is_shared": isShared,
            "is_imported": isImported
        ]
        if let se = seRating {
            props["se_rating"] = se
        }
        return capture(event: "game_started", properties: props)
    }

    /// Track game completion (win or loss).
    @MainActor
    @discardableResult
    func captureGameCompleted(game: GameViewModel, won: Bool) -> Task<Void, Never> {
        let puzzleString = game.getPuzzleString()
        let puzzleHash = canonicalPuzzleHash(puzzleString: puzzleString)
        let shortCode = game.getShortCode()
        let difficulty = game.difficulty.rawValue
        let seRating = game.seRating
        let timeSecs = Int(game.elapsedTime)
        let hintsUsed = game.hintsUsed
        let mistakes = game.mistakes

        var props: [String: Any] = [
            "result": won ? "Win" : "Loss",
            "difficulty": difficulty,
            "puzzle_hash": puzzleHash,
            "se_rating": seRating,
            "time_secs": timeSecs,
            "hints_used": hintsUsed,
            "mistakes": mistakes
        ]
        if let code = shortCode, !code.isEmpty {
            props["short_code"] = code
        }

        return capture(event: "game_completed", properties: props)
    }

    /// Track puzzle sharing (via link or QR code).
    @discardableResult
    func capturePuzzleShared(shortCode: String?, difficulty: String? = nil) -> Task<Void, Never> {
        var props: [String: Any] = [:]
        if let code = shortCode, !code.isEmpty {
            props["short_code"] = code
        }
        if let diff = difficulty {
            props["difficulty"] = diff
        }
        return capture(event: "puzzle_shared", properties: props)
    }

    /// Track theme changes.
    @discardableResult
    func captureThemeChanged(theme: String) -> Task<Void, Never> {
        return capture(event: "theme_changed", properties: ["theme": theme])
    }

    /// Manually trigger a flush of pending events (e.g., when app moves to background).
    @discardableResult
    func flush() -> Task<Bool, Never> {
        return Task.detached(priority: .utility) { [queue] in
            await queue.flush()
        }
    }

    /// Direct async flush for tests.
    func flushNow() async -> Bool {
        await queue.flush()
    }

    /// Direct async queue count check for tests.
    func queuedEventCount() async -> Int {
        await queue.getQueuedEvents().count
    }

    /// Clear queue for tests.
    func clearQueue() async {
        await queue.clearQueue()
    }

    // MARK: - Device Context

    private func defaultContext() -> [String: Any] {
        return [
            "$os": "iOS",
            "$os_version": Self.osVersion(),
            "$app_version": Self.appVersion(),
            "$app_build": Self.appBuild(),
            "$device_model": Self.deviceModel(),
            "$lib": "sudoku-ios",
            "$lib_version": Self.appVersion(),
            "platform": "ios"
        ]
    }

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

    private static func appBuild() -> String {
        Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "unknown"
    }
}
