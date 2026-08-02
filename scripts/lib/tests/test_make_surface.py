"""Lib Tests Test Make Surface.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
from __future__ import annotations

import subprocess
import sys
import time
import tomllib
from pathlib import Path
from re import MULTILINE, Match, search

import pytest

ROOT = Path(__file__).resolve().parents[3]
MAKEFILE = ROOT / "Makefile"
DISPATCH = ROOT / "makefiles" / "dispatch.mk"
MCB_SH = ROOT / "scripts" / "lib" / "mcb.sh"
NEXTEST_CONFIG = ROOT / ".config" / "nextest.toml"
PYPROJECT = ROOT / "pyproject.toml"
PLAYWRIGHT_CONFIG = ROOT / "tests" / "playwright.config.ts"


def _make_deadline(name: str) -> int:
    match: Match[str] | None = search(
        rf"^export {name} \?= ([1-9][0-9]*)$",
        MAKEFILE.read_text(encoding="utf-8"),
        flags=MULTILINE,
    )
    assert match is not None, f"missing canonical deadline {name}"
    return int(match.group(1))


def test_help_does_not_imply_apply_for_read_only_checks() -> None:
    help_source = MAKEFILE.read_text(encoding="utf-8")

    assert "check WHAT=guard | WHAT=ci | WHAT=optimize [APPLY=Y]" not in help_source
    assert "check WHAT=guard | WHAT=ci" in help_source
    assert "check WHAT=optimize ACT=%s [APPLY=Y]" in help_source
    assert "check WHAT=fix     ACT=%s [APPLY=Y]" in help_source


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
        assert marker in dispatch, f"{label} is not gated"

    version_bump = dispatch.split("define MCB_VERSION_BUMP", 1)[1].split("endef", 1)[0]
    assert "$(call gate,version bump" in version_bump


def test_docs_fix_and_e2e_have_explicit_gates() -> None:
    dispatch = DISPATCH.read_text(encoding="utf-8")

    assert 'if [ "$(FIX)" = "1" ]; then $(call gate,fix markdown docs);' in dispatch
    assert "define MCB_E2E\n$(call gate,run Playwright E2E" in dispatch


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
        assert result.returncode == 2, f"{' '.join(command)}: expected rc 2, got {result.returncode}\n{combined}"
        assert "ERRO:" in combined, f"{' '.join(command)}: missing ERRO marker"


def test_surface_command_runs_safe_matrix() -> None:
    command = ROOT / "scripts" / "check" / "surface.py"

    result = subprocess.run([sys.executable, str(command)], cwd=ROOT, check=False, capture_output=True, text=True)

    assert result.returncode == 0, result.stdout + result.stderr
    assert "SURFACE OK" in result.stdout


def test_make_exports_canonical_test_deadlines() -> None:
    dispatch = DISPATCH.read_text(encoding="utf-8")

    assert _make_deadline("MCB_TEST_TIMEOUT_SECONDS") == 10
    assert _make_deadline("MCB_PROCESS_TIMEOUT_SECONDS") == 60
    assert "MCB_DEADLINE = $(MCB_TOOL) deadline $(MCB_PROCESS_TIMEOUT_SECONDS)" in dispatch
    assert "$(MCB_DEADLINE)" in dispatch

    runner_commands = (
        "cargo nextest run",
        "cargo test",
        "uv run --no-sync pytest",
        "playwright test",
    )
    for command in runner_commands:
        for line in dispatch.splitlines():
            if command in line and not line.lstrip().startswith("#"):
                assert "$(MCB_DEADLINE)" in line, f"runner bypasses watchdog: {line.strip()}"


def test_nextest_enforces_ten_second_tests_without_retries() -> None:
    config = tomllib.loads(NEXTEST_CONFIG.read_text(encoding="utf-8"))
    expected = {
        "period": f'{_make_deadline("MCB_TEST_TIMEOUT_SECONDS")}s',
        "terminate-after": 1,
        "grace-period": "0s",
    }

    assert config["profile"]["default"]["slow-timeout"] == expected
    assert config["profile"]["ci"]["slow-timeout"] == expected
    assert "retries" not in NEXTEST_CONFIG.read_text(encoding="utf-8")


def test_pytest_enforces_ten_second_whole_lifecycle_timeout() -> None:
    config = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))
    dev_dependencies = config["project"]["optional-dependencies"]["dev"]
    pytest_config = config["tool"]["pytest"]["ini_options"]

    assert any(dependency.startswith("pytest-timeout") for dependency in dev_dependencies)
    assert pytest_config["timeout"] == _make_deadline("MCB_TEST_TIMEOUT_SECONDS")
    assert pytest_config["timeout_method"] == "thread"
    assert pytest_config["timeout_func_only"] is False


def test_playwright_consumes_canonical_deadlines_without_retries() -> None:
    config = PLAYWRIGHT_CONFIG.read_text(encoding="utf-8")

    assert "process.env.MCB_TEST_TIMEOUT_SECONDS" in config
    assert "process.env.MCB_PROCESS_TIMEOUT_SECONDS" in config
    assert "?? '10'" not in config
    assert "?? '60'" not in config
    assert "timeout: testTimeoutSeconds * 1000" in config
    assert "retries: 0" in config
    assert "timeout: processTimeoutSeconds * 1000" in config


def test_deadline_kills_python_process_group_before_child_writes(tmp_path: Path) -> None:
    marker = tmp_path / "child-survived"
    child_code = f"import time; time.sleep(2); open({str(marker)!r}, 'w').write('leaked')"
    parent_code = (
        "import subprocess, sys, time; "
        f"subprocess.Popen([sys.executable, '-c', {child_code!r}]); "
        "time.sleep(5)"
    )

    result = subprocess.run(
        ["bash", str(MCB_SH), "deadline", "1", sys.executable, "-c", parent_code],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
        timeout=5,
    )
    time.sleep(2)

    assert result.returncode == 124, result.stdout + result.stderr
    assert not marker.exists(), "deadline left a child process alive"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
