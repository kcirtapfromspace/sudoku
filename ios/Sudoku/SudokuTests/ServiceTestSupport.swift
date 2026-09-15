import XCTest
import UIKit
@testable import Sudoku

final class ServiceURLProtocol: URLProtocol {
    static let lock = NSLock()
    static var handlers: [String: (URLRequest) throws -> (Data, URLResponse)] = [:]
    static func session(_ handler: @escaping (URLRequest) throws -> (Data, URLResponse)) -> URLSession {
        let key = UUID().uuidString
        lock.lock(); handlers[key] = handler; lock.unlock()
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [ServiceURLProtocol.self]
        configuration.httpAdditionalHeaders = ["X-Test-Session": key]
        return URLSession(configuration: configuration)
    }
    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        Self.lock.lock()
        let handler = Self.handlers[request.value(forHTTPHeaderField: "X-Test-Session") ?? ""]
        Self.lock.unlock()
        do {
            guard let handler else { throw URLError(.unsupportedURL) }
            let (data, response) = try handler(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data)
            client?.urlProtocolDidFinishLoading(self)
        } catch { client?.urlProtocol(self, didFailWithError: error) }
    }
    override func stopLoading() {}
}

enum ServiceFixtures {
    static let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
    static let solution = "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    static func defaults() -> UserDefaults { UserDefaults(suiteName: "sudoku-services-\(UUID().uuidString)")! }
    static func game() -> SudokuGame {
        gameFromPregenerated(puzzleString: puzzle, solutionString: solution, difficulty: "Medium", seRating: 2.0)!
    }
    static func response(_ request: URLRequest, status: Int = 200, body: Any = [:], headers: [String: String]? = nil) throws -> (Data, URLResponse) {
        (try JSONSerialization.data(withJSONObject: body), HTTPURLResponse(url: request.url!, statusCode: status, httpVersion: nil, headerFields: headers)!)
    }
    static func image(size: CGSize = CGSize(width: 180, height: 180), draw: (UIGraphicsImageRendererContext) -> Void = { _ in }) -> UIImage {
        let format = UIGraphicsImageRendererFormat(); format.scale = 1
        return UIGraphicsImageRenderer(size: size, format: format).image { context in
            UIColor.white.setFill(); context.fill(CGRect(origin: .zero, size: size)); draw(context)
        }
    }
}
