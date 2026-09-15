# iOS coverage goals and verification

## Required thresholds

The iOS gates follow the sister `ukodus` repository's authored frontend standard:
95% lines and functions, with complete coverage for critical gameplay state.
These are separate Swift measurements, not a combined percentage with the Rust
engine or another repository's frontend.

| Scope | Minimum line coverage | Minimum function coverage |
| --- | ---: | ---: |
| All authored iOS application Swift | 95% | 95% |
| `Services/GameManager.swift` | 100% | 100% |
| `ViewModels/GameViewModel.swift` | 100% | 100% |

These thresholds do not justify deleting assertions or excluding difficult
application files. Persistence, permissions, imports, gameplay transitions and
error recovery need behavioral tests even when a numeric gate passes. Swift
branch coverage is not measured by this collector; it does not claim the sister
frontend's separate 85% branch gate.

## Run acceptance

Prerequisites: macOS, a current Xcode with an available iPhone simulator runtime,
XcodeGen, Python 3, and Rust with rustup. CI uses Rust 1.95.0. Install XcodeGen
with `brew install xcodegen`, then run from the repository root:

```sh
./ios/scripts/test-coverage.sh
```

The helper builds the Rust FFI library for the host's simulator architecture and
the native UniFFI generator, generates Swift bindings and the Xcode project,
creates a disposable iPhone simulator, and runs the complete `Sudoku` test scheme
with coverage. The scheme includes unit and UI tests. Test mode isolates saved
state and external app integrations. Screenshot tooling is not a substitute for
assertions in the acceptance suites.

Each invocation uses a new `DerivedData` directory, coverage profiles, result
bundle and simulator. Previous execution counts cannot satisfy a later run after
a test is removed. The helper never erases a user's existing simulator. Its EXIT
trap shuts down and deletes only the simulator it created, retaining results
after success or failure. `IOS_COVERAGE_DIR` can change the parent artifact
directory; each run still gets a unique child directory.

Run the collector regression suite independently:

```sh
python3 -m unittest discover -s ios/scripts -p test_coverage.py -v
```

## Authoritative measurement

`ios/scripts/coverage.py` uses the raw output of
`xcrun xccov view --archive --json`, counting each unique authored
`(source path, executable line)` once. A line is covered when any execution hits
it; zero-hit lines remain in the denominator. Duplicate records are unioned.

The ordinary `xccov --report` line totals sum overlapping functions and nested
closures. SwiftUI therefore produces totals larger than the actual source file.
Those aggregate line percentages are not the production gate. The report is
used for distinct `(source path, function start line, function name)` mappings and
execution counts, including closures and initializers mapped to authored source.
Duplicate function records are also unioned.

The collector independently inventories every `ios/Sudoku/Sudoku/**/*.swift`
file. Only the root `Generated/` UniFFI directory is excluded from that inventory;
unit/UI test targets and third-party libraries are outside the app source tree.
Nested authored folders named `Generated` are not excluded. Importing XCTest or
Swift Testing inside the application scope is rejected so test code cannot
inflate production coverage.

The Swift compiler's parser inventories explicit functions, initializers and
computed accessors. A missing application file or declared body mapping fails
the gate, even if the remaining reported files meet 95%. Comment nesting, raw
strings and generic signatures cannot hide those declarations. Matching checks
both source coordinates and demangled function names; another function or a
getter on the same line cannot stand in for a missing function or setter. Conditional
directives are flattened for this syntax-only inventory so debug functions
cannot disappear from the denominator. Declaration-only source files may have
no executable mappings, but files with initializers or macros cannot evade the
missing-file check by having no `func` keyword.

The acceptance helper hashes authored app and test source files before running
XCTest. The collector rejects changes made during that run, including removed
tests and source edits that keep the same line count. It also rejects malformed/negative execution counts,
source lines beyond file bounds, missing target data and empty production
coverage. Treat a report as accepted only when its `passed` field is true and its
fresh source manifest was verified.

### Baseline

The September 15, 2026 baseline at
`c371f588aec8a80bb5402d853b6d7414cdb9013c` ran the five original Swift unit tests
with coverage. Its 27 authored app files measured **355 / 5,815 unique executable
lines (6.10%)** and **112 / 1,019 mapped functions (10.99%)**. The UI suites were
not executed in that baseline. Replaying its archive against the matching source
snapshot produced no missing declared-function mappings.

### Accepted result — September 15, 2026

The `codex/ios-coverage` checkout passed the full acceptance command on Xcode
26.6 / iOS 26.5 with a fresh iPhone 17 Pro simulator and fresh build data:

| Scope | Unique executable lines | Mapped functions |
| --- | ---: | ---: |
| All 32 authored Swift files | **5,962 / 6,007 (99.25%)** | **1,205 / 1,245 (96.79%)** |
| Game manager | **301 / 301 (100%)** | **71 / 71 (100%)** |
| Game view model | **732 / 732 (100%)** | **181 / 181 (100%)** |

**150 unit tests, 9 UI tests, and 33 collector regression tests passed.** Both
XCTest and the coverage gate exited zero. App and test source hashes matched the
pre-test manifest, with no missing source or declaration mappings.

The [accepted summary](review-artifacts/ios-2026-09-15/after/summary-production.json)
and [source manifest](review-artifacts/ios-2026-09-15/after/source-manifest.json)
record the exact tested checkout. [Results and screenshots](ios-coverage-results-2026-09-15.md)
explain the behavior covered. Full local results remain in
`ios/Sudoku/build/coverage/run.7qiUb9/`. This is a local acceptance run; the CI
workflow change has not been pushed or run on GitHub.

Source and compiler changes can change both denominators. Re-run acceptance
for subsequent changes; this result does not transfer to a different checkout.

## Artifacts and CI

The default output parent is `ios/Sudoku/build/coverage/`. `latest-run.txt` points
to the latest unique `run.*` directory. Each run retains:

- `summary-production.json`: coverage gates, uncovered lines/functions, file
  inventory, source hashes and mapping errors.
- `xccov-archive.json` and `xccov-report.json`: raw Xcode measurement inputs.
- `tests.xcresult`: test results, screenshots and Xcode diagnostic attachments.
- `source-manifest.json`, tool versions and `acceptance-status.txt`: source and
  execution provenance.
- Build, XCTest and collector logs, including failures.

The `iOS tests and coverage` job in `.github/workflows/build-ios.yml` runs for
pull requests and pushes to `main` affecting iOS, FFI, the Rust core or Cargo inputs, and for
manual dispatches. It performs the full acceptance command and uploads reports
even after failures. TestFlight deployment now depends on that job succeeding.
Deployment generates the same project definition and selects the same Rust and
Xcode versions as acceptance.
Repository branch protection is a separate GitHub setting; these source changes
do not configure it.

## Limits

Coverage shows code ran in asserted scenarios; it does not prove every puzzle,
camera device or race is correct. Deterministic tests replace hardware, clock,
network and scheduling boundaries while exercising production state transitions.
The Apple camera adapter remains in the measured source scope. Actual device
camera behavior, Game Center integration and full accessibility audits still
benefit from device testing.

Compiler-generated `#Preview` macro expansions are not surfaced as application
coverage mappings by Xcode. Ordinary authored preview or debug functions that
have executable mappings stay included. SwiftUI nested-closure accounting and
the absence of a branch metric are disclosed rather than treated as exclusions
that can be adjusted to make a gate pass.
