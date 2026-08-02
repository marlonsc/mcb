"""Lib Tests Test Make Surface.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from ._utilities.matchers import tm

ROOT = Path(__file__).resolve().parents[3]
MAKEFILE = ROOT / "Makefile"
DISPATCH = ROOT / "makefiles" / "dispatch.mk"


def test_help_does_not_imply_apply_for_read_only_checks() -> None:
    help_source = MAKEFILE.read_text(encoding="utf-8")

    tm.that("check WHAT=guard | WHAT=ci | WHAT=optimize [APPLY=Y]" not in help_source)
    tm.that("check WHAT=guard | WHAT=ci" in help_source)
    tm.that("check WHAT=optimize ACT=%s [APPLY=Y]" in help_source)
    tm.that("check WHAT=fix     ACT=%s [APPLY=Y]" in help_source)


def test_mutating_make_arms_are_apply_gated() -> None:
    dispatch = DISPATCH.read_text(encoding="utf-8")

    gated_markers = {
        "ship add": "$(call gate,stage files",
        "ship commit before add": "$(call gate,commit",
        "ship pull": "$(call gate,pull",
        "ship branch create": "$(call gate,create branch",
        "ship checkout": "$(call gate,checkout",
        "ship tag": "$(call gate,tag",
        "ship stash": "$(call gate,stash",
        "ship stash-pop": "$(call gate,stash pop",
        "ship unstage": "$(call gate,unstage",
        "pr rerun": "$(call gate,rerun",

        "release package": "$(call gate,package release",
        "fix fmt": "$(call gate,auto-fix fmt",
        "fix lint": "$(call gate,auto-fix lint",
        "fix docs": "$(call gate,auto-fix docs",
        "dev run": "$(call gate,start dev server",
        "dev docker-up": "$(call gate,start Docker test services",
        "dev docker-down": "$(call gate,stop Docker test services",
        "dev docker-logs": "$(call gate,follow Docker logs",
        "dev docker-test": "$(call gate,run Docker test services",
    }

    for label, marker in gated_markers.items():
        tm.that(marker in dispatch, f"{label} is not gated")

    version_bump = dispatch.split("define MCB_VERSION_BUMP", 1)[1].split("endef", 1)[0]
    tm.that("$(call gate,version bump" in version_bump)


def test_docs_fix_and_e2e_have_explicit_gates() -> None:
    dispatch = DISPATCH.read_text(encoding="utf-8")

    tm.that('if [ "$(FIX)" = "1" ]; then $(call gate,fix markdown docs);' in dispatch)
    tm.that("define MCB_E2E\n$(call gate,run Playwright E2E" in dispatch)


def test_invalid_nested_choices_fail_before_dry_run_gates() -> None:
    commands = [
        ["make", "build", "WHAT=codegen", "ACT=__invalid__"],
        ["make", "check", "WHAT=fix", "ACT=__invalid__"],
        ["make", "check", "WHAT=dev", "ACT=__invalid__"],
        ["make", "ship", "WHAT=release", "ACT=__invalid__"],
        ["make", "ship", "WHAT=pr", "ACT=__invalid__"],
        ["make", "ship", "WHAT=sub", "ACT=__invalid__"],
        ["make", "clean", "WHAT=__invalid__"],
    ]

    for command in commands:
        result = subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)
        combined = result.stdout + result.stderr
        tm.that(result.returncode == 2, f"{' '.join(command)}: expected rc 2, got {result.returncode}\n{combined}")
        tm.that("ERRO:" in combined, f"{' '.join(command)}: missing ERRO marker")


def test_surface_command_runs_safe_matrix() -> None:
    command = ROOT / "scripts" / "check" / "surface.py"

    result = subprocess.run([sys.executable, str(command)], cwd=ROOT, check=False, capture_output=True, text=True)

    tm.that(result.returncode == 0, result.stdout + result.stderr)
    tm.that("SURFACE OK" in result.stdout)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
