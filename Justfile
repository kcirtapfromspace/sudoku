# Default: fast local check (compile + clippy on core only)
default: check

# --- Fast local targets (use during development) ---

# Quick compile check — no codegen, no tests
check:
    cargo check --workspace --exclude sudoku-wasm

# Check all native crates (excludes wasm)
check-all:
    cargo check --workspace --exclude sudoku-wasm

# Run TUI tests only
test-tui:
    cargo test -p sudoku-tui

# Clippy on workspace (excludes wasm)
lint:
    cargo clippy --workspace --exclude sudoku-wasm -- -D warnings

# Format check
fmt:
    cargo fmt --all -- --check

# Format fix
fmt-fix:
    cargo fmt --all

# --- Crate-specific builds ---

# Build the TUI binary (debug)
build-tui:
    cargo build -p sudoku-tui

# Build the TUI binary (release)
build-tui-release:
    cargo build -p sudoku-tui --release

# Build WASM (requires wasm32-unknown-unknown target)
build-wasm:
    cd crates/sudoku-wasm && wasm-pack build --target web --release

# Build FFI (UniFFI for iOS)
build-ffi:
    cargo build -p sudoku-ffi

# --- Full CI-equivalent targets ---

# Full workspace test excluding soundness (mirrors CI test job)
test-all:
    cargo test --workspace --exclude sudoku-wasm --all-features -- --skip soundness

# Full CI pipeline: fmt + clippy + test + wasm build
ci: fmt lint-all test-all build-wasm
    @echo "CI passed."

# Clippy on full workspace
lint-all:
    cargo clippy --workspace --exclude sudoku-wasm --all-features -- -D warnings

# --- Convenience ---

# Run the TUI
run *args:
    cargo run -p sudoku-tui -- {{args}}

# Clean build artifacts
clean:
    cargo clean
