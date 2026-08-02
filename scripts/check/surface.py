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
from dataclasses import dataclass
from pathlib import Path
from typing import cast

SCRIPTS = Path(__file__).resolve().parents[1]
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

from lib.core import BaseCommandSettings, McbResult, get_logger  # noqa: E402

logger = get_logger(__name__)

ROOT = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class SurfaceCase:
    name: str
    args: tuple[str, ...]
    expected_rc: int = 0
    must_contain: tuple[str, ...] = ()


READ_ONLY_CASES = (
    SurfaceCase("help", ("help",), must_contain=("MCB", "make <verb>")),
    SurfaceCase("ship status", ("ship", "WHAT=status")),
    SurfaceCase("ship tags", ("ship", "WHAT=tags")),
    SurfaceCase("ship branch list", ("ship", "WHAT=branch")),
    SurfaceCase("ship sub status", ("ship", "WHAT=sub", "ACT=status")),
    SurfaceCase("ship release version list", ("ship", "WHAT=release", "ACT=version"), must_contain=("Current:",)),
    SurfaceCase("docs check", ("build", "WHAT=docs", "ACT=check")),
    SurfaceCase("docs adr list", ("build", "WHAT=docs", "ACT=adr"), must_contain=("Architecture Decision Records",)),
    SurfaceCase("gitops check", ("check", "WHAT=gitops"), must_contain=("GITOPS",)),
    SurfaceCase("optimize dry-run script", ("check", "WHAT=optimize"), must_contain=("DRY-RUN",)),
)

DRY_RUN_CASES = (
    SurfaceCase("codegen all", ("build", "WHAT=codegen", "ACT=all"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs build", ("build", "WHAT=docs", "ACT=build"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs serve", ("build", "WHAT=docs", "ACT=serve"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs sync", ("build", "WHAT=docs", "ACT=sync"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs setup", ("build", "WHAT=docs", "ACT=setup"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs adr-new", ("build", "WHAT=docs", "ACT=adr-new"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs diagrams", ("build", "WHAT=docs", "ACT=diagrams"), must_contain=("DRY-RUN",)),
    SurfaceCase("docs lint fix", ("build", "WHAT=docs", "ACT=lint", "FIX=1"), must_contain=("DRY-RUN",)),
    SurfaceCase("test e2e", ("test", "SCOPE=e2e"), must_contain=("DRY-RUN",)),
    SurfaceCase("check fix", ("check", "WHAT=fix", "ACT=all"), must_contain=("DRY-RUN",)),
    SurfaceCase("check dev", ("check", "WHAT=dev", "ACT=run"), must_contain=("DRY-RUN",)),
    SurfaceCase("clean all", ("clean", "WHAT=all"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship add", ("ship", "WHAT=add", "FILES=Makefile"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship commit", ("ship", "WHAT=commit", "MSG=surface-check"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship push", ("ship", "WHAT=push"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship pull", ("ship", "WHAT=pull"), must_contain=("DRY-RUN",)),
    SurfaceCase(
        "ship branch create",
        ("ship", "WHAT=branch", "REF=surface-check-probe", "BASE=HEAD"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase("ship checkout", ("ship", "WHAT=checkout", "REF=HEAD"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship tag", ("ship", "WHAT=tag", "TAG=surface-check-probe"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship stash", ("ship", "WHAT=stash"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship stash-pop", ("ship", "WHAT=stash-pop"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship merge", ("ship", "WHAT=merge", "REF=HEAD"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship rebase", ("ship", "WHAT=rebase", "BASE=HEAD"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship unstage", ("ship", "WHAT=unstage", "FILES=Makefile"), must_contain=("DRY-RUN",)),
    SurfaceCase("ship push-tags", ("ship", "WHAT=push-tags", "TAG=surface-check-probe"), must_contain=("DRY-RUN",)),
    SurfaceCase("pr merge", ("ship", "WHAT=pr", "ACT=merge", "PR=1"), must_contain=("DRY-RUN",)),
    SurfaceCase("pr rerun", ("ship", "WHAT=pr", "ACT=rerun", "RUN=1"), must_contain=("DRY-RUN",)),

    SurfaceCase("release package", ("ship", "WHAT=release", "ACT=package"), must_contain=("DRY-RUN",)),
    SurfaceCase(
        "release version bump",
        ("ship", "WHAT=release", "ACT=version", "BUMP=patch"),
        must_contain=("DRY-RUN",),
    ),
    SurfaceCase("release install", ("ship", "WHAT=release", "ACT=install"), must_contain=("DRY-RUN",)),
)

INVALID_CASES = (
    SurfaceCase("boot invalid", ("boot", "WHAT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("build invalid", ("build", "WHAT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("test invalid", ("test", "SCOPE=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("check invalid", ("check", "WHAT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("ship invalid", ("ship", "WHAT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("clean invalid", ("clean", "WHAT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase(
        "codegen invalid",
        ("build", "WHAT=codegen", "ACT=__invalid__"),
        expected_rc=2,
        must_contain=("ERRO:",),
    ),
    SurfaceCase("docs invalid", ("build", "WHAT=docs", "ACT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("fix invalid", ("check", "WHAT=fix", "ACT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("dev invalid", ("check", "WHAT=dev", "ACT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("pr invalid", ("ship", "WHAT=pr", "ACT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase("sub invalid", ("ship", "WHAT=sub", "ACT=__invalid__"), expected_rc=2, must_contain=("ERRO:",)),
    SurfaceCase(
        "release invalid",
        ("ship", "WHAT=release", "ACT=__invalid__"),
        expected_rc=2,
        must_contain=("ERRO:",),
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

    result = subprocess.run(
        ("make", "APPLY=N", *case.args),
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
        return cast(McbResult[int], McbResult[int].fail("surface validation failed"))

    logger.info(f"SURFACE OK: {len(cases)} command cases validated")
    logger.info("External/long-running operations are represented by APPLY-gated dry-runs.")
    return cast(McbResult[int], McbResult[int].ok(len(cases)))


def main() -> int:
    """Entrypoint used by the cosmos-command dispatcher."""
    result = run(SurfaceSettings())
    if result.failure:
        logger.error(result.error or "surface validation failed")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
