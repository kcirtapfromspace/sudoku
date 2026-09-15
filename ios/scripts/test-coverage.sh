#!/bin/bash
# Full simulator acceptance run. Each invocation owns fresh profiles and a new
# simulator; old test hits cannot satisfy a later run. Artifacts remain on failure.
set -euo pipefail

TASK_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TASK_OUTPUT_BASE="${IOS_COVERAGE_DIR:-$TASK_ROOT/ios/Sudoku/build/coverage}"
mkdir -p "$TASK_OUTPUT_BASE"
TASK_RUN_DIR="$(mktemp -d "$TASK_OUTPUT_BASE/run.XXXXXX")"
TASK_SIMULATOR=""
printf '%s\n' "$TASK_RUN_DIR" > "$TASK_OUTPUT_BASE/latest-run.txt"

# shellcheck disable=SC2317 # Invoked indirectly by the EXIT trap.
cleanup() {
    local status=$?
    if [ -n "$TASK_SIMULATOR" ]; then
        xcrun simctl shutdown "$TASK_SIMULATOR" >/dev/null 2>&1 || true
        xcrun simctl delete "$TASK_SIMULATOR" >/dev/null 2>&1 || true
    fi
    printf 'Acceptance artifacts: %s\n' "$TASK_RUN_DIR"
    exit "$status"
}
trap cleanup EXIT

for task_tool in xcrun xcodebuild xcodegen python3 rustup cargo; do
    command -v "$task_tool" >/dev/null || { printf 'Missing tool: %s\n' "$task_tool" >&2; exit 2; }
done

cd "$TASK_ROOT"
python3 -m unittest discover -s ios/scripts -p test_coverage.py -v 2>&1 | tee "$TASK_RUN_DIR/collector-tests.log"
xcodebuild -version > "$TASK_RUN_DIR/xcode-version.txt"
rustc --version > "$TASK_RUN_DIR/rust-version.txt"
git rev-parse HEAD > "$TASK_RUN_DIR/git-commit.txt"
git diff --stat > "$TASK_RUN_DIR/source-changes.txt"

case "$(uname -m)" in
    arm64) TASK_RUST_TARGET="aarch64-apple-ios-sim" ;;
    x86_64) TASK_RUST_TARGET="x86_64-apple-ios" ;;
    *) printf 'Unsupported simulator host architecture\n' >&2; exit 2 ;;
esac
rustup target add "$TASK_RUST_TARGET"
cargo build --locked -p sudoku-ffi --release --target "$TASK_RUST_TARGET" 2>&1 | tee "$TASK_RUN_DIR/rust-simulator-build.log"
cargo build --locked -p sudoku-ffi --release 2>&1 | tee "$TASK_RUN_DIR/rust-bindgen-build.log"
TASK_CARGO_TARGET="$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
mkdir -p ios/Frameworks ios/Sudoku/Sudoku/Generated
cp "$TASK_CARGO_TARGET/$TASK_RUST_TARGET/release/libsudoku_ffi.a" ios/Frameworks/libsudoku_ffi_sim.a
"$TASK_CARGO_TARGET/release/uniffi-bindgen" generate \
    --library "$TASK_CARGO_TARGET/release/libsudoku_ffi.dylib" \
    --language swift --out-dir ios/Sudoku/Sudoku/Generated
cp ios/Sudoku/Sudoku/Generated/SudokuEngineFFI.h ios/Frameworks/
cp ios/Sudoku/Sudoku/Generated/SudokuEngineFFI.modulemap ios/Frameworks/module.modulemap

xcodegen generate --spec ios/Sudoku/project.yml
python3 ios/scripts/coverage.py --write-manifest "$TASK_RUN_DIR/source-manifest.json"

# Select a device/runtime combination already supported by the installed Xcode,
# then create our own empty simulator instead of erasing an existing device.
xcrun simctl list devices available --json > "$TASK_RUN_DIR/available-simulators.json"
python3 - "$TASK_RUN_DIR/available-simulators.json" > "$TASK_RUN_DIR/simulator-config.txt" <<'PY'
import json, re, sys
devices = json.load(open(sys.argv[1]))["devices"]
def version(runtime):
    return tuple(map(int, re.findall(r"\d+", runtime)))
for runtime in sorted(devices, key=version, reverse=True):
    if ".iOS-" not in runtime:
        continue
    for device in devices[runtime]:
        if device.get("isAvailable") and device["name"].startswith("iPhone") and device.get("deviceTypeIdentifier"):
            print(device["deviceTypeIdentifier"])
            print(runtime)
            raise SystemExit(0)
raise SystemExit("No available iPhone simulator; install an iOS runtime in Xcode settings.")
PY
TASK_DEVICE_TYPE="$(sed -n '1p' "$TASK_RUN_DIR/simulator-config.txt")"
TASK_RUNTIME="$(sed -n '2p' "$TASK_RUN_DIR/simulator-config.txt")"
TASK_SIMULATOR="$(xcrun simctl create "Sudoku coverage $(basename "$TASK_RUN_DIR")" "$TASK_DEVICE_TYPE" "$TASK_RUNTIME")"
printf '%s\n' "$TASK_SIMULATOR" > "$TASK_RUN_DIR/simulator-udid.txt"
xcrun simctl boot "$TASK_SIMULATOR"
xcrun simctl bootstatus "$TASK_SIMULATOR" -b

set +e
xcodebuild test \
    -project ios/Sudoku/Sudoku.xcodeproj -scheme Sudoku \
    -configuration Debug -destination "platform=iOS Simulator,id=$TASK_SIMULATOR" \
    -derivedDataPath "$TASK_RUN_DIR/DerivedData" \
    -resultBundlePath "$TASK_RUN_DIR/tests.xcresult" \
    -enableCodeCoverage YES -parallel-testing-enabled NO \
    CODE_SIGNING_ALLOWED=NO \
    2>&1 | tee "$TASK_RUN_DIR/xcodebuild.log"
TASK_TEST_STATUS=${PIPESTATUS[0]}
set -e

# Export even when XCTest fails. xccov errors are failures, never an empty pass.
TASK_COVERAGE_STATUS=2
if [ -d "$TASK_RUN_DIR/tests.xcresult" ]; then
    if xcrun xccov view --archive --json "$TASK_RUN_DIR/tests.xcresult" > "$TASK_RUN_DIR/xccov-archive.json" \
        && xcrun xccov view --report --json "$TASK_RUN_DIR/tests.xcresult" > "$TASK_RUN_DIR/xccov-report.json"; then
        set +e
        python3 ios/scripts/coverage.py \
            --archive "$TASK_RUN_DIR/xccov-archive.json" \
            --report "$TASK_RUN_DIR/xccov-report.json" \
            --manifest "$TASK_RUN_DIR/source-manifest.json" \
            --output "$TASK_RUN_DIR/summary-production.json" \
            2>&1 | tee "$TASK_RUN_DIR/coverage-gate.log"
        TASK_COVERAGE_STATUS=${PIPESTATUS[0]}
        set -e
    fi
fi
printf 'XCTest exit: %s\nCoverage gate exit: %s\n' "$TASK_TEST_STATUS" "$TASK_COVERAGE_STATUS" > "$TASK_RUN_DIR/acceptance-status.txt"
if [ "$TASK_TEST_STATUS" -ne 0 ]; then
    exit "$TASK_TEST_STATUS"
fi
exit "$TASK_COVERAGE_STATUS"
