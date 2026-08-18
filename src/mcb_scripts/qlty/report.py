"""Qlty Report.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from collections import Counter, defaultdict
from dataclasses import dataclass, field

from flext_core import p
from mcb_scripts.core import r

from mcb_scripts.qlty.model import SarifIssue, Severity
from mcb_scripts.qlty.strategies import get_strategy


@dataclass
class AnalysisReport:
    """Statistical analysis of SARIF issues."""

    total_issues: int = 0
    by_severity: Counter[Severity] = field(default_factory=Counter)
    by_rule: Counter[str] = field(default_factory=Counter)
    by_category: Counter[str] = field(default_factory=Counter)
    by_file: Counter[str] = field(default_factory=Counter)
    top_files: list[tuple[str, int]] = field(default_factory=list)
    top_rules: list[tuple[str, int]] = field(default_factory=list)
    issues: list[SarifIssue] = field(default_factory=list)

    def generate_summary(self) -> str:
        """Generate human-readable summary."""
        lines: list[str] = []
        lines.extend((
            "━" * 72,
            f"📊 ANALYSIS SUMMARY: {self.total_issues} issues",
            "━" * 72,
            "",
            # Severity breakdown
            "## By Severity",
            "",
        ))
        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"{sev.to_emoji()} {sev.name:8s} {count:4d} ({pct:5.1f}%)")
        lines.extend(("", "## By Category", ""))
        for cat, count in self.by_category.most_common(10):
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"  {cat:20s} {count:4d} ({pct:5.1f}%)")
        lines.extend(("", "## Top 10 Rules", ""))
        for rule, count in self.top_rules[:10]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"  {count:4d} ({pct:5.1f}%)  {rule}")
        lines.extend(("", "## Top 10 Files", ""))
        for file_path, count in self.top_files[:10]:
            lines.append(f"  {count:4d}  {file_path}")
        lines.extend(("", "━" * 72))
        return "\n".join(lines)

    def _generate_severity_table(self, lines: list[str]) -> None:
        lines.extend((
            "## Severity Distribution",
            "",
            "| Severity | Count | Percentage |",
            "| ---------- | ------- | ------------ |",
        ))
        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {sev.to_emoji()} {sev.name} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_category_table(self, lines: list[str]) -> None:
        lines.extend((
            "## Category Breakdown",
            "",
            "| Category | Count | Percentage |",
            "| ---------- | ------- | ------------ |",
        ))
        for cat, count in self.by_category.most_common():
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {cat} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_rules_table(self, lines: list[str]) -> None:
        lines.extend((
            "## Top Rules",
            "",
            "| Rule | Count | Percentage |",
            "| ------ | ------- | ------------ |",
        ))
        for rule, count in self.top_rules[:20]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| `{rule}` | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_files_table(self, lines: list[str]) -> None:
        lines.extend((
            "## Most Affected Files",
            "",
            "| File | Issues |",
            "| ------ | -------- |",
        ))
        for file_path, count in self.top_files[:20]:
            lines.append(f"| `{file_path}` | {count} |")
        lines.append("")

    def _generate_rule_section(
        self, lines: list[str], rule: str, rule_issues: list[SarifIssue]
    ) -> None:
        lines.extend((f"### {rule} ({len(rule_issues)} issues)", ""))

        strategy = get_strategy(rule)
        if strategy:
            # Blank line before the list keeps MD032 satisfied.
            lines.extend((
                f"**Strategy:** {strategy.title}",
                "",
                strategy.instructions.replace(":\\n-", ":\\n\\n-"),
            ))
            lines.append("")

        # Show up to 50 issues per rule to avoid massive files
        limit = 50
        count = len(rule_issues)

        for issue in rule_issues[:limit]:
            lines.extend((f"#### `{issue.location_str}`", ""))

            func = issue.fingerprints.get("function.name")
            if func:
                lines.append(f"- **Function:** `{func}`")

            msg = issue.message
            if msg:
                lines.append(f"- **Message:** {msg}")
            lines.append("")

        if count > limit:
            lines.extend((f"*...and {count - limit} more issues.*", ""))

    def _generate_severity_section(self, lines: list[str], sev: Severity) -> None:
        sev_issues = [i for i in self.issues if i.level == sev]
        if not sev_issues:
            return

        lines.extend((f"## {sev.to_emoji()} {sev.name} Issues ({len(sev_issues)})", ""))

        by_rule: defaultdict[str, list[SarifIssue]] = defaultdict(list)
        for issue in sev_issues:
            by_rule[issue.rule_id].append(issue)

        def _issue_count(item: tuple[str, list[SarifIssue]]) -> int:
            return len(item[1])

        for rule, rule_issues in sorted(
            by_rule.items(), key=_issue_count, reverse=True
        ):
            self._generate_rule_section(lines, rule, rule_issues)

    def generate_markdown(self, title: str = "Quality Analysis Report") -> str:
        """Generate detailed markdown report."""
        lines: list[str] = []
        lines.extend((f"# {title}", "", f"**Total Issues:** {self.total_issues}", ""))

        self._generate_severity_table(lines)
        self._generate_category_table(lines)
        self._generate_rules_table(lines)
        self._generate_files_table(lines)

        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            self._generate_severity_section(lines, sev)

        return "\n".join(lines)


def _populate_severity_counts(report: AnalysisReport, issues: list[SarifIssue]) -> None:
    for issue in issues:
        report.by_severity[issue.level] += 1


def _populate_category_and_rule_counts(
    report: AnalysisReport, issues: list[SarifIssue]
) -> None:
    for issue in issues:
        report.by_rule[issue.rule_id] += 1
        report.by_category[issue.rule_category] += 1


def _populate_file_counts(report: AnalysisReport, issues: list[SarifIssue]) -> None:
    for issue in issues:
        report.by_file[issue.file_path] += 1


def analyze_issues(issues: list[SarifIssue]) -> p.Result[AnalysisReport]:
    """Generate statistical analysis of issues."""
    report = AnalysisReport()
    report.total_issues = len(issues)
    report.issues = issues

    _populate_severity_counts(report, issues)
    _populate_category_and_rule_counts(report, issues)
    _populate_file_counts(report, issues)

    report.top_files = report.by_file.most_common(20)
    report.top_rules = report.by_rule.most_common(20)

    return r[AnalysisReport].ok(report)
