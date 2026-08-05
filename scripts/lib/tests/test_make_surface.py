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
DOMAIN = ROOT / "makefiles" / "domain.mk"
CUSTOM = ROOT / "custom.mk"


def test_help_lists_flext_public_verbs() -> None:
    makefile = MAKEFILE.read_text(encoding="utf-8")
    custom = CUSTOM.read_text(encoding="utf-8")

    tm.that("@flext-managed: continuous" in makefile)
    tm.that("PUBLIC_VERBS :=" in makefile)
    tm.that("_custom_work_commit" in custom)
    tm.that("_custom_run_mcb-hooks" in custom)


def test_mutating_make_arms_are_apply_gated() -> None:
    domain = DOMAIN.read_text(encoding="utf-8")

    gated_markers = {
        "work add": "$(call gate,stage files",
        "work commit before add": "$(call gate,commit",
        "work pull": "$(call gate,pull",
        "work branch create": "$(call gate,create branch",
        "work checkout": "$(call gate,checkout",
        "work tag": "$(call gate,tag",
        "work stash": "$(call gate,stash",
        "work stash-pop": "$(call gate,stash pop",
        "work unstage": "$(call gate,unstage",
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
        tm.that(marker in domain, f"{label} is not gated")

    version_bump = domain.split("define MCB_VERSION_BUMP", 1)[1].split("endef", 1)[0]
    tm.that("$(call gate,version bump" in version_bump)


def test_docs_fix_and_e2e_have_explicit_gates() -> None:
    domain = DOMAIN.read_text(encoding="utf-8")

    tm.that('if [ "$(FIX)" = "1" ]; then $(call gate,fix markdown docs);' in domain)
    tm.that("define MCB_E2E\n$(call gate,run Playwright E2E" in domain)


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
        result = subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)
        combined = result.stdout + result.stderr
        tm.that(result.returncode != 0, f"{' '.join(command)}: expected failure, got {result.returncode}\n{combined}")
        tm.that(
            "ERROR:" in combined or "unsupported" in combined,
            f"{' '.join(command)}: missing flext error marker",
        )


@pytest.mark.slow
@pytest.mark.timeout(900)
def test_surface_command_runs_safe_matrix() -> None:
    command = ROOT / "scripts" / "check" / "surface.py"

    result = subprocess.run([sys.executable, str(command)], cwd=ROOT, check=False, capture_output=True, text=True)

    tm.that(result.returncode == 0, result.stdout + result.stderr)
    tm.that("SURFACE OK" in result.stdout)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
