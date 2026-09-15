import Foundation

extension CellModel {
    /// One coherent announcement for each cell, including empty and note-only cells.
    func accessibilityValue(showErrors: Bool = true) -> String {
        var parts: [String] = []
        if value == 0 {
            parts.append("Empty")
            if !candidates.isEmpty {
                parts.append("Notes " + candidates.sorted().map(String.init).joined(separator: ", "))
            }
        } else {
            parts.append(String(value))
        }
        if isGiven { parts.append("Given") }
        if hasConflict && showErrors { parts.append("Conflict") }
        return parts.joined(separator: ". ")
    }
}
