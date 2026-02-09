#!/usr/bin/env python3
"""Unified SARIF analyzer for qlty check and qlty smells reports.

Processes SARIF output from both `qlty check --all` and `qlty smells`
to generate comprehensive reports, prioritized fix plans, and actionable
remediation strategies.

Usage examples::

    # Analyze checks only
    python scripts/analyze_qlty.py --type checks --summary

    # Analyze smells only
    python scripts/analyze_qlty.py --type smells --summary

    # Analyze both with full report
    python scripts/analyze_qlty.py --type both --summary --markdown FULL_REPORT.md

    # Filter by severity
    python scripts/analyze_qlty.py --type checks --severity error --summary

    # Filter by rule
    python scripts/analyze_qlty.py --type checks --rule rustfmt:fmt --summary

    # Generate combined report
    python scripts/analyze_qlty.py --type both --plan --output combined_plan.json
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import IntEnum
from pathlib import Path
from typing import Any, Sequence

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

    def generate_markdown(self, title: str = "Quality Analysis Report") -> str:
        """Generate detailed markdown report."""
        lines = []
        lines.append(f"# {title}")
        lines.append("")
        lines.append(f"**Total Issues:** {self.total_issues}")
        lines.append("")

        # Severity table
        lines.append("## Severity Distribution")
        lines.append("")
        lines.append("| Severity | Count | Percentage |")
        lines.append("|----------|-------|------------|")
        for sev in [Severity.ERROR, Severity.WARNING, Severity.NOTE]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {sev.to_emoji()} {sev.name} | {count} | {pct:.1f}% |")
        lines.append("")

        # Category table
        lines.append("## Category Breakdown")
        lines.append("")
        lines.append("| Category | Count | Percentage |")
        lines.append("|----------|-------|------------|")
        for cat, count in self.by_category.most_common():
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {cat} | {count} | {pct:.1f}% |")
        lines.append("")

        # Top rules
        lines.append("## Top Rules")
        lines.append("")
        lines.append("| Rule | Count | Percentage |")
        lines.append("|------|-------|------------|")
        for rule, count in self.top_rules[:20]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| `{rule}` | {count} | {pct:.1f}% |")
        lines.append("")

        # Top files
        lines.append("## Most Affected Files")
        lines.append("")
        lines.append("| File | Issues |")
        lines.append("|------|--------|")
        for file_path, count in self.top_files[:20]:
            lines.append(f"| `{file_path}` | {count} |")
        lines.append("")

        # Detailed issues by severity
        for sev in [Severity.ERROR, Severity.WARNING, Severity.NOTE]:
            sev_issues = [i for i in self.issues if i.level == sev]
            if not sev_issues:
                continue

            lines.append(f"## {sev.to_emoji()} {sev.name} Issues ({len(sev_issues)})")
            lines.append("")

            # Group by rule
            by_rule = defaultdict(list)
            for issue in sev_issues:
                by_rule[issue.rule_id].append(issue)

            for rule, rule_issues in sorted(
                by_rule.items(), key=lambda x: len(x[1]), reverse=True
            ):
                lines.append(f"### {rule} ({len(rule_issues)} issues)")
                lines.append("")

                # Show up to 10 examples
                for issue in rule_issues[:10]:
                    lines.append(f"- `{issue.location_str}`")
                    if issue.message:
                        # Truncate long messages
                        msg = (
                            issue.message[:100] + "..."
                            if len(issue.message) > 100
                            else issue.message
                        )
                        lines.append(f"  > {msg}")

                if len(rule_issues) > 10:
                    lines.append(f"  ... and {len(rule_issues) - 10} more")
                lines.append("")

        return "\n".join(lines)


def analyze_issues(issues: list[SarifIssue]) -> AnalysisReport:
    """Generate statistical analysis of issues."""
    report = AnalysisReport()
    report.total_issues = len(issues)
    report.issues = issues

    for issue in issues:
        report.by_severity[issue.level] += 1
        report.by_rule[issue.rule_id] += 1
        report.by_category[issue.rule_category] += 1
        report.by_file[issue.file_path] += 1

    report.top_files = report.by_file.most_common(20)
    report.top_rules = report.by_rule.most_common(20)

    return report


# ────────────────────────────────────────────────
# CLI Interface
# ────────────────────────────────────────────────


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Analyze qlty SARIF reports (checks and smells)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    # Input options
    parser.add_argument(
        "--type",
        choices=["checks", "smells", "both"],
        default="checks",
        help="Type of analysis to perform (default: checks)",
    )
    parser.add_argument(
        "--checks-file",
        type=Path,
        default=Path("qlty.check.lst"),
        help="Path to checks SARIF file (default: qlty.check.lst)",
    )
    parser.add_argument(
        "--smells-file",
        type=Path,
        default=Path("qlty.smells.lst"),
        help="Path to smells SARIF file (default: qlty.smells.lst)",
    )

    # Filtering options
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

    # Output options
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

    # Collect issues based on type
    all_issues = []

    if args.type in ("checks", "both"):
        if args.checks_file.exists():
            print(f"📖 Reading checks from {args.checks_file}")
            checks = parse_sarif_file(args.checks_file)
            for check in checks:
                check.category = "check"
            all_issues.extend(checks)
            print(f"   Found {len(checks)} check issues")
        else:
            print(f"⚠️  Checks file not found: {args.checks_file}", file=sys.stderr)

    if args.type in ("smells", "both"):
        if args.smells_file.exists():
            print(f"📖 Reading smells from {args.smells_file}")
            smells = parse_sarif_file(args.smells_file)
            for smell in smells:
                smell.category = "smell"
            all_issues.extend(smells)
            print(f"   Found {len(smells)} code smells")
        else:
            print(f"⚠️  Smells file not found: {args.smells_file}", file=sys.stderr)

    if not all_issues:
        print("❌ No issues found", file=sys.stderr)
        return 1

    # Apply filters
    filtered = all_issues

    if args.severity:
        target_sev = Severity.from_str(args.severity)
        filtered = [i for i in filtered if i.level == target_sev]
        print(f"🔍 Filtered to {len(filtered)} {args.severity} issues")

    if args.rule:
        filtered = [i for i in filtered if args.rule in i.rule_id]
        print(f"🔍 Filtered to {len(filtered)} issues matching rule '{args.rule}'")

    if args.category:
        filtered = [i for i in filtered if args.category in i.rule_category]
        print(f"🔍 Filtered to {len(filtered)} issues in category '{args.category}'")

    if args.file:
        import fnmatch

        filtered = [i for i in filtered if fnmatch.fnmatch(i.file_path, args.file)]
        print(f"🔍 Filtered to {len(filtered)} issues in files matching '{args.file}'")

    # Generate report
    report = analyze_issues(filtered)

    # Output
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

    return 0


if __name__ == "__main__":
    sys.exit(main())
