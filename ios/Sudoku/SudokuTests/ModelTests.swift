import XCTest
@testable import Sudoku

final class ModelTests: XCTestCase {
    func testDifficultyMetadataMatchesProgressionAndEngine() {
        let ranges: [ClosedRange<Float>] = [1.5...2, 2...2.5, 2.5...3.4, 3.4...3.8, 3.8...4.5, 4.5...5.5, 5.5...7, 7...11]
        for (index, difficulty) in Difficulty.allCases.enumerated() {
            XCTAssertEqual(difficulty.id, difficulty.displayName)
            XCTAssertEqual(difficulty.seRange, ranges[index])
            XCTAssertEqual(difficulty.defaultSE, (ranges[index].lowerBound + ranges[index].upperBound) / 2)
            XCTAssertFalse(difficulty.seDescription.isEmpty)
            XCTAssertEqual(Difficulty.from(difficulty.toGameDifficulty()), difficulty)
            XCTAssertEqual(difficulty.requiresUnlock, index >= 6)
        }
        XCTAssertEqual(Difficulty.alwaysUnlocked.count, 6)
        XCTAssertNil(Difficulty.easy.unlockRequirement)
        XCTAssertEqual(Difficulty.master.unlockRequirement?.difficulty, .expert)
        XCTAssertEqual(Difficulty.extreme.unlockRequirement?.wins, 50)
    }

    func testStatisticsTrackBestTimesStreaksAndUnlocks() throws {
        var statistics = GameStatistics()
        XCTAssertEqual(statistics.winRate, 0)
        XCTAssertEqual(statistics.availableDifficulties, Difficulty.alwaysUnlocked)
        XCTAssertEqual(statistics.unlockProgress(for: .easy), 1)
        XCTAssertEqual(statistics.unlockProgress(for: .master), 0)
        XCTAssertFalse(statistics.isUnlocked(.master))
        for _ in 0..<50 { statistics.recordWin(difficulty: .expert, time: 120) }
        XCTAssertTrue(statistics.isUnlocked(.master))
        XCTAssertEqual(statistics.unlockProgress(for: .master), 1)
        XCTAssertEqual(statistics.currentStreak, 50)
        statistics.recordWin(difficulty: .expert, time: 80)
        statistics.recordWin(difficulty: .expert, time: 100)
        XCTAssertEqual(statistics.bestTimes[.expert], 80)
        XCTAssertEqual(statistics.wins(for: .expert), 52)
        statistics.recordLoss(time: 60)
        XCTAssertEqual(statistics.currentStreak, 0)
        XCTAssertEqual(statistics.bestStreak, 52)
        XCTAssertEqual(statistics.gamesPlayed, 53)
        XCTAssertEqual(statistics.winRate, 52.0 / 53.0)
        statistics.recordSequentialCompletion()
        XCTAssertEqual(statistics.sequentialCompletions, 1)
        statistics.activateEasterEgg()
        XCTAssertEqual(statistics.availableDifficulties, Difficulty.allCases)
        let restored = try JSONDecoder().decode(GameStatistics.self, from: JSONEncoder().encode(statistics))
        XCTAssertEqual(restored.bestTimes, statistics.bestTimes)
        XCTAssertEqual(restored.totalPlayTime, 6240)
    }

    func testCellsAndInputModesHaveStableIdentityAndState() {
        var cell = CellModel.empty(row: 4, col: 7)
        XCTAssertEqual(cell.id, "4-7")
        XCTAssertEqual(cell.position.row, 4)
        XCTAssertEqual(cell.position.col, 7)
        XCTAssertEqual(cell.boxIndex, 5)
        XCTAssertTrue(cell.isEmpty)
        cell.value = 8
        XCTAssertFalse(cell.isEmpty)
        for mode in [InputMode.normal, .candidate, .temporaryCandidate] {
            XCTAssertFalse(mode.displayName.isEmpty)
            XCTAssertEqual(mode.isNotesMode, mode != .normal)
            var toggled = mode
            toggled.toggle()
            XCTAssertEqual(toggled, mode == .normal ? .candidate : .normal)
        }
    }

    func testSettingsRoundTripPreservesEveryOption() throws {
        var settings = GameSettings()
        XCTAssertEqual(settings.mistakeLimit, 3)
        XCTAssertTrue(settings.mistakeLimitEnabled)
        XCTAssertFalse(settings.autoFillCandidates)
        for theme in GameSettings.ThemeSetting.allCases {
            settings.theme = theme
            settings.mistakeLimitEnabled = false
            settings.mistakeLimit = 9
            settings.cameraImportEnabled = true
            settings.autoFillCandidates = true
            let restored = try JSONDecoder().decode(GameSettings.self, from: JSONEncoder().encode(settings))
            XCTAssertEqual(restored.theme, theme)
            XCTAssertEqual(restored.mistakeLimit, 9)
            XCTAssertFalse(restored.mistakeLimitEnabled)
            XCTAssertTrue(restored.cameraImportEnabled)
            XCTAssertTrue(restored.autoFillCandidates)
        }
    }

    func testPuzzleRecordCountsStartsSeparatelyFromOutcomesAndHashesIdentity() throws {
        let start = Date(timeIntervalSince1970: 10)
        var record = PuzzleRecord(puzzleString: "abc", difficulty: .easy, now: start)
        XCTAssertEqual(record.id, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        XCTAssertEqual(record.firstPlayedAt, start)
        XCTAssertFalse(record.hasBeenSolved)
        record.recordResult(won: false, time: nil, now: start.addingTimeInterval(10))
        XCTAssertEqual(record.losses, 1)
        XCTAssertEqual(record.playCount, 1)
        record.recordResult(won: true, time: nil)
        XCTAssertTrue(record.hasBeenSolved)
        XCTAssertNil(record.bestTime)
        record.recordResult(won: true, time: 60)
        record.recordResult(won: true, time: 90)
        record.recordResult(won: true, time: 30)
        XCTAssertEqual(record.bestTime, 30)
        XCTAssertEqual(record.wins, 4)
        let restored = try JSONDecoder().decode(PuzzleRecord.self, from: JSONEncoder().encode(record))
        XCTAssertEqual(restored.puzzleHash, record.puzzleHash)
        XCTAssertEqual(restored.bestTime, 30)
        var library = PuzzleLibraryStats()
        XCTAssertEqual(library.completionRate, 0)
        library.totalPuzzles = 4
        library.solvedPuzzles = 3
        XCTAssertEqual(library.completionRate, 0.75)
    }

    func testKonamiSequenceRequiresContiguousCompleteInputsAndCanReset() {
        let detector = KonamiCodeDetector()
        detector.input(.up)
        XCTAssertEqual(detector.progress, 1)
        detector.input(.left)
        XCTAssertFalse(detector.isActivated)
        for _ in 0..<12 { detector.input(.a) }
        for input in KonamiCodeDetector.sequence { detector.input(input) }
        XCTAssertTrue(detector.isActivated)
        XCTAssertEqual(detector.progress, 0)
        detector.reset()
        XCTAssertFalse(detector.isActivated)
        XCTAssertEqual(detector.progress, 0)
        for input in KonamiCodeDetector.sequence { detector.input(input) }
        XCTAssertTrue(detector.isActivated)
    }
}
