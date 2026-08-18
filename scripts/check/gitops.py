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

from pathlib import Path


from flext_cli import cli
from mcb_scripts.core import BaseCommandSettings, get_logger, r
from mcb_scripts.gitops import summarize
from mcb_scripts.settings import McbSettings
from pydantic import Field

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


# `from __future__ import annotations` defers every annotation to a string, and
# the CLI facade resolves the model in ITS namespace, where names like Path are
# absent. Rebuilding here binds them in the module that actually declares them.
GitopsSettings.model_rebuild()


def run(settings: GitopsSettings) -> r[str]:
    """Discover and validate GitOps manifests.

    Returns the status string rather than the whole summary: the CLI facade
    serializes a successful result as a JSON value, and GitOpsSummary carries
    Path and nested report objects that are not JSON values. The summary is
    still reported in full through the logger below.
    """
    k8s_root = settings.root / str(McbSettings().k8s_dir)
    summary_result = summarize(k8s_root)
    if summary_result.failure:
        logger.error(summary_result.error or "gitops discovery failed")
        return r[str].fail(summary_result.error or "gitops discovery failed")
    summary = summary_result.unwrap()
    logger.info(f"GITOPS {summary.status}: {summary.message}")
    if summary.report.total_issues:
        logger.info(summary.report.generate_summary())
    for target in summary.targets:
        logger.info(f"{target.kind}\t{target.path}")
    if summary.status not in {"OK", "SKIP"}:
        return r[str].fail(summary.message)
    return r[str].ok(summary.status)


def main() -> None:
    """Entrypoint used by the cosmos-command dispatcher and direct CLI runs."""
    app = cli.create_app_with_common_params(
        name="check-gitops", help_text="Run MCB GitOps validation discovery."
    )
    cli.register_result_command(
        app,
        name="run",
        help_text="Discover and validate GitOps manifests.",
        model_cls=GitopsSettings,
        handler=run,
    )
    result = cli.execute_app(app, prog_name="check-gitops")
    if result.failure:
        logger.error(result.error or "cli execution failed")
        raise SystemExit(1)


if __name__ == "__main__":
    main()
