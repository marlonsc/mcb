"""Main CLI entry point for qlty analysis."""

import argparse
import sys
from pathlib import Path

from qlty.model import Severity, SarifIssue
from qlty.parser import parse_sarif_file
from qlty.runner import run_qlty_check, run_qlty_smells
from qlty.report import analyze_issues


def _load_checks_from_file(
    args: argparse.Namespace, all_issues: list[SarifIssue]
) -> bool:
    if args.checks_file and args.checks_file.exists():
        print(f"📖 Reading checks from {args.checks_file}")
        checks = parse_sarif_file(args.checks_file)
        for check in checks:
            check.category = "check"
        all_issues.extend(checks)
        print(f"   Found {len(checks)} check issues")
        return True
    return False


def _collect_smells_issues(
    args: argparse.Namespace, all_issues: list[SarifIssue]
) -> None:
    if args.smells_file.exists() and not args.scan:
        print(f"📖 Reading smells from {args.smells_file}")
        smells = parse_sarif_file(args.smells_file)
        for smell in smells:
            smell.category = "smell"
        all_issues.extend(smells)
        print(f"   Found {len(smells)} code smells")
    elif args.scan:
        smells = run_qlty_smells(args.smells_file or Path("qlty.smells.sarif"))
        all_issues.extend(smells)
    else:
        print(f"⚠️  Smells file not found: {args.smells_file}", file=sys.stderr)


def _collect_checks_issues(
    args: argparse.Namespace, all_issues: list[SarifIssue]
) -> None:
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


def _collect_all_issues(args: argparse.Namespace) -> list[SarifIssue]:
    all_issues: list[SarifIssue] = []

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

    if args.check or args.smells:
        do_checks = args.check
        do_smells = args.smells

    if do_checks:
        _collect_checks_issues(args, all_issues)

    if do_smells:
        _collect_smells_issues(args, all_issues)

    return all_issues


def _apply_severity_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.severity:
        target_sev = Severity.from_str(args.severity)
        filtered = [i for i in filtered if i.level == target_sev]
        print(f"🔍 Filtered to {len(filtered)} {args.severity} issues")
    return filtered


def _apply_rule_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.rule:
        filtered = [i for i in filtered if args.rule in i.rule_id]
        print(f"🔍 Filtered to {len(filtered)} issues matching rule '{args.rule}'")
    return filtered


def _apply_category_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.category:
        filtered = [i for i in filtered if args.category in i.rule_category]
        print(f"🔍 Filtered to {len(filtered)} issues in category '{args.category}'")
    return filtered


def _apply_file_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.file:
        import fnmatch

        filtered = [i for i in filtered if fnmatch.fnmatch(i.file_path, args.file)]
        print(f"🔍 Filtered to {len(filtered)} issues in files matching '{args.file}'")
    return filtered


def _apply_exclude_rule_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.exclude_rule:
        for rule in args.exclude_rule:
            filtered = [i for i in filtered if rule not in i.rule_id]
            print(f"🔍 Excluded issues matching rule '{rule}'")
    return filtered


def _apply_exclude_category_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.exclude_category:
        for cat in args.exclude_category:
            filtered = [i for i in filtered if cat not in i.rule_category]
            print(f"🔍 Excluded issues in category '{cat}'")
    return filtered


def _apply_exclude_file_filter(
    args: argparse.Namespace, filtered: list[SarifIssue]
) -> list[SarifIssue]:
    if args.exclude_file:
        import fnmatch

        for pattern in args.exclude_file:
            filtered = [
                i for i in filtered if not fnmatch.fnmatch(i.file_path, pattern)
            ]
            print(f"🔍 Excluded issues in files matching '{pattern}'")
    return filtered


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Analyze SARIF quality reports")

    # Mode selection
    parser.add_argument(
        "--scan", action="store_true", help="Run qlty scan instead of reading files"
    )

    # Input files
    parser.add_argument(
        "--checks-file",
        type=Path,
        help="SARIF file for qlty checks (default: qlty.check.current.sarif)",
    )
    parser.add_argument(
        "--smells-file",
        type=Path,
        default=Path("qlty.smells.sarif"),
        help="SARIF file for qlty smells (default: qlty.smells.sarif)",
    )

    # Issue type selection
    parser.add_argument(
        "--type",
        choices=["both", "checks", "smells"],
        default="both",
        help="Types of issues to analyze (default: both)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Analyze checks only (alias for --type checks)",
    )
    parser.add_argument(
        "--smells",
        action="store_true",
        help="Analyze smells only (alias for --type smells)",
    )

    # Filters
    parser.add_argument(
        "--severity",
        choices=["error", "warning", "note"],
        help="Filter by severity",
    )
    parser.add_argument("--rule", help="Filter by rule ID substring")
    parser.add_argument("--category", help="Filter by rule category")
    parser.add_argument("--file", help="Filter by file path pattern (glob)")

    # Exclusions
    parser.add_argument(
        "--exclude-rule",
        action="append",
        help="Exclude rule ID substring (can be used multiple times)",
    )
    parser.add_argument(
        "--exclude-category",
        action="append",
        help="Exclude rule category (can be used multiple times)",
    )
    parser.add_argument(
        "--exclude-file",
        action="append",
        help="Exclude file path pattern (can be used multiple times)",
    )

    # Output control
    parser.add_argument(
        "--summary-only", action="store_true", help="Print only summary, no report"
    )
    parser.add_argument(
        "--report-file",
        type=Path,
        default=Path("QUALITY_REPORT.md"),
        help="Output markdown report file (default: QUALITY_REPORT.md)",
    )

    args = parser.parse_args()

    # Collect issues
    all_issues = _collect_all_issues(args)

    if not all_issues:
        print("✅ No issues found to analyze")
        return 0

    # Apply filters
    filtered = all_issues
    filtered = _apply_severity_filter(args, filtered)
    filtered = _apply_rule_filter(args, filtered)
    filtered = _apply_category_filter(args, filtered)
    filtered = _apply_file_filter(args, filtered)
    filtered = _apply_exclude_rule_filter(args, filtered)
    filtered = _apply_exclude_category_filter(args, filtered)
    filtered = _apply_exclude_file_filter(args, filtered)

    if not filtered:
        print("✅ No issues matched filters")
        return 0

    # Analyze
    report = analyze_issues(filtered)

    # Print summary
    print("\n" + report.generate_summary())

    # Generate Markdown Report
    if not args.summary_only:
        md_content = report.generate_markdown()
        args.report_file.write_text(md_content, encoding="utf-8")
        print(f"\n📝 Detailed report written to {args.report_file}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
