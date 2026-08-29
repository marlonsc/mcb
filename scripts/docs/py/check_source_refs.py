#!/usr/bin/env python3
"""Docs Py Check Source Refs.

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


class CheckSourceRefsSettings(BaseMcbSettings):
    """Settings for the broken source-reference documentation check."""

    root: Path = Field(default=Path("."), description="Project root directory")


def _check_files(docs_dir: str, project_root: str) -> tuple[list[tuple[str, str]], int]:
    issues: list[tuple[str, str]] = []
    checked = 0

    md_files = utils.find_md_files(docs_dir)

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked += 1

        try:
            with open(filepath, encoding="utf-8") as file:
                content = file.read()
        except Exception as e:  # noqa: BLE001
            logger.error(f"Error reading {rel_filepath}: {e}")
            continue

        content = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)
        refs = re.findall(r"`(crates/[^`]+)`", content)

        for ref in refs:
            if " " in ref or "(" in ref or "::" in ref or "..." in ref:
                continue

            target = os.path.join(project_root, ref.rstrip("/"))
            if not os.path.exists(target) and not os.path.exists(target + ".rs"):
                issues.append((rel_filepath, ref))

    return issues, checked


def run(settings: CheckSourceRefsSettings) -> r[int]:
    """Check broken source references in documentation."""
    project_root = os.path.abspath(settings.root)
    if settings.root == Path("."):
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, str(McbSettings().docs_dir))

    if not os.path.exists(docs_dir):
        return r[int].fail(f"docs directory not found at {docs_dir}")

    issues, checked = _check_files(docs_dir, project_root)

    logger.info(f"Checked source refs in {checked} docs")

    if issues:
        logger.info(f"Found {len(issues)} broken source references:")
        for fp, ref in sorted(set(issues)):
            logger.info(f"  {fp}: `{ref}` -> Not found")
        return r[int].fail(f"{len(issues)} broken source reference(s) found")

    logger.info("No broken source references found.")
    return r[int].ok(checked)


def main() -> None:
    app = create_app_with_common_params(
        name="check-source-refs", help_text="Check broken source references in docs."
    )
    register_result_command(
        app,
        name="run",
        help_text="Check broken source references in documentation.",
        model_cls=CheckSourceRefsSettings,
        handler=run,
    )
    app()


if __name__ == "__main__":
    main()
