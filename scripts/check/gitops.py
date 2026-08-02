#!/usr/bin/env python3
"""Check Gitops.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
# /// cosmos-command
# verb = "check"
# what = "gitops"
# domain = "quality"
# summary = "Validate GitOps manifests through the shared qlty-style pipeline"
# description = "Discovers Helm/Kustomize targets under k8s/ and reports SKIP when the tree has no manifests yet."
# example = "make check WHAT=gitops"
# mutates = false
# ///
from __future__ import annotations

import sys
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

from lib.cli import create_app_with_common_params, register_result_command  # noqa: E402
from lib.core import BaseCommandSettings, get_logger, r  # noqa: E402
from lib.gitops import GitOpsSummary, summarize  # noqa: E402
from lib.settings import McbSettings  # noqa: E402
from pydantic import Field  # noqa: E402

logger = get_logger(__name__)


class GitopsSettings(BaseCommandSettings):
    """Settings for the GitOps validation command.

    cosmos-command exposes parameters unprefixed, so this base disables the
    default ``MCB_`` prefix while keeping the FLEXT settings lifecycle. The
    ``root`` option is also exposed as ``--root`` for direct CLI usage.
    """

    root: Path = Field(
        default=Path(__file__).resolve().parents[2],
        description="Project root directory",
    )


def run(settings: GitopsSettings) -> r[GitOpsSummary]:
    """Discover and validate GitOps manifests."""
    k8s_root = settings.root / str(McbSettings().k8s_dir)
    summary_result = summarize(k8s_root)
    if summary_result.failure:
        logger.error(summary_result.error or "gitops discovery failed")
        return summary_result
    summary = summary_result.unwrap()
    logger.info(f"GITOPS {summary.status}: {summary.message}")
    if summary.report.total_issues:
        logger.info(summary.report.generate_summary())
    for target in summary.targets:
        logger.info(f"{target.kind}\t{target.path}")
    if summary.status not in {"OK", "SKIP"}:
        return r[GitOpsSummary].fail(summary.message)
    return r[GitOpsSummary].ok(summary)


def main() -> None:
    """Entrypoint used by the cosmos-command dispatcher and direct CLI runs."""
    app = create_app_with_common_params(
        name="check-gitops",
        help_text="Run MCB GitOps validation discovery.",
    )
    register_result_command(
        app,
        name="run",
        help_text="Discover and validate GitOps manifests.",
        model_cls=GitopsSettings,
        handler=run,
    )
    app()


if __name__ == "__main__":
    main()
