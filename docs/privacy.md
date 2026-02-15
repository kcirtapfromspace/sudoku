# Privacy Policy for Ukodus

**Last updated: February 15, 2026**

## Overview

Ukodus ("the App") is a Sudoku puzzle game developed by Patrick Deutsch. It is available as a web app at [ukodus.com](https://ukodus.com/play/), as an iOS app, and as a terminal (TUI) app. This privacy policy explains what data we collect across all platforms and how we use it.

## Local Data Storage

The following data is stored **locally on your device** and never transmitted:
- Game progress and saved games
- Statistics (games played, win rate, best times)
- App settings and preferences
- Candidate/note annotations

## Telemetry Data Collected

When you complete a game (win or loss), Ukodus submits gameplay data to our server. **This happens automatically on all platforms.** There is currently no opt-out mechanism for telemetry submission.

### Data Fields Submitted

The following fields are sent upon game completion:

| Field | Description | All Platforms |
|---|---|---|
| `puzzle_hash` | DJB2 hash of the puzzle string (identifies the puzzle) | Yes |
| `puzzle_string` | The 81-character initial puzzle layout | Yes |
| `difficulty` | Difficulty tier (Beginner through Extreme) | Yes |
| `se_rating` | Sudoku Explainer numerical rating of the puzzle | Yes |
| `result` | Win or Loss | Yes |
| `time_secs` | Total solve time in seconds (pauses excluded) | Yes |
| `hints_used` | Number of hints the player requested | Yes |
| `mistakes` | Number of incorrect digit placements | Yes |
| `moves_count` | Total number of moves made | Yes |
| `avg_move_time_ms` | Average time between moves in milliseconds | Yes |
| `min_move_time_ms` | Fastest time between two consecutive moves | Yes |
| `move_time_std_dev` | Standard deviation of move timing | Yes |
| `player_id` | A randomly generated UUID (see below) | Yes |
| `short_code` | Compact puzzle identifier for sharing (if available) | Yes |
| `platform` | One of `"web"`, `"ios"`, or `"tui"` | Yes |

### Additional Fields by Platform

| Field | Description | Platforms |
|---|---|---|
| `device_model` | Hardware model identifier (e.g., `iPhone15,2`) | iOS only |
| `os_version` | Operating system version (e.g., `iOS 18.2`) | iOS only |
| `app_version` | Application version string | iOS, TUI |

### Player ID

Each platform generates a random UUID the first time you play. This ID is **not tied to your identity** in any way -- it exists solely to associate your results on the leaderboard and Galaxy visualization.

- **Web (ukodus.com):** Stored in the browser's `localStorage`
- **iOS:** Stored in `UserDefaults`
- **TUI:** Stored in a file in your local data directory

The player ID contains no personal information and cannot be used to identify you. Clearing your browser data, reinstalling the app, or deleting the stored file will generate a new ID.

### Move Timing Data

The `avg_move_time_ms`, `min_move_time_ms`, and `move_time_std_dev` fields are used for **anti-cheat verification**. Abnormal timing patterns (such as sub-human move speeds) may flag a result for review on the leaderboard.

## What We Do NOT Collect

Across all platforms, Ukodus does **not** collect:

- Your name, email address, or any contact information
- Location data (GPS, IP-based geolocation, or otherwise)
- IP addresses (not logged by the application)
- Advertising identifiers (IDFA/GAID)
- Device fingerprints beyond the fields listed above
- Any data for marketing or advertising purposes
- Usage analytics or behavioral tracking outside of game results

## How We Use Your Data

Telemetry data is used exclusively for:

1. **Galaxy visualization** -- an interactive display of all games played across the community
2. **Leaderboards** -- ranking solve times for each puzzle and difficulty tier
3. **Anti-cheat verification** -- detecting automated solvers via move timing analysis
4. **Puzzle analytics** -- understanding solve rates and difficulty calibration

Data is never sold, shared with third parties, or used for advertising.

## Optional Services

### Apple Game Center (iOS)

If you choose to sign in to Game Center:
- Your scores may appear on leaderboards
- Achievements are synced with your Game Center account
- This data is managed by Apple according to their privacy policy

### Apple iCloud (iOS)

If you enable iCloud sync:
- Game progress syncs across your Apple devices
- Data is stored in your personal iCloud account
- This data is managed by Apple according to their privacy policy

## Third-Party Services

Beyond Apple Game Center and iCloud (both opt-in on iOS), the App does not integrate with any third-party analytics, advertising, or tracking services.

## Children's Privacy

Ukodus does not knowingly collect personal information from anyone, including children under 13 years of age. The telemetry data described above contains no personally identifiable information.

## Data Security

Local data is protected by your device's security measures. Telemetry data is transmitted over HTTPS. Server-side data consists only of the anonymous gameplay fields described above.

## Opt-Out

There is currently no in-app mechanism to disable telemetry submission. If this is a concern, you can block network requests to the API endpoint at the network level (e.g., via a firewall or DNS filter). We are considering adding an opt-out toggle in a future release.

## Data Deletion

You can delete your local data at any time by:
- **Web:** Clearing your browser's localStorage for ukodus.com
- **iOS:** Deleting the App from your device, or using the "Reset Statistics" option in Settings
- **TUI:** Deleting the stats and player ID files from your local data directory

Server-side telemetry data is anonymous and not linked to a recoverable identity. If you would like your data removed, contact us with your player ID (found in your platform's storage) and we will delete all associated records.

## Changes to This Policy

We may update this privacy policy from time to time. Any changes will be reflected with a new "Last updated" date.

## Contact

If you have questions about this privacy policy, please contact:

**Patrick Deutsch**
GitHub: [@kcirtapfromspace](https://github.com/kcirtapfromspace)
