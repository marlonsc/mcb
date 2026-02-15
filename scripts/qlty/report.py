"""Reporting and analysis logic."""

from collections import Counter, defaultdict
from dataclasses import dataclass, field

from .model import SarifIssue, Severity
from .strategies import get_strategy


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
        return "\\n".join(lines)

    def _generate_severity_table(self, lines: list[str]) -> None:
        lines.append("## Severity Distribution")
        lines.append("")
        lines.append("| Severity | Count | Percentage |")
        lines.append("| ---------- | ------- | ------------ |")
        for sev in [Severity.ERROR, Severity.WARNING, Severity.INFO]:
            count = self.by_severity.get(sev, 0)
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {sev.to_emoji()} {sev.name} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_category_table(self, lines: list[str]) -> None:
        lines.append("## Category Breakdown")
        lines.append("")
        lines.append("| Category | Count | Percentage |")
        lines.append("| ---------- | ------- | ------------ |")
        for cat, count in self.by_category.most_common():
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| {cat} | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_rules_table(self, lines: list[str]) -> None:
        lines.append("## Top Rules")
        lines.append("")
        lines.append("| Rule | Count | Percentage |")
        lines.append("| ------ | ------- | ------------ |")
        for rule, count in self.top_rules[:20]:
            pct = (count / self.total_issues * 100) if self.total_issues > 0 else 0
            lines.append(f"| `{rule}` | {count} | {pct:.1f}% |")
        lines.append("")

    def _generate_files_table(self, lines: list[str]) -> None:
        lines.append("## Most Affected Files")
        lines.append("")
        lines.append("| File | Issues |")
        lines.append("| ------ | -------- |")
        for file_path, count in self.top_files[:20]:
            lines.append(f"| `{file_path}` | {count} |")
        lines.append("")

    def _generate_rule_section(
        self, lines: list[str], rule: str, rule_issues: list[SarifIssue]
    ) -> None:
        lines.append(f"### {rule} ({len(rule_issues)} issues)")
        lines.append("")

        strategy = get_strategy(rule)
        if strategy:
            lines.append(f"**Strategy:** {strategy.title}")
            lines.append("")
            # Ensure blank line before list for MD032 compliance
            lines.append(strategy.instructions.replace(":\\n-", ":\\n\\n-"))
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

    def _generate_severity_section(self, lines: list[str], sev: Severity) -> None:
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

        return "\\n".join(lines)


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
