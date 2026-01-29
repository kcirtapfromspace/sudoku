# Sudoku Engine Examples

## Basic Usage

Run the basic example to see puzzle generation and solving:

```bash
cargo run --example basic
```

## TUI Application

Run the interactive terminal interface:

```bash
cargo run --bin sudoku
```

### Controls

| Key | Action |
|-----|--------|
| Arrow keys / hjkl | Navigate cells |
| 1-9 | Enter number |
| Shift + 1-9 | Toggle candidate |
| 0 / Delete / Backspace | Clear cell |
| c | Toggle candidate mode |
| u | Undo |
| Ctrl+r | Redo |
| ? | Show hint |
| ! | Apply hint |
| n | New game |
| p | Pause/Resume |
| t | Change theme |
| Shift+S | Save game |
| Shift+L | Load game |
| q | Quit |

## Building for Mobile

### iOS (Swift)

```bash
cd crates/sudoku-ffi
cargo build --release --target aarch64-apple-ios
cargo run --bin uniffi-bindgen generate --library ../target/release/libsudoku_ffi.dylib --language swift --out-dir ./generated
```

### Android (Kotlin)

```bash
cd crates/sudoku-ffi
cargo build --release --target aarch64-linux-android
cargo run --bin uniffi-bindgen generate --library ../target/release/libsudoku_ffi.so --language kotlin --out-dir ./generated
```
