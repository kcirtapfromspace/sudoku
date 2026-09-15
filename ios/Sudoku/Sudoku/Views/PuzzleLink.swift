import Foundation

/// The same validated payload rules apply to camera QR codes and incoming links.
enum PuzzleLink {
    static func extract(from content: String) -> String? {
        let trimmed = content.trimmingCharacters(in: .whitespacesAndNewlines)
        if isGrid(trimmed) || isShortCode(trimmed) { return trimmed }
        guard let components = URLComponents(string: trimmed) else { return nil }
        if let code = components.queryItems?.first(where: { $0.name == "s" })?.value, isShortCode(code) {
            return code
        }
        if let grid = components.queryItems?.first(where: { $0.name == "p" })?.value, isGrid(grid) {
            return grid
        }
        return nil
    }

    private static func isGrid(_ value: String) -> Bool {
        value.utf8.count == 81 && value.utf8.allSatisfy { (48...57).contains($0) || $0 == 46 }
    }

    private static func isShortCode(_ value: String) -> Bool {
        value.utf8.count == 8 && value.utf8.allSatisfy {
            (48...57).contains($0) || (65...90).contains($0) || (97...122).contains($0)
        }
    }
}
