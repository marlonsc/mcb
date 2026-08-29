#!/usr/bin/env python3
"""Check Surface.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

# /// cosmos-command
# verb = "check"  # noqa: ERA001
# what = "surface"  # noqa: ERA001
# domain = "quality"  # noqa: ERA001
# summary = "Validate the public make verb/WHAT/ACT command surface"  # noqa: ERA001
# description = "Runs the safe command-surface matrix: read-only checks, invalid-choice errors, and mutating dry-runs."  # noqa: ERA001
# example = "make check WHAT=surface"  # noqa: ERA001
# mutates = false  # noqa: ERA001
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

from lib.core import BaseCommandSettings, McbResult, get_logger  # noqa: E402

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
    SurfaceCase("help", ("help",), must_contain=("mcb-scripts",)),
    SurfaceCase("status diagnostics", ("status", "WHAT=diagnostics")),
    SurfaceCase("work status", ("work", "WHAT=status")),
    SurfaceCase("gitops check", ("check", "WHAT=gitops"), must_contain=("GITOPS",)),
    SurfaceCase("check validate", ("check", "WHAT=validate")),
)

# Mutation commands that must be blocked when APPLY != Y.
# The env guard (env["APPLY"] = "N") exercises the same gate as passing
# APPLY=N on the make command line.
BLOCKED_CASES = (
    SurfaceCase(
        "run mcb-hooks blocked",
        ("run", "WHAT=mcb-hooks"),
        expected_rc=2,
        must_contain=("requires APPLY=Y",),
    ),
    SurfaceCase(
        "gen agent-pointers blocked",
        ("gen", "WHAT=agent-pointers"),
        expected_rc=2,
        must_contain=("requires APPLY=Y",),
    ),
    SurfaceCase(
        "fmt apply blocked",
        ("fmt", "WHAT=apply"),
        expected_rc=2,
        must_contain=("requires APPLY=Y",),
    ),
    SurfaceCase(
        "fix apply blocked",
        ("fix", "WHAT=apply"),
        expected_rc=2,
        must_contain=("requires APPLY=Y",),
    ),
)

INVALID_CASES = (
    SurfaceCase(
        "run invalid",
        ("run", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported run WHAT=",),
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
        "gen invalid",
        ("gen", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported gen WHAT=",),
    ),
    SurfaceCase(
        "release invalid",
        ("release", "WHAT=__invalid__"),
        expected_rc=2,
        must_contain=("unsupported release WHAT=",),
    ),
    SurfaceCase(
        "fix invalid",
        ("check", "WHAT=fix-__invalid__"),
        expected_rc=2,
        must_contain=("unsupported check WHAT=",),
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
        (make_bin, *case.args),
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
    cases = (*READ_ONLY_CASES, *BLOCKED_CASES, *INVALID_CASES)
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
