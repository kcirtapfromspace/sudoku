#!/usr/bin/env python3
"""Gate authored iOS Swift coverage using unique source lines, not function sums.

`xccov view --report` sums closure/function line ranges; nested SwiftUI closures
therefore count the same source line repeatedly. The archive contains the real
per-source-line execution counts. Inventory source files independently and use
the Swift parser to reject missing mappings for declared function/accessor bodies.
Generated UniFFI bindings and separate test targets are outside the app scope.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


APP_SOURCE = Path("ios/Sudoku/Sudoku")
CRITICAL_FILES = ("Services/GameManager.swift", "ViewModels/GameViewModel.swift")


class CoverageError(ValueError):
    """A report is incomplete, malformed, or does not match the source tree."""


@dataclass(frozen=True)
class Declaration:
    name: str
    kind: str
    start: int
    body_start: int
    end: int
    accessor: str = ""


def integer(value: object, label: str, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise CoverageError(f"{label} must be an integer >= {minimum}: {value!r}")
    return value


def inventory(source_root: Path) -> list[Path]:
    files = sorted(path.resolve() for path in source_root.rglob("*.swift")
                   if path.relative_to(source_root).parts[0] != "Generated")
    if not files:
        raise CoverageError(f"No authored Swift sources found in {source_root}")
    return files


def source_manifest(source_root: Path) -> dict[str, str]:
    return {str(path.relative_to(source_root.resolve())):
            hashlib.sha256(path.read_bytes()).hexdigest() for path in inventory(source_root)}


def run_manifest(source_root: Path) -> dict:
    tests = {}
    project_root = source_root.parent
    for target in ("SudokuTests", "SudokuUITests"):
        for path in sorted((project_root / target).rglob("*.swift")):
            tests[str(path.relative_to(project_root))] = hashlib.sha256(path.read_bytes()).hexdigest()
    return {"authored_sources": source_manifest(source_root), "test_sources": tests}


def declarations_from_ast(ast: str) -> list[Declaration]:
    """Extract explicit body-bearing functions, initializers and accessors.

    The compiler handles Swift syntax, comments, raw/multiline strings, generic
    signatures, nested functions and property accessors. Protocol declarations
    without bodies legitimately have no executable coverage mapping.
    """
    node = re.compile(r"^(\s*)\((func_decl|constructor_decl|destructor_decl|accessor_decl)\b")
    location = re.compile(r"range=\[.*?:(\d+):\d+ - (?:line:)?(\d+):\d+\]")
    lines = ast.splitlines()
    found = set()
    for index, line in enumerate(lines):
        match = node.match(line)
        if not match or " implicit " in line:
            continue
        extent = location.search(line)
        if not extent:
            raise CoverageError(f"Swift parser returned an unknown declaration format: {line.strip()}")
        indent = len(match[1])
        start, end = map(int, extent.groups())
        name_match = re.search(r'"([^"]+)"', line[extent.end():])
        name = name_match[1] if name_match else match[2]
        accessor = ""
        if match[2] == "accessor_decl":
            accessor_match = re.search(r'\b(get|set|willSet|didSet|_read|_modify) for="', line)
            if not accessor_match:
                raise CoverageError(f"Unknown Swift accessor format: {line.strip()}")
            accessor = accessor_match[1]
        for child in lines[index + 1:]:
            child_indent = len(child) - len(child.lstrip())
            if child.strip() and child_indent <= indent:
                break
            if child_indent == indent + 2 and child.lstrip().startswith("(brace_stmt"):
                body = location.search(child)
                if not body:
                    raise CoverageError(f"Missing Swift function body range: {child.strip()}")
                found.add(Declaration(name, match[2], start, int(body[1]), end, accessor))
                break
    return sorted(found, key=lambda decl: (decl.start, decl.name, decl.kind))


def matches_declaration(name: str, declaration: Declaration) -> bool:
    """Match Xcode's demangled declaration name, not just an overlapping line.

    Nested functions add an ordinal and their enclosing context. Generic
    functions add type parameters. Accessor kinds remain distinct, so a getter
    cannot conceal an unreported setter on the same source line.
    """
    if re.match(r"(?:implicit )?closure #", name) or name.startswith("variable initialization expression"):
        return False
    name = name.split(" in ", 1)[0]
    name = re.sub(r" #\d+\s*", "", name)
    name = re.sub(r"(?<=\w)<[^<>]*>(?=\()", "", name)
    name = name.replace(".__allocating_init(", ".init(")
    expected = declaration.name
    if declaration.kind == "accessor_decl":
        suffixes = {"get": "getter", "set": "setter", "willSet": "willset", "didSet": "didset",
                    "_read": "read", "_modify": "modify"}
        if declaration.accessor not in suffixes:
            raise CoverageError(f"Unknown accessor kind for {expected}: {declaration.accessor}")
        expected += "." + suffixes[declaration.accessor]
    return bool(re.search(r"(?:^|[. ])" + re.escape(expected) + r"$", name))


def flatten_conditionals(source: str) -> str:
    """Blank compiler directives only in code, preserving source coordinates."""
    result = list(source)
    index = 0
    block_depth = 0
    string_opening = re.compile(r'(#+)?("""|")')
    conditional = re.compile(r"#(?:if|elseif|else|endif)\b[^\n]*")
    while index < len(source):
        if block_depth:
            if source.startswith("/*", index):
                block_depth += 1; index += 2
            elif source.startswith("*/", index):
                block_depth -= 1; index += 2
            else:
                index += 1
            continue
        if source.startswith("//", index):
            end = source.find("\n", index)
            index = len(source) if end < 0 else end + 1
            continue
        if source.startswith("/*", index):
            block_depth = 1; index += 2
            continue
        opening = string_opening.match(source, index)
        if opening:
            hashes, quote = opening[1] or "", opening[2]
            index += len(opening[0])
            closing = quote + hashes
            while index < len(source):
                if source.startswith("\\" + hashes, index):
                    index += len(hashes) + 2
                elif source.startswith(closing, index):
                    index += len(closing)
                    break
                else:
                    index += 1
            continue
        if source[index] == "#" and not source[source.rfind("\n", 0, index) + 1:index].strip():
            directive = conditional.match(source, index)
            if directive:
                end = index + len(directive[0])
                result[index:end] = " " * (end - index)
                index = end
                continue
        index += 1
    return "".join(result)


def without_parse_sentinel(ast: str, line: int) -> str | None:
    """Verify that the dumper reached EOF, then remove our declaration-only marker.

    Swift's text AST does not consistently balance delimiters for attributed
    stored properties, so counting parentheses cannot detect a truncated dump.
    A final empty struct has a stable one-line representation and no function or
    initializer bodies. Its exact source location prevents matching authored text.
    """
    marker = re.search(
        rf'^  \(struct_decl\b[^\n]*range=\[<stdin>:{line}:1 - (?:line:)?{line}:29\] '
        r'"__CoverageParserEOF"\)\)\s*\Z', ast, re.MULTILINE)
    if not ast.startswith('(source_file "<stdin>"\n') or not marker:
        return None
    return ast[:marker.start()] + ")"


def known_dump_type_errors(stderr: str) -> bool:
    """Only recognize the Swift 6.2 default-closure dumper compatibility errors."""
    primary = re.compile(
        r"^<stdin>:\d+:\d+: error: (cannot find type '[^'\n]+' in scope|"
        r"@escaping attribute only applies to function types)$")
    source_context = re.compile(r"^\s*\d+\s+\|")
    caret_context = re.compile(r"^\s*\|\s+`- error: (.+)$")
    seen = set()
    for line in stderr.splitlines():
        if "error:" not in line or source_context.match(line):
            continue
        if match := primary.fullmatch(line):
            seen.add(match[1])
            continue
        # Swift repeats each diagnostic beside a caret under the source line.
        # Accept only exact repetitions of an already validated primary error;
        # unlocated/unknown diagnostics and unmatched caret errors remain fatal.
        if match := caret_context.fullmatch(line):
            if match[1] in seen:
                continue
        return False
    return bool(seen)


def parse_source(path: Path) -> str:
    # -dump-parse defers #if bodies. Parse all authored conditional bodies so
    # moving code behind #if DEBUG cannot conceal a missing coverage mapping.
    # This is syntax parsing only: duplicate alternatives need no type checking.
    source = flatten_conditionals(path.read_text())
    marker_line = source.count("\n") + 2
    source += "\nstruct __CoverageParserEOF {}\n"
    result = subprocess.run(["xcrun", "swiftc", "-frontend", "-dump-parse", "-"],
                            input=source, text=True, capture_output=True)
    ast = without_parse_sentinel(result.stdout, marker_line)
    if result.returncode:
        # Swift 6.2's AST dumper computes default-closure discriminators even in
        # -dump-parse mode, which resolves parameter types from other files.
        # Swift 6.3 avoids that work for parsed trees. For the older dumper's
        # known semantic errors only, validate syntax independently and require
        # a complete dump. Never suppress syntax errors, unknown diagnostics,
        # crashes or incomplete trees; mapping validation still runs unchanged.
        if (result.returncode != 1 or not known_dump_type_errors(result.stderr)
                or ast is None):
            raise CoverageError(f"Could not parse {path}: {result.stderr.strip()}")
        syntax = subprocess.run(["xcrun", "swiftc", "-frontend", "-parse", "-"],
                                input=source, text=True, capture_output=True)
        if syntax.returncode:
            raise CoverageError(f"Could not parse {path}: {syntax.stderr.strip()}")
    if ast is None:
        raise CoverageError(f"Incomplete Swift parser output for {path}")
    return ast


def source_declarations(path: Path) -> list[Declaration]:
    ast = parse_source(path)
    if re.search(r'\(import_decl[^\n]*module="(?:XCTest|Testing)"', ast):
        raise CoverageError(f"Test framework imported inside production coverage scope: {path}")
    return declarations_from_ast(ast)


def has_initializers_or_macros(path: Path) -> bool:
    # A file consisting of stored/top-level initializers or a macro body is not
    # declaration-only just because it has no explicit `func` keyword.
    ast = parse_source(path)
    return "(original_init=" in ast or "(macro_expansion_expr " in ast


def normalize_paths(values: dict, label: str) -> dict[Path, list]:
    if not isinstance(values, dict):
        raise CoverageError(f"{label} must be a path-to-records object")
    result: dict[Path, list] = {}
    for raw_path, records in values.items():
        if not isinstance(raw_path, str) or not isinstance(records, list):
            raise CoverageError(f"Malformed {label} entry")
        result.setdefault(Path(raw_path).resolve(), []).extend(records)
    return result


def line_records(records: list, path: Path, source_lines: int) -> dict[int, int]:
    hits: dict[int, int] = {}
    for row in records:
        if not isinstance(row, dict) or not isinstance(row.get("isExecutable"), bool):
            raise CoverageError(f"Invalid executable flag for {path}: {row!r}")
        line = integer(row.get("line"), "Source line", 1)
        if line > source_lines:
            raise CoverageError(f"Stale coverage: {path}:{line} exceeds {source_lines} source lines")
        if row["isExecutable"]:
            count = integer(row.get("executionCount"), "Line execution count")
            hits[line] = max(hits.get(line, 0), count)
    return hits


def function_records(report: dict, app_target: str) -> dict[Path, list]:
    if not isinstance(report, dict) or not isinstance(report.get("targets"), list):
        raise CoverageError("xccov report has no targets")
    result: dict[Path, list] = {}
    matched = False
    for target in report["targets"]:
        if target.get("name") != app_target:
            continue
        matched = True
        if not isinstance(target.get("files"), list):
            raise CoverageError(f"No files for {app_target}")
        for file in target["files"]:
            if not isinstance(file.get("path"), str) or not isinstance(file.get("functions"), list):
                raise CoverageError(f"Malformed report file: {file!r}")
            result.setdefault(Path(file["path"]).resolve(), []).extend(file["functions"])
    if not matched:
        raise CoverageError(f"No coverage report for application target {app_target}")
    return result


def metric(covered: int, total: int) -> dict:
    return {"covered": covered, "total": total,
            "percent": 100.0 * covered / total if total else 100.0}


def collect(source_root: Path, archive: dict, report: dict,
            declarations: dict[Path, list[Declaration]], minimum_lines: float = 95,
            minimum_functions: float = 95, critical_files=CRITICAL_FILES,
            app_target: str = "Sudoku.app") -> dict:
    source_root = source_root.resolve()
    paths = inventory(source_root)
    archive_files = normalize_paths(archive, "xccov archive")
    functions = function_records(report, app_target)
    errors: list[str] = []
    files = []
    for path in paths:
        relative = str(path.relative_to(source_root))
        declared = declarations.get(path)
        if declared is None:
            raise CoverageError(f"No independent function inventory for {relative}")
        source_lines = len(path.read_text().splitlines())
        if path not in archive_files or path not in functions:
            if declared or has_initializers_or_macros(path):
                errors.append(f"Missing authored file mapping: {relative}")
            else:
                # Declaration-only files may be absent, but remain in inventory.
                files.append({"path": relative, "lines": metric(0, 0),
                              "functions": metric(0, 0), "uncovered_lines": [],
                              "uncovered_functions": [], "declared_functions": []})
                continue
        hits = line_records(archive_files.get(path, []), path, source_lines)
        mapped: dict[tuple[int, str], int] = {}
        for function in functions.get(path, []):
            line = integer(function.get("lineNumber"), "Function source line", 1)
            count = integer(function.get("executionCount"), "Function execution count")
            executable = integer(function.get("executableLines"), "Function executable lines")
            if not isinstance(function.get("name"), str) or not function["name"]:
                raise CoverageError(f"Missing function name in {relative}:{line}")
            if line > source_lines:
                raise CoverageError(f"Stale function mapping: {relative}:{line}")
            if executable:
                key = (line, function["name"])
                mapped[key] = max(mapped.get(key, 0), count)
                if line not in hits:
                    errors.append(f"Function has no archive line mapping: {relative}:{line} {function['name']}")
        for decl in declared:
            # Coverage can start at the declaration or its opening brace.
            # A different function/getter/closure on that line cannot satisfy it.
            candidates = [(line, name) for line, name in mapped
                          if decl.start <= line <= decl.body_start
                          and matches_declaration(name, decl)]
            if not candidates:
                errors.append(f"Missing authored function mapping: {relative}:{decl.start} {decl.name}")
        files.append({"path": relative,
                      "lines": metric(sum(count > 0 for count in hits.values()), len(hits)),
                      "functions": metric(sum(count > 0 for count in mapped.values()), len(mapped)),
                      "uncovered_lines": sorted(line for line, count in hits.items() if not count),
                      "uncovered_functions": [{"line": line, "name": name}
                                              for (line, name), count in sorted(mapped.items()) if not count],
                      "declared_functions": [asdict(decl) for decl in declared]})
    totals = {kind: metric(sum(file[kind]["covered"] for file in files),
                           sum(file[kind]["total"] for file in files))
              for kind in ("lines", "functions")}
    failures = list(errors)
    for kind, threshold in (("lines", minimum_lines), ("functions", minimum_functions)):
        if not totals[kind]["total"]:
            failures.append(f"No executable production {kind} measured")
        elif totals[kind]["covered"] * 100 < totals[kind]["total"] * threshold:
            failures.append(f"Production {kind}: {totals[kind]['percent']:.2f}% < {threshold:g}%")
    critical = []
    by_path = {file["path"]: file for file in files}
    for path in critical_files:
        if path not in by_path:
            failures.append(f"Missing critical gameplay file: {path}")
            continue
        file = by_path[path]
        critical.append({key: file[key] for key in ("path", "lines", "functions")})
        for kind in ("lines", "functions"):
            if not file[kind]["total"] or file[kind]["covered"] != file[kind]["total"]:
                failures.append(f"Critical gameplay {path} {kind}: {file[kind]['percent']:.2f}% < 100%")
    return {"schema_version": 1, "measurement": "unique authored Swift source lines and mapped functions",
            "source_root": str(source_root), "thresholds": {"lines": minimum_lines,
            "functions": minimum_functions, "critical_gameplay": 100},
            "excluded": ["Generated/ (UniFFI)", "separate test targets"],
            "totals": totals, "critical": critical, "files": files,
            "mapping_errors": errors, "failures": failures, "passed": not failures,
            "limitations": "Swift/Xcode line and function coverage; branch coverage is not measured."}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--manifest", type=Path, help="Require exact pre-test source hashes")
    parser.add_argument("--write-manifest", type=Path, help="Inventory sources before a fresh test run")
    args = parser.parse_args()
    source_root = args.root.resolve() / APP_SOURCE
    try:
        manifest = run_manifest(source_root)
        if args.write_manifest:
            args.write_manifest.parent.mkdir(parents=True, exist_ok=True)
            args.write_manifest.write_text(json.dumps(manifest, indent=2) + "\n")
            return 0
        if not all((args.archive, args.report, args.output)):
            parser.error("--archive, --report and --output are required for collection")
        if args.manifest and json.loads(args.manifest.read_text()) != manifest:
            raise CoverageError("Application or test Swift sources changed after the test-run manifest was created")
        declarations = {path: source_declarations(path) for path in inventory(source_root)}
        result = collect(source_root, json.loads(args.archive.read_text()),
                         json.loads(args.report.read_text()), declarations)
        result["source_sha256"] = manifest["authored_sources"]
        result["test_source_sha256"] = manifest["test_sources"]
        result["fresh_source_manifest_verified"] = args.manifest is not None
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2) + "\n")
        for kind, value in result["totals"].items():
            print(f"Authored Swift {kind}: {value['covered']}/{value['total']} ({value['percent']:.2f}%)")
        for failure in result["failures"]:
            print(f"FAIL: {failure}", file=sys.stderr)
        print(f"Coverage details: {args.output}")
        return 0 if result["passed"] else 1
    except (CoverageError, OSError, json.JSONDecodeError) as error:
        print(f"Coverage collection failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
