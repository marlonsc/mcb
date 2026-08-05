"""Qlty Runner.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import subprocess  # nosec B404
from pathlib import Path

from lib.core import get_logger, r
from lib.settings import McbSettings

from qlty.model import SarifIssue
from qlty.parser import parse_sarif_file

logger = get_logger(__name__)


def run_qlty_check(
    output_file: Path | None = None,
) -> r[list[SarifIssue]]:
    """Run qlty check --all --sarif, save to file, and parse SARIF output."""
    output_file = output_file or McbSettings().qlty_check_sarif
    logger.info("Running qlty check --all --sarif...")

    try:
        result = subprocess.run(  # nosec B603 B607
            ["qlty", "check", "--all", "--sarif"],
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return r[list[SarifIssue]].fail("qlty check timed out after 300s")
    except (OSError, subprocess.SubprocessError) as exc:
        return r[list[SarifIssue]].fail(f"error running qlty check: {exc}")

    if not result.stdout.strip():
        logger.info("No issues found (clean)")
        return r[list[SarifIssue]].ok([])

    output_file.write_text(result.stdout, encoding="utf-8")
    logger.info(f"Saved SARIF to {output_file}")

    parsed = parse_sarif_file(output_file)
    if parsed.failure:
        return parsed
    issues = parsed.unwrap()
    logger.info(f"Found {len(issues)} issues")
    return r[list[SarifIssue]].ok(issues)


def run_qlty_smells(
    output_file: Path | None = None,
) -> r[list[SarifIssue]]:
    """Run qlty smells --all --sarif, save to file, and parse SARIF output."""
    output_file = output_file or McbSettings().qlty_smells_sarif
    logger.info("Running qlty smells --all --sarif...")

    try:
        result = subprocess.run(  # nosec B603 B607
            ["qlty", "smells", "--all", "--sarif"],
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return r[list[SarifIssue]].fail("qlty smells timed out after 300s")
    except (OSError, subprocess.SubprocessError) as exc:
        return r[list[SarifIssue]].fail(f"error running qlty smells: {exc}")

    if not result.stdout.strip():
        logger.info("No smells found (clean)")
        return r[list[SarifIssue]].ok([])

    output_file.write_text(result.stdout, encoding="utf-8")
    logger.info(f"Saved SARIF to {output_file}")

    parsed = parse_sarif_file(output_file)
    if parsed.failure:
        return parsed
    issues = parsed.unwrap()
    # Mark issues as 'smell' category if not present
    for issue in issues:
        if not issue.category:
            issue.category = "smell"

    logger.info(f"Found {len(issues)} smells")
    return r[list[SarifIssue]].ok(issues)
