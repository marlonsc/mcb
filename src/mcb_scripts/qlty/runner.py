"""Qlty Runner.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import shutil
import subprocess  # nosec B404 -- qlty runner: executes the pinned qlty CLI
from pathlib import Path

from flext_core import p
from mcb_scripts.core import get_logger, r
from mcb_scripts.qlty.model import QltyCategory, SarifIssue
from mcb_scripts.qlty.parser import parse_sarif_file
from mcb_scripts.settings import McbSettings

logger = get_logger(__name__)


def _resolve_qlty() -> str | None:
    """Resolve the qlty executable to an absolute path.

    Passing a bare name lets PATH order decide which binary runs; resolving
    it here pins the decision and turns a missing tool into a typed failure
    instead of an OSError raised from deep inside subprocess.

    Returns:
        The resulting ``str | None``.
    """
    return shutil.which("qlty")


def run_qlty_check(output_file: Path | None = None) -> p.Result[list[SarifIssue]]:
    """Run qlty check --all --sarif, save to file, and parse SARIF output.

    Returns:
        The resulting ``p.Result[list[SarifIssue]]``.
    """
    output_file = output_file or McbSettings().qlty_check_sarif
    logger.info("Running qlty check --all --sarif...")

    executable = _resolve_qlty()
    if executable is None:
        return r[list[SarifIssue]].fail("qlty executable not found on PATH")

    try:
        result = subprocess.run(
            [executable, "check", "--all", "--sarif"],  # nosec B603 -- executable is the mise-pinned qlty binary
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return r[list[SarifIssue]].fail("qlty check timed out after 300s")
    except (OSError, subprocess.SubprocessError) as exc:
        return r[list[SarifIssue]].fail(
            f"error running qlty check: {exc}", exception=exc,
        )

    if not result.stdout.strip():
        logger.info("No issues found (clean)")
        issues: list[SarifIssue] = []
        return r[list[SarifIssue]].ok(issues)

    output_file.write_text(result.stdout, encoding="utf-8")
    logger.info("Saved SARIF to %s", output_file)

    parsed = parse_sarif_file(output_file)
    if parsed.failure:
        return parsed
    issues = parsed.unwrap()
    logger.info(f"Found {len(issues)} issues")
    return r[list[SarifIssue]].ok(issues)


def run_qlty_smells(output_file: Path | None = None) -> p.Result[list[SarifIssue]]:
    """Run qlty smells --all --sarif, save to file, and parse SARIF output.

    Returns:
        The resulting ``p.Result[list[SarifIssue]]``.
    """
    output_file = output_file or McbSettings().qlty_smells_sarif
    logger.info("Running qlty smells --all --sarif...")

    executable = _resolve_qlty()
    if executable is None:
        return r[list[SarifIssue]].fail("qlty executable not found on PATH")

    try:
        result = subprocess.run(
            [executable, "smells", "--all", "--sarif"],  # nosec B603 -- executable is the mise-pinned qlty binary
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return r[list[SarifIssue]].fail("qlty smells timed out after 300s")
    except (OSError, subprocess.SubprocessError) as exc:
        return r[list[SarifIssue]].fail(
            f"error running qlty smells: {exc}", exception=exc,
        )

    if not result.stdout.strip():
        logger.info("No smells found (clean)")
        issues: list[SarifIssue] = []
        return r[list[SarifIssue]].ok(issues)

    output_file.write_text(result.stdout, encoding="utf-8")
    logger.info("Saved SARIF to %s", output_file)

    parsed = parse_sarif_file(output_file)
    if parsed.failure:
        return parsed
    issues = parsed.unwrap()
    # Mark issues as 'smell' category if not present
    for issue in issues:
        if not issue.category:
            issue.category = QltyCategory.SMELL

    logger.info(f"Found {len(issues)} smells")
    return r[list[SarifIssue]].ok(issues)
