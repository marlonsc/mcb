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

from flext_core import m, p
from mcb_scripts.core import McbScriptsBaseSettings, get_logger, r
from mcb_scripts.docs import utils
from mcb_scripts.settings import McbScriptsSettings

logger = get_logger(__name__)


class CheckSourceRefsSettings(McbScriptsBaseSettings):
    """Settings for the broken source-reference documentation check."""

    root: Path = m.Field(default=Path(), description="Project root directory")


def _check_files(
    docs_dir: str,
    project_root: Path,
) -> tuple[list[tuple[str, str]], int, list[str]]:
    issues: list[tuple[str, str]] = []
    checked = 0
    unreadable: list[str] = []

    # docs/adr is an immutable decision record: its source refs describe the
    # architecture as it was WHEN the ADR was written — history, not living
    # docs to validate against the current tree.
    md_files = utils.find_md_files(docs_dir, exclude_dirs={"adr"})

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked += 1

        try:
            content = Path(filepath).read_text(encoding="utf-8")
        except OSError as e:
            # An unreadable doc must fail the check, never vanish from it.
            unreadable.append(f"{rel_filepath}: {e}")
            continue

        content = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)
        refs = re.findall(r"`(crates/[^`]+)`", content)

        for ref in refs:
            if " " in ref or "(" in ref or "::" in ref or "..." in ref:
                continue

            target = os.path.join(project_root, ref.rstrip("/"))
            if not Path(target).exists() and not Path(target + ".rs").exists():
                issues.append((rel_filepath, ref))

    return issues, checked, unreadable


def run(settings: CheckSourceRefsSettings) -> p.Result[int]:
    """Check broken source references in documentation.

    Returns:
        The resulting ``p.Result[int]``.
    """
    project_root = Path(settings.root).resolve()
    if settings.root == Path():
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, str(McbScriptsSettings().docs_dir))

    if not Path(docs_dir).exists():
        return r[int].fail(f"docs directory not found at {docs_dir}")

    issues, checked, unreadable = _check_files(docs_dir, project_root)

    logger.info("Checked source refs in %s docs", checked)

    if unreadable:
        for entry in sorted(unreadable):
            logger.error("Unreadable documentation file: %s", entry)
        return r[int].fail(f"{len(unreadable)} unreadable documentation file(s)")

    if issues:
        logger.info("Found %s broken source references:", len(issues))
        for fp, ref in sorted(set(issues)):
            logger.info("  %s: `%s` -> Not found", fp, ref)
        return r[int].fail(f"{len(issues)} broken source reference(s) found")

    logger.info("No broken source references found.")
    return r[int].ok(checked)


def main() -> None:
    app = cli.create_app_with_common_params(
        name="check-source-refs",
        help_text="Check broken source references in docs.",
    )
    cli.register_result_command(
        app,
        name="run",
        help_text="Check broken source references in documentation.",
        model_cls=CheckSourceRefsSettings,
        handler=run,
    )
    cli.finalize_result(cli.execute_app(app, prog_name="check-source-refs"))


if __name__ == "__main__":
    main()
