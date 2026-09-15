import XCTest
@testable import Sudoku

final class PuzzleAPIAndCacheTests: XCTestCase {
    func testAPIRequiresKeyAndValidURL() async {
        let defaults = ServiceFixtures.defaults()
        let session = ServiceURLProtocol.session { _ in XCTFail("Request should not be sent"); throw URLError(.badURL) }
        let api = PuzzleAPIService(session: session, defaults: defaults, bundleAPIKey: nil)
        let noKey = await api.fetchPuzzle(difficulty: .hard); XCTAssertNil(noKey)
        defaults.set("", forKey: "mining_api_key")
        let emptyKey = await api.fetchPuzzle(difficulty: .hard); XCTAssertNil(emptyKey)
        defaults.set("test", forKey: "mining_api_key")
        defaults.set("http://[", forKey: "api_base_url")
        let invalidURL = await api.fetchPuzzle(difficulty: .hard); XCTAssertNil(invalidURL)
        defaults.set("file:///tmp", forKey: "api_base_url")
        let invalidScheme = await api.fetchPuzzle(difficulty: .hard); XCTAssertNil(invalidScheme)
    }
    func testAPIDecodesRealPuzzleAndSendsAuthenticationAndDifficulty() async {
        let session = ServiceURLProtocol.session { request in
            XCTAssertEqual(request.value(forHTTPHeaderField: "X-Api-Key"), "test-key")
            XCTAssertEqual(request.url?.path, "/api/v1/internal/puzzles/undiscovered")
            XCTAssertEqual(URLComponents(url: request.url!, resolvingAgainstBaseURL: false)?.queryItems?.first?.value, "Hard")
            return try ServiceFixtures.response(request, body: ["puzzle_string": ServiceFixtures.puzzle, "solution_string": ServiceFixtures.solution, "difficulty": "Hard", "se_rating": 4.2])
        }
        let api = PuzzleAPIService(session: session, defaults: ServiceFixtures.defaults(), bundleAPIKey: "test-key")
        let game = await api.fetchPuzzle(difficulty: .hard)
        XCTAssertNotNil(game)
        XCTAssertEqual(game?.getAllCells()[0].value, 5)
        XCTAssertEqual(PuzzleAPIService.eligibleDifficulties, [.hard, .expert, .master, .extreme])
    }
    func testAPIFailuresReturnNil() async {
        let handlers: [(URLRequest) throws -> (Data, URLResponse)] = [
            { try ServiceFixtures.response($0, status: 401) },
            { (Data(), URLResponse(url: $0.url!, mimeType: nil, expectedContentLength: 0, textEncodingName: nil)) },
            { (Data("invalid JSON".utf8), HTTPURLResponse(url: $0.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!) },
            { try ServiceFixtures.response($0, body: ["puzzle_string": "bad"]) },
            { try ServiceFixtures.response($0, body: ["puzzle_string": "bad", "solution_string": "bad", "difficulty": "Hard", "se_rating": 4.2]) },
            { _ in throw URLError(.timedOut) }
        ]
        for handler in handlers {
            let api = PuzzleAPIService(session: ServiceURLProtocol.session(handler), defaults: ServiceFixtures.defaults(), bundleAPIKey: "key")
            let game = await api.fetchPuzzle(difficulty: .expert); XCTAssertNil(game)
        }
    }
    func testCacheWarmupConsumptionAndFallback() async {
        let cache = PuzzleCache(fetch: { _ in ServiceFixtures.game() }, generate: { _ in ServiceFixtures.game() })
        let initial = await cache.getCacheStatus(); XCTAssertTrue(initial.values.allSatisfy { !$0 })
        await cache.prefetchAll()
        let full = await cache.getCacheStatus(); XCTAssertTrue(full.values.allSatisfy { $0 })
        await cache.ensureCached(difficulty: .medium)
        let cached = await cache.getPuzzle(difficulty: .medium); XCTAssertEqual(cached.getAllCells()[0].value, 5)
        let api = PuzzleCache(fetch: { _ in ServiceFixtures.game() }, generate: { _ in XCTFail("API hit must not generate"); return ServiceFixtures.game() })
        let fetched = await api.getPuzzle(difficulty: .hard); XCTAssertEqual(fetched.getAllCells()[0].value, 5)
        let fallback = PuzzleCache(fetch: { _ in nil }, generate: { _ in ServiceFixtures.game() })
        let generated = await fallback.getPuzzle(difficulty: .hard); XCTAssertEqual(generated.getAllCells()[0].value, 5)
        await fallback.prefetch(difficulty: .easy).value
        let status = await fallback.getCacheStatus(); XCTAssertEqual(status[.easy], true)
    }
    func testConcurrentWarmupsGenerateOnlyOnce() async {
        actor Counter {
            var count = 0
            func generate() async -> SudokuGame { count += 1; try? await Task.sleep(nanoseconds: 10_000_000); return ServiceFixtures.game() }
        }
        let counter = Counter()
        let cache = PuzzleCache(fetch: { _ in nil }, generate: { _ in await counter.generate() })
        await withTaskGroup(of: Void.self) { group in
            for _ in 0..<10 { group.addTask { await cache.ensureCached(difficulty: .beginner) } }
        }
        let count = await counter.count; XCTAssertEqual(count, 1)
        let status = await cache.getCacheStatus(); XCTAssertEqual(status[.beginner], true)
    }
    func testDefaultGeneratorProducesAPlayableGrid() async {
        let cache = PuzzleCache(fetch: { _ in nil })
        let game = await cache.getPuzzle(difficulty: .beginner)
        let cells = game.getAllCells()
        XCTAssertEqual(cells.count, 81)
        XCTAssertTrue(cells.contains { $0.isGiven && $0.value > 0 })
        XCTAssertTrue(cells.contains { !$0.isGiven && $0.value == 0 })
    }

}
