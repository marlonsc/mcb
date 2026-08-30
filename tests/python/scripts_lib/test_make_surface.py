"""Lib Tests Test Make Surface.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest
from flext_cli import cli, t as flext_t


ROOT = Path(__file__).resolve().parents[3]


def _json_list(value: flext_t.JsonValue) -> list[flext_t.JsonValue]:
    """Return `value` as a list, or an empty list when it is anything else.

    cli.read_yaml_file yields a JsonValue union, so every nested lookup has to
    state which shape it expects before iterating.
    """
    return value if isinstance(value, list) else []


def _json_dict(value: flext_t.JsonValue) -> dict[str, flext_t.JsonValue]:
    """Return `value` as a mapping, or an empty mapping when it is anything else."""
    return value if isinstance(value, dict) else {}


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

    assert result.returncode == 0, combined
    # Why (gastown): the `work` lane-lifecycle verb was removed; lanes are
    # owned by Gas Town (gt sling / gt done), not make.
    assert "work" not in result.stdout

    assert "golden" in result.stdout


def test_gitops_check_executes_registered_run_command() -> None:
    result = _run_make("check", "WHAT=gitops")
    combined = result.stdout + result.stderr

    assert result.returncode == 0, combined
    assert "GITOPS SKIP" in combined


def test_default_check_runs_conflict_marker_guard() -> None:
    default_check = _run_make("-n", "check")
    pre_check = _run_make("-n", "pre-check")
    combined = (
        default_check.stdout
        + default_check.stderr
        + pre_check.stdout
        + pre_check.stderr
    )

    assert default_check.returncode == 0, combined
    assert pre_check.returncode == 0, combined
    assert '"pre-check"' in default_check.stdout
    assert "bash scripts/lib/mcb.sh conflict-markers" in pre_check.stdout


def test_custom_mutations_require_apply() -> None:
    # The public verbs are the contract a caller can invoke; the internal
    # `_serialized_*` targets are a generator implementation detail and were
    # removed when flext-infra dropped the `serialize-make` CLI route.
    commands = [
        ("fmt", "WHAT=apply", "APPLY=N"),
        ("fix", "WHAT=apply", "APPLY=N"),
        ("gen", "WHAT=agent-pointers", "APPLY=N"),
    ]

    for command in commands:
        result = _run_make(*command)
        combined = result.stdout + result.stderr
        assert result.returncode != 0, (
            f"{command}: mutation ran without APPLY=Y\n{combined}"
        )
        assert "requires APPLY=Y" in combined, (
            f"{command}: missing APPLY gate\n{combined}"
        )


@pytest.mark.slow
@pytest.mark.timeout(900)
def test_invalid_nested_choices_fail_before_dry_run_gates() -> None:
    commands = [
        ["make", "build", "WHAT=codegen-__invalid__"],
        ["make", "check", "WHAT=fix-__invalid__"],
        ["make", "check", "WHAT=dev-__invalid__"],
        ["make", "release", "WHAT=__invalid__"],
        ["make", "clean", "WHAT=__invalid__"],
    ]

    for command in commands:
        result = subprocess.run(
            command, cwd=ROOT, check=False, capture_output=True, text=True
        )
        combined = result.stdout + result.stderr
        assert result.returncode != 0, (
            f"{' '.join(command)}: expected failure, got {result.returncode}\n{combined}"
        )
        assert "ERROR:" in combined or "unsupported" in combined, (
            f"{' '.join(command)}: missing flext error marker"
        )


def test_generated_gitignore_keeps_declared_project_exceptions() -> None:
    """Regeneration must not drop the project's own ignore rules.

    `.gitignore` is a generated projection, so the project's rules live in the
    `extra_ignored_patterns` overlay of config/workspace.yaml (the mro-jnm1.3
    seam). Both sides are read from their real files here: if the overlay ever
    stops reaching the rendered artifact, the barrier that keeps machine
    config and tool output out of version control silently disappears.
    """
    loaded = cli.read_yaml_file(ROOT / "config" / "workspace.yaml").unwrap()
    assert isinstance(loaded, dict), "workspace.yaml must parse to a mapping"
    manifest: dict[str, flext_t.JsonValue] = loaded
    declared: list[str] = [
        str(pattern)
        for overlay in _json_list(manifest.get("repository_policy_overlays"))
        for pattern in _json_list(_json_dict(overlay).get("extra_ignored_patterns"))
    ]

    assert bool(declared), (
        "config/workspace.yaml declares no extra_ignored_patterns, so the "
        "project's ignore rules are not owned by the generator input"
    )

    rendered = {
        line.strip()
        for line in (ROOT / ".gitignore").read_text().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    }
    missing = [pattern for pattern in declared if pattern not in rendered]
    assert not missing, (
        "declared ignore patterns absent from the generated .gitignore:\n"
        + "\n".join(missing)
    )


def test_git_hooks_have_exactly_one_owner() -> None:
    """Only beads (bd hooks) may own the installed hook shims.

    Gas Town owns the lane lifecycle and Beads owns the git hooks; the
    installed shims must delegate to `bd hooks run`, never to a foreign
    runner (pre-commit or a copied script), so commit gating is identical in
    every checkout.
    """
    hooks_dir = subprocess.run(
        ["git", "rev-parse", "--git-path", "hooks"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    hooks_path = (ROOT / hooks_dir).resolve()

    foreign: list[str] = []
    for stage in ("pre-commit", "pre-push"):
        shim = hooks_path / stage
        if not shim.exists():
            foreign.append(f"{stage}: missing (run `bd hooks install`)")
            continue
        head = shim.read_text(errors="replace")[:400]
        if "bd hooks run" not in head:
            lines = head.splitlines()
            foreign.append(f"{stage}: {lines[1] if len(lines) > 1 else head!r}")

    assert not foreign, (
        "installed git hooks are not owned by beads:\n" + "\n".join(foreign)
    )


def test_generated_hook_entries_are_executable_argv() -> None:
    """Installed git hooks must delegate to the single canonical runner.

    Why (gastown): flext_infra codegen conform unconditionally regenerates
    .pre-commit-config.yaml as a managed file (SSOT: flext_infra
    _constants/check.py PRE_COMMIT_CONFIG) whose first hook already routes
    through `bd hooks run pre-commit` — pre-commit is the dispatch framework,
    not a competing owner. The installed git hook shims are the actual commit
    gate and must delegate to `bd hooks run <stage>`.
    """
    hooks_dir = subprocess.run(
        ["git", "rev-parse", "--git-path", "hooks"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    hooks_path = (ROOT / hooks_dir).resolve()

    for stage in ("pre-commit", "pre-push"):
        shim = hooks_path / stage
        assert shim.exists(), f"{stage} shim missing; run `bd hooks install`"
        head = shim.read_text(errors="replace")[:400]
        assert "bd hooks run" in head, (
            f"{stage} shim does not delegate to bd hooks:\n{head}"
        )

    pre_commit_config = ROOT / ".pre-commit-config.yaml"
    assert not pre_commit_config.exists(), (
        ".pre-commit-config.yaml should not exist; hooks are owned by beads (bd hooks)"
    )


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
