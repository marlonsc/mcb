"""Lib Tests Test Make Surface.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest

from ._utilities.matchers import tm

ROOT = Path(__file__).resolve().parents[3]


def _run_make(*args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    for name in ("MAKEFLAGS", "MFLAGS", "MAKELEVEL"):
        env.pop(name, None)
    return subprocess.run(
        ["make", *args], cwd=ROOT, check=False, capture_output=True, text=True, env=env
    )


def test_help_lists_flext_public_verbs() -> None:
    result = _run_make("help", "WHAT=usage")
    combined = result.stdout + result.stderr

    tm.that(result.returncode == 0, combined)
    tm.that("work       WHAT=start|status|land|finish" in result.stdout)
    tm.that("_custom_run_mcb-hooks" in result.stdout)
    tm.that("golden" in result.stdout)


def test_custom_mutations_require_apply() -> None:
    # The public verbs are the contract a caller can invoke; the internal
    # `_serialized_*` targets are a generator implementation detail and were
    # removed when flext-infra dropped the `serialize-make` CLI route.
    commands = [
        ("fmt", "WHAT=apply", "APPLY=N"),
        ("fix", "WHAT=apply", "APPLY=N"),
        ("run", "WHAT=mcb-hooks", "APPLY=N"),
        ("gen", "WHAT=agent-pointers", "APPLY=N"),
    ]

    for command in commands:
        result = _run_make(*command)
        combined = result.stdout + result.stderr
        tm.that(
            result.returncode != 0,
            f"{command}: mutation ran without APPLY=Y\n{combined}",
        )
        tm.that(
            "requires APPLY=Y" in combined, f"{command}: missing APPLY gate\n{combined}"
        )


@pytest.mark.slow
@pytest.mark.timeout(900)
def test_invalid_nested_choices_fail_before_dry_run_gates() -> None:
    commands = [
        ["make", "build", "WHAT=codegen-__invalid__"],
        ["make", "check", "WHAT=fix-__invalid__"],
        ["make", "check", "WHAT=dev-__invalid__"],
        ["make", "release", "WHAT=__invalid__"],
        ["make", "work", "WHAT=pr-__invalid__"],
        ["make", "work", "WHAT=sub-__invalid__"],
        ["make", "clean", "WHAT=__invalid__"],
    ]

    for command in commands:
        result = subprocess.run(
            command, cwd=ROOT, check=False, capture_output=True, text=True
        )
        combined = result.stdout + result.stderr
        tm.that(
            result.returncode != 0,
            f"{' '.join(command)}: expected failure, got {result.returncode}\n{combined}",
        )
        tm.that(
            "ERROR:" in combined or "unsupported" in combined,
            f"{' '.join(command)}: missing flext error marker",
        )


@pytest.mark.slow
@pytest.mark.timeout(900)
def test_surface_command_runs_safe_matrix() -> None:
    command = ROOT / "scripts" / "check" / "surface.py"

    result = subprocess.run(
        [sys.executable, str(command)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )

    tm.that(result.returncode == 0, result.stdout + result.stderr)
    tm.that("SURFACE OK" in result.stdout)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
