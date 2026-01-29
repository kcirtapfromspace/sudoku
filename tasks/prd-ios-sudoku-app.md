# PRD: iOS Sudoku App

## Introduction

A native iOS Sudoku app powered by the existing Rust `sudoku-core` engine via UniFFI bindings. The app brings full-featured Sudoku gameplay to iPhone and iPad with a polished native iOS design, Apple ecosystem integrations (Game Center, iCloud, widgets), and multiple input methods optimized for touch.

## Goals

- Deliver a polished, native iOS Sudoku experience on iPhone and iPad
- Achieve full feature parity with the TUI version (gameplay, stats, leaderboards, themes, animations)
- Integrate with Apple services: Game Center achievements/leaderboards, iCloud sync, Home Screen widgets
- Support multiple input methods: tap, drag-and-drop, and external keyboard
- Leverage the existing Rust core engine via UniFFI Swift bindings
- Support iOS 16+ for broad device compatibility

## User Stories

### US-001: Generate Swift bindings from Rust core
**Description:** As a developer, I need Swift bindings for sudoku-core so the iOS app can use the existing game logic.

**Acceptance Criteria:**
- [ ] UniFFI generates `SudokuEngine.swift` from `sudoku-ffi` crate
- [ ] Bindings compile without errors in Xcode
- [ ] Can create a new puzzle and solve it from Swift
- [ ] XCTest validates core functionality works through bindings

---

### US-002: Create Xcode project structure
**Description:** As a developer, I need a properly structured Xcode project to build the iOS app.

**Acceptance Criteria:**
- [ ] Xcode project created at `ios/Sudoku/` in workspace
- [ ] Universal app target (iPhone + iPad)
- [ ] Minimum deployment target iOS 16.0
- [ ] Rust library linked as XCFramework
- [ ] Project builds and runs on simulator

---

### US-003: Display Sudoku grid with native UI
**Description:** As a user, I want to see the Sudoku puzzle displayed clearly so I can play the game.

**Acceptance Criteria:**
- [ ] 9x9 grid rendered using SwiftUI
- [ ] Given numbers displayed in bold/distinct style
- [ ] Player-entered numbers visually differentiated
- [ ] 3x3 box boundaries clearly visible
- [ ] Grid scales appropriately on iPhone and iPad
- [ ] Supports light and dark mode
- [ ] Verify on device/simulator

---

### US-004: Select cells by tapping
**Description:** As a user, I want to tap a cell to select it so I can enter a number.

**Acceptance Criteria:**
- [ ] Tapping empty cell selects it with visual highlight
- [ ] Tapping given cell shows it's not editable (subtle feedback)
- [ ] Selected cell clearly highlighted
- [ ] Related cells (same row/col/box) subtly highlighted
- [ ] Cells with same number highlighted
- [ ] Haptic feedback on selection (light tap)
- [ ] Verify on device

---

### US-005: Enter numbers via on-screen number pad
**Description:** As a user, I want a number pad to enter values so I can play without a keyboard.

**Acceptance Criteria:**
- [ ] Number pad (1-9) displayed below or beside grid
- [ ] Tapping number enters it in selected cell
- [ ] Clear/delete button to remove value
- [ ] Numbers that are complete (9 placed) shown as disabled/dimmed
- [ ] Haptic feedback on number entry
- [ ] Pad repositions appropriately in landscape
- [ ] Verify on device

---

### US-006: Toggle notes/candidates mode
**Description:** As a user, I want to enter pencil marks so I can track possible values.

**Acceptance Criteria:**
- [ ] Toggle button switches between normal and notes mode
- [ ] In notes mode, tapping number toggles it as candidate
- [ ] Candidates displayed as small numbers in cell (3x3 grid)
- [ ] Visual indicator shows current mode
- [ ] Long-press on number pad enters as note (alternative)
- [ ] Verify on device

---

### US-007: Drag numbers onto cells
**Description:** As a user, I want to drag numbers from the pad to cells for a tactile experience.

**Acceptance Criteria:**
- [ ] Can drag number from pad onto any empty cell
- [ ] Drop preview shows where number will land
- [ ] Invalid drops (given cells) show rejection feedback
- [ ] Drag works for both values and notes
- [ ] Haptic feedback on successful drop
- [ ] Verify on device

---

### US-008: Support external keyboard input
**Description:** As an iPad user with keyboard, I want to use arrow keys and numbers for faster input.

**Acceptance Criteria:**
- [ ] Arrow keys navigate between cells
- [ ] Number keys (1-9) enter values
- [ ] Delete/Backspace clears cell
- [ ] Shift+number toggles note
- [ ] H key requests hint
- [ ] U key undoes last action
- [ ] Cmd+Z also undoes
- [ ] Verify on iPad with keyboard

---

### US-009: Implement undo/redo
**Description:** As a user, I want to undo mistakes so I can correct errors easily.

**Acceptance Criteria:**
- [ ] Undo button reverts last action
- [ ] Redo button restores undone action
- [ ] Unlimited undo history for current game
- [ ] Buttons disabled when stack is empty
- [ ] Swipe gesture (left) for undo (optional)
- [ ] Verify on device

---

### US-010: Provide hints
**Description:** As a user, I want hints when stuck so I can learn and progress.

**Acceptance Criteria:**
- [ ] Hint button available during gameplay
- [ ] Hint highlights a cell and shows the technique
- [ ] "Apply hint" fills in the value or eliminates candidates
- [ ] Hints used counter displayed
- [ ] Confirmation before using hint (optional setting)
- [ ] Verify on device

---

### US-011: Start new game with difficulty selection
**Description:** As a user, I want to choose difficulty when starting a new game.

**Acceptance Criteria:**
- [ ] New game button shows difficulty picker
- [ ] Difficulties: Beginner, Easy, Medium, Intermediate, Hard, Expert
- [ ] Selected difficulty generates appropriate puzzle
- [ ] Confirm before abandoning in-progress game
- [ ] Last used difficulty remembered
- [ ] Verify on device

---

### US-012: Display game timer
**Description:** As a user, I want to see elapsed time so I can track my performance.

**Acceptance Criteria:**
- [ ] Timer displayed during gameplay (MM:SS format)
- [ ] Timer pauses when app backgrounds
- [ ] Timer pauses on pause screen
- [ ] Timer stops on completion
- [ ] Option to hide timer in settings
- [ ] Verify on device

---

### US-013: Track and limit mistakes
**Description:** As a user, I want feedback on mistakes so I can learn correct solving.

**Acceptance Criteria:**
- [ ] Incorrect entries highlighted in red
- [ ] Mistake counter displayed (e.g., 3 hearts)
- [ ] Game over after 3 mistakes (configurable)
- [ ] Option to disable mistake limit in settings
- [ ] Verify on device

---

### US-014: Display win screen with animations
**Description:** As a user, I want a celebratory win screen so completing puzzles feels rewarding.

**Acceptance Criteria:**
- [ ] Win screen appears on puzzle completion
- [ ] Shows completion time, hints used, mistakes
- [ ] Confetti/particle animation plays
- [ ] Options: New Game, Share, View Leaderboard
- [ ] Haptic success feedback
- [ ] Verify on device

---

### US-015: Display game over screen
**Description:** As a user, I want clear feedback when I lose so I understand what happened.

**Acceptance Criteria:**
- [ ] Game over screen on 3rd mistake
- [ ] Shows "Too many mistakes" message
- [ ] Options: Try Again (same puzzle), New Game
- [ ] Somber animation (subtle, not punishing)
- [ ] Verify on device

---

### US-016: Implement pause functionality
**Description:** As a user, I want to pause the game so I can take breaks without the timer running.

**Acceptance Criteria:**
- [ ] Pause button stops timer
- [ ] Puzzle grid hidden while paused (no cheating)
- [ ] Resume button continues game
- [ ] Auto-pause when app enters background
- [ ] Verify on device

---

### US-017: Auto-save game progress
**Description:** As a user, I want my game saved automatically so I don't lose progress.

**Acceptance Criteria:**
- [ ] Game state saved after each move
- [ ] Saved to local storage (UserDefaults/files)
- [ ] Game restored on app launch
- [ ] Multiple save slots (current + last completed)
- [ ] Verify by force-quitting and relaunching

---

### US-018: Implement ghost hints feature
**Description:** As a user, I want to optionally see valid candidates as faded numbers for learning.

**Acceptance Criteria:**
- [ ] Toggle in settings to enable ghost hints
- [ ] Empty cells show valid candidates faded
- [ ] Only shows when user hasn't entered notes
- [ ] Clearly differentiated from user-entered notes
- [ ] Verify on device

---

### US-019: Highlight valid cells (naked singles)
**Description:** As a user, I want to optionally highlight cells with only one valid number.

**Acceptance Criteria:**
- [ ] Toggle in settings to enable
- [ ] Cells with single valid candidate have subtle highlight
- [ ] Helps learning without giving away the answer
- [ ] Verify on device

---

### US-020: Support multiple color themes
**Description:** As a user, I want theme options so the app matches my preferences.

**Acceptance Criteria:**
- [ ] Theme options: System (auto), Light, Dark, High Contrast
- [ ] All UI elements respect selected theme
- [ ] Theme persists across launches
- [ ] Accessible colors in high contrast mode
- [ ] Verify on device in all themes

---

### US-021: Track player statistics
**Description:** As a user, I want to see my playing statistics so I can track improvement.

**Acceptance Criteria:**
- [ ] Stats screen shows: games played, games won, win rate
- [ ] Best times per difficulty level
- [ ] Average completion time per difficulty
- [ ] Current and best win streak
- [ ] Stats persist across app updates
- [ ] Verify stats update after completing games

---

### US-022: Implement local leaderboard
**Description:** As a user, I want to see my best times so I can compete with myself.

**Acceptance Criteria:**
- [ ] Leaderboard shows top 10 times per difficulty
- [ ] Entries show: rank, time, date, hints used
- [ ] Current game highlighted if it ranks
- [ ] Clear leaderboard option in settings
- [ ] Verify on device

---

### US-023: Integrate Game Center achievements
**Description:** As a user, I want to earn achievements so I have goals to work toward.

**Acceptance Criteria:**
- [ ] Game Center authentication on launch
- [ ] Achievements for: first win, win each difficulty, streak milestones, no-hint wins
- [ ] Achievement unlocked notification displays
- [ ] Can view achievements from settings
- [ ] Graceful handling when Game Center unavailable
- [ ] Verify achievements unlock correctly

---

### US-024: Integrate Game Center leaderboards
**Description:** As a user, I want to compete globally so I can compare with other players.

**Acceptance Criteria:**
- [ ] Game Center leaderboard per difficulty (6 total)
- [ ] Score = completion time (lower is better)
- [ ] Submit score on puzzle completion
- [ ] View leaderboards from app
- [ ] Friend leaderboard filter
- [ ] Verify scores submit and appear

---

### US-025: Sync progress via iCloud
**Description:** As a user, I want my progress synced across devices so I can play anywhere.

**Acceptance Criteria:**
- [ ] Statistics sync via iCloud Key-Value Store
- [ ] Current game syncs via iCloud Documents
- [ ] Handles merge conflicts gracefully
- [ ] Works when iCloud unavailable (local fallback)
- [ ] Sync indicator in settings
- [ ] Verify sync between two devices

---

### US-026: Create Home Screen widget
**Description:** As a user, I want a widget showing my stats or quick-start so the app is accessible.

**Acceptance Criteria:**
- [ ] Small widget: shows current streak and quick-start button
- [ ] Medium widget: shows today's stats and continue game
- [ ] Widget updates after each game completion
- [ ] Tapping widget opens app to appropriate screen
- [ ] Supports light/dark mode
- [ ] Verify widget displays and updates

---

### US-027: Implement haptic feedback throughout
**Description:** As a user, I want tactile feedback so the app feels responsive and polished.

**Acceptance Criteria:**
- [ ] Light tap on cell selection
- [ ] Medium tap on number entry
- [ ] Success haptic on correct completion
- [ ] Error haptic on mistake
- [ ] Option to disable haptics in settings
- [ ] Verify haptics feel appropriate

---

### US-028: Create settings screen
**Description:** As a user, I want to customize the app behavior to my preferences.

**Acceptance Criteria:**
- [ ] Settings accessible from main menu
- [ ] Options: theme, haptics, timer visibility, mistake limit, ghost hints, valid cells
- [ ] Game Center and iCloud status shown
- [ ] Reset statistics option (with confirmation)
- [ ] About section with version info
- [ ] Verify all settings persist and apply

---

### US-029: Implement iPad-optimized layout
**Description:** As an iPad user, I want a layout that uses the larger screen effectively.

**Acceptance Criteria:**
- [ ] Grid and number pad side-by-side in landscape
- [ ] Stats/controls in sidebar
- [ ] Larger touch targets appropriate for iPad
- [ ] Supports Split View and Slide Over
- [ ] Keyboard shortcuts work
- [ ] Verify on iPad simulator and device

---

### US-030: Add accessibility support
**Description:** As a user with accessibility needs, I want the app to work with assistive technologies.

**Acceptance Criteria:**
- [ ] Full VoiceOver support for all controls
- [ ] Cells announce position and value
- [ ] Dynamic Type support for text elements
- [ ] Sufficient color contrast (WCAG AA)
- [ ] Reduce Motion respected for animations
- [ ] Verify with VoiceOver enabled

---

## Functional Requirements

- **FR-1:** App uses `sudoku-core` Rust engine via UniFFI Swift bindings for all game logic
- **FR-2:** Grid displays 9x9 cells with clear 3x3 box boundaries
- **FR-3:** Given cells are non-editable and visually distinct
- **FR-4:** Tapping a cell selects it; related cells highlight
- **FR-5:** Number pad (1-9 + delete) allows value entry in selected cell
- **FR-6:** Notes mode toggles candidate entry (small numbers in cells)
- **FR-7:** Drag-and-drop from number pad to cells is supported
- **FR-8:** External keyboard navigation and input works on iPad
- **FR-9:** Unlimited undo/redo within a game session
- **FR-10:** Hints use solver to provide next logical step
- **FR-11:** Timer tracks elapsed time, pauses appropriately
- **FR-12:** Mistakes are tracked; game over after limit (configurable)
- **FR-13:** Win screen shows stats and celebration animation
- **FR-14:** Game auto-saves after each move to local storage
- **FR-15:** Statistics persist locally and sync via iCloud
- **FR-16:** Game Center achievements and leaderboards integrate
- **FR-17:** Home Screen widgets show stats and quick actions
- **FR-18:** All screens support light, dark, and high contrast themes
- **FR-19:** Haptic feedback enhances interactions
- **FR-20:** VoiceOver and Dynamic Type are fully supported

## Non-Goals

- No online multiplayer or real-time competitive play
- No daily challenges or puzzle-of-the-day (future consideration)
- No puzzle editor or custom puzzle import
- No Killer Sudoku or variant rules in v1 (core supports it, but iOS v1 is classic only)
- No Apple Watch app
- No macOS Catalyst build in v1
- No in-app purchases or ads
- No social features beyond Game Center

## Design Considerations

- **Design Language:** Native iOS using SwiftUI, following Human Interface Guidelines
- **Typography:** San Francisco system font; monospace only for numbers in grid
- **Colors:** System colors for automatic light/dark support; custom palette for game elements
- **Grid:** Clean, minimal lines; subtle shadows for depth; generous touch targets (44pt minimum)
- **Animations:** Subtle, purposeful; respect Reduce Motion setting
- **iPad:** Utilize space with sidebar layout; avoid stretched iPhone UI

## Technical Considerations

- **Architecture:** SwiftUI + MVVM; Rust core via UniFFI XCFramework
- **Minimum iOS:** 16.0 (SwiftUI improvements, WidgetKit enhancements)
- **Dependencies:** Minimal - primarily system frameworks + Rust bindings
- **Data Storage:** UserDefaults for settings, file-based for game state, iCloud for sync
- **Build:** XCFramework built via `cargo-xcode` or manual `lipo` for simulator + device
- **Testing:** XCTest for unit tests, XCUITest for UI automation
- **CI:** GitHub Actions or Xcode Cloud for builds and TestFlight

## Success Metrics

- App Store rating of 4.5+ stars
- Crash-free rate > 99.5%
- Average session length > 5 minutes
- Day 7 retention > 40%
- Game Center leaderboard participation > 20% of players
- Widget adoption > 15% of users

## Open Questions

1. Should we include a tutorial for new Sudoku players?
2. Should difficulty auto-adjust based on player performance?
3. Should we support SharePlay for cooperative solving?
4. What App Store pricing model? (Free, Paid, Freemium?)
5. Should we include colorblind-specific themes?
