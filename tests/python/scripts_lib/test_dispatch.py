"""Unit tests for the project script-verb dispatcher.

The dispatcher is the project-owned executor behind every script verb
(``make <verb> WHAT=<what>`` routes here through the generated Makefile).
Its rejection contract is exercised directly — no make bootstrap — because
a single make invocation pays the direnv/mise/uv harness cost and cannot fit
the configured item budgets.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DISPATCHER = ROOT / "scripts" / "dispatch.py"


def _run(verb: str, what: str | None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    if what is None:
        env.pop("WHAT", None)
    else:
        env["WHAT"] = what
    return subprocess.run(
        [str(ROOT / ".venv" / "bin" / "python"), str(DISPATCHER), verb],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


def test_unknown_selector_is_rejected_with_marker() -> None:
    """Test unknown selector is rejected with marker."""
    result = _run("rust", "__invalid__")
    combined = result.stdout + result.stderr
    assert result.returncode != 0, combined
    assert "unsupported" in combined, combined[-1000:]


def test_unknown_selector_on_gitops_is_rejected() -> None:
    """Test unknown selector on gitops is rejected."""
    result = _run("gitops", "__invalid__")
    combined = result.stdout + result.stderr
    assert result.returncode != 0, combined
    assert "unsupported" in combined, combined[-1000:]


def test_bare_verb_prints_handler_menu() -> None:
    """Test bare verb prints handler menu."""
    result = _run("rust", None)
    combined = result.stdout + result.stderr
    assert result.returncode == 0, combined
    for selector in ("all", "lint", "unit", "integration", "doc", "golden", "test"):
        assert f"  {selector}" in combined, combined


def test_invalid_characters_are_rejected_before_resolution() -> None:
    """Test invalid characters are rejected before resolution."""
    result = _run("rust", "bad;rm")
    combined = result.stdout + result.stderr
    assert result.returncode != 0, combined
    assert "invalid WHAT" in combined, combined


def test_gitops_all_selector_runs_the_registered_handler() -> None:
    """Test gitops all selector runs the registered handler."""
    result = _run("gitops", "all")
    combined = result.stdout + result.stderr
    assert result.returncode == 0, combined
    assert "GITOPS summary" in combined, combined
    assert "✅ SKIP" in combined, combined
