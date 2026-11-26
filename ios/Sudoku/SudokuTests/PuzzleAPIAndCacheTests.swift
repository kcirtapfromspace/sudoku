import XCTest
@testable import Sudoku

final class PuzzleAPIAndCacheTests: XCTestCase {
    private let hard = "002601000407080000008090050780009005500030001200800093040020700000010904000306500"

    func testPublicCatalogChecksBoardAndComputesSolutionAndRating() async throws {
        let board = hard
        let session = ServiceURLProtocol.session { request in
            XCTAssertNil(request.value(forHTTPHeaderField: "X-Api-Key"))
            XCTAssertEqual(request.url?.path, "/api/v1/puzzles/random")
            XCTAssertEqual(URLComponents(url: request.url!, resolvingAgainstBaseURL: false)?.queryItems,
                           [URLQueryItem(name: "difficulty", value: "Hard")])
            return try ServiceFixtures.response(request, body: ["puzzle_string": board, "difficulty": "Hard", "se_rating": 0])
        }
        let api = PuzzleAPIService(session: session, defaults: ServiceFixtures.defaults())
        let result = await api.fetchPuzzle(difficulty: .hard)
        let game = try XCTUnwrap(result)
        XCTAssertEqual(game.getRatedDifficulty(), .hard)
        XCTAssertEqual(game.getSeRating(), 2.6, accuracy: 0.001)
        XCTAssertGreaterThan(game.getSolutionValue(row: 0, col: 0), 0)
    }

    func testInvalidCatalogResponseFallsBack() async {
        let handlers: [(URLRequest) throws -> (Data, URLResponse)] = [
            { try ServiceFixtures.response($0, status: 404) },
            { try ServiceFixtures.response($0, body: ["difficulty": "Expert", "puzzle_string": self.hard]) },
            { try ServiceFixtures.response($0, body: ["difficulty": "Hard", "puzzle_string": ServiceFixtures.puzzle]) },
            { try ServiceFixtures.response($0, body: ["difficulty": "Hard", "puzzle_string": String(repeating: "0", count: 81)]) },
            { try ServiceFixtures.response($0, body: ["difficulty": "Hard", "puzzle_string": "bad"]) },
            { _ in throw URLError(.notConnectedToInternet) }
        ]
        for handler in handlers {
            let api = PuzzleAPIService(session: ServiceURLProtocol.session(handler), defaults: ServiceFixtures.defaults())
            let game = await api.fetchPuzzle(difficulty: .hard)
            XCTAssertNil(game)
        }
        let defaults = ServiceFixtures.defaults()
        defaults.set("file:///tmp", forKey: "api_base_url")
        let api = PuzzleAPIService(session: ServiceURLProtocol.session { _ in XCTFail(); throw URLError(.badURL) }, defaults: defaults)
        let result = await api.fetchPuzzle(difficulty: .hard)
        XCTAssertNil(result)
    }

    func testBundledReserveIsUniqueRatedAndRotatesAcrossColdStarts() throws {
        let defaults = ServiceFixtures.defaults()
        var all = Set<String>()
        for difficulty in PuzzleAPIService.eligibleDifficulties {
            let choices = try XCTUnwrap(StarterPuzzles.boards[difficulty.rawValue])
            XCTAssertGreaterThanOrEqual(choices.count, 8)
            if difficulty == .extreme {
                XCTAssertTrue(choices.contains { $0.rating == 11 })
            }
            var cycle: [String] = []
            for _ in choices {
                let game = try XCTUnwrap(StarterPuzzles.next(for: difficulty, defaults: defaults))
                let board = game.getPuzzleString()
                guard case .valid = validatePuzzleString(puzzle: board) else { return XCTFail("Invalid starter") }
                let solved = try XCTUnwrap(gameFromString(puzzle: board))
                XCTAssertEqual(solved.getRatedDifficulty(), difficulty.toGameDifficulty())
                XCTAssertEqual(game.getSeRating(), solved.getSeRating())
                for i in 0..<81 {
                    let row = UInt8(i / 9), col = UInt8(i % 9)
                    XCTAssertEqual(game.getSolutionValue(row: row, col: col), solved.getSolutionValue(row: row, col: col))
                }
                XCTAssertTrue(all.insert(board).inserted)
                cycle.append(board)
            }
            XCTAssertEqual(StarterPuzzles.next(for: difficulty, defaults: defaults)?.getPuzzleString(), cycle.first)
        }
        XCTAssertNil(StarterPuzzles.next(for: .easy, defaults: defaults))
    }

    func testColdExactStartsRotateMatchingBoardsDuringPendingGeneration() async {
        let gate = AsyncStream<Void>.makeStream()
        let cache = PuzzleCache(fetch: { _ in XCTFail("Unexpected tier fetch"); return nil },
                                generateSE: { _ in
                                    for await _ in gate.stream {}
                                    return ServiceFixtures.game()
                                })
        let first = await cache.getPuzzle(targetSE: 11)
        let second = await cache.getPuzzle(targetSE: 11)
        let otherRating = await cache.getPuzzle(targetSE: 9.3)
        gate.continuation.finish()
        XCTAssertEqual(first.getSeRating(), 11)
        XCTAssertEqual(second.getSeRating(), 11)
        XCTAssertNotEqual(first.getPuzzleString(), second.getPuzzleString())
        XCTAssertEqual(otherRating.getSeRating(), 9.3)
    }

    func testColdStartDoesNotWaitForCatalogAndConsumesReadyGameNext() async {
        let gate = AsyncStream<Void>.makeStream()
        let catalog = ServiceFixtures.game()
        let cache = PuzzleCache(fetch: { _ in
            for await _ in gate.stream { break }
            return catalog
        }, generate: { _ in XCTFail("Unexpected generation"); return ServiceFixtures.game() },
        starter: { _ in ServiceFixtures.game() })
        let cold = await cache.getPuzzle(difficulty: .hard)
        XCTAssertFalse(cold === catalog)
        gate.continuation.finish()
        await cache.ensureCached(difficulty: .hard)
        let next = await cache.getPuzzle(difficulty: .hard)
        XCTAssertTrue(next === catalog)
    }

    func testConcurrentWarmupsAndStartsConsumeDistinctGames() async {
        actor Counter {
            var count = 0
            func generate() async -> SudokuGame {
                count += 1
                await Task.yield()
                return ServiceFixtures.game()
            }
        }
        let counter = Counter()
        let cache = PuzzleCache(fetch: { _ in nil }, generate: { _ in await counter.generate() }, starter: { _ in nil })
        await withTaskGroup(of: Void.self) { group in
            for _ in 0..<8 { group.addTask { await cache.ensureCached(difficulty: .medium) } }
        }
        let count = await counter.count
        XCTAssertEqual(count, 1)
        async let first = cache.getPuzzle(difficulty: .medium)
        async let second = cache.getPuzzle(difficulty: .medium)
        let games = await (first, second)
        XCTAssertFalse(games.0 === games.1)
    }

    func testOfflineRefillAndMissingBundleStillProducePlayableGames() async {
        let replacement = StarterPuzzles.next(for: .hard, defaults: ServiceFixtures.defaults())!
        let cache = PuzzleCache(fetch: { _ in nil }, generate: { _ in ServiceFixtures.game() }, starter: { _ in replacement })
        await cache.prefetch(difficulty: .hard).value
        let result = await cache.getPuzzle(difficulty: .hard)
        XCTAssertTrue(result === replacement)
        let offline = PuzzleCache(fetch: { _ in nil }, starter: { _ in nil })
        let easy = await offline.getPuzzle(difficulty: .beginner)
        XCTAssertTrue(easy.getAllCells().contains { $0.isGiven })
    }
}
