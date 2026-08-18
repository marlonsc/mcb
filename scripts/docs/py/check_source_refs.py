#!/usr/bin/env python3
"""Docs Py Check Source Refs.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import re
from pathlib import Path


from flext_cli import cli
from mcb_scripts.core import BaseMcbSettings, get_logger, r
from mcb_scripts.settings import McbSettings
from pydantic import Field

from mcb_scripts.docs import utils

logger = get_logger(__name__)


class CheckSourceRefsSettings(BaseMcbSettings):
    """Settings for the broken source-reference documentation check."""

    root: Path = Field(default=Path(), description="Project root directory")


# `from __future__ import annotations` defers every annotation to a string, and
# the CLI facade resolves the model in ITS namespace, where names like Path are
# absent. Rebuilding here binds them in the module that actually declares them.
CheckSourceRefsSettings.model_rebuild()


def _check_files(
    docs_dir: str, project_root: Path
) -> tuple[list[tuple[str, str]], int]:
    issues: list[tuple[str, str]] = []
    checked = 0

    md_files = utils.find_md_files(docs_dir)

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked += 1

        try:
            content = Path(filepath).read_text(encoding="utf-8")
        except Exception as e:  # noqa: BLE001
            logger.error(f"Error reading {rel_filepath}: {e}")
            continue

        content = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)
        refs = re.findall(r"`(crates/[^`]+)`", content)

        for ref in refs:
            if " " in ref or "(" in ref or "::" in ref or "..." in ref:
                continue

            target = os.path.join(project_root, ref.rstrip("/"))
            if not Path(target).exists() and not Path(target + ".rs").exists():
                issues.append((rel_filepath, ref))

    return issues, checked


def run(settings: CheckSourceRefsSettings) -> r[int]:
    """Check broken source references in documentation."""
    project_root = Path(settings.root).resolve()
    if settings.root == Path():
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, str(McbSettings().docs_dir))

    if not Path(docs_dir).exists():
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
    app = cli.create_app_with_common_params(
        name="check-source-refs", help_text="Check broken source references in docs."
    )
    cli.register_result_command(
        app,
        name="run",
        help_text="Check broken source references in documentation.",
        model_cls=CheckSourceRefsSettings,
        handler=run,
    )
    result = cli.execute_app(app, prog_name="check-source-refs")
    if result.failure:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
