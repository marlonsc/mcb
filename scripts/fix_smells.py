#!/usr/bin/env python3
"""Unified SARIF code-smell analyser, planner and auto-refactorer.

Wraps ``qlty smells`` to produce a SARIF report, parses it,
prioritises smells, generates actionable fix instructions,
and can apply mechanical AST-based refactoring for known
patterns.

Works with **any** project and **any** language supported by
``qlty`` (Rust, Python, TypeScript, Go, Java, C#, …).

Usage examples::

    # Scan and show summary
    python scripts/fix_smells.py --scan --summary

    # Plan for a specific module
    python scripts/fix_smells.py --plan --module my-lib

    # Apply auto-fixes on one file
    python scripts/fix_smells.py --fix --file routes.py

    # Full pipeline: scan → plan → fix
    python scripts/fix_smells.py --scan --plan --fix
"""

from __future__ import annotations

import abc
import argparse
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Any, Sequence

# ──────────────────────────────────────────────────────────
# Constants & Configuration
# ──────────────────────────────────────────────────────────

# -- Paths ------------------------------------------------
DEFAULT_SARIF = "qlty.smells.lst"
SARIF_VERSION = "2.1.0"

# -- qlty CLI defaults ------------------------------------
QLTY_BIN = "qlty"
QLTY_TIMEOUT = 300

# -- Thresholds -------------------------------------------
NESTING_WARN = 5
COMPLEXITY_WARN = 15
LINES_HIGH = 30
TOP_FILES = 20
TOP_MODULES = 15

# -- Output -----------------------------------------------
SEP = "\u2501" * 60
THIN = "\u2500" * 60
HASH = "#" * 60

# -- Project root markers (order matters) -----------------
ROOT_MARKERS = (
    ".git",
    "Cargo.toml",
    "pyproject.toml",
    "package.json",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "Makefile",
    ".hg",
)

# -- Language detection by extension ----------------------
_LANG_MAP: dict[str, str] = {
    ".rs": "rust",
    ".py": "python",
    ".ts": "typescript",
    ".tsx": "typescript",
    ".js": "javascript",
    ".jsx": "javascript",
    ".go": "go",
    ".java": "java",
    ".cs": "csharp",
    ".rb": "ruby",
    ".kt": "kotlin",
    ".swift": "swift",
    ".cpp": "cpp",
    ".c": "c",
    ".scala": "scala",
    ".php": "php",
    ".lua": "lua",
    ".ex": "elixir",
    ".exs": "elixir",
}


# ──────────────────────────────────────────────────────────
# Priority
# ──────────────────────────────────────────────────────────


class Priority(IntEnum):
    """Fix priority — lower ordinal = fix first."""

    CRITICAL = 1
    HIGH = 2
    MEDIUM = 3
    LOW = 4
    INFO = 5


RULE_PRIORITY: dict[str, Priority] = {
    "identical-code": Priority.CRITICAL,
    "similar-code": Priority.HIGH,
    "function-complexity": Priority.MEDIUM,
    "nested-control-flow": Priority.MEDIUM,
    "file-complexity": Priority.MEDIUM,
    "boolean-logic": Priority.LOW,
    "function-parameters": Priority.LOW,
}


# ──────────────────────────────────────────────────────────
# Data model
# ──────────────────────────────────────────────────────────


@dataclass(frozen=True, slots=True)
class SmellLocation:
    """Physical location of a smell in source code."""

    uri: str
    start_line: int
    start_column: int
    end_line: int
    end_column: int
    language: str = ""

    @property
    def file_name(self) -> str:
        """Base file name from the URI."""
        return Path(self.uri).name

    @property
    def detected_language(self) -> str:
        """Detect language from the file extension."""
        if self.language:
            return self.language
        ext = Path(self.uri).suffix.lower()
        return _LANG_MAP.get(ext, "")

    @property
    def module_name(self) -> str:
        """Extract the top-level module/package/crate.

        Recognises common project layouts:
        ``src/<mod>/…``, ``lib/<mod>/…``,
        ``crates/<mod>/…``, ``packages/<mod>/…``,
        ``apps/<mod>/…``.
        """
        parts = Path(self.uri).parts
        top_dirs = {
            "crates",
            "packages",
            "apps",
            "libs",
            "modules",
            "services",
        }
        for i, part in enumerate(parts):
            if part in top_dirs and i + 1 < len(parts):
                return parts[i + 1]
        if len(parts) >= 2 and parts[0] in ("src", "lib"):
            return parts[1]
        return ""

    @property
    def line_count(self) -> int:
        """Number of lines spanned by this location."""
        return max(1, self.end_line - self.start_line + 1)


@dataclass(frozen=True, slots=True)
class Smell:
    """A single code smell parsed from SARIF."""

    rule_id: str
    level: str
    message: str
    location: SmellLocation
    fingerprints: dict[str, str] = field(default_factory=dict)
    taxa: tuple[str, ...] = ()

    # -- derived properties --------------------------------

    @property
    def rule_short(self) -> str:
        """Short rule name after the last ``:``.

        ``qlty:similar-code`` → ``similar-code``.
        """
        return self.rule_id.split(":")[-1]

    @property
    def function_name(self) -> str:
        """Function name from SARIF fingerprints."""
        return self.fingerprints.get("function.name", "")

    @property
    def priority(self) -> Priority:
        """Fix priority for this smell."""
        return RULE_PRIORITY.get(self.rule_short, Priority.INFO)

    @property
    def complexity_count(self) -> int | None:
        """Numeric complexity from the message."""
        m = re.search(r"count\s*=\s*(\d+)", self.message)
        return int(m.group(1)) if m else None

    @property
    def nesting_level(self) -> int | None:
        """Nesting depth from the message."""
        m = re.search(r"level\s*=\s*(\d+)", self.message)
        return int(m.group(1)) if m else None

    @property
    def similar_lines(self) -> int | None:
        """Number of similar lines from the message."""
        m = re.search(
            r"(\d+)\s+lines?\s+of\s+similar",
            self.message,
        )
        return int(m.group(1)) if m else None

    @property
    def similar_locations(self) -> int | None:
        """Number of locations with similar code."""
        m = re.search(
            r"in\s+(\d+)\s+locations?",
            self.message,
        )
        return int(m.group(1)) if m else None


# ──────────────────────────────────────────────────────────
# SARIF parser
# ──────────────────────────────────────────────────────────


def parse_sarif(path: Path) -> list[Smell]:
    """Parse a SARIF 2.1.0 JSON into Smell objects."""
    with open(path, encoding="utf-8") as fh:
        data = json.load(fh)

    vsn = data.get("version", "")
    if vsn != SARIF_VERSION:
        _warn(f"Expected SARIF {SARIF_VERSION}, got {vsn}")

    smells: list[Smell] = []
    for run in data.get("runs", []):
        for result in run.get("results", []):
            loc = _extract_location(result)
            if loc is None:
                continue
            fps = result.get("partialFingerprints", {})
            taxa = tuple(t.get("id", "") for t in result.get("taxa", []))
            smells.append(
                Smell(
                    rule_id=result.get("ruleId", ""),
                    level=result.get("level", "warning"),
                    message=result.get("message", {}).get("text", ""),
                    location=loc,
                    fingerprints=fps,
                    taxa=taxa,
                )
            )
    return smells


def _extract_location(
    result: dict[str, Any],
) -> SmellLocation | None:
    """Pull the first physical location from result."""
    locations = result.get("locations", [])
    if not locations:
        return None
    phys = locations[0].get("physicalLocation", {})
    art = phys.get("artifactLocation", {})
    region = phys.get("region", {})
    uri = art.get("uri", "")
    if not uri:
        return None
    return SmellLocation(
        uri=uri,
        start_line=region.get("startLine", 0),
        start_column=region.get("startColumn", 0),
        end_line=region.get("endLine", 0),
        end_column=region.get("endColumn", 0),
        language=region.get("sourceLanguage", ""),
    )


# ──────────────────────────────────────────────────────────
# qlty execution
# ──────────────────────────────────────────────────────────


def run_qlty_smells(
    root: Path,
    *,
    all_files: bool = True,
    include_tests: bool = False,
    no_duplication: bool = False,
    upstream: str | None = None,
    paths: Sequence[str] = (),
) -> Path:
    """Run ``qlty smells --sarif`` and save output.

    Returns the path to the generated SARIF file.
    """
    out = root / DEFAULT_SARIF

    cmd: list[str] = [
        QLTY_BIN,
        "smells",
        "--sarif",
        "--quiet",
    ]
    if all_files:
        cmd.append("--all")
    if include_tests:
        cmd.append("--include-tests")
    if no_duplication:
        cmd.append("--no-duplication")
    if upstream:
        cmd.extend(["--upstream", upstream])
    cmd.extend(paths)

    _info(f"Running: {' '.join(cmd)}")
    _info(f"Working directory: {root}")

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=QLTY_TIMEOUT,
            cwd=str(root),
        )
    except FileNotFoundError:
        _error(f"'{QLTY_BIN}' not found. See https://github.com/qlty/qlty")
        sys.exit(1)
    except subprocess.TimeoutExpired:
        _error(f"qlty timed out after {QLTY_TIMEOUT}s. Narrow scope with --paths.")
        sys.exit(1)

    text = result.stdout.strip()
    if not text:
        if result.stderr.strip():
            _error("qlty stderr: " + result.stderr.strip()[:300])
        _error("qlty produced no SARIF output.")
        sys.exit(1)

    try:
        json.loads(text)
    except json.JSONDecodeError as exc:
        _error(f"Invalid JSON from qlty: {exc}")
        _error(f"First 200 chars: {text[:200]}")
        sys.exit(1)

    out.write_text(text, encoding="utf-8")
    _ok(f"SARIF written to {out}")
    return out


# ──────────────────────────────────────────────────────────
# Fix strategy registry (plugin architecture)
# ──────────────────────────────────────────────────────────


class FixStrategy(abc.ABC):
    """Base class for smell-fix strategies.

    Subclass and add instances to ``STRATEGY_REGISTRY``.
    """

    @property
    @abc.abstractmethod
    def rule(self) -> str:
        """Short rule name, e.g. ``similar-code``."""

    @property
    @abc.abstractmethod
    def title(self) -> str:
        """Human-readable title for the plan."""

    @property
    @abc.abstractmethod
    def instructions(self) -> str:
        """Multi-line English fix instructions."""

    @property
    def auto_fixable(self) -> bool:
        """Whether this strategy can auto-fix."""
        return False

    def attempt_fix(self, root: Path, smell: Smell) -> bool:
        """Try to auto-fix *smell*. True on success."""
        return False


# -- Concrete strategies ----------------------------------


class IdenticalCodeStrategy(FixStrategy):
    """Eliminate identical (copy-pasted) code."""

    rule = "identical-code"
    title = "Eliminate identical code blocks"
    auto_fixable = True
    instructions = (
        "Extract the duplicated block into a shared"
        " helper.\n"
        "  \u2022 If the duplication lives across sibling"
        " modules, place the helper in the\n"
        "    nearest common ancestor module.\n"
        "  \u2022 When duplicated logic implements a"
        " contract, delegate to the contract's\n"
        "    default implementation instead of"
        " re-coding it.\n"
        "  \u2022 For identical branches in a switch/match,"
        " merge them with pattern\n"
        "    alternation or a shared handler."
    )

    def attempt_fix(self, root: Path, smell: Smell) -> bool:
        """Auto-fix severity delegation patterns."""
        if "severity" in smell.message.lower():
            return _fix_delegation(root, smell)
        return False


class SimilarCodeStrategy(FixStrategy):
    """Deduplicate similar (not identical) code."""

    rule = "similar-code"
    title = "Deduplicate similar code blocks"
    instructions = (
        "Identify the varying parts between similar"
        " blocks; extract the common\n"
        "logic into a single helper parameterised by"
        " the differences.\n"
        "  \u2022 Repeated traversal/looping boilerplate"
        " \u2192 a generic visitor or iterator\n"
        "    helper accepting a callback.\n"
        "  \u2022 Near-identical branches \u2192 unify into one"
        " branch with a configuration\n"
        "    parameter or lookup table.\n"
        "  \u2022 Similar validation functions \u2192 a"
        " table-driven approach that iterates\n"
        "    (pattern, handler) pairs."
    )


class FunctionComplexityStrategy(FixStrategy):
    """Reduce cyclomatic complexity of a function."""

    rule = "function-complexity"
    title = "Reduce function complexity"
    instructions = (
        "Break the function into smaller, focused"
        " helpers:\n"
        "  \u2022 Extract nested loops into iterator or"
        " helper methods.\n"
        "  \u2022 Replace deep conditional chains with"
        " early returns (guard clauses).\n"
        "  \u2022 Move complex conditionals into named"
        " boolean variables.\n"
        "  \u2022 Extract validation into predicate"
        " functions.\n"
        "  \u2022 Use a table-driven pattern: define"
        " behaviours in a static lookup\n"
        "    and iterate over it."
    )


class NestedControlFlowStrategy(FixStrategy):
    """Flatten deeply nested control flow."""

    rule = "nested-control-flow"
    title = "Flatten deeply nested control flow"
    auto_fixable = True
    instructions = (
        "Reduce nesting depth to \u22644 levels:\n"
        "  \u2022 Invert conditions and return/continue"
        " early (guard clauses).\n"
        "  \u2022 Extract inner blocks into small, named"
        " helper functions.\n"
        "  \u2022 Chain optional/nullable access with"
        " built-in combinators\n"
        "    (e.g. ``?.``, ``and_then``, ``map``,"
        " ``flatMap``).\n"
        "  \u2022 Propagate errors with the language's"
        " idiomatic short-circuit\n"
        "    mechanism (``?``, ``try``,"
        " ``raise``/``throw``)."
    )

    def attempt_fix(self, root: Path, smell: Smell) -> bool:
        """Apply ast-grep to flatten nesting."""
        lang = smell.location.detected_language
        if not lang:
            return False
        return _run_ast_grep(
            root,
            _NESTING_PATTERNS.get(lang, ("", "")),
            target=smell.location.uri,
            language=lang,
        )


class FileComplexityStrategy(FixStrategy):
    """Split a high-complexity file into modules."""

    rule = "file-complexity"
    title = "Split complex file into modules"
    instructions = (
        "Split the file into focused sub-modules:\n"
        "  \u2022 Move helper/utility functions into a"
        " dedicated helpers module.\n"
        "  \u2022 Separate each concern (e.g. parsing,"
        " validation, formatting)\n"
        "    into its own module.\n"
        "  \u2022 Extract types/data structures into a"
        " shared types module.\n"
        "  \u2022 Re-export public items from the"
        " parent module index."
    )


class BooleanLogicStrategy(FixStrategy):
    """Simplify complex boolean expressions."""

    rule = "boolean-logic"
    title = "Simplify complex boolean expressions"
    auto_fixable = True
    instructions = (
        "Break complex boolean expressions into"
        " named predicates:\n"
        "  \u2022 Extract ``a && b || c && d`` into a"
        " named variable\n"
        "    (``is_valid = \u2026``).\n"
        "  \u2022 Use helper functions:"
        " ``is_test_file(path)``.\n"
        "  \u2022 Apply De Morgan\u2019s laws to simplify"
        " negations.\n"
        "  \u2022 Use language-specific pattern-matching"
        " utilities."
    )


class FunctionParametersStrategy(FixStrategy):
    """Reduce function parameter count."""

    rule = "function-parameters"
    title = "Reduce function parameter count"
    instructions = (
        "Group related parameters into a single"
        " configuration object:\n"
        "  \u2022 Create a Config/Options structure"
        " for 4+ parameters.\n"
        "  \u2022 Use the builder pattern for complex"
        " construction.\n"
        "  \u2022 Check if parameters can be derived"
        " from existing context.\n"
        "  \u2022 Accept flexible input types to"
        " reduce overloads."
    )


# -- Nesting patterns per language (pattern, fix) ---------
_NESTING_PATTERNS: dict[str, tuple[str, str]] = {
    "rust": (
        "if let Some($X) = $Y { if let Some($Z) = $W { $$$B } }",
        "let Some($X) = $Y else { return; };\n"
        "let Some($Z) = $W else { return; };\n"
        "$$$B",
    ),
    "python": (
        "if $X:\n    if $Y:\n        $$$B",
        "if not $X:\n    return\nif not $Y:\n    return\n$$$B",
    ),
}


# -- Registry ---------------------------------------------

STRATEGY_REGISTRY: dict[str, FixStrategy] = {}


def _register_strategies() -> None:
    """Populate the global strategy registry."""
    for cls in (
        IdenticalCodeStrategy,
        SimilarCodeStrategy,
        FunctionComplexityStrategy,
        NestedControlFlowStrategy,
        FileComplexityStrategy,
        BooleanLogicStrategy,
        FunctionParametersStrategy,
    ):
        inst = cls()
        STRATEGY_REGISTRY[inst.rule] = inst


_register_strategies()


def get_strategy(
    rule_short: str,
) -> FixStrategy | None:
    """Look up a fix strategy by rule short name."""
    return STRATEGY_REGISTRY.get(rule_short)


def register_strategy(s: FixStrategy) -> None:
    """Register a custom strategy at runtime."""
    STRATEGY_REGISTRY[s.rule] = s


# ──────────────────────────────────────────────────────────
# Auto-fix helpers
# ──────────────────────────────────────────────────────────


def _run_ast_grep(
    root: Path,
    pattern_fix: tuple[str, str],
    *,
    target: str | None = None,
    language: str = "rust",
) -> bool:
    """Run ast-grep to apply a pattern rewrite."""
    pat, fix = pattern_fix
    if not pat:
        return False

    cmd = [
        "ast-grep",
        "--pattern",
        pat,
        "--rewrite",
        fix,
        "--lang",
        language,
    ]
    if target:
        cmd.append(str(root / target))
    else:
        cmd.append(str(root))

    try:
        res = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=30,
            cwd=str(root),
        )
    except FileNotFoundError:
        _error("ast-grep not found \u2014 install: cargo install ast-grep")
        return False
    except subprocess.TimeoutExpired:
        _error("ast-grep timed out after 30s")
        return False

    if res.returncode == 0:
        _ok("ast-grep fix applied")
        out = res.stdout.strip()
        if out:
            _info(f"  {out[:200]}")
        return True

    _warn(f"ast-grep exit {res.returncode}")
    err = res.stderr.strip()
    if err:
        _info(f"  {err[:200]}")
    return False


def _fix_delegation(root: Path, smell: Smell) -> bool:
    """Replace an inherent method with delegation.

    Works for any method that duplicates a trait or
    interface default implementation.
    """
    fpath = root / smell.location.uri
    if not fpath.exists():
        _error(f"File not found: {fpath}")
        return False

    content = fpath.read_text(encoding="utf-8")

    pat = re.compile(
        r"(\s+pub fn severity\(&self\)"
        r" -> Severity \{)\s*\n"
        r"(\s+match self \{.*?\n\s+\})\s*\n"
        r"(\s+\})",
        re.DOTALL,
    )

    match = pat.search(content)
    if not match:
        _warn("No delegatable pattern found")
        return False

    indent = "        "
    replacement = (
        f"{match.group(1)}\n"
        f"{indent}<Self as Violation>"
        f"::severity(self)\n"
        f"{match.group(3)}"
    )

    before = content[: match.start()]
    after = content[match.end() :]
    new = before + replacement + after
    fpath.write_text(new, encoding="utf-8")
    _ok("Replaced with delegation to contract")
    return True


# ──────────────────────────────────────────────────────────
# Instruction generator
# ──────────────────────────────────────────────────────────


def generate_fix_instruction(smell: Smell) -> str:
    """Generate an actionable English instruction."""
    strategy = get_strategy(smell.rule_short)
    title = strategy.title if strategy else f"Fix {smell.rule_short}"
    instr = strategy.instructions if strategy else "Review and refactor manually."

    pname = smell.priority.name
    parts: list[str] = [f"## [{pname}] {title}"]
    parts.append(f"**File:** `{smell.location.uri}`")
    sl = smell.location.start_line
    el = smell.location.end_line
    parts.append(f"**Location:** lines {sl}\u2013{el}")

    if smell.function_name:
        parts.append(f"**Function:** `{smell.function_name}`")
    cc = smell.complexity_count
    if cc is not None:
        parts.append(f"**Complexity:** {cc}")
    nl = smell.nesting_level
    if nl is not None:
        parts.append(f"**Nesting depth:** {nl}")
    sl_count = smell.similar_lines
    if sl_count is not None:
        locs = smell.similar_locations or "?"
        parts.append(f"**Duplication:** {sl_count} similar lines in {locs} locations")

    parts.append(f"\n**Message:** {smell.message}")
    parts.append(f"\n### Fix Strategy\n{instr}")

    auto = strategy.auto_fixable if strategy else False
    if auto:
        parts.append("\n> \u2705 **Auto-fixable** \u2014 run with ``--fix``.")
    else:
        parts.append(
            "\n> \u26a0\ufe0f  **Manual review required**"
            " \u2014 needs design decisions."
        )

    return "\n".join(parts)


# ──────────────────────────────────────────────────────────
# Filtering
# ──────────────────────────────────────────────────────────


def filter_smells(
    smells: list[Smell],
    *,
    file_filter: str | None = None,
    rule_filter: str | None = None,
    module_filter: str | None = None,
    priority_filter: str | None = None,
) -> list[Smell]:
    """Apply zero or more filters to the list."""
    out = smells

    if file_filter:
        out = [s for s in out if file_filter in s.location.uri]
    if rule_filter:
        out = [s for s in out if rule_filter in s.rule_short]
    if module_filter:
        out = [s for s in out if module_filter in s.location.module_name]
    if priority_filter:
        try:
            prio = Priority[priority_filter.upper()]
            out = [s for s in out if s.priority == prio]
        except KeyError:
            _warn(f"Unknown priority: {priority_filter}")

    return out


# ──────────────────────────────────────────────────────────
# Reporters
# ──────────────────────────────────────────────────────────


def report_summary(smells: list[Smell]) -> None:
    """Print a concise count summary."""
    print(SEP)
    print("CODE SMELL SUMMARY")
    print(SEP)
    print(f"\nTotal smells: {len(smells)}\n")

    _print_counter(
        "By rule:",
        Counter(s.rule_short for s in smells),
    )
    _print_counter(
        "By module:",
        Counter(s.location.module_name for s in smells),
        limit=TOP_MODULES,
        width=40,
    )

    print("\nBy priority:")
    for prio in Priority:
        cnt = sum(1 for s in smells if s.priority == prio)
        if cnt:
            print(f"  {prio.name:.<20s} {cnt:>4d}")

    print(f"\nBy file (top {TOP_FILES}):")
    fc = Counter(s.location.uri for s in smells)
    for uri, cnt in fc.most_common(TOP_FILES):
        print(f"  {cnt:>3d}  {uri}")


def report_detail(smells: list[Smell]) -> None:
    """Print smells grouped by file."""
    by_file: dict[str, list[Smell]] = defaultdict(list)
    for s in smells:
        by_file[s.location.uri].append(s)

    for uri in sorted(by_file, key=lambda u: -len(by_file[u])):
        items = sorted(
            by_file[uri],
            key=lambda s: s.location.start_line,
        )
        print(f"\n{THIN}")
        n = len(items)
        print(f"  {uri} ({n} smells)")
        print(THIN)
        for it in items:
            fn = ""
            if it.function_name:
                fn = f" [{it.function_name}]"
            pc = it.priority.name[0]
            msg = it.message[:50]
            ln = it.location.start_line
            rule = it.rule_short
            print(f"  L{ln:>4d} ({pc}) {rule:<25s} {msg}{fn}")


def report_plan(smells: list[Smell]) -> None:
    """Print a prioritised refactoring plan."""
    print(SEP)
    print("REFACTORING PLAN")
    print(SEP)

    by_prio: dict[Priority, list[Smell]] = defaultdict(list)
    for s in smells:
        by_prio[s.priority].append(s)

    for prio in Priority:
        items = by_prio.get(prio, [])
        if not items:
            continue

        n = len(items)
        print(f"\n{HASH}")
        print(f"# PRIORITY: {prio.name} ({n} items)")
        print(HASH)

        by_file: dict[str, list[Smell]] = defaultdict(list)
        for itm in items:
            by_file[itm.location.uri].append(itm)

        for uri in sorted(
            by_file,
            key=lambda u: -len(by_file[u]),
        ):
            fs = by_file[uri]
            n2 = len(fs)
            print(f"\n{THIN}")
            print(f"File: {uri} ({n2} smells)")
            print(THIN)

            seen: set[str] = set()
            for s in sorted(
                fs,
                key=lambda x: x.location.start_line,
            ):
                fn = s.function_name or "file"
                key = f"{s.rule_short}:{fn}"
                if key in seen:
                    fnl = ""
                    if s.function_name:
                        fnl = f" `{s.function_name}`"
                    ln = s.location.start_line
                    msg = s.message[:60]
                    print(f"  \u21b3 Also at line {ln}{fnl}: {msg}")
                    continue
                seen.add(key)
                print()
                print(generate_fix_instruction(s))


def report_json(smells: list[Smell]) -> None:
    """Output smells as JSON."""
    out = [
        {
            "rule": s.rule_short,
            "priority": s.priority.name,
            "file": s.location.uri,
            "start_line": s.location.start_line,
            "end_line": s.location.end_line,
            "function": s.function_name,
            "message": s.message,
            "language": (s.location.detected_language),
            "module": s.location.module_name,
        }
        for s in sorted(
            smells,
            key=lambda x: (x.priority, x.location.uri),
        )
    ]
    print(json.dumps(out, indent=2))


def apply_fixes(smells: list[Smell], root: Path) -> None:
    """Attempt to auto-fix all eligible smells."""
    print(SEP)
    print("AUTO-FIX ENGINE")
    print(SEP)

    fixable = [
        s for s in smells if (st := get_strategy(s.rule_short)) and st.auto_fixable
    ]
    total = len(smells)
    nfix = len(fixable)
    print(f"\nFound {nfix} auto-fixable smells out of {total} total")

    fixed = 0
    skipped = 0

    for smell in sorted(
        fixable,
        key=lambda s: (s.priority, s.location.uri),
    ):
        uri = smell.location.uri
        ln = smell.location.start_line
        rule = smell.rule_short
        print(f"\n\u2192 [{rule}] {uri}:{ln}")
        print(f"  {smell.message[:70]}")
        st = get_strategy(smell.rule_short)
        if st and st.attempt_fix(root, smell):
            fixed += 1
        else:
            skipped += 1

    manual = total - nfix
    print(f"\n{THIN}")
    print(f"Results: {fixed} fixed, {skipped} skipped")
    print(f"Manual review needed: {manual} smells")


# ──────────────────────────────────────────────────────────
# IO helpers
# ──────────────────────────────────────────────────────────


def _info(msg: str) -> None:
    """Print an informational message to stderr."""
    print(f"  \u2139\ufe0f  {msg}", file=sys.stderr)


def _ok(msg: str) -> None:
    """Print a success message to stderr."""
    print(f"  \u2705 {msg}", file=sys.stderr)


def _warn(msg: str) -> None:
    """Print a warning message to stderr."""
    print(f"  \u26a0\ufe0f  {msg}", file=sys.stderr)


def _error(msg: str) -> None:
    """Print an error message to stderr."""
    print(f"  \u274c {msg}", file=sys.stderr)


def _print_counter(
    header: str,
    counter: Counter[str],
    *,
    limit: int | None = None,
    width: int = 30,
) -> None:
    """Print a formatted counter distribution."""
    print(f"\n{header}")
    for name, cnt in counter.most_common(limit):
        label = name or "(unknown)"
        print(f"  {label:.<{width}s} {cnt:>4d}")


# ──────────────────────────────────────────────────────────
# Project root discovery
# ──────────────────────────────────────────────────────────


def find_project_root() -> Path:
    """Walk up to find a project root marker.

    Recognises: .git, Cargo.toml, pyproject.toml,
    package.json, go.mod, pom.xml, build.gradle,
    Makefile, .hg.
    """
    cur = Path(__file__).resolve().parent
    for d in [cur, *cur.parents]:
        for marker in ROOT_MARKERS:
            if (d / marker).exists():
                return d
    return cur


# ──────────────────────────────────────────────────────────
# CLI
# ──────────────────────────────────────────────────────────


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser."""
    p = argparse.ArgumentParser(
        description=("Unified SARIF smell analyser and auto-refactorer"),
        formatter_class=(argparse.RawDescriptionHelpFormatter),
        epilog=(
            "Examples:\n"
            "  fix_smells.py --scan --summary\n"
            "  fix_smells.py --plan --module lib\n"
            "  fix_smells.py --fix --file app.py\n"
            "  fix_smells.py --detail --priority high"
        ),
    )

    # -- Scanning ---
    sg = p.add_argument_group("scanning (qlty smells)")
    sg.add_argument(
        "--scan",
        action="store_true",
        help="Run qlty smells before analysing",
    )
    sg.add_argument(
        "--include-tests",
        action="store_true",
        help="Include test files in scan",
    )
    sg.add_argument(
        "--no-duplication",
        action="store_true",
        help="Skip duplication checks",
    )
    sg.add_argument(
        "--upstream",
        metavar="REF",
        help="Upstream base ref (e.g. main)",
    )
    sg.add_argument(
        "--paths",
        nargs="*",
        default=[],
        metavar="PATH",
        help="Paths to pass to qlty smells",
    )

    # -- Actions ---
    ag = p.add_argument_group("actions")
    ag.add_argument(
        "--summary",
        action="store_true",
        help="Show smell count summary",
    )
    ag.add_argument(
        "--detail",
        action="store_true",
        help="Show smells grouped by file",
    )
    ag.add_argument(
        "--plan",
        action="store_true",
        help="Generate prioritised fix plan",
    )
    ag.add_argument(
        "--fix",
        action="store_true",
        help="Apply auto-fixes where possible",
    )
    ag.add_argument(
        "--json",
        action="store_true",
        help="Output smells as JSON",
    )

    # -- Input ---
    p.add_argument(
        "--sarif",
        metavar="FILE",
        default=DEFAULT_SARIF,
        help=f"SARIF file (default: {DEFAULT_SARIF})",
    )

    # -- Filters ---
    fg = p.add_argument_group("filters")
    fg.add_argument(
        "--file",
        dest="file_filter",
        metavar="SUBSTR",
        help="Filter by file path substring",
    )
    fg.add_argument(
        "--rule",
        dest="rule_filter",
        metavar="SUBSTR",
        help="Filter by rule name substring",
    )
    fg.add_argument(
        "--module",
        dest="module_filter",
        metavar="SUBSTR",
        help="Filter by module/package name",
    )
    fg.add_argument(
        "--priority",
        dest="priority_filter",
        choices=[pr.name.lower() for pr in Priority],
        metavar="LEVEL",
        help="Filter by priority level",
    )

    return p


def main() -> None:
    """Entry point."""
    parser = build_parser()
    args = parser.parse_args()

    root = find_project_root()
    sarif = root / args.sarif

    # -- Optional scanning ---
    if args.scan:
        sarif = run_qlty_smells(
            root,
            include_tests=args.include_tests,
            no_duplication=args.no_duplication,
            upstream=args.upstream,
            paths=args.paths,
        )

    # -- Parse SARIF ---
    if not sarif.exists():
        _error(f"SARIF not found: {sarif}\n  Use --scan or --sarif.")
        sys.exit(1)

    _info(f"Loading SARIF from: {sarif}")
    smells = parse_sarif(sarif)
    _info(f"Parsed {len(smells)} smells")

    # -- Apply filters ---
    smells = filter_smells(
        smells,
        file_filter=args.file_filter,
        rule_filter=args.rule_filter,
        module_filter=args.module_filter,
        priority_filter=args.priority_filter,
    )

    if not smells:
        print("No smells match the given filters.")
        sys.exit(0)

    n = len(smells)
    print(f"After filters: {n} smells\n")

    # Default action
    has_action = any(
        [
            args.summary,
            args.detail,
            args.plan,
            args.fix,
            args.json,
        ]
    )
    if not has_action:
        args.summary = True

    # -- Execute actions ---
    if args.summary:
        report_summary(smells)
    if args.detail:
        report_detail(smells)
    if args.plan:
        report_plan(smells)
    if args.fix:
        apply_fixes(smells, root)
    if args.json:
        report_json(smells)


if __name__ == "__main__":
    main()
