"""Qlty Main.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import fnmatch
from pathlib import Path

from flext_cli import cli

from flext_core import m, p
from mcb_scripts.core import get_logger, r
from mcb_scripts.qlty.model import QltyCategory, SarifIssue, Severity
from mcb_scripts.qlty.parser import parse_sarif_file
from mcb_scripts.qlty.report import analyze_issues
from mcb_scripts.qlty.runner import run_qlty_check, run_qlty_smells
from mcb_scripts.settings import McbSettings

logger = get_logger(__name__)


class QltyParams(m.BaseModel):
    """Command parameters for the qlty analysis verb."""

    scan: bool = False
    checks_file: Path | None = None
    smells_file: Path = m.Field(default_factory=lambda: McbSettings().qlty_smells_sarif)
    type: str = "both"
    check: bool = False
    smells: bool = False
    severity: str | None = None
    rule: str | None = None
    category: str | None = None
    file: str | None = None
    exclude_rule: list[str] = m.Field(default_factory=list)
    exclude_category: list[str] = m.Field(default_factory=list)
    exclude_file: list[str] = m.Field(default_factory=list)
    summary_only: bool = False
    report_file: Path = m.Field(default_factory=lambda: McbSettings().qlty_report_md)


def _load_checks_from_file(
    checks_file: Path, all_issues: list[SarifIssue],
) -> p.Result[None]:
    if not checks_file.exists():
        return r[None].ok(None)
    logger.info("📖 Reading checks from %s", checks_file)
    checks_result = parse_sarif_file(checks_file)
    if checks_result.failure:
        return r[None].from_failure(checks_result)
    checks = checks_result.unwrap()
    for check in checks:
        check.category = QltyCategory.CHECK
    all_issues.extend(checks)
    logger.info("   Found %s check issues", len(checks))
    return r[None].ok(None)


def _collect_smells_issues(
    params: QltyParams, all_issues: list[SarifIssue],
) -> p.Result[None]:
    if params.smells_file.exists() and not params.scan:
        logger.info("📖 Reading smells from %s", params.smells_file)
        smells_result = parse_sarif_file(params.smells_file)
        if smells_result.failure:
            return r[None].from_failure(smells_result)
        smells = smells_result.unwrap()
        for smell in smells:
            smell.category = QltyCategory.SMELL
        all_issues.extend(smells)
        logger.info("   Found %s code smells", len(smells))
    elif params.scan:
        smells_result = run_qlty_smells(
            params.smells_file or McbSettings().qlty_smells_sarif,
        )
        if smells_result.failure:
            return r[None].from_failure(smells_result)
        smells = smells_result.unwrap()
        all_issues.extend(smells)
    else:
        logger.error("⚠️  Smells file not found: %s", params.smells_file)
    return r[None].ok(None)


def _collect_checks_issues(
    params: QltyParams, all_issues: list[SarifIssue],
) -> p.Result[None]:
    if params.scan:
        outfile = params.checks_file or McbSettings().qlty_check_sarif
        checks_result = run_qlty_check(output_file=outfile)
        if checks_result.failure:
            return r[None].from_failure(checks_result)
        checks = checks_result.unwrap()
        for check in checks:
            check.category = QltyCategory.CHECK
        all_issues.extend(checks)
        return r[None].ok(None)

    if params.checks_file:
        loaded = _load_checks_from_file(params.checks_file, all_issues)
        if loaded.failure:
            return loaded
        return r[None].ok(None)

    logger.error("⚠️  No checks file specified and --scan not set")
    return r[None].ok(None)


def _resolve_issue_types(params: QltyParams) -> tuple[bool, bool]:
    do_checks = params.type in {"checks", "both"}
    do_smells = params.type in {"smells", "both"}

    if params.check:
        do_checks = True
        if not params.smells and params.type == "both":
            do_smells = False

    if params.smells:
        do_smells = True
        if not params.check and params.type == "both":
            do_checks = False

    if params.check or params.smells:
        do_checks = params.check
        do_smells = params.smells

    return do_checks, do_smells


def _collect_all_issues(params: QltyParams) -> p.Result[list[SarifIssue]]:
    all_issues: list[SarifIssue] = []
    do_checks, do_smells = _resolve_issue_types(params)

    if do_checks:
        checks_result = _collect_checks_issues(params, all_issues)
        if checks_result.failure:
            return r[list[SarifIssue]].from_failure(checks_result)

    if do_smells:
        smells_result = _collect_smells_issues(params, all_issues)
        if smells_result.failure:
            return r[list[SarifIssue]].from_failure(smells_result)

    return r[list[SarifIssue]].ok(all_issues)


def _apply_severity_filter(
    severity: str | None, filtered: list[SarifIssue],
) -> list[SarifIssue]:
    if severity:
        target_sev = Severity.from_str(severity)
        filtered = [i for i in filtered if i.level == target_sev]
        logger.info("🔍 Filtered to %s %s issues", len(filtered), severity)
    return filtered


def _apply_rule_filter(
    rule: str | None, filtered: list[SarifIssue],
) -> list[SarifIssue]:
    if rule:
        filtered = [i for i in filtered if rule in i.rule_id]
        logger.info("🔍 Filtered to %s issues matching rule '%s'", len(filtered), rule)
    return filtered


def _apply_category_filter(
    category: str | None, filtered: list[SarifIssue],
) -> list[SarifIssue]:
    if category:
        filtered = [i for i in filtered if category in i.rule_category]
        logger.info(
            "🔍 Filtered to %s issues in category '%s'", len(filtered), category,
        )
    return filtered


def _apply_file_filter(
    file_pattern: str | None, filtered: list[SarifIssue],
) -> list[SarifIssue]:
    if file_pattern:
        filtered = [i for i in filtered if fnmatch.fnmatch(i.file_path, file_pattern)]
        logger.info(
            "🔍 Filtered to %s issues in files matching '%s'",
            len(filtered),
            file_pattern,
        )
    return filtered


def _apply_exclude_rule_filter(
    exclude_rules: list[str], filtered: list[SarifIssue],
) -> list[SarifIssue]:
    for rule in exclude_rules:
        filtered = [i for i in filtered if rule not in i.rule_id]
        logger.info("🔍 Excluded issues matching rule '%s'", rule)
    return filtered


def _apply_exclude_category_filter(
    exclude_categories: list[str], filtered: list[SarifIssue],
) -> list[SarifIssue]:
    for cat in exclude_categories:
        filtered = [i for i in filtered if cat not in i.rule_category]
        logger.info("🔍 Excluded issues in category '%s'", cat)
    return filtered


def _apply_exclude_file_filter(
    exclude_files: list[str], filtered: list[SarifIssue],
) -> list[SarifIssue]:
    for pattern in exclude_files:
        filtered = [i for i in filtered if not fnmatch.fnmatch(i.file_path, pattern)]
        logger.info("🔍 Excluded issues in files matching '%s'", pattern)
    return filtered


def analyze(params: QltyParams) -> p.Result[str]:
    """Analyze SARIF quality reports.

    Returns a short status string rather than the report object: the CLI facade
    serializes a successful result as a JSON value, and AnalysisReport carries
    Counter and dataclass members that are not JSON values. The full report is
    still emitted through the logger and, unless --summary-only, written to the
    report file.

    Returns:
        The resulting ``p.Result[str]``.
    """
    issues_result = _collect_all_issues(params)
    if issues_result.failure:
        return r[str].from_failure(issues_result)

    all_issues = issues_result.unwrap()

    if not all_issues:
        logger.info("✅ No issues found to analyze")
        return r[str].ok("no issues matched filters")

    filtered = all_issues
    filtered = _apply_severity_filter(params.severity, filtered)
    filtered = _apply_rule_filter(params.rule, filtered)
    filtered = _apply_category_filter(params.category, filtered)
    filtered = _apply_file_filter(params.file, filtered)
    filtered = _apply_exclude_rule_filter(params.exclude_rule, filtered)
    filtered = _apply_exclude_category_filter(params.exclude_category, filtered)
    filtered = _apply_exclude_file_filter(params.exclude_file, filtered)

    if not filtered:
        logger.info("✅ No issues matched filters")
        return r[str].ok("no issues matched filters")

    report_result = analyze_issues(filtered)
    if report_result.failure:
        return r[str].from_failure(report_result)
    report = report_result.unwrap()

    logger.info("\n%s", report.generate_summary())

    if not params.summary_only:
        md_content = report.generate_markdown()
        params.report_file.write_text(md_content, encoding="utf-8")
        logger.info("\n📝 Detailed report written to %s", params.report_file)

    return r[str].ok(f"{report.total_issues} issues analyzed")


def main() -> None:
    """Entry point for the qlty SARIF analysis command.

    Raises:
        SystemExit: If ``result.failure``.
    """
    app = cli.create_app_with_common_params(
        name="qlty", help_text="Analyze SARIF quality reports.",
    )
    cli.register_result_command(
        app,
        name="analyze",
        help_text="Analyze SARIF quality reports.",
        model_cls=QltyParams,
        handler=analyze,
    )
    result = cli.execute_app(app, prog_name="qlty")
    if result.failure:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
