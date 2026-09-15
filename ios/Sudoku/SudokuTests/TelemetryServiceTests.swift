import XCTest
@testable import Sudoku

final class TelemetryServiceTests: XCTestCase {
    func testTokenCachingExpiryAndClear() async {
        let expiry = Date().addingTimeInterval(3600).timeIntervalSince1970
        var requests = 0
        let session = ServiceURLProtocol.session { request in
            requests += 1
            XCTAssertEqual(request.httpMethod, "POST")
            XCTAssertEqual(request.value(forHTTPHeaderField: "Content-Type"), "application/json")
            return try ServiceFixtures.response(request, body: ["token": "token-\(requests)", "expires_at": expiry])
        }
        let manager = TokenManager(session: session)
        let first = await manager.getToken(playerId: "player"); XCTAssertEqual(first, "token-1")
        let cached = await manager.getToken(playerId: "player"); XCTAssertEqual(cached, first); XCTAssertEqual(requests, 1)
        await manager.clearToken()
        let cleared = await manager.getToken(playerId: "player"); XCTAssertEqual(cleared, "token-2")
        let expired = TokenManager(session: session, now: { Date(timeIntervalSince1970: expiry - 59) })
        _ = await expired.getToken(playerId: "player")
        _ = await expired.getToken(playerId: "player")
        XCTAssertEqual(requests, 4, "A token within 60 seconds of expiry must be renewed")
    }
    func testTokenFailuresReturnNil() async {
        let handlers: [(URLRequest) throws -> (Data, URLResponse)] = [
            { try ServiceFixtures.response($0, status: 401) },
            { try ServiceFixtures.response($0, body: ["token": "missing expiry"]) },
            { (Data("bad".utf8), HTTPURLResponse(url: $0.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!) },
            { _ in throw URLError(.notConnectedToInternet) }
        ]
        for handler in handlers {
            let manager = TokenManager(session: ServiceURLProtocol.session(handler))
            let result = await manager.getToken(playerId: "player"); XCTAssertNil(result)
        }
    }
    @MainActor
    func testTelemetryPayloadIdentityStatusHandlingAndTokenInvalidation() async throws {
        let defaults = ServiceFixtures.defaults()
        var resultStatus = 201
        var tokenRequests = 0
        var resultRequests = 0
        var payloads: [[String: Any]] = []
        let session = ServiceURLProtocol.session { request in
            if request.url?.path.hasSuffix("token") == true {
                tokenRequests += 1
                return try ServiceFixtures.response(request, body: ["token": "auth", "expires_at": Date().addingTimeInterval(3600).timeIntervalSince1970])
            }
            resultRequests += 1
            XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer auth")
            XCTAssertEqual(request.httpMethod, "POST")
            payloads.append(try JSONSerialization.jsonObject(with: Self.body(request)) as! [String: Any])
            return try ServiceFixtures.response(request, status: resultStatus, headers: ["Retry-After": "30"])
        }
        let service = TelemetryService(session: session, defaults: defaults)
        let id = service.playerId; XCTAssertEqual(service.playerId, id)
        XCTAssertEqual(TelemetryService(session: session, defaults: defaults).playerId, id)
        let game = GameViewModel(cachedGame: ServiceFixtures.game(), difficulty: .medium)
        await service.submitResult(game: game, won: true).value
        XCTAssertEqual(payloads[0]["result"] as? String, "Win")
        XCTAssertEqual(payloads[0]["player_id"] as? String, id)
        XCTAssertEqual(payloads[0]["platform"] as? String, "ios")
        XCTAssertEqual(payloads[0]["puzzle_hash"] as? String, service.hashPuzzle(game.getPuzzleString()))
        XCTAssertEqual(payloads[0]["difficulty"] as? String, "Medium")
        XCTAssertNotNil(payloads[0]["device_model"])
        XCTAssertTrue((payloads[0]["os_version"] as? String)?.hasPrefix("iOS ") == true)
        for status in [401, 429, 500] {
            resultStatus = status
            await service.submitResult(game: game, won: false).value
        }
        XCTAssertEqual(resultRequests, 4); XCTAssertEqual(tokenRequests, 2)
        XCTAssertEqual(payloads.last?["result"] as? String, "Loss")
    }
    @MainActor
    func testTelemetryWithoutTokenAndNetworkFailure() async {
        for networkFailure in [false, true] {
            let session = ServiceURLProtocol.session { request in
                if request.url?.path.hasSuffix("token") == true { return try ServiceFixtures.response(request, status: 404) }
                XCTAssertNil(request.value(forHTTPHeaderField: "Authorization"))
                if networkFailure { throw URLError(.timedOut) }
                return (Data(), URLResponse(url: request.url!, mimeType: nil, expectedContentLength: 0, textEncodingName: nil))
            }
            let service = TelemetryService(session: session, defaults: ServiceFixtures.defaults())
            let game = GameViewModel(cachedGame: ServiceFixtures.game(), difficulty: .easy)
            await service.submitResult(game: game, won: false).value
        }
    }
    @MainActor
    func testSeededPuzzleSubmitsShareCodeAndCounters() async {
        var body: [String: Any] = [:]
        let session = ServiceURLProtocol.session { request in
            if request.url?.path.hasSuffix("token") == true { return try ServiceFixtures.response(request, status: 404) }
            body = try JSONSerialization.jsonObject(with: Self.body(request)) as! [String: Any]
            return try ServiceFixtures.response(request, status: 204)
        }
        let service = TelemetryService(session: session, defaults: ServiceFixtures.defaults())
        let game = GameViewModel(difficulty: .beginner)
        await service.submitResult(game: game, won: false).value
        XCTAssertEqual(body["short_code"] as? String, game.getShortCode())
        XCTAssertNotNil(body["short_code"])
        XCTAssertEqual(body["hints_used"] as? Int, 0)
        XCTAssertEqual(body["mistakes"] as? Int, 0)
        XCTAssertGreaterThanOrEqual(body["time_secs"] as? Int ?? -1, 0)
    }
    private static func body(_ request: URLRequest) -> Data {
        if let body = request.httpBody { return body }
        guard let stream = request.httpBodyStream else { return Data() }
        stream.open(); defer { stream.close() }
        var result = Data(), bytes = [UInt8](repeating: 0, count: 1024)
        while stream.hasBytesAvailable {
            let count = stream.read(&bytes, maxLength: bytes.count)
            if count <= 0 { break }
            result.append(bytes, count: count)
        }
        return result
    }
}
