# Sudoku (Rust Engine + TUI + WASM + iOS)

Shared Sudoku engine written in Rust, with:

- Core engine ([sudoku-core](https://github.com/kcirtapfromspace/sudoku-core))
- Terminal UI (`crates/sudoku-tui`)
- WebAssembly build (`crates/sudoku-wasm`)
- iOS app via UniFFI (`crates/sudoku-ffi` + `ios/`)

App Store: https://apps.apple.com/us/app/sudoku/id6758485043

Brand page (GitHub Pages): https://kcirtapfromspace.github.io/sudoku/

## Demos

### TUI

![TUI demo](docs/demos/tui.gif)

### WASM

![WASM demo](docs/demos/wasm.gif)

## Quickstart

### TUI

```bash
cargo run -p sudoku-tui --bin sudoku
```

### WASM

```bash
wasm-pack build crates/sudoku-wasm --target web --out-dir crates/sudoku-wasm/www/pkg --release
cd crates/sudoku-wasm/www
python3 serve.py 8080
```

Then open `http://127.0.0.1:8080/`.

Live at [ukodus.com/play](https://ukodus.com/play/).

### iOS

Open `ios/Sudoku/Sudoku.xcodeproj` in Xcode and run the `Sudoku` scheme.

## Puzzle Generation (Rust)

Puzzle generation lives in the Rust core crate:

- Generator: [`generator.rs`](https://github.com/kcirtapfromspace/sudoku-core/blob/main/src/generator.rs)
- Solver + uniqueness + difficulty rating: [`src/solver/`](https://github.com/kcirtapfromspace/sudoku-core/tree/main/src/solver) (modular directory with engines for fish, ALS, AIC, uniqueness, etc.)

At a high level:

1. **Create a fully solved grid** (a complete valid Sudoku solution).
2. **Remove givens while preserving uniqueness**:
   - Remove values (using symmetric pairs) to form a puzzle.
   - After each removal, verify the puzzle still has **exactly one solution** (the solver stops once it finds 2).
3. **Rate the puzzle difficulty** using a human-style technique simulation and retry generation until it matches the requested difficulty.

The `PuzzleId` system ([`puzzle_id.rs`](https://github.com/kcirtapfromspace/sudoku-core/blob/main/src/puzzle_id.rs)) encodes puzzle parameters into short alphanumeric codes, enabling deterministic regeneration and shareable puzzle links.

The iOS app uses this same generator through the Rust FFI layer (`crates/sudoku-ffi`), and stores the solved grid alongside the puzzle so it can power hints and validation. The WASM build powers [ukodus.com/play](https://ukodus.com/play/) and includes an anti-cheat move log that records timestamped actions for leaderboard verification.
