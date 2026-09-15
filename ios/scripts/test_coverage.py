"""Regression tests for production coverage accounting and mapping safeguards."""

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import coverage


class CoverageTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name).resolve()
        self.path = self.root / "Game.swift"
        self.path.write_text("func play() {\n    print(1)\n}\n")
        self.declarations = {self.path: [coverage.Declaration("play()", "func_decl", 1, 1, 3)]}
        self.archive = {str(self.path): [
            {"line": line, "isExecutable": True, "executionCount": 1} for line in (1, 2, 3)]}
        self.function = {"name": "play()", "lineNumber": 1, "executionCount": 1,
                         "executableLines": 3}
        self.report = {"targets": [{"name": "Sudoku.app", "files": [
            {"path": str(self.path), "functions": [self.function]}]}]}

    def collect(self, **kwargs):
        return coverage.collect(self.root, self.archive, self.report, self.declarations,
                                critical_files=kwargs.pop("critical_files", ()), **kwargs)

    def test_all_authored_swift_sources_parse_before_app_build(self):
        # test-coverage.sh runs this suite before building Rust or launching
        # Xcode tests. Exercise the exact full source inventory on the runner's
        # compiler so toolchain incompatibilities fail during this preflight.
        source_root = Path(__file__).resolve().parents[2] / coverage.APP_SOURCE
        paths = coverage.inventory(source_root)
        self.assertTrue(paths)
        for path in paths:
            with self.subTest(source=str(path.relative_to(source_root))):
                declarations = coverage.source_declarations(path)
                if not declarations:
                    coverage.has_initializers_or_macros(path)
                source_lines = len(path.read_text().splitlines())
                for declaration in declarations:
                    self.assertLessEqual(declaration.end, source_lines)

    def test_complete_coverage_passes(self):
        self.assertTrue(self.collect()["passed"])

    def test_duplicate_line_records_are_unioned_not_summed(self):
        self.archive[str(self.path)] += [
            {"line": 1, "isExecutable": True, "executionCount": 0},
            {"line": 2, "isExecutable": True, "executionCount": 8}]
        result = self.collect()
        self.assertEqual(result["totals"]["lines"], coverage.metric(3, 3))

    def test_duplicate_function_records_are_unioned(self):
        duplicate = dict(self.function, executionCount=0)
        self.report["targets"][0]["files"][0]["functions"].append(duplicate)
        self.assertEqual(self.collect()["totals"]["functions"], coverage.metric(1, 1))

    def test_zero_hit_lines_remain_in_denominator(self):
        self.archive[str(self.path)][1]["executionCount"] = 0
        result = self.collect()
        self.assertFalse(result["passed"])
        self.assertEqual(result["totals"]["lines"], coverage.metric(2, 3))
        self.assertEqual(result["files"][0]["uncovered_lines"], [2])

    def test_uncovered_function_fails_independently_of_lines(self):
        self.function["executionCount"] = 0
        self.assertIn("Production functions", " ".join(self.collect()["failures"]))

    def test_missing_source_file_fails(self):
        self.archive.clear()
        self.assertIn("Missing authored file mapping", " ".join(self.collect()["failures"]))

    def test_new_unreported_source_file_fails(self):
        added = self.root / "Uncalled.swift"
        added.write_text("func missing() {}\n")
        self.declarations[added] = [coverage.Declaration("missing()", "func_decl", 1, 1, 1)]
        self.assertIn("Uncalled.swift", " ".join(self.collect()["failures"]))

    def test_file_of_initializers_cannot_evade_missing_file_guard(self):
        added = self.root / "Defaults.swift"
        added.write_text("struct Defaults { let retries = 3 }")
        self.declarations[added] = []
        self.assertIn("Defaults.swift", " ".join(self.collect()["failures"]))

    def test_missing_function_mapping_fails_despite_covered_parent_lines(self):
        self.declarations[self.path].append(coverage.Declaration("nested()", "func_decl", 2, 2, 2))
        self.assertIn("nested()", " ".join(self.collect()["failures"]))

    def test_closure_cannot_substitute_for_missing_function(self):
        self.function["name"] = "closure #1 in play()"
        self.assertIn("Missing authored function mapping", " ".join(self.collect()["failures"]))

    def test_same_line_function_cannot_substitute_for_missing_function(self):
        self.declarations[self.path].append(coverage.Declaration("missing()", "func_decl", 1, 1, 1))
        self.assertIn("missing()", " ".join(self.collect()["mapping_errors"]))

    def test_same_line_getter_cannot_substitute_for_missing_setter(self):
        self.declarations[self.path] = [coverage.Declaration("value", "accessor_decl", 1, 1, 3, "get"),
                                        coverage.Declaration("value", "accessor_decl", 1, 1, 3, "set")]
        self.function["name"] = "Game.value.getter"
        errors = self.collect()["mapping_errors"]
        self.assertEqual(len(errors), 1)
        self.assertIn("value", errors[0])

    def test_demangled_function_names_match_compiler_declarations(self):
        cases = [("static Game.play()", "play()", "func_decl"),
                 ("nested #2 (_:) in Game.play()", "nested(_:)", "func_decl"),
                 ("Game.generic<A>(_:)", "generic(_:)", "func_decl"),
                 ("Game.__allocating_init(difficulty:)", "init(difficulty:)", "constructor_decl"),
                 ("Game.deinit", "deinit", "destructor_decl")]
        for name, declared, kind in cases:
            with self.subTest(name=name):
                self.assertTrue(coverage.matches_declaration(name, coverage.Declaration(declared, kind, 1, 1, 1)))

    def test_unknown_accessor_format_fails(self):
        ast = '(accessor_decl range=[<stdin>:1:1 - line:1:30] <anonymous> unknown for="value")'
        with self.assertRaisesRegex(coverage.CoverageError, "Unknown Swift accessor"):
            coverage.declarations_from_ast(ast)

    def test_function_requires_archive_mapping(self):
        self.archive[str(self.path)][0] = {"line": 1, "isExecutable": False}
        self.assertIn("no archive line mapping", " ".join(self.collect()["failures"]))

    def test_excludes_only_generated_directory(self):
        generated = self.root / "Generated"
        generated.mkdir()
        (generated / "Engine.swift").write_text("func binding() {}")
        authored = self.root / "GeneratedHelper.swift"
        authored.write_text("func helper() {}")
        self.assertEqual(coverage.inventory(self.root), [self.path, authored])

    def test_nested_directory_named_generated_remains_authored(self):
        nested = self.root / "Views" / "Generated"
        nested.mkdir(parents=True)
        authored = nested / "Screen.swift"
        authored.write_text("func render() {}")
        self.assertIn(authored, coverage.inventory(self.root))

    def test_declaration_only_file_remains_in_inventory(self):
        types = self.root / "Types.swift"
        types.write_text("struct Empty {}")
        self.declarations[types] = []
        result = self.collect()
        self.assertTrue(result["passed"])
        self.assertEqual(len(result["files"]), 2)

    def test_test_target_cannot_supply_app_mapping(self):
        self.report["targets"][0]["name"] = "SudokuTests.xctest"
        with self.assertRaisesRegex(coverage.CoverageError, "No coverage report"):
            self.collect()

    def test_rejects_negative_and_boolean_counts(self):
        for value in (-1, True, "1", None):
            with self.subTest(value=value):
                self.archive[str(self.path)][0]["executionCount"] = value
                with self.assertRaises(coverage.CoverageError):
                    self.collect()

    def test_rejects_missing_execution_count(self):
        del self.archive[str(self.path)][0]["executionCount"]
        with self.assertRaises(coverage.CoverageError):
            self.collect()

    def test_rejects_out_of_bounds_stale_source_lines(self):
        self.archive[str(self.path)][0]["line"] = 100
        with self.assertRaisesRegex(coverage.CoverageError, "Stale coverage"):
            self.collect()

    def test_manifest_detects_equal_length_source_changes(self):
        before = coverage.source_manifest(self.root)
        self.path.write_text(self.path.read_text().replace("print(1)", "print(2)"))
        self.assertNotEqual(before, coverage.source_manifest(self.root))

    def test_run_manifest_detects_tests_removed_during_execution(self):
        source = self.root / "Application"
        source.mkdir()
        (source / "Game.swift").write_text("func game() {}")
        tests = self.root / "SudokuTests"
        tests.mkdir()
        test = tests / "GameplayTests.swift"
        test.write_text("import XCTest\nfunc testGame() {}")
        before = coverage.run_manifest(source)
        test.unlink()
        self.assertNotEqual(before, coverage.run_manifest(source))
        self.assertEqual(before["authored_sources"], coverage.run_manifest(source)["authored_sources"])

    def test_critical_file_requires_all_lines_and_functions(self):
        self.archive[str(self.path)][1]["executionCount"] = 0
        result = self.collect(critical_files=("Game.swift",), minimum_lines=50)
        self.assertIn("Critical gameplay Game.swift lines", " ".join(result["failures"]))

    def test_missing_critical_file_fails(self):
        result = self.collect(critical_files=("Missing.swift",))
        self.assertIn("Missing critical gameplay file", " ".join(result["failures"]))

    def test_report_function_totals_do_not_inflate_line_denominator(self):
        self.function["executableLines"] = 9000
        self.assertEqual(self.collect()["totals"]["lines"]["total"], 3)

    def test_unknown_ast_declaration_format_fails(self):
        with self.assertRaises(coverage.CoverageError):
            coverage.declarations_from_ast('(func_decl missing_source_location "broken()")')

    def test_protocol_requirements_have_no_body(self):
        ast = '''(source_file "<stdin>"
  (func_decl range=[<stdin>:1:1 - line:1:15] "required()"
    (parameter_list range=[<stdin>:1:14 - line:1:15])))'''
        self.assertEqual(coverage.declarations_from_ast(ast), [])

    def test_accessor_and_nested_body_inventory(self):
        ast = '''(source_file "<stdin>"
  (accessor_decl range=[<stdin>:1:10 - line:4:1] <anonymous> get for="value"
    (brace_stmt range=[<stdin>:1:10 - line:4:1]
      (func_decl range=[<stdin>:2:1 - line:2:18] "nested()"
        (brace_stmt range=[<stdin>:2:15 - line:2:18])))))'''
        declared = coverage.declarations_from_ast(ast)
        self.assertEqual([decl.name for decl in declared], ["value", "nested()"])

    def test_actual_swift_parser_handles_nested_comments_raw_strings_and_debug(self):
        if subprocess.run(["/usr/bin/which", "xcrun"], capture_output=True).returncode:
            self.skipTest("Swift parser integration requires Xcode")
        self.path.write_text('''/* outer /* func fake() {} */ comment */
let literal = #"func fake2() {}"#
#if DEBUG
func actual<T>(_ value: [T]) -> Int {
    func nested() -> Int { 1 }
    return nested()
}
#endif
var count: Int { 42 }
''')
        declared = coverage.source_declarations(self.path)
        self.assertEqual([decl.name for decl in declared], ["actual(_:)", "nested()", "count"])

    def test_parser_inventories_async_default_closures_with_types_in_another_file(self):
        if subprocess.run(["/usr/bin/which", "xcrun"], capture_output=True).returncode:
            self.skipTest("Swift parser integration requires Xcode")
        (self.root / "Models.swift").write_text("enum Difficulty {}\nstruct SudokuGame {}\n")
        self.path.write_text('''actor PuzzleCache {
    private let fetch: @Sendable (Difficulty) async -> SudokuGame?
    private let generate: @Sendable (Difficulty) async -> SudokuGame
    init(fetch: @escaping @Sendable (Difficulty) async -> SudokuGame? = {
        func nested() -> SudokuGame? { nil }
        return nested()
    }, generate: @escaping @Sendable (Difficulty) async -> SudokuGame = { difficulty in
        await Task.detached { SudokuGame.newClassic(difficulty: difficulty) }.value
    }) {}
    func afterDefaultClosure() {}
}
''')
        self.assertEqual([decl.name for decl in coverage.source_declarations(self.path)],
                         ["init(fetch:generate:)", "nested()", "afterDefaultClosure()"])

    def test_parser_still_rejects_invalid_swift_syntax(self):
        self.path.write_text("func broken( {\n")
        with self.assertRaisesRegex(coverage.CoverageError, "Could not parse"):
            coverage.source_declarations(self.path)

    def test_known_dumper_type_errors_require_independent_syntax_validation(self):
        ast = '''(source_file "<stdin>"
  (func_decl range=[<stdin>:1:1 - line:3:1] "play()"
    (brace_stmt range=[<stdin>:1:13 - line:3:1]))
  (struct_decl range=[<stdin>:5:1 - line:5:29] "__CoverageParserEOF"))'''
        error = "<stdin>:1:1: error: cannot find type 'Difficulty' in scope\n"
        error += "<stdin>:1:2: error: @escaping attribute only applies to function types\n"
        for syntax_exit in (0, 1):
            with self.subTest(syntax_exit=syntax_exit), patch.object(coverage.subprocess, "run") as run:
                run.side_effect = [subprocess.CompletedProcess([], 1, ast, error),
                                   subprocess.CompletedProcess([], syntax_exit, "", "syntax error")]
                if syntax_exit:
                    with self.assertRaisesRegex(coverage.CoverageError, "syntax error"):
                        coverage.parse_source(self.path)
                else:
                    self.assertEqual(coverage.source_declarations(self.path), self.declarations[self.path])
                self.assertEqual(run.call_args.args[0], ["xcrun", "swiftc", "-frontend", "-parse", "-"])
                self.assertEqual(run.call_args.kwargs["input"],
                                 self.path.read_text() + "\nstruct __CoverageParserEOF {}\n")

    def test_actual_swift_62_diagnostic_context_preserves_unknown_error_rejection(self):
        fixture = Path(__file__).with_name("fixtures") / "swift-6.2-default-closure.stderr.txt"
        stderr = fixture.read_text()
        self.assertTrue(coverage.known_dump_type_errors(stderr))
        # Source excerpts may contain arbitrary authored strings. They are not
        # compiler diagnostics, unlike primary errors and repeated caret errors.
        self.assertTrue(coverage.known_dump_type_errors(
            stderr + '\n 18 | let message = "error: expected expression"\n'))
        for extra in ("<stdin>:20:1: error: expected expression\n",
                      "error: unknown argument: '-dump-parse'\n",
                      "<unknown>:0: error: missing required module 'Foundation'\n",
                      "    |              `- error: expected expression\n"):
            with self.subTest(extra=extra):
                self.assertFalse(coverage.known_dump_type_errors(stderr + extra))
        caret_only = "    | `- error: cannot find type 'Difficulty' in scope\n"
        self.assertFalse(coverage.known_dump_type_errors(caret_only))

    def test_standard_main_actor_dumper_error_does_not_allow_unknown_attributes(self):
        fixture = Path(__file__).with_name("fixtures") / "swift-6.2-win-screen.stderr.txt"
        stderr = fixture.read_text()
        self.assertTrue(coverage.known_dump_type_errors(stderr))
        for attribute in ("MainActorr", "Mainactor", "CustomActor", "Unknown"):
            with self.subTest(attribute=attribute):
                self.assertFalse(coverage.known_dump_type_errors(
                    stderr.replace("unknown attribute 'MainActor'", f"unknown attribute '{attribute}'")))

    def test_dumper_compatibility_rejects_unknown_errors_crashes_and_partial_trees(self):
        ast = '''(source_file "<stdin>"
  (struct_decl range=[<stdin>:5:1 - line:5:29] "__CoverageParserEOF"))'''
        known = "<stdin>:1:1: error: cannot find type 'Difficulty' in scope\n"
        self.assertEqual(coverage.without_parse_sentinel(ast, 5), '(source_file "<stdin>"\n)')
        cases = [(1, ast, known + "<stdin>:1:2: error: expected expression\n"),
                 (1, ast, "error: unknown argument: '-dump-parse'\n"),
                 (-11, ast, known), (2, ast, known), (1, "", known),
                 (1, ast[:-1], known), (1, ast.replace(":5:", ":4:"), known),
                 (1, ast.replace("  (struct_decl", "    (struct_decl"), known),
                 (1, ast + "\n(source_file \"<stdin>\")", known)]
        for code, output, error in cases:
            with self.subTest(code=code, output=output, error=error), patch.object(coverage.subprocess, "run") as run:
                run.return_value = subprocess.CompletedProcess([], code, output, error)
                with self.assertRaisesRegex(coverage.CoverageError, "Could not parse"):
                    coverage.parse_source(self.path)
                self.assertEqual(run.call_count, 1)

    def test_successful_dump_must_still_reach_the_source_end(self):
        with patch.object(coverage.subprocess, "run") as run:
            run.return_value = subprocess.CompletedProcess([], 0, '(source_file "<stdin>")', "")
            with self.assertRaisesRegex(coverage.CoverageError, "Incomplete Swift parser output"):
                coverage.parse_source(self.path)

    def test_actual_puzzle_cache_ast_survives_older_dumper_type_errors(self):
        path = Path(__file__).resolve().parents[1] / "Sudoku/Sudoku/Services/PuzzleCache.swift"
        expected = coverage.source_declarations(path)
        actual_run = subprocess.run

        def older_dumper(command, **kwargs):
            result = actual_run(command, **kwargs)
            if "-dump-parse" in command and result.returncode == 0:
                # Keep the real AST, including its unbalanced attributed-property
                # output, and emulate Swift 6.2's extra semantic diagnostics.
                fixture = Path(__file__).with_name("fixtures") / "swift-6.2-default-closure.stderr.txt"
                return subprocess.CompletedProcess(command, 1, result.stdout, fixture.read_text())
            return result

        with patch.object(coverage.subprocess, "run", side_effect=older_dumper):
            self.assertEqual(coverage.source_declarations(path), expected)

    def test_actual_win_screen_ast_survives_standard_actor_dumper_error(self):
        path = Path(__file__).resolve().parents[1] / "Sudoku/Sudoku/Views/WinScreenView.swift"
        expected = coverage.source_declarations(path)
        actual_run = subprocess.run
        fixture = Path(__file__).with_name("fixtures") / "swift-6.2-win-screen.stderr.txt"

        def older_dumper(command, **kwargs):
            result = actual_run(command, **kwargs)
            if "-dump-parse" in command and result.returncode == 0:
                return subprocess.CompletedProcess(command, 1, result.stdout, fixture.read_text())
            return result

        with patch.object(coverage.subprocess, "run", side_effect=older_dumper):
            self.assertEqual(coverage.source_declarations(path), expected)

    def test_test_framework_inside_application_source_is_rejected(self):
        self.path.write_text("import XCTest\nfunc testGame() {}\n")
        with self.assertRaisesRegex(coverage.CoverageError, "Test framework imported"):
            coverage.source_declarations(self.path)

    def test_conditional_inventory_preserves_nested_comments_and_multiline_raw_strings(self):
        source = '''/* outer /* nested */
#if DEBUG */ func visible() {}
let literal = ##"""
#if DEBUG
func imaginary() {}
#endif
"""##
#if DEBUG
func debugOnly() {}
#endif
'''
        flattened = coverage.flatten_conditionals(source)
        self.assertIn("#if DEBUG */ func visible() {}", flattened)
        self.assertEqual(flattened.count("#if DEBUG"), 2)
        self.assertEqual(flattened.count("#endif"), 1)
        self.assertEqual(len(source), len(flattened))
        self.path.write_text(source)
        self.assertEqual([d.name for d in coverage.source_declarations(self.path)], ["visible()", "debugOnly()"])


if __name__ == "__main__":
    unittest.main()
