"""Qlty Main.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import fnmatch
from pathlib import Path

import typer
from flext_core import p
from lib.cli import create_app_with_common_params, register_result_command
from lib.core import get_logger, r
from lib.settings import McbSettings
from pydantic import BaseModel, Field

from qlty.model import SarifIssue, Severity
from qlty.parser import parse_sarif_file
from qlty.report import AnalysisReport, analyze_issues
from qlty.runner import run_qlty_check, run_qlty_smells

logger = get_logger(__name__)


class QltyParams(BaseModel):
    scan: bool = False
    checks_file: Path | None = None
    smells_file: Path = Field(default_factory=lambda: McbSettings().qlty_smells_sarif)
    type: str = "both"
    check: bool = False
    smells: bool = False
    severity: str | None = None
    rule: str | None = None
    category: str | None = None
    file: str | None = None
    exclude_rule: list[str] = Field(default_factory=list)
    exclude_category: list[str] = Field(default_factory=list)
    exclude_file: list[str] = Field(default_factory=list)
    summary_only: bool = False
    report_file: Path = Field(default_factory=lambda: McbSettings().qlty_report_md)


def _load_checks_from_file(checks_file: Path, all_issues: list[SarifIssue]) -> r[None]:
    if not checks_file.exists():
        return r[None].ok(None)
    logger.info(f"📖 Reading checks from {checks_file}")
    checks_result = parse_sarif_file(checks_file)
    if checks_result.failure:
        return r[None].fail(checks_result.error or "failed to parse checks file")
    checks = checks_result.unwrap()
    for check in checks:
        check.category = "check"
    all_issues.extend(checks)
    logger.info(f"   Found {len(checks)} check issues")
    return r[None].ok(None)


def _collect_smells_issues(params: QltyParams, all_issues: list[SarifIssue]) -> r[None]:
    if params.smells_file.exists() and not params.scan:
        logger.info(f"📖 Reading smells from {params.smells_file}")
        smells_result = parse_sarif_file(params.smells_file)
        if smells_result.failure:
            return r[None].fail(smells_result.error or "failed to parse smells file")
        smells = smells_result.unwrap()
        for smell in smells:
            smell.category = "smell"
        all_issues.extend(smells)
        logger.info(f"   Found {len(smells)} code smells")
    elif params.scan:
        smells_result = run_qlty_smells(
            params.smells_file or McbSettings().qlty_smells_sarif
        )
        if smells_result.failure:
            return r[None].fail(smells_result.error or "qlty smells failed")
        smells = smells_result.unwrap()
        all_issues.extend(smells)
    else:
        logger.error(f"⚠️  Smells file not found: {params.smells_file}")
    return r[None].ok(None)


def _collect_checks_issues(params: QltyParams, all_issues: list[SarifIssue]) -> r[None]:
    if params.scan:
        outfile = (
            params.checks_file if params.checks_file else McbSettings().qlty_check_sarif
        )
        checks_result = run_qlty_check(output_file=outfile)
        if checks_result.failure:
            return r[None].fail(checks_result.error or "qlty check failed")
        checks = checks_result.unwrap()
        for check in checks:
            check.category = "check"
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
    do_checks = params.type in ("checks", "both")
    do_smells = params.type in ("smells", "both")

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


def _collect_all_issues(params: QltyParams) -> r[list[SarifIssue]]:
    all_issues: list[SarifIssue] = []
    do_checks, do_smells = _resolve_issue_types(params)

    if do_checks:
        checks_result = _collect_checks_issues(params, all_issues)
        if checks_result.failure:
            return r[list[SarifIssue]].fail(
                checks_result.error or "checks collection failed"
            )

    if do_smells:
        smells_result = _collect_smells_issues(params, all_issues)
        if smells_result.failure:
            return r[list[SarifIssue]].fail(
                smells_result.error or "smells collection failed"
            )

    return r[list[SarifIssue]].ok(all_issues)


def _apply_severity_filter(
    severity: str | None, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if severity:
        target_sev = Severity.from_str(severity)
        filtered = [i for i in filtered if i.level == target_sev]
        logger.info(f"🔍 Filtered to {len(filtered)} {severity} issues")
    return filtered


def _apply_rule_filter(
    rule: str | None, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if rule:
        filtered = [i for i in filtered if rule in i.rule_id]
        logger.info(f"🔍 Filtered to {len(filtered)} issues matching rule '{rule}'")
    return filtered


def _apply_category_filter(
    category: str | None, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if category:
        filtered = [i for i in filtered if category in i.rule_category]
        logger.info(f"🔍 Filtered to {len(filtered)} issues in category '{category}'")
    return filtered


def _apply_file_filter(
    file_pattern: str | None, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if file_pattern:
        filtered = [i for i in filtered if fnmatch.fnmatch(i.file_path, file_pattern)]
        logger.info(
            f"🔍 Filtered to {len(filtered)} issues in files matching '{file_pattern}'"
        )
    return filtered


def _apply_exclude_rule_filter(
    exclude_rules: list[str], filtered: list[SarifIssue]
) -> list[SarifIssue]:
    for rule in exclude_rules:
        filtered = [i for i in filtered if rule not in i.rule_id]
        logger.info(f"🔍 Excluded issues matching rule '{rule}'")
    return filtered


def _apply_exclude_category_filter(
    exclude_categories: list[str], filtered: list[SarifIssue]
) -> list[SarifIssue]:
    for cat in exclude_categories:
        filtered = [i for i in filtered if cat not in i.rule_category]
        logger.info(f"🔍 Excluded issues in category '{cat}'")
    return filtered


def _apply_exclude_file_filter(
    exclude_files: list[str], filtered: list[SarifIssue]
) -> list[SarifIssue]:
    for pattern in exclude_files:
        filtered = [i for i in filtered if not fnmatch.fnmatch(i.file_path, pattern)]
        logger.info(f"🔍 Excluded issues in files matching '{pattern}'")
    return filtered


def analyze(params: QltyParams) -> p.Result[AnalysisReport]:
    """Analyze SARIF quality reports."""
    issues_result = _collect_all_issues(params)
    if issues_result.failure:
        return r[AnalysisReport].fail(issues_result.error or "issue collection failed")

    all_issues = issues_result.unwrap()

    if not all_issues:
        logger.info("✅ No issues found to analyze")
        return r[AnalysisReport].ok(AnalysisReport(issues=[]))

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
        return r[AnalysisReport].ok(AnalysisReport(issues=[]))

    report_result = analyze_issues(filtered)
    if report_result.failure:
        return r[AnalysisReport].fail(report_result.error or "analysis failed")
    report = report_result.unwrap()

    logger.info("\n" + report.generate_summary())

    if not params.summary_only:
        md_content = report.generate_markdown()
        params.report_file.write_text(md_content, encoding="utf-8")
        logger.info(f"\n📝 Detailed report written to {params.report_file}")

    return r[AnalysisReport].ok(report)


def main() -> None:
    app = create_app_with_common_params(
        name="qlty", help_text="Analyze SARIF quality reports."
    )
    register_result_command(
        app,
        name="analyze",
        help_text="Analyze SARIF quality reports.",
        model_cls=QltyParams,
        handler=analyze,
    )
    typer.main.get_command(app)()


if __name__ == "__main__":
    main()
