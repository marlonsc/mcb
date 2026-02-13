#!/usr/bin/env python3
"""
SARIF Quality Analysis Tool - Parses qlty check SARIF output and provides statistics.
"""

import argparse
import abc
import json
import subprocess  # nosec B404
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Any


class Severity(IntEnum):
    """Severity levels mapped from SARIF."""

    ERROR = 3
    WARNING = 2
    INFO = 1
    NONE = 0

    @classmethod
    def from_str(cls, s: str) -> "Severity":
        mapping = {
            "error": cls.ERROR,
            "warning": cls.WARNING,
            "note": cls.INFO,
        }
        return mapping.get(s.lower(), cls.NONE)

    def to_emoji(self) -> str:
        return {
            self.ERROR: "🔴",
            self.WARNING: "🟠",
            self.INFO: "🔵",
            self.NONE: "⚪",
        }[self]


@dataclass
class SarifIssue:
    """Unified representation of a SARIF result (check or smell)."""

    rule_id: str
    level: Severity
    message: str
    file_path: str
    start_line: int
    end_line: int | None = None
    category: str = ""  # check, smell, security, format, etc.
    help_uri: str = ""
    metadata: dict[str, Any] = field(default_factory=dict)
    fingerprints: dict[str, str] = field(default_factory=dict)

    @property
    def location_str(self) -> str:
        if self.end_line and self.end_line != self.start_line:
            return f"{self.file_path}:{self.start_line}-{self.end_line}"
        return f"{self.file_path}:{self.start_line}"

    @property
    def rule_category(self) -> str:
        """Extract category from rule_id (e.g., 'rustfmt', 'zizmor', 'osv-scanner')."""
        if ":" in self.rule_id:
            return self.rule_id.split(":")[0]
        return "unknown"


# ────────────────────────────────────────────────
# Fix Strategies (Ported from fix_smells.py)
# ────────────────────────────────────────────────


class FixStrategy(abc.ABC):
    """Base for smell-fix strategies."""

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


class IdenticalCodeStrategy(FixStrategy):
    rule = "identical-code"
    title = "Eliminate identical code blocks"
    instructions = "\n".join(
        [
            "Refactor duplicated logic into shared abstractions:",
            "- **Domain Logic**: Move shared business rules to `mcb-domain` entities or services.",
            "- **Infrastructure**: Extract common technical implementations to `mcb-infrastructure::utils`.",
            "- **Tests**: Use `mcb_domain::test_services_config` or shared test fixtures.",
        ]
    )


class SimilarCodeStrategy(FixStrategy):
    rule = "similar-code"
    title = "Refactor similar code blocks"
    instructions = "\n".join(
        [
            "Unify similar patterns using Rust's powerful type system:",
            "- **Traits**: Define a trait in `mcb-domain::ports` and implement variations in `mcb-providers`.",
            "- **Generics**: Use generic parameters for slight variations in types.",
            "- **Macros**: Use `macro_rules!` (sparingly) for structural repetition that generics can't handle.",
        ]
    )


class FunctionComplexityStrategy(FixStrategy):
    rule = "function-complexity"
    title = "Reduce function complexity"
    instructions = "\n".join(
        [
            "Simplify complex functions by extracting logic:",
            "- **Abstraction**: Move distinct steps into private helper methods or `impl` blocks.",
            "- **Guard Clauses**: Use `if ... { return ... }` to reduce nesting depth.",
            "- **Pattern Matching**: Use `match` expressions instead of complex `if/else` chains.",
            "- **Error Handling**: Use the `?` operator for clean error propagation.",
        ]
    )


class MethodComplexityStrategy(FixStrategy):
    rule = "method-complexity"
    title = "Reduce method complexity"
    instructions = FunctionComplexityStrategy.instructions


class CognitiveComplexityStrategy(FixStrategy):
    rule = "cognitive-complexity"
    title = "Lower cognitive complexity"
    instructions = "\n".join(
        [
            "Make the code easier to reason about:",
            "- **Encapsulation**: Hide complex details behind descriptive function names.",
            "- **Boolean Logic**: Extract complex conditions into `is_valid()` styling methods.",
            "- **Control Flow**: Prefer iterators (`map`, `filter`, `fold`) over manual loops with state.",
        ]
    )


class NestedControlFlowStrategy(FixStrategy):
    rule = "nested-control-flow"
    title = "Flatten deeply nested control flow"
    instructions = "\n".join(
        [
            "Reduce nesting depth (target ≤ 4 levels):",
            "- **Guard Clauses**: Check preconditions early and return.",
            "- **Iterators**: Use functional combinators to transform collections flatly.",
            "- **Lets**: Use `let ... = match ...` to assign results instead of nesting logic.",
        ]
    )


class DeepNestingStrategy(FixStrategy):
    rule = "deep-nesting"
    title = "Flatten deep nesting"
    instructions = NestedControlFlowStrategy.instructions


class FileComplexityStrategy(FixStrategy):
    rule = "file-complexity"
    title = "Split complex file into modules"
    instructions = "\n".join(
        [
            "Break down large files into focused modules:",
            "- **Modularity**: Create a directory with `mod.rs` and split concerns into separate files.",
            "- **Clean Architecture**: Ensure the file strictly belongs to one layer (Domain, Infra, App).",
            "- **Helpers**: Move utility functions to `utils.rs` or specialized submodules.",
        ]
    )


class LongMethodStrategy(FixStrategy):
    rule = "long-method"
    title = "Shorten long method"
    instructions = "\n".join(
        [
            "Break methods into single-responsibility steps:",
            "- **Steps**: Identify logical sections (setup, process, output) and extract them.",
            "- **Size**: Aim for methods that fit on a single screen (≤ 25 lines).",
            "- **Context**: If passing many variables, consider a context struct.",
        ]
    )


class LargeClassStrategy(FixStrategy):
    rule = "large-class"
    title = "Decompose large struct/class"
    instructions = "\n".join(
        [
            "Redistribute responsibilities from this large struct:",
            "- **Composition**: Extract groups of fields into smaller Value Objects (in `mcb-domain::value_objects`).",
            "- **Behavior**: Move complex logic to Domain Services if it involves multiple entities.",
            "- **Traits**: Implement standard traits (`From`, `TryFrom`, `Display`) to offload conversion logic.",
        ]
    )


class GodClassStrategy(FixStrategy):
    rule = "god-class"
    title = "Decompose God Class"
    instructions = "\n".join(
        [
            "This struct violates Single Responsibility Principle:",
            "- **Domain Services**: Split orchestration logic into specific Application Services.",
            "- **Rich Entities**: Move business rules to the Entities that hold the data.",
            "- **Providers**: Delegate external interaction to `mcb-providers` via Ports.",
        ]
    )


class FeatureEnvyStrategy(FixStrategy):
    rule = "feature-envy"
    title = "Resolve Feature Envy"
    instructions = "\n".join(
        [
            "Move logic closer to the data it operates on:",
            "- **Move Method**: If a method primarily uses another struct's data, move it there.",
            "- **Encapsulation**: Keep data and behavior together in `mcb-domain` entities.",
            "- **Getters**: If you are accessing many getters, it's a sign that logic belongs in that object.",
        ]
    )


class DataClumpStrategy(FixStrategy):
    rule = "data-clump"
    title = "Encapsulate Data Clumps"
    instructions = "\n".join(
        [
            "Group frequently appearing parameters or fields:",
            "- **Value Object**: Create a new struct in `mcb-domain::value_objects`.",
            "- **Validation**: Enforce invariants in the new type's constructor (`new()`).",
            "- **Type Safety**: Replace loose parameters with this strongly-typed value.",
        ]
    )


class BooleanLogicStrategy(FixStrategy):
    rule = "boolean-logic"
    title = "Simplify boolean expressions"
    instructions = "\n".join(
        [
            "Improve readability of boolean logic:",
            "- **Predicates**: Extract conditions into named methods returning `bool`.",
            "- **De Morgan**: Simplify negated groups.",
            "- **Matches**: Consider if a `match` expression is clearer than complex boolean operators.",
        ]
    )


class ComplexConditionStrategy(FixStrategy):
    rule = "complex-condition"
    title = "Simplify complex conditional"
    instructions = BooleanLogicStrategy.instructions


class FunctionParametersStrategy(FixStrategy):
    rule = "function-parameters"
    title = "Reduce function parameter count"
    instructions = "\n".join(
        [
            "Too many arguments indicate missing abstractions:",
            "- **Config Struct**: Group related parameters into a configuration struct.",
            "- **Builder**: Use the Builder pattern for complex instance construction.",
            "- **Context**: Use a `Context` struct for passing cross-cutting data.",
        ]
    )


class TooManyArgumentsStrategy(FixStrategy):
    rule = "too-many-arguments"
    title = "Reduce argument count"
    instructions = FunctionParametersStrategy.instructions


class ReturnStatementsStrategy(FixStrategy):
    rule = "return-statements"
    title = "Consolidate return points"
    instructions = "\n".join(
        [
            "Simplify control flow exits:",
            "- **Expression-Oriented**: In Rust, the last expression is the return value. Use it.",
            "- **Guard Clauses**: Return early for error checks, then have a single success path.",
            "- **Result**: Propagate errors with `?` rather than manual early returns.",
        ]
    )


STRATEGIES = {
    s.rule: s()
    for s in (
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
    )
}


def get_strategy(rule_id: str) -> FixStrategy | None:
    # Rule ID might be "qlty:similar-code" or just "similar-code"
    short = rule_id.split(":")[-1]
    return STRATEGIES.get(short)


def parse_sarif_file(path: Path) -> list[SarifIssue]:
    """Parse SARIF JSON and extract all issues."""
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    issues = []
    for run in data.get("runs", []):
        results = run.get("results", [])
        for result in results:
            rule_id = result.get("ruleId", "unknown")
            level_str = result.get("level", "note")
            level = Severity.from_str(level_str)
            message = result.get("message", {}).get("text", "")

            # Extract location
            locations = result.get("locations", [])
            if not locations:
                continue

            physical_loc = locations[0].get("physicalLocation", {})
            artifact_loc = physical_loc.get("artifactLocation", {})
            file_path = artifact_loc.get("uri", "unknown")

            region = physical_loc.get("region", {})
            start_line = region.get("startLine", 0)
            end_line = region.get("endLine", start_line)

            # Extract metadata and fingerprints
            metadata = {}
            if "properties" in result:
                metadata = result["properties"]

            fingerprints = result.get("partialFingerprints", {})
            if not fingerprints:
                fingerprints = result.get("fingerprints", {})

            issues.append(
                SarifIssue(
                    rule_id=rule_id,
                    level=level,
                    message=message,
                    file_path=file_path,
                    start_line=start_line,
                    end_line=end_line,
                    metadata=metadata,
                    fingerprints=fingerprints,
                )
            )

    return issues


def run_qlty_check(
    output_file: Path = Path("qlty.check.current.sarif"),
) -> list[SarifIssue]:
    """Run qlty check --all --sarif, save to file, and parse SARIF output."""
    print("🔄 Running qlty check --all --sarif...")

    try:
        result = subprocess.run(  # nosec B603 B607
            ["qlty", "check", "--all", "--sarif"],
            capture_output=True,
            text=True,
            timeout=300,
        )

        if not result.stdout.strip():
            print("   ✅ No issues found (clean)")
            return []

        output_file.write_text(result.stdout)
        print(f"   💾 Saved SARIF to {output_file}")

        issues = parse_sarif_file(output_file)
        print(f"   📊 Found {len(issues)} issues")
        return issues

    except subprocess.TimeoutExpired:
        print("   ❌ qlty check timed out after 300s", file=sys.stderr)
        return []
    except Exception as e:
        print(f"   ❌ Error running qlty: {e}", file=sys.stderr)
        return []


def run_qlty_smells(
    output_file: Path = Path("qlty.smells.sarif"),
) -> list[SarifIssue]:
    """Run qlty smells --all --sarif, save to file, and parse SARIF output."""
    print("🔄 Running qlty smells --all --sarif...")

    try:
        result = subprocess.run(  # nosec B603 B607
            ["qlty", "smells", "--all", "--sarif"],
            capture_output=True,
            text=True,
            timeout=300,
        )

        if not result.stdout.strip():
            print("   ✅ No smells found (clean)")
            return []

        output_file.write_text(result.stdout)
        print(f"   💾 Saved SARIF to {output_file}")

        issues = parse_sarif_file(output_file)
        # Mark issues as 'smell' category if not present
        for issue in issues:
            if not issue.category:
                issue.category = "smell"

        print(f"   📊 Found {len(issues)} smells")
        return issues

    except subprocess.TimeoutExpired:
        print("   ❌ qlty smells timed out after 300s", file=sys.stderr)
        return []
    except Exception as e:
        print(f"   ❌ Error running qlty smells: {e}", file=sys.stderr)
        return []


@dataclass
class AnalysisReport:
    """Statistical analysis of SARIF issues."""

    total_issues: int = 0
    by_severity: Counter = field(default_factory=Counter)
    by_rule: Counter = field(default_factory=Counter)
    by_category: Counter = field(default_factory=Counter)
    by_file: Counter = field(default_factory=Counter)
    top_files: list[tuple[str, int]] = field(default_factory=list)
    top_rules: list[tuple[str, int]] = field(default_factory=list)
    issues: list[SarifIssue] = field(default_factory=list)

    def generate_summary(self) -> str:
        """Generate human-readable summary."""
        lines = []
        lines.append("━" * 72)
        lines.append(f"📊 ANALYSIS SUMMARY: {self.total_issues} issues")
        lines.append("━" * 72)
        lines.append("")

        # Severity breakdown
        lines.append("## By Severity")
        lines.append("")
        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"{sev.to_emoji()} {sev.name:8s} {count:4d} ({pct:5.1f}%)")
        lines.append("")

        # Category breakdown
        lines.append("## By Category")
        lines.append("")
        for cat, count in self.by_category.most_common(10):
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"  {cat:20s} {count:4d} ({pct:5.1f}%)")
        lines.append("")

        # Top rules
        lines.append("## Top 10 Rules")
        lines.append("")
        for rule, count in self.top_rules[:10]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"  {count:4d} ({pct:5.1f}%)  {rule}")
        lines.append("")

        # Top files
        lines.append("## Top 10 Files")
        lines.append("")
        for file_path, count in self.top_files[:10]:
            lines.append(f"  {count:4d}  {file_path}")
        lines.append("")

        lines.append("━" * 72)
        return "\n".join(lines)

    def _generate_severity_table(self, lines):
        lines.append("## Severity Distribution")
        lines.append("")
        lines.append("| Severity | Count | Percentage |")
        lines.append("| ---------- | ------- | ------------ |")
        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {sev.to_emoji()} {sev.name} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_category_table(self, lines):
        lines.append("## Category Breakdown")
        lines.append("")
        lines.append("| Category | Count | Percentage |")
        lines.append("| ---------- | ------- | ------------ |")
        for cat, count in self.by_category.most_common():
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {cat} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_rules_table(self, lines):
        lines.append("## Top Rules")
        lines.append("")
        lines.append("| Rule | Count | Percentage |")
        lines.append("| ------ | ------- | ------------ |")
        for rule, count in self.top_rules[:20]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| `{rule}` | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_files_table(self, lines):
        lines.append("## Most Affected Files")
        lines.append("")
        lines.append("| File | Issues |")
        lines.append("| ------ | -------- |")
        for file_path, count in self.top_files[:20]:
            lines.append(f"| `{file_path}` | {count} |")
        lines.append("")

    def _generate_rule_section(self, lines, rule, rule_issues):
        lines.append(f"### {rule} ({len(rule_issues)} issues)")
        lines.append("")

        strategy = get_strategy(rule)
        if strategy:
            lines.append(f"**Strategy:** {strategy.title}")
            lines.append("")
            # Ensure blank line before list for MD032 compliance
            lines.append(strategy.instructions.replace(":\n-", ":\n\n-"))
            lines.append("")

        # Show up to 50 issues per rule to avoid massive files
        limit = 50
        count = len(rule_issues)

        for issue in rule_issues[:limit]:
            lines.append(f"#### `{issue.location_str}`")
            lines.append("")

            func = issue.fingerprints.get("function.name")
            if func:
                lines.append(f"- **Function:** `{func}`")

            msg = issue.message
            if msg:
                lines.append(f"- **Message:** {msg}")
            lines.append("")

        if count > limit:
            lines.append(f"*...and {count - limit} more issues.*")
            lines.append("")

    def _generate_severity_section(self, lines, sev):
        sev_issues = [i for i in self.issues if i.level == sev]
        if not sev_issues:
            return

        lines.append(f"## {sev.to_emoji()} {sev.name} Issues ({len(sev_issues)})")
        lines.append("")

        by_rule = defaultdict(list)
        for issue in sev_issues:
            by_rule[issue.rule_id].append(issue)

        for rule, rule_issues in sorted(
            by_rule.items(), key=lambda x: len(x[1]), reverse=True
        ):
            self._generate_rule_section(lines, rule, rule_issues)

    def generate_markdown(self, title: str = "Quality Analysis Report") -> str:
        """Generate detailed markdown report."""
        lines = []
        lines.append(f"# {title}")
        lines.append("")
        lines.append(f"**Total Issues:** {self.total_issues}")
        lines.append("")

        self._generate_severity_table(lines)
        self._generate_category_table(lines)
        self._generate_rules_table(lines)
        self._generate_files_table(lines)

        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            self._generate_severity_section(lines, sev)

        return "\n".join(lines)


def _populate_severity_counts(report, issues):
    for issue in issues:
        report.by_severity[issue.level] += 1


def _populate_category_and_rule_counts(report, issues):
    for issue in issues:
        report.by_rule[issue.rule_id] += 1
        report.by_category[issue.rule_category] += 1


def _populate_file_counts(report, issues):
    for issue in issues:
        report.by_file[issue.file_path] += 1


def analyze_issues(issues: list[SarifIssue]) -> AnalysisReport:
    """Generate statistical analysis of issues."""
    report = AnalysisReport()
    report.total_issues = len(issues)
    report.issues = issues

    _populate_severity_counts(report, issues)
    _populate_category_and_rule_counts(report, issues)
    _populate_file_counts(report, issues)

    report.top_files = report.by_file.most_common(20)
    report.top_rules = report.by_rule.most_common(20)

    return report


# ────────────────────────────────────────────────
# CLI Interface
# ────────────────────────────────────────────────


def _load_checks_from_file(args, all_issues):
    if args.checks_file and args.checks_file.exists():
        print(f"📖 Reading checks from {args.checks_file}")
        checks = parse_sarif_file(args.checks_file)
        for check in checks:
            check.category = "check"
        all_issues.extend(checks)
        print(f"   Found {len(checks)} check issues")
        return True
    return False


def _collect_smells_issues(args, all_issues):
    if args.smells_file.exists() and not args.scan:
        print(f"📖 Reading smells from {args.smells_file}")
        smells = parse_sarif_file(args.smells_file)
        for smell in smells:
            smell.category = "smell"
        all_issues.extend(smells)
        print(f"   Found {len(smells)} code smells")
    elif args.scan:
        smells = run_qlty_smells(args.smells_file)
        all_issues.extend(smells)
    else:
        print(f"⚠️  Smells file not found: {args.smells_file}", file=sys.stderr)


def _collect_checks_issues(args, all_issues) -> None:
    if args.scan:
        outfile = (
            args.checks_file if args.checks_file else Path("qlty.check.current.sarif")
        )
        checks = run_qlty_check(output_file=outfile)
        for check in checks:
            check.category = "check"
        all_issues.extend(checks)
    elif _load_checks_from_file(args, all_issues):
        return
    else:
        if args.checks_file:
            print(f"⚠️  Checks file not found: {args.checks_file}", file=sys.stderr)


def _collect_all_issues(args):
    all_issues = []

    # Determine types based on flags or default
    do_checks = args.type in ("checks", "both")
    do_smells = args.type in ("smells", "both")

    if args.check:
        do_checks = True
        if not args.smells and not args.type:
            do_smells = False  # Exclusive if only check specified without type

    if args.smells:
        do_smells = True
        if not args.check and not args.type:
            do_checks = False  # Exclusive if only smells specified without type

    # If explicit flags are used, they override default type behavior if contradictory?
    # Actually, simpler logic:
    # If --check or --smells provided, use them.
    # If neither provided, fall back to args.type.

    if args.check or args.smells:
        do_checks = args.check
        do_smells = args.smells
        # If user provided --check but not --smells, do_smells=False (default from args)
        # But wait, args.check is boolean.

    if do_checks:
        _collect_checks_issues(args, all_issues)

    if do_smells:
        _collect_smells_issues(args, all_issues)

    return all_issues


def _apply_severity_filter(args, filtered):
    if args.severity:
        target_sev = Severity.from_str(args.severity)
        filtered = [i for i in filtered if i.level == target_sev]
        print(f"🔍 Filtered to {len(filtered)} {args.severity} issues")
    return filtered


def _apply_rule_filter(args, filtered):
    if args.rule:
        filtered = [i for i in filtered if args.rule in i.rule_id]
        print(f"🔍 Filtered to {len(filtered)} issues matching rule '{args.rule}'")
    return filtered


def _apply_category_filter(args, filtered):
    if args.category:
        filtered = [i for i in filtered if args.category in i.rule_category]
        print(f"🔍 Filtered to {len(filtered)} issues in category '{args.category}'")
    return filtered


def _apply_file_filter(args, filtered):
    if args.file:
        import fnmatch

        filtered = [i for i in filtered if fnmatch.fnmatch(i.file_path, args.file)]
        print(f"🔍 Filtered to {len(filtered)} issues in files matching '{args.file}'")
    return filtered


def _apply_exclude_rule_filter(args, filtered):
    if args.exclude_rule:
        for rule in args.exclude_rule:
            filtered = [i for i in filtered if rule not in i.rule_id]
            print(f"🔍 Excluded issues matching rule '{rule}'")
    return filtered


def _apply_exclude_category_filter(args, filtered):
    if args.exclude_category:
        for cat in args.exclude_category:
            filtered = [i for i in filtered if cat not in i.rule_category]
            print(f"🔍 Excluded issues in category '{cat}'")
    return filtered


def _apply_exclude_file_filter(args, filtered):
    if args.exclude_file:
        import fnmatch

        for pattern in args.exclude_file:
            filtered = [
                i for i in filtered if not fnmatch.fnmatch(i.file_path, pattern)
            ]
            print(f"🔍 Excluded issues in files matching '{pattern}'")
    return filtered


def _apply_all_filters(args, all_issues):
    filtered = all_issues
    filtered = _apply_severity_filter(args, filtered)
    filtered = _apply_rule_filter(args, filtered)
    filtered = _apply_category_filter(args, filtered)
    filtered = _apply_file_filter(args, filtered)
    filtered = _apply_exclude_rule_filter(args, filtered)
    filtered = _apply_exclude_category_filter(args, filtered)
    filtered = _apply_exclude_file_filter(args, filtered)
    return filtered


def _validate_markdown_output_path(markdown_path: Path) -> Path:
    workspace_root = Path.cwd().resolve()
    resolved = markdown_path.resolve()

    if (
        resolved.suffix.lower() == ".md"
        and resolved.parent == workspace_root
        and resolved.name != "README.md"
    ):
        raise ValueError(
            "Writing markdown reports to workspace root is disabled; "
            "use README.md or a subdirectory (for example docs/reports/...)."
        )

    return resolved


def _generate_outputs(args, report, filtered):
    if args.summary:
        print()
        print(report.generate_summary())

    if args.markdown:
        title = f"Quality Report: {args.type.title()}"
        content = report.generate_markdown(title)
        try:
            markdown_path = _validate_markdown_output_path(args.markdown)
        except ValueError as e:
            print(f"❌ {e}", file=sys.stderr)
            raise SystemExit(2) from e
        markdown_path.write_text(content, encoding="utf-8")
        print(f"✅ Markdown report written to {markdown_path}")

    if args.json:
        output = {
            "total": report.total_issues,
            "by_severity": {k.name: v for k, v in report.by_severity.items()},
            "by_rule": dict(report.by_rule),
            "by_category": dict(report.by_category),
            "by_file": dict(report.by_file),
            "issues": [
                {
                    "rule_id": i.rule_id,
                    "level": i.level.name,
                    "message": i.message,
                    "location": i.location_str,
                    "file": i.file_path,
                    "line": i.start_line,
                    "category": i.category,
                }
                for i in filtered
            ],
        }
        args.json.write_text(json.dumps(output, indent=2), encoding="utf-8")
        print(f"✅ JSON report written to {args.json}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Analyze qlty SARIF reports (checks and smells)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "--type",
        choices=["checks", "smells", "both"],
        default="checks",
        help="Type of analysis to perform (default: checks)",
    )
    parser.add_argument(
        "--checks-file",
        type=Path,
        default=None,
        help="Path to checks SARIF file (default: run qlty check --all)",
    )
    parser.add_argument(
        "--smells-file",
        type=Path,
        default=Path("qlty.smells.lst"),
        help="Path to smells SARIF file (default: qlty.smells.lst)",
    )
    parser.add_argument(
        "--scan",
        action="store_true",
        help="Run qlty scan (check or smell) instead of just reading files",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Analyze checks",
    )
    parser.add_argument(
        "--smells",
        action="store_true",
        help="Analyze smells",
    )
    parser.add_argument(
        "--no-run",
        action="store_true",
        help="Deprecated: Use --scan to enable running. By default only reads files.",
    )
    parser.add_argument(
        "--severity",
        choices=["error", "warning", "note"],
        help="Filter by severity level",
    )
    parser.add_argument(
        "--rule",
        help="Filter by specific rule ID (e.g., rustfmt:fmt)",
    )
    parser.add_argument(
        "--category",
        help="Filter by category (e.g., rustfmt, zizmor, osv-scanner)",
    )
    parser.add_argument(
        "--file",
        help="Filter by file path (glob pattern supported)",
    )
    parser.add_argument(
        "--exclude-rule",
        action="append",
        help="Exclude issues matching rule ID (can be used multiple times)",
    )
    parser.add_argument(
        "--exclude-category",
        action="append",
        help="Exclude issues matching category (can be used multiple times)",
    )
    parser.add_argument(
        "--exclude-file",
        action="append",
        help="Exclude issues in files matching glob pattern (can be used multiple times)",
    )
    parser.add_argument(
        "--summary",
        action="store_true",
        help="Print summary to stdout",
    )
    parser.add_argument(
        "--markdown",
        type=Path,
        metavar="FILE",
        help="Generate markdown report",
    )
    parser.add_argument(
        "--json",
        type=Path,
        metavar="FILE",
        help="Export issues as JSON",
    )

    args = parser.parse_args()

    all_issues = _collect_all_issues(args)

    if not all_issues:
        print("❌ No issues found", file=sys.stderr)
        return 1

    filtered = _apply_all_filters(args, all_issues)
    report = analyze_issues(filtered)
    _generate_outputs(args, report, filtered)

    return 0


if __name__ == "__main__":
    sys.exit(main())
