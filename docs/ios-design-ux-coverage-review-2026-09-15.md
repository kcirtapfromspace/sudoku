# iOS design, UX, and test coverage review

Reviewed September 15, 2026 against `kcirtapfromspace/sudoku` main, commit `c371f588aec8a80bb5402d853b6d7414cdb9013c`.

## Status of this review

This is the historical review of the commit named above. Follow-up implementation
and regression tests passed acceptance on `codex/ios-coverage`; the acceptance
standard and reproducible command are in [iOS coverage goals](ios-coverage-goals.md).
The findings below describe the reviewed baseline, not the updated checkout.
See [accepted results and updated screenshots](ios-coverage-results-2026-09-15.md).

A subsequent measurement audit found that Xcode's ordinary line summary counts
nested SwiftUI function ranges more than once. The baseline's authoritative
unique-source measurement is **355 / 5,815 lines (6.10%)** and **112 / 1,019 mapped
functions (10.99%)**. The original aggregate table is retained below with its
accounting method identified.

## Summary

The iOS app has tests, but CI does not run the Swift tests or collect coverage. A local run completed during this review: **all five unit tests passed, covering 6.10% of unique handwritten Swift executable source lines**. The gameplay screen, grid, cells, and number pad each measured **0% in this unit-test-only run**. Existing tests provide limited protection for a design change. The highest-priority UX problems are missing landscape input, lost or inconsistent pencil notes, and incomplete board accessibility. Mistake settings and pause timing also disagree with their displayed behavior.

This review combines current SwiftUI/source inspection, CI configuration and run inspection, a local coverage-enabled unit test run, and fresh simulator interaction on iPhone 17 Pro / iOS 26.5. Portrait-to-landscape rotation reproduced the missing keypad. The simulator accessibility tree confirmed digits without cell coordinates and mislabeled icon actions. Historical committed screenshots were also inspected: their menus differ from current source, and some filenames do not describe the pictured screen. Findings below distinguish source-confirmed behavior from design recommendations. A full VoiceOver session and the separate UI test suite were not run.

## Original Xcode aggregate summary

These line totals sum overlapping function ranges; they are not unique-source coverage or the new acceptance gate.

| Scope | Covered / executable lines | Line coverage |
| --- | ---: | ---: |
| Handwritten Swift, excluding `Generated/` | 829 / 11,115 | **7.46%** |
| SwiftUI views | 444 / 8,177 | **5.43%** |
| View models | 86 / 897 | 9.59% |
| Services | 165 / 1,709 | 9.65% |
| App target including generated Swift bindings | 1,230 / 12,653 | 9.72% |

The view coverage comes entirely from `ContentView.swift` executing during app startup. `GameView`, `GridView`, `CellView`, `NumberPadView`, `SettingsView`, `StatsView`, import/confirmation, history, and win/loss views each recorded 0%. Executed lines do not imply asserted behavior: launching the app covers some menu code without verifying its UX.

These numbers describe **five unit tests only**, not combined unit/UI coverage. The Rust engine was built as a release library and is not measured by this Swift coverage report. [Raw coverage JSON](/Users/thinkstudio/Documents/ChatGPT/sudoku/docs/review-artifacts/ios-2026-09-15/unit-coverage.json).

## Coverage: what exists

| Layer | Evidence | What it establishes |
| --- | --- | --- |
| Swift unit tests | Five methods in [SudokuTests.swift](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/SudokuTests/SudokuTests.swift#L4) | Difficulty count, empty-cell creation, initial game state, basic statistics, input-mode toggle. No assertions for move workflows, restored notes, pause timing, or settings integration. |
| UI flow tests | Four methods in `ConsolidatedMenuTests.swift` | Menu consolidation, difficulty-picker navigation, progress tabs, and an attempted camera/OCR flow. |
| Screenshot helpers | Four methods in [ScreenshotTests.swift](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/SudokuUITests/ScreenshotTests.swift#L28) | Screenshot capture and manual helpers, including a five-minute manual session. These are not image-diff regression tests. |
| Xcode schemes | [Sudoku.xcscheme:26](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku.xcodeproj/xcshareddata/xcschemes/Sudoku.xcscheme#L26) | Unit and UI test targets are listed, but coverage collection is not enabled. `onlyGenerateCoverageForSpecifiedTargets` controls scope; it does not enable coverage. |
| iOS CI | [build-ios.yml:64](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/.github/workflows/build-ios.yml#L64) | The PR job named “Build & Test” runs `xcodebuild build`, with no test action or coverage export. |
| Rust CI | [ci.yml:69](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/.github/workflows/ci.yml#L69) | Runs Rust workspace tests. This does not establish SwiftUI or iOS user-flow coverage. |

The latest inspected [iOS workflow run](https://github.com/kcirtapfromspace/sudoku/actions/runs/22538499958), dated March 1, 2026, succeeded at TestFlight deployment; its “Build & Test” job was skipped and it had no artifacts. No existing repository-configured iOS percentage, report, or minimum coverage gate was found. The measurements above were produced locally for this review, not by CI. Test counts alone cannot be converted into a meaningful percentage.

### Existing UI tests need repairs before they provide confidence

- [ScreenshotTests.swift:42](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/SudokuUITests/ScreenshotTests.swift#L42) taps Medium once and assumes gameplay begins. The current picker expands the difficulty first; it requires Play or a second tap. Replace fixed sleeps with assertions that the board and digit buttons are present before taking a screenshot.
- The screenshot tests pass `--reset-state` and `--screenshot-mode`, but app source does not read those arguments. Add a deterministic test launch configuration, isolated saved data, and a fixed puzzle fixture.
- [ConsolidatedMenuTests.swift:151](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/SudokuUITests/ConsolidatedMenuTests.swift#L151) can return if the OCR fixture control is missing and does not assert the resulting recognized puzzle. Require the fixture and assert its cells and import outcome.
- The committed `ios/screenshots/03_Gameplay.png` shows a loading spinner; `ios/screenshots/ipad/03_gameplay.png` shows the main menu. Retake screenshots from verified states. Do not use these filenames as evidence that gameplay layouts passed.

## Priority UX findings

### P1 — Landscape removes the number pad

[GameView.swift:23](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/Views/GameView.swift#L23): the landscape branch renders the board and action controls. It omits `numberPadSection`, the status header, and the hint panel. Compact controls only contain action icons and the input-mode toggle. Rotation therefore removes touch entry of digits and hides hint explanations.

**Reproduced in the simulator.** Compare [portrait](/Users/thinkstudio/Documents/ChatGPT/sudoku/docs/review-artifacts/ios-2026-09-15/gameplay-portrait.png) with [landscape](/Users/thinkstudio/Documents/ChatGPT/sudoku/docs/review-artifacts/ios-2026-09-15/gameplay-landscape.png).

**Change:** Include the number pad, status, and hints in an adaptive side panel. **Verify:** Start a fixed puzzle, rotate, enter a digit, switch to notes, request a hint, and rotate back without losing state.

### P1 — Pencil notes disappear on restore and resist Clear/Undo

[GameViewModel.swift:143](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/ViewModels/GameViewModel.swift#L143): manual notes render from `userCandidates`, separate from the Rust engine. Clear and Undo mutate the engine but do not restore that local array. Serialization stores engine state plus time and difficulty; deserialization leaves `userCandidates` empty and `usingAutoFill` false. Restored games therefore do not display the saved manual notes.

**Change:** Give displayed notes one authoritative state and include it in undo/redo and persistence. **Verify:** Enter notes, clear, undo, redo, save, terminate, and restore; compare the exact displayed candidates at every step.

### P1 — The board lacks meaningful VoiceOver cell semantics

[GridView.swift:25](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/Views/GridView.swift#L25): cells expose digit text and gestures without an explicit combined accessibility element, coordinates, value/notes, given status, selection, or error semantics. A grid-level test identifier does not supply that information. A player using VoiceOver cannot determine the same board context as a sighted player.

The simulator accessibility tree exposed only the given digits as text with the repeated `SudokuGrid` identifier; it supplied no row/column context or empty-cell entries. It also labeled the keypad erase X “Close,” the hint action “lightbulb,” and hearts “Love.”

**Change:** Expose each cell as one actionable element, with announcements such as “Row 4, column 6, empty, notes 2 and 7, selected.” Give icon controls action labels. **Verify:** Complete selection, digit entry, notes, correction, and hint flows using VoiceOver. Follow [Apple’s accessibility guidance](https://developer.apple.com/design/human-interface-guidelines/accessibility).

### P2 — Mistake-limit controls do not affect gameplay

[SettingsView.swift:24](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/Views/SettingsView.swift#L24) offers an unlimited option and limits from 1–10. [GameViewModel.swift:32](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/ViewModels/GameViewModel.swift#L32) hard-codes three; `isGameOver` always applies it. Users selecting relaxed play still lose at three mistakes.

**Change:** Pass the selected rule into gameplay and the hearts display, defining whether setting changes apply immediately or to the next game. **Verify:** Disabled, one, three, and ten mistakes, including saved-game restoration.

### P2 — Pause and backgrounding corrupt elapsed-time behavior

[GameManager.swift:208](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/Services/GameManager.swift#L208): manual pause/resume changes the screen without calling the view model’s timing methods. Save & Exit also leaves the clock’s start time running. Backgrounding calls `pause()`, but [GameViewModel.swift:58](https://github.com/kcirtapfromspace/sudoku/blob/c371f588aec8a80bb5402d853b6d7414cdb9013c/ios/Sudoku/Sudoku/ViewModels/GameViewModel.swift#L58) then calculates time since the pause began rather than preserving active play time. `resume()` has no callers.

**Change:** Track accumulated active time using one pause/resume lifecycle and stop timing at completion. **Verify:** With a controllable clock, play 30 seconds, pause 60 seconds, resume for 10 seconds; active time should be 40 seconds. Include repeated inactive/background events and Save & Exit.

## Design recommendations

These are design judgments from the code, fresh iPhone gameplay captures, and historical menu/iPad images, separate from the confirmed behavior above.

1. **Make difficulty selection easier to understand.** The current picker exposes an unexplained SE number in its slider and primary Play label, and a repeated tap on a difficulty immediately starts the game. Keep named difficulty and a clear Play action primary; explain Sudoku Explainer ratings and put precision adjustment behind an optional control. Source: `ContentView.swift:225–277`.
2. **Clarify the gameplay toolbar.** The keypad’s X and the toolbar’s delete icon call the same clear action, while “Normal” describes the current mode without saying what tapping does. Use a consistent erase control and explicit “Notes” on/off state; label key actions. Preserve the existing generous keypad buttons. Source: `NumberPadView.swift:19–28`, `GameView.swift:433–438,496–508`.
3. **Improve compact-screen and large-text legibility.** Board digits and pencil marks use fixed proportions of cell size; a 375-point-wide layout gives pencil marks about 9.5 points before any height constraint. There is no board zoom or scaled typography path. Provide a readable selected-cell/notes view or zoom, test accessibility text sizes, and adapt the toolbar instead of squeezing it into one row. The 36-point camera controls should have larger hit areas. See [Apple’s UI design guidance](https://developer.apple.com/design/tips/).
4. **Give iPad layouts a deliberate content width.** The menu uses full-width buttons and multiple spacers without a maximum content width. Historical iPad captures show a stretched button and very large empty regions. Constrain the menu and use available space for useful continuation details such as difficulty and elapsed time. Refresh screenshots before judging exact spacing.

## Recommended coverage work

1. Run `xcodebuild test` with coverage enabled in macOS CI, after generating the required Rust libraries and Swift bindings. Upload `.xcresult` and a readable coverage summary. Keep generated UniFFI bindings and Rust instrumentation distinguishable from handwritten Swift application coverage.
2. Prioritize deterministic behavioral tests for notes/undo/persistence, mistake settings, and pause/resume. These test the gaps identified in this review more directly than increasing constructor-test counts.
3. Add asserted UI flows for new game → entry → notes → pause → restore, rotation, hints, and completion. Give cells and controls stable accessibility identifiers in addition to meaningful accessibility labels.
4. Add visual regression checks for compact iPhone and iPad, portrait/landscape, light/dark, and large text. Capture known fixtures only after asserting the expected screen. Run a manual VoiceOver pass as well.
5. Adopt a repeatable baseline in CI, then set a threshold that prevents regression. Use this local unit-test result as a starting point and report UI-inclusive results separately when those tests are repaired. A high line percentage alone does not verify readability, reachability, or preserved player progress.

## Local execution

Rust simulator/native libraries and UniFFI bindings built successfully. The existing checked-in Xcode project then ran all five `SudokuTests` methods successfully on iPhone 17 Pro / iOS 26.5. App/test source and CI configuration were not changed. The build emitted a non-Sendable capture warning at `PuzzleOCRService.swift:52`.

After generating the engine libraries and bindings, the coverage command was:

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  xcodebuild test \
  -project ios/Sudoku/Sudoku.xcodeproj \
  -scheme Sudoku \
  -destination 'platform=iOS Simulator,id=3DADE1A1-F1AC-4E21-802C-D1F1CF4E191B' \
  -only-testing:SudokuTests \
  -enableCodeCoverage YES \
  -derivedDataPath /tmp/sudoku-ios-review-derived \
  -resultBundlePath /tmp/sudoku-ios-review-unit-coverage.xcresult \
  CODE_SIGNING_ALLOWED=NO

DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  xcrun xccov view --report --json \
  /tmp/sudoku-ios-review-unit-coverage.xcresult
```

The result bundle and build log are temporary local artifacts: [Xcode result bundle](/tmp/sudoku-ios-review-unit-coverage.xcresult), [test/build log](/tmp/sudoku-ios-review-xcodebuild.log). A durable copy of the coverage JSON and fresh gameplay screenshots is saved under `docs/review-artifacts/ios-2026-09-15/`.
