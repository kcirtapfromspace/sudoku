import SwiftUI

/// Keeps a selected difficulty and its rating together as the picker expands.
@MainActor
final class NewGameSelection: ObservableObject {
    @Published var expanded: Difficulty?
    @Published var targetSE: Float = 2.0
}
