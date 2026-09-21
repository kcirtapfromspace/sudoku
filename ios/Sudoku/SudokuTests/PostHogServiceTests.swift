import XCTest
@testable import Sudoku

final class PostHogServiceTests: XCTestCase {
    private static func extractBody(_ request: URLRequest) -> [String: Any]? {
        let data: Data
        if let body = request.httpBody {
            data = body
        } else if let stream = request.httpBodyStream {
            stream.open()
            defer { stream.close() }
            var buffer = Data()
            var bytes = [UInt8](repeating: 0, count: 1024)
            while stream.hasBytesAvailable {
                let count = stream.read(&bytes, maxLength: bytes.count)
                if count <= 0 { break }
                buffer.append(bytes, count: count)
            }
            data = buffer
        } else {
            return nil
        }
        return try? JSONSerialization.jsonObject(with: data) as? [String: Any]
    }

    func testPostHogEventBatchingAndPayloadStructure() async throws {
        let defs = ServiceFixtures.defaults()
        var receivedRequests = 0
        var receivedBody: [String: Any]?

        let session = ServiceURLProtocol.session { request in
            receivedRequests += 1
            XCTAssertEqual(request.httpMethod, "POST")
            XCTAssertEqual(request.url?.path, "/batch")
            XCTAssertEqual(request.value(forHTTPHeaderField: "Content-Type"), "application/json")
            receivedBody = Self.extractBody(request)
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        // Capture an event
        await service.capture(event: "test_event", properties: ["custom_key": "custom_val"]).value

        XCTAssertEqual(receivedRequests, 1)
        let body = try XCTUnwrap(receivedBody)
        XCTAssertEqual(body["api_key"] as? String, "test_key")

        let batch = try XCTUnwrap(body["batch"] as? [[String: Any]])
        XCTAssertEqual(batch.count, 1)

        let event = batch[0]
        XCTAssertEqual(event["event"] as? String, "test_event")
        XCTAssertEqual(event["distinct_id"] as? String, service.distinctId)
        XCTAssertNotNil(event["timestamp"])

        let props = try XCTUnwrap(event["properties"] as? [String: Any])
        XCTAssertEqual(props["custom_key"] as? String, "custom_val")
        XCTAssertEqual(props["$os"] as? String, "iOS")
        XCTAssertEqual(props["$lib"] as? String, "sudoku-ios")
        XCTAssertNotNil(props["$os_version"])
        XCTAssertNotNil(props["$app_version"])
        XCTAssertNotNil(props["$device_model"])

        // Verify queue is empty after successful flush
        let remaining = await service.queuedEventCount()
        XCTAssertEqual(remaining, 0)
    }

    func testIdentifyAndScreenTracking() async throws {
        let defs = ServiceFixtures.defaults()
        var capturedEvents: [[String: Any]] = []

        let session = ServiceURLProtocol.session { request in
            if let body = Self.extractBody(request),
               let batch = body["batch"] as? [[String: Any]] {
                capturedEvents.append(contentsOf: batch)
            }
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        await service.identify(playerId: "custom_pid", playerTag: "SudokuMaster").value
        await service.screen(name: "game_board", properties: ["difficulty": "Hard"]).value

        XCTAssertEqual(capturedEvents.count, 2)

        let identifyEvent = capturedEvents[0]
        XCTAssertEqual(identifyEvent["event"] as? String, "$identify")
        let identifyProps = try XCTUnwrap(identifyEvent["properties"] as? [String: Any])
        let setProps = try XCTUnwrap(identifyProps["$set"] as? [String: Any])
        XCTAssertEqual(setProps["player_tag"] as? String, "SudokuMaster")

        let screenEvent = capturedEvents[1]
        XCTAssertEqual(screenEvent["event"] as? String, "$screen")
        let screenProps = try XCTUnwrap(screenEvent["properties"] as? [String: Any])
        XCTAssertEqual(screenProps["$screen_name"] as? String, "game_board")
        XCTAssertEqual(screenProps["difficulty"] as? String, "Hard")
    }

    @MainActor
    func testGameStartedAndCompletedEvents() async throws {
        let defs = ServiceFixtures.defaults()
        var capturedEvents: [[String: Any]] = []

        let session = ServiceURLProtocol.session { request in
            if let body = Self.extractBody(request),
               let batch = body["batch"] as? [[String: Any]] {
                capturedEvents.append(contentsOf: batch)
            }
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        // Game Started
        await service.captureGameStarted(
            puzzleHash: "hash123",
            difficulty: "Medium",
            seRating: 2.5,
            isShared: true,
            isImported: false
        ).value

        let game = GameViewModel(cachedGame: ServiceFixtures.game(), difficulty: .medium)
        await service.captureGameCompleted(game: game, won: true).value

        XCTAssertEqual(capturedEvents.count, 2)

        let started = capturedEvents[0]
        XCTAssertEqual(started["event"] as? String, "game_started")
        let startedProps = try XCTUnwrap(started["properties"] as? [String: Any])
        XCTAssertEqual(startedProps["puzzle_hash"] as? String, "hash123")
        XCTAssertEqual(startedProps["difficulty"] as? String, "Medium")
        XCTAssertEqual(startedProps["se_rating"] as? Float, 2.5)
        XCTAssertEqual(startedProps["is_shared"] as? Bool, true)

        let completed = capturedEvents[1]
        XCTAssertEqual(completed["event"] as? String, "game_completed")
        let completedProps = try XCTUnwrap(completed["properties"] as? [String: Any])
        XCTAssertEqual(completedProps["result"] as? String, "Win")
        XCTAssertEqual(completedProps["difficulty"] as? String, "Medium")
        XCTAssertNotNil(completedProps["puzzle_hash"])
        XCTAssertNotNil(completedProps["time_secs"])
    }

    func testPuzzleSharedAndThemeChanged() async throws {
        let defs = ServiceFixtures.defaults()
        var capturedEvents: [[String: Any]] = []

        let session = ServiceURLProtocol.session { request in
            if let body = Self.extractBody(request),
               let batch = body["batch"] as? [[String: Any]] {
                capturedEvents.append(contentsOf: batch)
            }
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        await service.capturePuzzleShared(shortCode: "ABCDEFGH", difficulty: "Expert").value
        await service.captureThemeChanged(theme: "Dark").value

        XCTAssertEqual(capturedEvents.count, 2)
        XCTAssertEqual(capturedEvents[0]["event"] as? String, "puzzle_shared")
        let shareProps = try XCTUnwrap(capturedEvents[0]["properties"] as? [String: Any])
        XCTAssertEqual(shareProps["short_code"] as? String, "ABCDEFGH")
        XCTAssertEqual(shareProps["difficulty"] as? String, "Expert")

        XCTAssertEqual(capturedEvents[1]["event"] as? String, "theme_changed")
        let themeProps = try XCTUnwrap(capturedEvents[1]["properties"] as? [String: Any])
        XCTAssertEqual(themeProps["theme"] as? String, "Dark")
    }

    func testRetryOnRateLimitAndServerFailure() async {
        let defs = ServiceFixtures.defaults()
        var callCount = 0

        let session = ServiceURLProtocol.session { request in
            callCount += 1
            if callCount == 1 {
                return try ServiceFixtures.response(request, status: 429, headers: ["Retry-After": "30"])
            }
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        // Enqueue event; first flush attempt returns 429
        await service.capture(event: "retry_event").value
        let countAfterFail = await service.queuedEventCount()
        XCTAssertEqual(countAfterFail, 1, "Event must remain in queue after 429")

        // Second flush attempt succeeds
        let success = await service.flushNow()
        XCTAssertTrue(success)
        let countAfterSuccess = await service.queuedEventCount()
        XCTAssertEqual(countAfterSuccess, 0, "Queue must be empty after successful retry")
    }

    func testDiscardOnClientError() async {
        let defs = ServiceFixtures.defaults()

        let session = ServiceURLProtocol.session { request in
            try ServiceFixtures.response(request, status: 400, body: ["error": "Invalid API Key"])
        }

        let service = PostHogService(
            apiKey: "bad_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        await service.capture(event: "bad_event").value
        let remaining = await service.queuedEventCount()
        XCTAssertEqual(remaining, 0, "400 Client error should drop batch to prevent infinite loop")
    }

    func testDisabledInTestingMode() async {
        let defs = ServiceFixtures.defaults()
        var networkAttempted = false

        let session = ServiceURLProtocol.session { request in
            networkAttempted = true
            return try ServiceFixtures.response(request, status: 200)
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: false
        )

        await service.capture(event: "ignore_me").value
        _ = await service.flushNow()

        XCTAssertFalse(networkAttempted)
        let count = await service.queuedEventCount()
        XCTAssertEqual(count, 0)
    }

    func testDistinctIdPersistenceAcrossInstances() {
        let defs = ServiceFixtures.defaults()
        let session = ServiceURLProtocol.session { try ServiceFixtures.response($0, status: 200) }

        let service1 = PostHogService(session: session, defaults: defs, isEnabled: false)
        let id1 = service1.distinctId

        let service2 = PostHogService(session: session, defaults: defs, isEnabled: false)
        let id2 = service2.distinctId

        XCTAssertEqual(id1, id2)
    }

    func testClearQueue() async {
        let defs = ServiceFixtures.defaults()
        let session = ServiceURLProtocol.session { _ in throw URLError(.timedOut) }

        let service = PostHogService(
            apiKey: "test_key",
            session: session,
            defaults: defs,
            isEnabled: true
        )

        await service.capture(event: "event_to_clear").value
        let beforeCount = await service.queuedEventCount()
        XCTAssertEqual(beforeCount, 1)

        await service.clearQueue()
        let afterCount = await service.queuedEventCount()
        XCTAssertEqual(afterCount, 0)
    }

    func testFlushTaskWrapperAndOptionalBranches() async throws {
        let defs = ServiceFixtures.defaults()
        var receivedEvents: [[String: Any]] = []

        let session = ServiceURLProtocol.session { request in
            if let body = Self.extractBody(request),
               let batch = body["batch"] as? [[String: Any]] {
                receivedEvents.append(contentsOf: batch)
            }
            return try ServiceFixtures.response(request, status: 200, body: ["status": "Ok"])
        }

        let service = PostHogService(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        // Identify without custom tag
        await service.identify().value
        // Game started without SE rating
        await service.captureGameStarted(puzzleHash: "hash_no_se", difficulty: "Easy").value
        // Puzzle shared without short code or difficulty
        await service.capturePuzzleShared(shortCode: nil, difficulty: nil).value
        // Background flush task
        let flushed = await service.flush().value
        XCTAssertTrue(flushed)

        XCTAssertEqual(receivedEvents.count, 3)
    }

    func testQueueCappingAtMaxLimit() async {
        let defs = ServiceFixtures.defaults()
        let session = ServiceURLProtocol.session { _ in throw URLError(.timedOut) }

        let queue = PostHogQueue(
            apiKey: "test_key",
            host: URL(string: "https://test.posthog.com")!,
            session: session,
            defaults: defs,
            isEnabled: true
        )

        // Enqueue 505 events
        for i in 0..<505 {
            await queue.enqueue(event: ["event": "capped_\(i)"])
        }

        let queued = await queue.getQueuedEvents()
        XCTAssertEqual(queued.count, 500, "Queue must be capped at 500 events")
        XCTAssertEqual(queued.first?["event"] as? String, "capped_5")
    }
}
