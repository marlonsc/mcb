#!/usr/bin/env python3
"""Docs Py Check Links.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
from pathlib import Path


from flext_cli import cli
from mcb_scripts.core import BaseMcbSettings, get_logger, r
from mcb_scripts.settings import McbSettings
from pydantic import Field

from mcb_scripts.docs import utils

logger = get_logger(__name__)


class CheckLinksSettings(BaseMcbSettings):
    """Settings for the broken-link documentation check."""

    root: Path = Field(default=Path(), description="Project root directory")


# `from __future__ import annotations` defers every annotation to a string, and
# the CLI facade resolves the model in ITS namespace, where names like Path are
# absent. Rebuilding here binds them in the module that actually declares them.
CheckLinksSettings.model_rebuild()


def _process_links(
    links: list[tuple[str, str]], filepath: str, rel_filepath: str, project_root: Path
) -> tuple[list[tuple[str, str, str, str]], int]:
    broken_in_file: list[tuple[str, str, str, str]] = []
    checked_in_file = 0

    for text, link in links:
        checked_in_file += 1
        if link.startswith(("http", "mailto:", "ftp:")):
            continue

        if link.startswith("/"):
            target = os.path.join(project_root, link.lstrip("/"))
        else:
            target = os.path.normpath(os.path.join(Path(filepath).parent, link))

        if not Path(target).exists():
            broken_in_file.append((
                rel_filepath,
                text,
                link,
                os.path.relpath(target, project_root),
            ))

    return broken_in_file, checked_in_file


def _check_files(
    docs_dir: str, project_root: Path
) -> tuple[list[tuple[str, str, str, str]], int, int]:
    broken: list[tuple[str, str, str, str]] = []
    checked_files = 0
    checked_links = 0

    md_files = utils.find_md_files(docs_dir)

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked_files += 1

        try:
            content = Path(filepath).read_text(encoding="utf-8")
        except Exception as e:  # noqa: BLE001
            logger.error(f"Error reading {rel_filepath}: {e}")
            continue

        links = utils.extract_links(content)
        file_broken, file_links = _process_links(
            links, filepath, rel_filepath, project_root
        )

        broken.extend(file_broken)
        checked_links += file_links

    return broken, checked_files, checked_links


def run(settings: CheckLinksSettings) -> r[int]:
    """Check broken internal links in documentation."""
    project_root = Path(settings.root).resolve()
    if settings.root == Path():
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, str(McbSettings().docs_dir))

    if not Path(docs_dir).exists():
        return r[int].fail(f"docs directory not found at {docs_dir}")

    broken, checked_files, checked_links = _check_files(docs_dir, project_root)

    logger.info(f"Checked {checked_files} files, {checked_links} internal links.")

    if broken:
        logger.info(f"Found {len(broken)} broken internal links:")
        for fp, text, link, target in sorted(broken):
            logger.info(f"  {fp}: [{text}]({link}) -> {target} (missing)")
        return r[int].fail(f"{len(broken)} broken internal link(s) found")

    logger.info("No broken internal links found.")
    return r[int].ok(checked_links)


def main() -> None:
    app = cli.create_app_with_common_params(
        name="check-links", help_text="Check broken internal links in docs."
    )
    cli.register_result_command(
        app,
        name="run",
        help_text="Check broken internal links in documentation.",
        model_cls=CheckLinksSettings,
        handler=run,
    )
    result = cli.execute_app(app, prog_name="check-links")
    if result.failure:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
