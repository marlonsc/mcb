"""Qlty Runner.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import shutil
import subprocess  # ruff: ignore [suspicious-subprocess-import] -- pinned-binary runner
from pathlib import Path
from typing import TYPE_CHECKING

from mcb_scripts.core import get_logger, r
from mcb_scripts.qlty.model import McbScriptsQltyCategory, McbScriptsSarifIssue
from mcb_scripts.qlty.parser import parse_sarif_file
from mcb_scripts.settings import McbScriptsSettings

if TYPE_CHECKING:
    from flext_core import p

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


def run_qlty_check(
    output_file: Path | None = None,
) -> p.Result[list[McbScriptsSarifIssue]]:
    """Run qlty check --all --sarif, save to file, and parse SARIF output.

    Returns:
        The resulting ``p.Result[list[McbScriptsSarifIssue]]``.
    """
    output_file = output_file or McbScriptsSettings().qlty_check_sarif
    logger.info("Running qlty check --all --sarif...")

    executable = _resolve_qlty()
    if executable is None:
        return r[list[McbScriptsSarifIssue]].fail("qlty executable not found on PATH")

    try:
        result = subprocess.run(  # ruff: ignore [subprocess-without-shell-equals-true] -- argv is the mise-pinned qlty binary plus literals
            [executable, "check", "--all", "--sarif"],
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
            shell=False,
        )
    except subprocess.TimeoutExpired:
        return r[list[McbScriptsSarifIssue]].fail("qlty check timed out after 300s")
    except (OSError, subprocess.SubprocessError) as exc:
        return r[list[McbScriptsSarifIssue]].fail(
            f"error running qlty check: {exc}",
            exception=exc,
        )

    if not result.stdout.strip():
        logger.info("No issues found (clean)")
        issues: list[McbScriptsSarifIssue] = []
        return r[list[McbScriptsSarifIssue]].ok(issues)

    output_file.write_text(result.stdout, encoding="utf-8")
    logger.info("Saved SARIF to %s", output_file)

    parsed = parse_sarif_file(output_file)
    if parsed.failure:
        return parsed
    issues = parsed.unwrap()
    logger.info("Found %s issues", len(issues))
    return r[list[McbScriptsSarifIssue]].ok(issues)


def run_qlty_smells(
    output_file: Path | None = None,
) -> p.Result[list[McbScriptsSarifIssue]]:
    """Run qlty smells --all --sarif, save to file, and parse SARIF output.

    Returns:
        The resulting ``p.Result[list[McbScriptsSarifIssue]]``.
    """
    output_file = output_file or McbScriptsSettings().qlty_smells_sarif
    logger.info("Running qlty smells --all --sarif...")

    executable = _resolve_qlty()
    if executable is None:
        return r[list[McbScriptsSarifIssue]].fail("qlty executable not found on PATH")

    try:
        result = subprocess.run(  # ruff: ignore [subprocess-without-shell-equals-true] -- argv is the mise-pinned qlty binary plus literals
            [executable, "smells", "--all", "--sarif"],
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
            shell=False,
        )
    except subprocess.TimeoutExpired:
        return r[list[McbScriptsSarifIssue]].fail("qlty smells timed out after 300s")
    except (OSError, subprocess.SubprocessError) as exc:
        return r[list[McbScriptsSarifIssue]].fail(
            f"error running qlty smells: {exc}",
            exception=exc,
        )

    if not result.stdout.strip():
        logger.info("No smells found (clean)")
        issues: list[McbScriptsSarifIssue] = []
        return r[list[McbScriptsSarifIssue]].ok(issues)

    output_file.write_text(result.stdout, encoding="utf-8")
    logger.info("Saved SARIF to %s", output_file)

    parsed = parse_sarif_file(output_file)
    if parsed.failure:
        return parsed
    issues = parsed.unwrap()
    # Mark issues as 'smell' category if not present
    for issue in issues:
        if not issue.category:
            issue.category = McbScriptsQltyCategory.SMELL

    logger.info("Found %s smells", len(issues))
    return r[list[McbScriptsSarifIssue]].ok(issues)
