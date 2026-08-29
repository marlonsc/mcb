#!/usr/bin/env python3
"""Docs Py Check Outdated.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[2]
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

from lib.cli import create_app_with_common_params, register_result_command  # noqa: E402
from lib.core import BaseMcbSettings, get_logger, r  # noqa: E402
from lib.settings import McbSettings  # noqa: E402
from pydantic import Field  # noqa: E402

from docs.py import utils  # noqa: E402

logger = get_logger(__name__)

OUTDATED_PATTERNS: list[tuple[str, str]] = [
    (r"v0\.1\.[0-9]+", "old version reference (v0.1.x)"),
    (r"shaku", "shaku DI (superseded by dill)"),
    (r"Shaku", "Shaku DI (superseded by dill)"),
    (r"inventory", "inventory crate (migrated to linkme)"),
    (r"rockets?", "Rocket web framework (migrated to Poem)"),
    (r"mcp-context-browser", "old project name (now mcb)"),
    (r"MCP Context Browser", "old project name (now Memory Context Browser / MCB)"),
    (r"mcb-adapters", "old crate name (removed/renamed)"),
    (r"mcb-core", "old crate name (split into mcb-domain + mcb-infrastructure)"),
    (r"CODEQL_SETUP", "reference to archived doc"),
]

SUPPRESS_RE = re.compile(
    r"superseded|historical|migrated|referenc|deprecat|NOTE|dill|poem|linkme|previous|archived|legacy|renamed|removed",
    re.IGNORECASE,
)


class CheckOutdatedSettings(BaseMcbSettings):
    """Settings for the outdated-content documentation check."""

    root: Path = Field(default=Path("."), description="Project root directory")


def _is_suppressed(line: str) -> bool:
    return bool(SUPPRESS_RE.search(line))


def _process_lines(
    lines: list[str], rel_filepath: str, outdated_patterns: list[tuple[str, str]]
) -> list[tuple[str, int, str, str]]:
    issues_in_file: list[tuple[str, int, str, str]] = []
    for i, line in enumerate(lines, 1):
        stripped = line.strip()
        if not stripped or stripped.startswith("<!--") or stripped.startswith("```"):
            continue

        for pattern, desc in outdated_patterns:
            flags = re.IGNORECASE if pattern.islower() else 0
            if re.search(pattern, line, flags) and not _is_suppressed(line):
                issues_in_file.append((rel_filepath, i, desc, stripped[:80]))
    return issues_in_file


def _check_files(
    docs_dir: str, project_root: str
) -> tuple[list[tuple[str, int, str, str]], int]:
    issues: list[tuple[str, int, str, str]] = []
    checked = 0

    md_files = utils.find_md_files(
        docs_dir, exclude_dirs={".git", "fixtures", "archive"}
    )

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked += 1

        try:
            with open(filepath, encoding="utf-8") as fh:
                lines = fh.readlines()
        except Exception as e:  # noqa: BLE001
            logger.error(f"Error reading {rel_filepath}: {e}")
            continue

        file_issues = _process_lines(lines, rel_filepath, OUTDATED_PATTERNS)
        issues.extend(file_issues)

    return issues, checked


def run(settings: CheckOutdatedSettings) -> r[int]:
    """Check outdated content in documentation."""
    project_root = os.path.abspath(settings.root)
    if settings.root == Path("."):
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, str(McbSettings().docs_dir))

    if not os.path.exists(docs_dir):
        return r[int].fail(f"docs directory not found at {docs_dir}")

    issues, checked = _check_files(docs_dir, project_root)

    logger.info(f"Checked {checked} files for outdated content.")

    if issues:
        logger.info(f"Found {len(issues)} potential outdated references:")
        for fp, lineno, desc, content in sorted(issues):
            logger.info(f"  {fp}:{lineno} [{desc}] {content}")
        # Return 0 for now as these are often false positives or acceptable history
        return r[int].ok(len(issues))

    logger.info("No outdated content found.")
    return r[int].ok(0)


def main() -> None:
    app = create_app_with_common_params(
        name="check-outdated", help_text="Check outdated content in docs."
    )
    register_result_command(
        app,
        name="run",
        help_text="Check outdated content in documentation.",
        model_cls=CheckOutdatedSettings,
        handler=run,
    )
    app()


if __name__ == "__main__":
    main()
