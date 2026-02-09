#!/usr/bin/env python3
"""
SARIF Quality Analysis Tool - Parses qlty check SARIF output and provides statistics.
"""

import argparse
import json
import subprocess  # nosec B404
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import Enum, IntEnum
from pathlib import Path
from typing import Any

# ────────────────────────────────────────────────
# Constants & Configuration
# ────────────────────────────────────────────────


class Severity(IntEnum):
    """Severity levels mapped from SARIF."""

    ERROR = 3
    WARNING = 2
    NOTE = 1
    NONE = 0

    @classmethod
    def from_str(cls, s: str) -> Severity:
        mapping = {
            "error": cls.ERROR,
            "warning": cls.WARNING,
            "note": cls.NOTE,
        }
        return mapping.get(s.lower(), cls.NONE)

    def to_emoji(self) -> str:
        return {
            self.ERROR: "🔴",
            self.WARNING: "🟠",
            self.NOTE: "🔵",
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
# SARIF Parsers
# ────────────────────────────────────────────────


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

            # Extract metadata
            metadata = {}
            if "properties" in result:
                metadata = result["properties"]

            issues.append(
                SarifIssue(
                    rule_id=rule_id,
                    level=level,
                    message=message,
                    file_path=file_path,
                    start_line=start_line,
                    end_line=end_line,
                    metadata=metadata,
                )
            )

    return issues


def run_qlty_check(
    output_file: Path = Path("qlty.check.current.sarif"),
) -> list[SarifIssue]:
    """Run qlty check --all --sarif, save to file, and parse SARIF output."""
    print(f"🔄 Running qlty check --all --sarif...")

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


# ────────────────────────────────────────────────
# Analysis & Reporting
# ────────────────────────────────────────────────


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
        for sev in [Severity.ERROR, Severity.WARNING, Severity.NOTE]:
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
        lines.append("|----------|-------|------------|")
        for sev in [Severity.ERROR, Severity.WARNING, Severity.NOTE]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {sev.to_emoji()} {sev.name} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_category_table(self, lines):
        lines.append("## Category Breakdown")
        lines.append("")
        lines.append("| Category | Count | Percentage |")
        lines.append("|----------|-------|------------|")
        for cat, count in self.by_category.most_common():
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {cat} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_rules_table(self, lines):
        lines.append("## Top Rules")
        lines.append("")
        lines.append("| Rule | Count | Percentage |")
        lines.append("|------|-------|------------|")
        for rule, count in self.top_rules[:20]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| `{rule}` | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_files_table(self, lines):
        lines.append("## Most Affected Files")
        lines.append("")
        lines.append("| File | Issues |")
        lines.append("|------|--------|")
        for file_path, count in self.top_files[:20]:
            lines.append(f"| `{file_path}` | {count} |")
        lines.append("")

    def _format_issue_message(self, issue):
        if not issue.message:
            return None
        if len(issue.message) > 100:
            return issue.message[:100] + "..."
        return issue.message

    def _generate_rule_section(self, lines, rule, rule_issues):
        lines.append(f"### {rule} ({len(rule_issues)} issues)")
        lines.append("")

        for issue in rule_issues[:10]:
            lines.append(f"- `{issue.location_str}`")
            msg = self._format_issue_message(issue)
            if msg:
                lines.append(f"  > {msg}")

        if len(rule_issues) > 10:
            lines.append(f"  ... and {len(rule_issues) - 10} more")
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

        for sev in [Severity.ERROR, Severity.WARNING, Severity.NOTE]:
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


def _run_qlty_checks(all_issues):
    checks = run_qlty_check()
    for check in checks:
        check.category = "check"
    all_issues.extend(checks)


def _load_smells_from_file(args, all_issues):
    if args.smells_file.exists():
        print(f"📖 Reading smells from {args.smells_file}")
        smells = parse_sarif_file(args.smells_file)
        for smell in smells:
            smell.category = "smell"
        all_issues.extend(smells)
        print(f"   Found {len(smells)} code smells")
    else:
        print(f"⚠️  Smells file not found: {args.smells_file}", file=sys.stderr)


def _collect_checks_issues(args, all_issues):
    if _load_checks_from_file(args, all_issues):
        return
    if not args.no_run:
        _run_qlty_checks(all_issues)
    else:
        print(f"⚠️  No checks file and --no-run specified", file=sys.stderr)


def _collect_all_issues(args):
    all_issues = []

    if args.type in ("checks", "both"):
        _collect_checks_issues(args, all_issues)

    if args.type in ("smells", "both"):
        _load_smells_from_file(args, all_issues)

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


def _apply_all_filters(args, all_issues):
    filtered = all_issues
    filtered = _apply_severity_filter(args, filtered)
    filtered = _apply_rule_filter(args, filtered)
    filtered = _apply_category_filter(args, filtered)
    filtered = _apply_file_filter(args, filtered)
    return filtered


def _generate_outputs(args, report, filtered):
    if args.summary:
        print()
        print(report.generate_summary())

    if args.markdown:
        title = f"Quality Report: {args.type.title()}"
        content = report.generate_markdown(title)
        args.markdown.write_text(content, encoding="utf-8")
        print(f"✅ Markdown report written to {args.markdown}")

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
        "--no-run",
        action="store_true",
        help="Don't run qlty, only use existing files",
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
