"""Qlty Runner.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from mcb_scripts.core import get_logger, r
from mcb_scripts.settings import McbSettings

from mcb_scripts.qlty.model import SarifIssue
from mcb_scripts.qlty.parser import parse_sarif_file

logger = get_logger(__name__)


def _resolve_qlty() -> str | None:
    """Resolve the qlty executable to an absolute path.

    Passing a bare name lets PATH order decide which binary runs; resolving
    it here pins the decision and turns a missing tool into a typed failure
    instead of an OSError raised from deep inside subprocess.
    """
    return shutil.which("qlty")


def run_qlty_check(output_file: Path | None = None) -> r[list[SarifIssue]]:
    """Run qlty check --all --sarif, save to file, and parse SARIF output."""
    output_file = output_file or McbSettings().qlty_check_sarif
    logger.info("Running qlty check --all --sarif...")

    executable = _resolve_qlty()
    if executable is None:
        return r[list[SarifIssue]].fail("qlty executable not found on PATH")

    try:
        result = subprocess.run(
            [executable, "check", "--all", "--sarif"],
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


def run_qlty_smells(output_file: Path | None = None) -> r[list[SarifIssue]]:
    """Run qlty smells --all --sarif, save to file, and parse SARIF output."""
    output_file = output_file or McbSettings().qlty_smells_sarif
    logger.info("Running qlty smells --all --sarif...")

    executable = _resolve_qlty()
    if executable is None:
        return r[list[SarifIssue]].fail("qlty executable not found on PATH")

    try:
        result = subprocess.run(
            [executable, "smells", "--all", "--sarif"],
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
