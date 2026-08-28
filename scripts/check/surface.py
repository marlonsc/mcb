#!/usr/bin/env python3
"""Check Surface.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

# /// cosmos-command
# verb = "check"
# what = "surface"
# domain = "quality"
# summary = "Validate the public make verb/WHAT/ACT command surface"
# description = "Runs the safe command-surface matrix: read-only checks, invalid-choice errors, and mutating dry-runs."
# example = "make check WHAT=surface"
# mutates = false
# ///
from __future__ import annotations

import os
import subprocess
import sys
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import cast

SCRIPTS = Path(__file__).resolve().parents[1]
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

from lib.core import BaseCommandSettings, McbResult, get_logger  # ruff: ignore[module-import-not-at-top-of-file]

logger = get_logger(__name__)

ROOT = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class SurfaceCase:
    """One public make invocation to validate."""

    name: str
    args: tuple[str, ...]
    expected_rc: int = 0
    must_contain: tuple[str, ...] = ()


READ_ONLY_CASES = (
    SurfaceCase("help", ("help",), must_contain=("mcb-scripts", "setup", "work")),
    SurfaceCase("status git", ("status", "WHAT=git")),
    SurfaceCase("work tags", ("work", "WHAT=tags")),
    SurfaceCase("work branch list", ("work", "WHAT=branch")),
    SurfaceCase("work sub status", ("work", "WHAT=sub-status")),
    SurfaceCase(
        "release version list", ("release", "WHAT=version"), must_contain=("Current:",)
    ),
    SurfaceCase("docs check", ("build", "WHAT=docs-check")),
    SurfaceCase(
        "docs adr list",
        ("build", "WHAT=docs-adr"),
        must_contain=("Architecture Decision Records",),
    ),
    SurfaceCase("gitops check", ("check", "WHAT=gitops"), must_contain=("GITOPS",)),
    SurfaceCase(
        "optimize cache dry-run",
        ("check", "WHAT=optimize-cache"),
        must_contain=("DRY-RUN",),
    ),
)

DRY_RUN_CASES = (
    SurfaceCase(
        "codegen all", ("build", "WHAT=codegen-all"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase("docs build", ("build", "WHAT=docs-build"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs serve", ("build", "WHAT=docs-serve"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs sync", ("build", "WHAT=docs-sync"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs setup", ("build", "WHAT=docs-setup"), must_contain=("DRY-RUN",)),
    SurfaceCase(
        "docs adr-new", ("build", "WHAT=docs-adr-new"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "docs diagrams", ("build", "WHAT=docs-diagrams"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "docs lint fix", ("build", "WHAT=docs-lint", "FIX=1"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase("test e2e", ("test", "WHAT=e2e"), must_contain=("DRY-RUN",)),
    SurfaceCase("check fix", ("check", "WHAT=fix-all"), must_contain=("DRY-RUN",)),
    SurfaceCase("run dev", ("run", "WHAT=dev-run"), must_contain=("DRY-RUN",)),
    SurfaceCase("clean all", ("clean", "WHAT=all"), must_contain=("DRY-RUN",)),
    SurfaceCase(
        "work add", ("work", "WHAT=add", "FILES=Makefile"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "work commit",
        ("work", "WHAT=commit", "MSG=surface-check"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase("work push", ("work", "WHAT=push"), must_contain=("DRY-RUN",)),
    SurfaceCase("work pull", ("work", "WHAT=pull"), must_contain=("DRY-RUN",)),
    SurfaceCase(
        "work branch create",
        ("work", "WHAT=branch", "REF=surface-check-probe", "BASE=HEAD"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase(
        "work checkout",
        ("work", "WHAT=checkout", "REF=HEAD"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase(
        "work tag",
        ("work", "WHAT=tag", "TAG=surface-check-probe"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase("work stash", ("work", "WHAT=stash"), must_contain=("DRY-RUN",)),
    SurfaceCase(
        "work stash-pop", ("work", "WHAT=stash-pop"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "work merge", ("work", "WHAT=merge", "REF=HEAD"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "work rebase", ("work", "WHAT=rebase", "BASE=HEAD"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "work unstage",
        ("work", "WHAT=unstage", "FILES=Makefile"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase(
        "work push-tags",
        ("work", "WHAT=push-tags", "TAG=surface-check-probe"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase(
        "pr merge", ("work", "WHAT=pr-merge", "PR=1"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "pr rerun", ("work", "WHAT=pr-rerun", "RUN=1"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "release package", ("release", "WHAT=package"), must_contain=("DRY-RUN",)
    ),
    SurfaceCase(
        "release version bump",
        ("release", "WHAT=version", "BUMP=patch"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase(
        "release install", ("release", "WHAT=install"), must_contain=("DRY-RUN",)
    ),
)

INVALID_CASES = (
    SurfaceCase(
        "run invalid",
        ("run", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("ERROR:",),
    ),
    SurfaceCase(
        "build invalid",
        ("build", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("ERROR:",),
    ),
    SurfaceCase(
        "test invalid",
        ("test", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported test WHAT=",),
    ),
    SurfaceCase(
        "check invalid",
        ("check", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported check WHAT=",),
    ),
    SurfaceCase(
        "work invalid",
        ("work", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported work WHAT=",),
    ),
    SurfaceCase(
        "clean invalid",
        ("clean", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported clean WHAT=",),
    ),
    SurfaceCase(
        "codegen invalid",
        ("build", "WHAT=codegen-__invalid__"),
        expected_rc=2,
        must_contain=("ERROR:",),
    ),
    SurfaceCase(
        "docs invalid",
        ("build", "WHAT=docs-__invalid__"),
        expected_rc=2,
        must_contain=("ERROR:",),
    ),
    SurfaceCase(
        "fix invalid",
        ("check", "WHAT=fix-__invalid__"),
        expected_rc=2,
        must_contain=("unsupported check WHAT=",),
    ),
    SurfaceCase(
        "dev invalid",
        ("check", "WHAT=dev-__invalid__"),
        expected_rc=2,
        must_contain=("unsupported check WHAT=",),
    ),
    SurfaceCase(
        "pr invalid",
        ("work", "WHAT=pr-__invalid__"),
        expected_rc=2,
        must_contain=("unsupported work WHAT=",),
    ),
    SurfaceCase(
        "sub invalid",
        ("work", "WHAT=sub-__invalid__"),
        expected_rc=2,
        must_contain=("unsupported work WHAT=",),
    ),
    SurfaceCase(
        "release invalid",
        ("release", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("ERROR:",),
    ),
)


class SurfaceSettings(BaseCommandSettings):
    """Settings for the surface check command.

    cosmos-command exposes parameters unprefixed, so this base disables the
    default ``MCB_`` prefix while keeping the FLEXT settings lifecycle.
    """


def _run_case(case: SurfaceCase) -> str | None:
    env = os.environ.copy()
    env["APPLY"] = "N"
    env["QUICK"] = "1"
    env["TERM"] = "dumb"

    make_bin = shutil.which("make") or "make"
    result = subprocess.run(
        (make_bin, "APPLY=N", *case.args),
        cwd=ROOT,
        env=env,
        check=False,
        capture_output=True,
        text=True,
        timeout=300,
    )
    combined = result.stdout + result.stderr
    if result.returncode != case.expected_rc:
        return f"{case.name}: expected rc {case.expected_rc}, got {result.returncode}\n{combined}"
    missing = [needle for needle in case.must_contain if needle not in combined]
    if missing:
        return f"{case.name}: missing {missing!r}\n{combined}"
    return None


def run(_settings: SurfaceSettings) -> McbResult[int]:
    """Validate the public make command surface."""
    failures: list[str] = []
    cases = (*READ_ONLY_CASES, *DRY_RUN_CASES, *INVALID_CASES)
    for case in cases:
        failure = _run_case(case)
        if failure:
            failures.append(failure)

    if failures:
        logger.info("SURFACE FAIL")
        for failure in failures:
            logger.info(f"\n--- {failure}")
        return cast("McbResult[int]", McbResult[int].fail("surface validation failed"))

    logger.info(f"SURFACE OK: {len(cases)} command cases validated")
    logger.info(
        "External/long-running operations are represented by APPLY-gated dry-runs."
    )
    return cast("McbResult[int]", McbResult[int].ok(len(cases)))


def main() -> int:
    """Entrypoint used by the cosmos-command dispatcher."""
    result = run(SurfaceSettings())
    if result.failure:
        logger.error(result.error or "surface validation failed")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
