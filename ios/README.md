# Sudoku iOS App

A native iOS Sudoku app powered by the Rust `sudoku-core` engine.

## Requirements

- Xcode 15.0+
- iOS 16.0+ deployment target
- Rust toolchain (for building the engine)
- [xcodegen](https://github.com/yonaskolb/XcodeGen) (for generating Xcode project)

## Quick Start

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install xcodegen
brew install xcodegen

# Add iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

### 2. Build the Project

```bash
# From the workspace root
cd ios
./build.sh
```

This will:
- Build the Rust library for iOS device and simulator
- Generate Swift bindings via UniFFI
- Create an XCFramework
- Generate the Xcode project

### 3. Open in Xcode

```bash
open Sudoku/Sudoku.xcodeproj
```

Or generate the project manually:

```bash
cd Sudoku
xcodegen generate
open Sudoku.xcodeproj
```

## Project Structure

```
ios/
├── build.sh              # Build script for Rust + Xcode project
├── Frameworks/           # Generated XCFramework (after build)
└── Sudoku/
    ├── project.yml       # xcodegen configuration
    ├── Sudoku/
    │   ├── SudokuApp.swift
    │   ├── Models/
    │   │   └── GameModels.swift
    │   ├── ViewModels/
    │   │   └── GameViewModel.swift
    │   ├── Views/
    │   │   ├── ContentView.swift
    │   │   ├── GameView.swift
    │   │   ├── GridView.swift
    │   │   ├── CellView.swift
    │   │   ├── NumberPadView.swift
    │   │   ├── SettingsView.swift
    │   │   └── StatsView.swift
    │   ├── Services/
    │   │   └── GameManager.swift
    │   ├── Resources/
    │   │   └── Assets.xcassets/
    │   └── Generated/    # UniFFI-generated Swift bindings
    └── SudokuTests/
```

## Features

- **Native SwiftUI interface** - Modern iOS design with full Dark Mode support
- **Universal app** - Optimized for both iPhone and iPad
- **Multiple input methods** - Tap, external keyboard support
- **Game features** - Undo/redo, hints, candidates/notes mode
- **Statistics** - Track games played, best times, win streaks
- **Persistence** - Auto-save games, statistics stored locally
- **Accessibility** - VoiceOver support, Dynamic Type

## Puzzle Generation (Rust)

When you start a new game, the iOS app asks the Rust engine to generate a brand-new Sudoku puzzle for the selected difficulty.

The call chain looks like this:

- Swift: `GameManager.newGame(...)` -> `PuzzleCache.shared.getPuzzle(...)`
- Swift -> Rust (UniFFI): `SudokuGame.newClassic(difficulty: ...)`
- Rust: `SudokuGame::new_classic` calls `sudoku_core::Generator::generate(...)` and then solves it with `sudoku_core::Solver`

### How the Generator Picks a Puzzle

The generator lives in `crates/sudoku-core/src/generator.rs` and works in three phases:

1. Generate a full valid solution grid
   - Start with an empty classic 9x9 grid.
   - Randomly fill the 3 diagonal 3x3 boxes (they don't constrain each other).
   - Use the solver to complete the rest of the grid.

2. Remove givens while keeping a unique solution
   - Shuffle all 81 positions.
   - Remove values in symmetric pairs (default is 180-degree rotational symmetry; `Extreme` uses no symmetry).
   - After each removal, the solver checks that the puzzle still has exactly one solution.

3. Rate difficulty and accept/retry
   - The solver runs `rate_difficulty` (in `crates/sudoku-core/src/solver.rs`) by applying human-style techniques in order.
   - The generator accepts puzzles that match the target difficulty (sometimes within one adjacent tier) and that land in a givens range for that difficulty.
   - It will retry up to `max_attempts` for that difficulty before falling back to the last attempt.

### Why the Engine Also Computes the Solution

In `crates/sudoku-ffi/src/lib.rs`, `SudokuGame::new_classic` generates the puzzle grid and also solves it immediately. The solved grid is used to:

- validate player moves (for mistake counting)
- generate hints
- support fast checks like "is this puzzle complete?"

Example usage with Rust engine:

```swift
import SudokuEngine

// Create a new game
let game = SudokuGame.newClassic(difficulty: .medium)

// Make a move
let result = game.makeMove(row: 0, col: 0, value: 5)

// Get hint
if let hint = game.getHint() {
    print("Hint: \(hint.explanation)")
}
```

## Development Notes

### Tests and coverage

Run the complete simulator acceptance suite from the repository root:

```sh
./ios/scripts/test-coverage.sh
```

It builds the simulator FFI dependency, regenerates the project, runs unit and UI
tests on a disposable simulator, and checks 95% authored Swift line/function
coverage plus 100% critical gameplay coverage. Each run starts with fresh
profiles and retains test/coverage artifacts on failure. See
[iOS coverage goals and verification](../docs/ios-coverage-goals.md) for scope,
measurement details and prerequisites.

### Rebuilding Rust vs Iterating on SwiftUI

Run `./build.sh` when you change Rust code or the UniFFI interface. If you're only changing SwiftUI/view code, you can usually just build/run from Xcode without regenerating bindings.

### Building for Release

For App Store submission, ensure you:
1. Build the Rust library in release mode
2. Set your development team in Xcode
3. Configure App Store Connect for Game Center and iCloud

### TestFlight versioning

Set `MARKETING_VERSION` in `Sudoku/project.yml` to the next app version when
the previous version has been approved and closed to new TestFlight uploads.
Regenerate the project from the repository root and commit both files:

```sh
xcodegen generate --spec ios/Sudoku/project.yml
```

CI regenerates the Xcode project from this spec. Changing only the generated
project, including through Fastlane's version-bump lanes, will be overwritten.
The `beta_manual` lane increments the build number automatically; it does not
advance the app version. A newer build number cannot reopen a closed version.

## License

MIT License - see the root LICENSE file.
