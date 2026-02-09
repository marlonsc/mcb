#!/usr/bin/env python3
"""Unified SARIF code-smell analyser, planner and auto-refactorer.

Wraps ``qlty smells`` to produce a SARIF report, parses it,
prioritises smells, generates actionable fix instructions,
and can apply mechanical AST-based refactoring for known
patterns.

Works with **any** project and **any** language supported by
``qlty`` (Rust, Python, TypeScript, Go, Java, C#, Ruby, …).

Usage examples::

    # Scan and show summary
    python scripts/fix_smells.py --scan --summary

    # Plan for a specific module
    python scripts/fix_smells.py --plan --module my-lib

    # Apply auto-fixes on one file
    python scripts/fix_smells.py --fix --file routes.py

    # Full pipeline: scan → plan → fix
    python scripts/fix_smells.py --scan --plan --fix

    # Export plan as Markdown
    python scripts/fix_smells.py --plan --markdown report.md

    # Load custom strategy config
    python scripts/fix_smells.py --config smells.toml --plan
"""

from __future__ import annotations

import abc
import argparse
import json
import re
import subprocess  # nosec B404
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Any, Sequence

# ────────────────────────────────────────────────
# Constants & Configuration
# ────────────────────────────────────────────────

# -- Paths ----------------------------------------
DEFAULT_SARIF = "qlty.smells.lst"
SARIF_VERSION = "2.1.0"

# -- qlty CLI defaults ----------------------------
QLTY_BIN = "qlty"
QLTY_TIMEOUT = 300

# -- Thresholds -----------------------------------
NESTING_WARN = 5
COMPLEXITY_WARN = 15
LINES_HIGH = 30
TOP_FILES = 20
TOP_MODULES = 15

# -- Output ---------------------------------------
SEP = "\u2501" * 56
THIN = "\u2500" * 56
HASH = "#" * 56

# -- Project root markers (order matters) ---------
ROOT_MARKERS = (
    ".git",
    "Cargo.toml",
    "pyproject.toml",
    "setup.py",
    "package.json",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "CMakeLists.txt",
    "Makefile",
    "mix.exs",
    "Gemfile",
    "composer.json",
    ".hg",
)

# -- Language detection by extension ---------------
_LANG_MAP: dict[str, str] = {
    ".rs": "rust",
    ".py": "python",
    ".pyi": "python",
    ".ts": "typescript",
    ".tsx": "typescript",
    ".js": "javascript",
    ".jsx": "javascript",
    ".mjs": "javascript",
    ".cjs": "javascript",
    ".go": "go",
    ".java": "java",
    ".cs": "csharp",
    ".rb": "ruby",
    ".kt": "kotlin",
    ".kts": "kotlin",
    ".swift": "swift",
    ".cpp": "cpp",
    ".cc": "cpp",
    ".cxx": "cpp",
    ".c": "c",
    ".h": "c",
    ".hpp": "hpp",
    ".scala": "scala",
    ".php": "php",
    ".lua": "lua",
    ".ex": "elixir",
    ".exs": "elixir",
    ".dart": "dart",
    ".r": "r",
    ".jl": "julia",
    ".zig": "zig",
    ".sol": "solidity",
    ".sh": "bash",
    ".bash": "bash",
}

# -- Monorepo / workspace top-level dirs ----------
_WORKSPACE_DIRS: frozenset[str] = frozenset(
    {
        "crates",
        "packages",
        "apps",
        "libs",
        "modules",
        "services",
        "projects",
        "components",
        "plugins",
        "extensions",
        "workspaces",
        "internal",
        "cmd",
        "pkg",
    }
)


# ────────────────────────────────────────────────
# Priority
# ────────────────────────────────────────────────


class Priority(IntEnum):
    """Fix priority — lower ordinal = fix first."""

    CRITICAL = 1
    HIGH = 2
    MEDIUM = 3
    LOW = 4
    INFO = 5


RULE_PRIORITY: dict[str, Priority] = {
    # duplication
    "identical-code": Priority.CRITICAL,
    "similar-code": Priority.HIGH,
    # complexity
    "function-complexity": Priority.MEDIUM,
    "method-complexity": Priority.MEDIUM,
    "cognitive-complexity": Priority.MEDIUM,
    "file-complexity": Priority.MEDIUM,
    # structure
    "nested-control-flow": Priority.MEDIUM,
    "deep-nesting": Priority.MEDIUM,
    "long-method": Priority.MEDIUM,
    "large-class": Priority.MEDIUM,
    # expression
    "boolean-logic": Priority.LOW,
    "complex-condition": Priority.LOW,
    # signature
    "function-parameters": Priority.LOW,
    "too-many-arguments": Priority.LOW,
    # misc
    "return-statements": Priority.LOW,
    "god-class": Priority.HIGH,
    "feature-envy": Priority.MEDIUM,
    "data-clump": Priority.MEDIUM,
}


# ────────────────────────────────────────────────
# Data model
# ────────────────────────────────────────────────


@dataclass(frozen=True, slots=True)
class SmellLocation:
    """Physical location of a smell in source."""

    uri: str
    start_line: int
    start_column: int
    end_line: int
    end_column: int
    language: str = ""

    @property
    def file_name(self) -> str:
        """Base file name."""
        return Path(self.uri).name

    @property
    def detected_language(self) -> str:
        """Language from metadata or extension."""
        if self.language:
            return self.language
        ext = Path(self.uri).suffix.lower()
        return _LANG_MAP.get(ext, "")

    @property
    def module_name(self) -> str:
        """Top-level module / package / crate.

        Recognises common monorepo layouts:
        ``crates/<mod>/…``, ``packages/<mod>/…``,
        ``apps/<mod>/…``, ``cmd/<mod>/…``, …
        Falls back to ``src/<first_dir>``.
        """
        parts = Path(self.uri).parts
        for i, part in enumerate(parts):
            if part in _WORKSPACE_DIRS and i + 1 < len(parts):
                return parts[i + 1]
        if len(parts) >= 2 and parts[0] in ("src", "lib"):
            return parts[1]
        return ""

    @property
    def line_count(self) -> int:
        """Number of lines the smell spans."""
        return max(
            1,
            self.end_line - self.start_line + 1,
        )

    @property
    def relative_dir(self) -> str:
        """Parent directory of the source file."""
        return str(Path(self.uri).parent)


@dataclass(frozen=True, slots=True)
class Smell:
    """A single code smell parsed from SARIF."""

    rule_id: str
    level: str
    message: str
    location: SmellLocation
    fingerprints: dict[str, str] = field(default_factory=dict)
    taxa: tuple[str, ...] = ()

    @property
    def rule_short(self) -> str:
        """``qlty:similar-code`` → ``similar-code``."""
        return self.rule_id.split(":")[-1]

    @property
    def function_name(self) -> str:
        """Enclosing function name, if any."""
        return self.fingerprints.get("function.name", "")

    @property
    def priority(self) -> Priority:
        """Lookup priority for this rule."""
        return RULE_PRIORITY.get(self.rule_short, Priority.INFO)

    @property
    def complexity_count(self) -> int | None:
        """Extract complexity count from message."""
        m = re.search(r"count\s*=\s*(\d+)", self.message)
        return int(m.group(1)) if m else None

    @property
    def nesting_level(self) -> int | None:
        """Extract nesting level from message."""
        m = re.search(r"level\s*=\s*(\d+)", self.message)
        return int(m.group(1)) if m else None

    @property
    def similar_lines(self) -> int | None:
        """Number of similar lines reported."""
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

    @property
    def severity_score(self) -> float:
        """Numeric score for sorting: higher = worse.

        Combines priority, span, and complexity.
        """
        base = (6 - self.priority) * 10.0
        span = min(self.location.line_count, 100)
        cc = self.complexity_count or 0
        return base + span * 0.3 + cc * 0.5


# ────────────────────────────────────────────────
# SARIF parser
# ────────────────────────────────────────────────


def parse_sarif(path: Path) -> list[Smell]:
    """Parse a SARIF 2.1.0 JSON file."""
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
    """Pull physical location from a result."""
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


# ────────────────────────────────────────────────
# qlty execution
# ────────────────────────────────────────────────


def run_qlty_smells(
    root: Path,
    *,
    all_files: bool = True,
    include_tests: bool = False,
    no_duplication: bool = False,
    upstream: str | None = None,
    paths: Sequence[str] = (),
) -> Path:
    """Run ``qlty smells --sarif``."""
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
        result = subprocess.run(  # nosec B603
            cmd,
            capture_output=True,
            text=True,
            timeout=QLTY_TIMEOUT,
            cwd=str(root),
        )
    except FileNotFoundError:
        _error(f"'{QLTY_BIN}' not found. Install: https://qlty.sh")
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
        sys.exit(1)

    out.write_text(text, encoding="utf-8")
    _ok(f"SARIF written to {out}")
    return out


# ────────────────────────────────────────────────
# Fix strategy registry (plugin architecture)
# ────────────────────────────────────────────────


class FixStrategy(abc.ABC):
    """Base for smell-fix strategies.

    Subclass, set class attributes, and add to
    ``STRATEGY_REGISTRY`` via ``register_strategy``.
    """

    @property
    @abc.abstractmethod
    def rule(self) -> str:
        """Short rule name."""

    @property
    @abc.abstractmethod
    def title(self) -> str:
        """Human-readable title."""

    @property
    @abc.abstractmethod
    def instructions(self) -> str:
        """English fix instructions."""

    @property
    def auto_fixable(self) -> bool:
        return False

    def attempt_fix(self, root: Path, smell: Smell) -> bool:
        return False


# -- Concrete strategies -------------------------


class IdenticalCodeStrategy(FixStrategy):
    """Eliminate identical (copy-pasted) code."""

    rule = "identical-code"
    title = "Eliminate identical code blocks"
    instructions = "\n".join(
        [
            "Extract the duplicated block into a shared helper.",
            "  \u2022 Place the helper in the nearest common ancestor module so all",
            "    call-sites can reach it without circular imports.",
            "  \u2022 When duplicated logic implements a contract (trait/interface),",
            "    delegate to the contract's default implementation instead.",
            "  \u2022 For identical branches in a switch/match, merge them with",
            "    pattern alternation or a shared handler.",
            "  \u2022 If the duplication exists in test code, extract a shared test",
            "    helper or fixture.",
        ]
    )


class SimilarCodeStrategy(FixStrategy):
    """Deduplicate similar (not identical) code."""

    rule = "similar-code"
    title = "Deduplicate similar code blocks"
    instructions = "\n".join(
        [
            "Identify the varying parts between the similar blocks; extract the",
            "common logic into a single helper parameterised by those differences.",
            "  \u2022 Repeated traversal/loop boilerplate \u2192 a generic visitor or",
            "    iterator helper accepting a callback or closure.",
            "  \u2022 Near-identical branches \u2192 unify"
            " with a configuration parameter",
            "    or data-driven lookup table.",
            "  \u2022 Similar validation functions \u2192"
            " table-driven approach iterating",
            "    (predicate, handler) pairs.",
            "  \u2022 Multiple functions differing only in type \u2192 use generics /",
            "    templates / type parameters.",
        ]
    )


class FunctionComplexityStrategy(FixStrategy):
    """Reduce cyclomatic complexity."""

    rule = "function-complexity"
    title = "Reduce function complexity"
    instructions = "\n".join(
        [
            "Break the function into smaller, focused helpers:",
            "  \u2022 Extract nested loops into iterator or helper methods.",
            "  \u2022 Replace deep conditional chains with early returns",
            "    (guard clauses).",
            "  \u2022 Move complex conditionals into named boolean variables.",
            "  \u2022 Extract validation into dedicated predicate functions.",
            "  \u2022 Use a table-driven pattern: define behaviours in a",
            "    static lookup and iterate.",
            "  \u2022 Each helper should do exactly one thing and be",
            "    independently testable.",
        ]
    )


class MethodComplexityStrategy(FixStrategy):
    """Reduce method complexity."""

    rule = "method-complexity"
    title = "Reduce method complexity"
    instructions = "\n".join(
        [
            "Simplify the method by extracting distinct responsibilities:",
            "  \u2022 Each extracted method should operate on a single concern.",
            "  \u2022 Use guard clauses at the top to handle edge cases early.",
            "  \u2022 Delegate orchestration: the method should read like a",
            "    high-level recipe calling well-named helpers.",
            "  \u2022 Consider whether the method belongs in this class or",
            "    should be moved to a collaborator.",
        ]
    )


class CognitiveComplexityStrategy(FixStrategy):
    """Lower cognitive complexity."""

    rule = "cognitive-complexity"
    title = "Lower cognitive complexity"
    instructions = "\n".join(
        [
            "Cognitive complexity measures how hard code is to understand.",
            "  \u2022 Flatten nesting: invert conditions and return early.",
            "  \u2022 Avoid breaks and continues; prefer structured iteration.",
            "  \u2022 Extract nested closures / lambdas into named functions.",
            "  \u2022 Simplify boolean expressions with named predicates.",
            "  \u2022 Replace recursive patterns with iterative ones when clarity",
            "    improves.",
        ]
    )


class NestedControlFlowStrategy(FixStrategy):
    """Flatten deeply nested control flow."""

    rule = "nested-control-flow"
    title = "Flatten deeply nested control flow"
    auto_fixable = True
    instructions = "\n".join(
        [
            "Reduce nesting depth to \u22644 levels:",
            "  \u2022 Invert conditions and return/continue early",
            "    (guard clauses).",
            "  \u2022 Extract inner blocks into small, named helper functions.",
            "  \u2022 Chain optional/nullable access with language combinators",
            "    (``?.``, ``and_then``, ``map``, ``flatMap``).",
            "  \u2022 Propagate errors with the language's idiomatic mechanism",
            "    (``?``, ``try``, ``raise``/``throw``).",
            "  \u2022 Consider the 'extract and compose' approach: each helper",
            "    adds at most one level of nesting.",
        ]
    )

    def attempt_fix(self, root: Path, smell: Smell) -> bool:
        lang = smell.location.detected_language
        if not lang:
            return False
        pattern = _NESTING_PATTERNS.get(lang)
        if not pattern:
            return False
        return _run_ast_grep(
            root,
            pattern,
            target=smell.location.uri,
            language=lang,
        )


class DeepNestingStrategy(FixStrategy):
    """Flatten deep nesting (alias rule)."""

    rule = "deep-nesting"
    title = "Flatten deep nesting"
    instructions = NestedControlFlowStrategy.instructions


class FileComplexityStrategy(FixStrategy):
    """Split a high-complexity file."""

    rule = "file-complexity"
    title = "Split complex file into modules"
    instructions = "\n".join(
        [
            "Split the file into focused sub-modules:",
            "  \u2022 Move helper/utility functions into a dedicated helpers module.",
            "  \u2022 Separate each concern (e.g. parsing, validation,",
            "    formatting) into its own module.",
            "  \u2022 Extract types/data structures into a shared types module.",
            "  \u2022 Re-export public items from the parent module index.",
            "  \u2022 Keep each file focused on a single cohesive purpose.",
        ]
    )


class LongMethodStrategy(FixStrategy):
    """Shorten an excessively long method."""

    rule = "long-method"
    title = "Shorten long method"
    instructions = "\n".join(
        [
            "Break the method into well-named steps:",
            "  \u2022 Identify logical sections (setup, process, output) and",
            "    extract each into a helper.",
            "  \u2022 If the method accumulates state across sections, consider",
            "    encapsulating that state in a small context object.",
            "  \u2022 Each extracted method should be \u226425 lines and fit on",
            "    one screen.",
            "  \u2022 Avoid flag arguments; prefer separate methods for",
            "    each mode of operation.",
        ]
    )


class LargeClassStrategy(FixStrategy):
    """Decompose a large class."""

    rule = "large-class"
    title = "Decompose large class"
    instructions = "\n".join(
        [
            "A class with too many responsibilities should be split:",
            "  \u2022 Group related fields and methods into cohesive clusters.",
            "  \u2022 Extract each cluster into a collaborator class.",
            "  \u2022 The original class delegates to collaborators.",
            "  \u2022 Consider composition over inheritance when sharing",
            "    behaviour.",
            "  \u2022 Aim for \u22645 public methods per class.",
        ]
    )


class GodClassStrategy(FixStrategy):
    """Break apart a god class."""

    rule = "god-class"
    title = "Break apart god class"
    instructions = "\n".join(
        [
            "A god class does too much; redistribute responsibilities:",
            "  \u2022 Identify distinct responsibilities by grouping",
            "    related fields and methods.",
            "  \u2022 Extract each group into a focused class with a clear",
            "    contract (interface/trait).",
            "  \u2022 The original class becomes a thin coordinator.",
            "  \u2022 Each new class should be independently testable.",
            "  \u2022 Apply dependency inversion: depend on abstractions,",
            "    not concrete classes.",
        ]
    )


class FeatureEnvyStrategy(FixStrategy):
    """Fix feature envy."""

    rule = "feature-envy"
    title = "Fix feature envy"
    instructions = "\n".join(
        [
            "This method uses more data from another class than its own:",
            "  \u2022 Move the method to the class whose data it accesses most.",
            "  \u2022 If only part of the method envies another class, extract",
            "    that part and move just the extracted piece.",
            "  \u2022 Consider whether the envied class should expose a",
            "    higher-level operation instead of raw data.",
        ]
    )


class DataClumpStrategy(FixStrategy):
    """Eliminate data clumps."""

    rule = "data-clump"
    title = "Eliminate data clumps"
    instructions = "\n".join(
        [
            "The same group of parameters or fields appears repeatedly:",
            "  \u2022 Introduce a value object / dataclass / struct that",
            "    bundles the related values.",
            "  \u2022 Replace the parameter list with this new type.",
            "  \u2022 Add behaviour to the new type when appropriate,",
            "    moving logic closer to the data it operates on.",
        ]
    )


class BooleanLogicStrategy(FixStrategy):
    """Simplify complex boolean expressions."""

    rule = "boolean-logic"
    title = "Simplify boolean expressions"
    auto_fixable = True
    instructions = "\n".join(
        [
            "Break complex boolean expressions into named predicates:",
            "  \u2022 Extract ``a && b || c && d`` into a named variable",
            "    (``is_valid = \u2026``).",
            "  \u2022 Use helper functions: ``is_test_file(path)``.",
            "  \u2022 Apply De Morgan\u2019s laws to simplify negations.",
            "  \u2022 Use pattern-matching or match/when constructs if",
            "    the language supports them.",
        ]
    )


class ComplexConditionStrategy(FixStrategy):
    """Simplify complex conditions."""

    rule = "complex-condition"
    title = "Simplify complex conditional"
    instructions = BooleanLogicStrategy.instructions


class FunctionParametersStrategy(FixStrategy):
    """Reduce function parameter count."""

    rule = "function-parameters"
    title = "Reduce function parameter count"
    instructions = "\n".join(
        [
            "Group related parameters into a single configuration object:",
            "  \u2022 Create a Config/Options type for 4+ parameters.",
            "  \u2022 Use the builder pattern for complex construction.",
            "  \u2022 Check if parameters can be derived from existing context.",
            "  \u2022 Accept flexible input types to reduce overloads.",
            "  \u2022 Default values should live in the config type, not the",
            "    function signature.",
        ]
    )


class TooManyArgumentsStrategy(FixStrategy):
    """Reduce argument count (alias rule)."""

    rule = "too-many-arguments"
    title = "Reduce argument count"
    instructions = FunctionParametersStrategy.instructions


class ReturnStatementsStrategy(FixStrategy):
    """Reduce excessive return statements."""

    rule = "return-statements"
    title = "Reduce return statements"
    instructions = "\n".join(
        [
            "Too many return points make flow hard to follow:",
            "  \u2022 Consolidate into a single exit by accumulating the",
            "    result in a variable.",
            "  \u2022 Use guard clauses only at the top for preconditions.",
            "  \u2022 Extract branches into small helper functions, each with",
            "    a single return.",
            "  \u2022 Consider replacing branching logic with a lookup table.",
        ]
    )


# -- AST patterns per language --------------------

_NESTING_PATTERNS: dict[str, tuple[str, str]] = {
    "rust": (
        "if let Some($X) = $Y { if let Some($Z) = $W { $$$B } }",
        "let Some($X) = $Y "
        "else { return; };\n"
        "let Some($Z) = $W "
        "else { return; };\n"
        "$$$B",
    ),
    "python": (
        "if $X:\n    if $Y:\n        $$$B",
        "if not $X:\n    return\nif not $Y:\n    return\n$$$B",
    ),
    "javascript": (
        "if ($X) { if ($Y) { $$$B } }",
        "if (!$X) { return; }\nif (!$Y) { return; }\n$$$B",
    ),
    "typescript": (
        "if ($X) { if ($Y) { $$$B } }",
        "if (!$X) { return; }\nif (!$Y) { return; }\n$$$B",
    ),
    "go": (
        "if $X { if $Y { $$$B } }",
        "if !($X) { return }\nif !($Y) { return }\n$$$B",
    ),
}

# -- All built-in strategy classes ----------------

_BUILTIN_STRATEGIES: list[type[FixStrategy]] = [
    IdenticalCodeStrategy,
    SimilarCodeStrategy,
    FunctionComplexityStrategy,
    MethodComplexityStrategy,
    CognitiveComplexityStrategy,
    NestedControlFlowStrategy,
    DeepNestingStrategy,
    FileComplexityStrategy,
    LongMethodStrategy,
    LargeClassStrategy,
    GodClassStrategy,
    FeatureEnvyStrategy,
    DataClumpStrategy,
    BooleanLogicStrategy,
    ComplexConditionStrategy,
    FunctionParametersStrategy,
    TooManyArgumentsStrategy,
    ReturnStatementsStrategy,
]

# -- Registry -------------------------------------

STRATEGY_REGISTRY: dict[str, FixStrategy] = {}


def _register_builtins() -> None:
    for cls in _BUILTIN_STRATEGIES:
        inst = cls()
        STRATEGY_REGISTRY[inst.rule] = inst


_register_builtins()


def get_strategy(
    rule_short: str,
) -> FixStrategy | None:
    return STRATEGY_REGISTRY.get(rule_short)


def register_strategy(s: FixStrategy) -> None:
    """Register a custom strategy at runtime."""
    STRATEGY_REGISTRY[s.rule] = s


# ────────────────────────────────────────────────
# Auto-fix helpers
# ────────────────────────────────────────────────


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
        res = subprocess.run(  # nosec B603
            cmd,
            capture_output=True,
            text=True,
            timeout=30,
            cwd=str(root),
        )
    except FileNotFoundError:
        _error("ast-grep not found. Install: npm i -g @ast-grep/cli")
        return False
    except subprocess.TimeoutExpired:
        _error("ast-grep timed out (30s)")
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


# ────────────────────────────────────────────────
# Instruction generator
# ────────────────────────────────────────────────


def generate_fix_instruction(
    smell: Smell,
) -> str:
    """Actionable English fix instruction."""
    strategy = get_strategy(smell.rule_short)
    title = strategy.title if strategy else f"Fix {smell.rule_short}"
    instr = strategy.instructions if strategy else "Review and refactor manually."

    pname = smell.priority.name
    parts: list[str] = [
        f"## [{pname}] {title}",
    ]
    parts.append(f"**File:** `{smell.location.uri}`")
    sl = smell.location.start_line
    el = smell.location.end_line
    parts.append(f"**Location:** lines {sl}\u2013{el}")

    if smell.function_name:
        parts.append(f"**Function:** `{smell.function_name}`")
    if smell.location.detected_language:
        parts.append(f"**Language:** {smell.location.detected_language}")
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
    parts.append(f"**Severity score:** {smell.severity_score:.1f}")

    parts.append(f"\n**Message:** {smell.message}")
    parts.append(f"\n### Fix Strategy\n{instr}")

    auto = strategy.auto_fixable if strategy else False
    if auto:
        parts.append("\n> \u2705 **Auto-fixable** \u2014 run with ``--fix``.")
    else:
        parts.append(
            "\n> \u26a0\ufe0f  **Manual review"
            " required** \u2014 needs design"
            " decisions."
        )

    return "\n".join(parts)


# ────────────────────────────────────────────────
# Filtering
# ────────────────────────────────────────────────


def filter_smells(
    smells: list[Smell],
    *,
    file_filter: str | None = None,
    rule_filter: str | None = None,
    module_filter: str | None = None,
    priority_filter: str | None = None,
    language_filter: str | None = None,
    min_severity: float | None = None,
) -> list[Smell]:
    """Apply zero or more filters."""
    out = smells

    if file_filter:
        out = [s for s in out if file_filter in s.location.uri]
    if rule_filter:
        out = [s for s in out if rule_filter in s.rule_short]
    if module_filter:
        out = [s for s in out if module_filter in s.location.module_name]
    if language_filter:
        lf = language_filter.lower()
        out = [s for s in out if s.location.detected_language == lf]
    if priority_filter:
        try:
            prio = Priority[priority_filter.upper()]
            out = [s for s in out if s.priority == prio]
        except KeyError:
            _warn(f"Unknown priority: {priority_filter}")
    if min_severity is not None:
        out = [s for s in out if s.severity_score >= min_severity]

    return out


# ────────────────────────────────────────────────
# Reporters
# ────────────────────────────────────────────────


def report_summary(
    smells: list[Smell],
) -> None:
    """Print a concise count summary."""
    print(SEP)
    print("CODE SMELL SUMMARY")
    print(SEP)
    n = len(smells)
    print(f"\nTotal smells: {n}\n")

    _print_counter(
        "By rule:",
        Counter(s.rule_short for s in smells),
    )
    _print_counter(
        "By language:",
        Counter(s.location.detected_language or "(unknown)" for s in smells),
        limit=TOP_MODULES,
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


def report_detail(
    smells: list[Smell],
) -> None:
    """Print smells grouped by file."""
    by_file: dict[str, list[Smell]] = defaultdict(list)
    for s in smells:
        by_file[s.location.uri].append(s)

    for uri in sorted(
        by_file,
        key=lambda u: -len(by_file[u]),
    ):
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


def report_plan(
    smells: list[Smell],
) -> None:
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
            fc = len(fs)
            print(f"\n{THIN}")
            print(f"File: {uri} ({fc} smells)")
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
                    msg = s.message[:55]
                    print(f"  \u21b3 Also at line {ln}{fnl}: {msg}")
                    continue
                seen.add(key)
                print()
                print(generate_fix_instruction(s))


def report_json(
    smells: list[Smell],
) -> None:
    """Output smells as JSON."""
    out = [
        {
            "rule": s.rule_short,
            "priority": s.priority.name,
            "severity_score": (s.severity_score),
            "file": s.location.uri,
            "start_line": (s.location.start_line),
            "end_line": s.location.end_line,
            "function": s.function_name,
            "message": s.message,
            "language": (s.location.detected_language),
            "module": s.location.module_name,
        }
        for s in sorted(
            smells,
            key=lambda x: (
                -x.severity_score,
                x.location.uri,
            ),
        )
    ]
    print(json.dumps(out, indent=2))


def report_markdown(
    smells: list[Smell],
    path: Path,
) -> None:
    """Export plan as a Markdown file."""
    lines: list[str] = [
        "# Refactoring Plan\n",
        f"**Total smells:** {len(smells)}\n",
    ]

    by_prio: dict[Priority, list[Smell]] = defaultdict(list)
    for s in smells:
        by_prio[s.priority].append(s)

    for prio in Priority:
        items = by_prio.get(prio, [])
        if not items:
            continue
        n = len(items)
        lines.append(f"\n## {prio.name} ({n} items)\n")
        for s in sorted(
            items,
            key=lambda x: (
                -x.severity_score,
                x.location.uri,
            ),
        ):
            fn = s.function_name or "-"
            lang = s.location.detected_language or "-"
            sl = s.location.start_line
            el = s.location.end_line
            score = s.severity_score
            lines.append(f"### {s.rule_short}: `{s.location.uri}` L{sl}\u2013{el}")
            lines.append(f"- **Function:** `{fn}`")
            lines.append(f"- **Language:** {lang}")
            lines.append(f"- **Score:** {score:.1f}")
            lines.append(f"- **Message:** {s.message}\n")
            st = get_strategy(s.rule_short)
            if st:
                lines.append(f"**Strategy:** {st.title}\n")
                lines.append(st.instructions + "\n")

    path.write_text("\n".join(lines), encoding="utf-8")
    _ok(f"Markdown plan written to {path}")


def apply_fixes(
    smells: list[Smell],
    root: Path,
) -> None:
    """Attempt to auto-fix eligible smells."""
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
        key=lambda s: (
            -s.severity_score,
            s.location.uri,
        ),
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


# ────────────────────────────────────────────────
# IO helpers
# ────────────────────────────────────────────────


def _info(msg: str) -> None:
    print(
        f"  \u2139\ufe0f  {msg}",
        file=sys.stderr,
    )


def _ok(msg: str) -> None:
    print(f"  \u2705 {msg}", file=sys.stderr)


def _warn(msg: str) -> None:
    print(
        f"  \u26a0\ufe0f  {msg}",
        file=sys.stderr,
    )


def _error(msg: str) -> None:
    print(f"  \u274c {msg}", file=sys.stderr)


def _print_counter(
    header: str,
    counter: Counter[str],
    *,
    limit: int | None = None,
    width: int = 30,
) -> None:
    print(f"\n{header}")
    for name, cnt in counter.most_common(limit):
        label = name or "(unknown)"
        print(f"  {label:.<{width}s} {cnt:>4d}")


# ────────────────────────────────────────────────
# Project root discovery
# ────────────────────────────────────────────────


def find_project_root() -> Path:
    """Walk up from script dir to find root."""
    cur = Path(__file__).resolve().parent
    for d in [cur, *cur.parents]:
        for marker in ROOT_MARKERS:
            if (d / marker).exists():
                return d
    return cur


# ────────────────────────────────────────────────
# Configuration loader (optional TOML/JSON)
# ────────────────────────────────────────────────


def load_config(path: Path) -> None:
    """Load custom configuration from file.

    Supports JSON format. Updates global
    RULE_PRIORITY and thresholds.
    """
    global NESTING_WARN, COMPLEXITY_WARN
    global LINES_HIGH, TOP_FILES, TOP_MODULES

    with open(path, encoding="utf-8") as fh:
        cfg = json.load(fh)

    # priorities
    for rule, prio_name in cfg.get("priorities", {}).items():
        try:
            RULE_PRIORITY[rule] = Priority[prio_name.upper()]
        except KeyError:
            _warn(f"Unknown priority '{prio_name}' for rule '{rule}'")

    # thresholds
    th = cfg.get("thresholds", {})
    NESTING_WARN = th.get("nesting_warn", NESTING_WARN)
    COMPLEXITY_WARN = th.get("complexity_warn", COMPLEXITY_WARN)
    LINES_HIGH = th.get("lines_high", LINES_HIGH)
    TOP_FILES = th.get("top_files", TOP_FILES)
    TOP_MODULES = th.get("top_modules", TOP_MODULES)

    _ok(f"Config loaded from {path}")


# ────────────────────────────────────────────────
# CLI
# ────────────────────────────────────────────────


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description=("Unified SARIF smell analyser and auto-refactorer"),
        formatter_class=(argparse.RawDescriptionHelpFormatter),
        epilog=(
            "Examples:\n"
            "  fix_smells.py --scan --summary\n"
            "  fix_smells.py --plan --module lib\n"
            "  fix_smells.py --fix --file app.py\n"
            "  fix_smells.py --plan --markdown r.md"
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
    ag.add_argument(
        "--markdown",
        metavar="FILE",
        help="Export plan as Markdown file",
    )

    # -- Input ---
    p.add_argument(
        "--sarif",
        metavar="FILE",
        default=DEFAULT_SARIF,
        help=(f"SARIF file (default: {DEFAULT_SARIF})"),
    )
    p.add_argument(
        "--config",
        metavar="FILE",
        help="JSON config file for custom priorities and thresholds",
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
    fg.add_argument(
        "--language",
        dest="language_filter",
        metavar="LANG",
        help="Filter by language (e.g. python)",
    )
    fg.add_argument(
        "--min-severity",
        dest="min_severity",
        type=float,
        metavar="SCORE",
        help="Minimum severity score",
    )

    return p


def main() -> None:
    """Entry point."""
    parser = build_parser()
    args = parser.parse_args()

    root = find_project_root()

    # -- Optional config ---
    if args.config:
        cfg_path = Path(args.config)
        if not cfg_path.is_absolute():
            cfg_path = root / cfg_path
        load_config(cfg_path)

    sarif = root / args.sarif

    # -- Optional scanning ---
    if args.scan:
        sarif = run_qlty_smells(
            root,
            include_tests=args.include_tests,
            no_duplication=(args.no_duplication),
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
        language_filter=args.language_filter,
        min_severity=args.min_severity,
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
            args.markdown,
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
    if args.markdown:
        report_markdown(smells, Path(args.markdown))


if __name__ == "__main__":
    main()
