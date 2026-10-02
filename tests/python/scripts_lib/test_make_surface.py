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
        ["make", *args], cwd=ROOT, check=False, capture_output=True, text=True, env=env,
    )


def test_help_lists_flext_public_verbs() -> None:
    """Test help lists flext public verbs."""
    result = _run_make("help", "WHAT=usage")
    combined = result.stdout + result.stderr

    assert result.returncode == 0, combined
    # Why (gastown): the `work` lane-lifecycle verb was removed; lanes are
    # owned by Gas City (gc sling / bead+PR closure), not make.
    verbs = [line.split()[0] for line in result.stdout.splitlines() if line.split()]
    assert "work" not in verbs, f"`work` is still a public verb:\n{result.stdout}"

    # Script-discovered project verbs are public surface too (file SSOT:
    # scripts/<verb>/all.sh), and the full suite owns its own verb.
    for expected in ("rust", "validate", "guard", "gitops", "test-full"):
        assert expected in verbs, f"`{expected}` missing from the public verbs"


def test_gitops_check_executes_registered_run_command() -> None:
    # gitops is a project-owned script verb (scripts/gitops/all.sh); WHAT=all
    # answers the whole-surface selector.
    """Test gitops check executes registered run command."""
    result = _run_make("gitops", "WHAT=all")
    combined = result.stdout + result.stderr

    assert result.returncode == 0, combined
    assert "GITOPS SKIP" in combined


def test_default_check_runs_conflict_marker_guard() -> None:
    """Test default check runs conflict marker guard."""
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
    # The regenerated surface re-enters make for the hook target; the hook
    # name appears unquoted in the recursive invocation line.
    assert "pre-check" in default_check.stdout
    assert "bash scripts/lib/mcb.sh conflict-markers" in pre_check.stdout


@pytest.mark.slow
def test_tree_is_at_generator_fixed_point() -> None:
    # The fixed-point contract: `gen WHAT=check` runs codegen conform in
    # check mode, failing when any managed projection drifts from the
    # canonical render. Running the full apply inside a test paid the whole
    # generation cost (60s+) for a property the check mode proves in
    # seconds — slowness is a defect, not a budget problem.
    """Test tree is at generator fixed point."""
    result = _run_make("gen", "WHAT=check")
    combined = result.stdout + result.stderr
    assert result.returncode == 0, combined[-2000:]


def _assert_make_fails(command: list[str], *, expect_unsupported: bool) -> None:
    result = subprocess.run(
        command, cwd=ROOT, check=False, capture_output=True, text=True,
    )
    combined = result.stdout + result.stderr
    assert result.returncode != 0, (
        f"{' '.join(command)}: expected failure, got {result.returncode}\n{combined}"
    )
    if expect_unsupported:
        assert "unsupported" in combined, (
            f"{' '.join(command)}: missing dispatcher rejection marker\n{combined[-1500:]}"
        )


# The full-make invalid-selector cases pay the direnv/mise/uv bootstrap per
# invocation (~20-25s each, worse under xdist contention) and cannot fit the
# 10s/60s item budgets; the dispatcher's rejection contract is unit-tested in
# test_dispatch.py, and this single fast make-level case locks the canonical
# split surface behavior.
@pytest.mark.slow
def test_invalid_release_selector_fails() -> None:
    """Test invalid release selector fails."""
    _assert_make_fails(
        ["make", "release", "WHAT=__invalid__"], expect_unsupported=False,
    )


def test_generated_gitignore_keeps_declared_project_exceptions() -> None:
    """Regeneration must not drop the project's own ignore rules.

    `.gitignore` is a generated projection, so the project's rules live in
    `ManagedArtifacts.Gitignore.patterns` of config/managed-artifacts.yaml, the
    generator's project-owned extension surface. Both sides are read from their
    real files here: if the declared patterns ever stop reaching the rendered
    artifact, the barrier that keeps machine config and tool output out of
    version control silently disappears.
    """
    loaded = cli.read_yaml_file(ROOT / "config" / "managed-artifacts.yaml").unwrap()
    assert isinstance(loaded, dict), "managed-artifacts.yaml must parse to a mapping"
    manifest: dict[str, flext_t.JsonValue] = loaded
    declared: list[str] = [
        str(pattern)
        for pattern in _json_list(
            _json_dict(
                _json_dict(manifest.get("ManagedArtifacts")).get("Gitignore"),
            ).get("patterns"),
        )
    ]

    assert bool(declared), (
        "config/managed-artifacts.yaml declares no ManagedArtifacts.Gitignore."
        "patterns, so the project's ignore rules are not owned by the generator input"
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


BEADS_HOOK_STAGES = ("pre-commit", "pre-push")


def _hook_shims(tmp_path: Path) -> dict[str, Path]:
    """Return the installed shim path for every Beads-managed stage.

    Both hook tests read the same shims the same way; keeping one reader stops
    them drifting apart, which is how one came to tolerate a missing shim while
    the other asserted it.

    The lookup honours only this repository's own git configuration: the
    operator's global and system config are replaced by an empty file under
    `tmp_path`, so a host-wide `core.hooksPath` (which git resolves before the
    repository's hooks directory) cannot turn a checkout-level contract into a
    statement about one workstation.
    """
    global_config = tmp_path / "gitconfig"
    global_config.write_text("", encoding="utf-8")
    env = os.environ.copy()
    env["GIT_CONFIG_GLOBAL"] = str(global_config)
    env["GIT_CONFIG_NOSYSTEM"] = "1"
    hooks_dir = subprocess.run(
        ["git", "rev-parse", "--git-path", "hooks"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        env=env,
    ).stdout.strip()
    hooks_path = (ROOT / hooks_dir).resolve()
    return {stage: hooks_path / stage for stage in BEADS_HOOK_STAGES}


def test_git_hooks_have_exactly_one_owner(tmp_path: Path) -> None:
    """Only beads (bd hooks) may own the installed hook shims.

    Gas City (gc) owns the lane lifecycle and Beads owns the git hooks; the
    installed shims must delegate to `bd hooks run`, never to a foreign
    runner (pre-commit or a copied script), so commit gating is identical in
    every checkout.
    """
    foreign: list[str] = []
    for stage, shim in _hook_shims(tmp_path).items():
        if not shim.exists():
            foreign.append(f"{stage}: missing (run `bd hooks install`)")
            continue
        head = shim.read_text(errors="replace")[:400]
        if "bd hooks run" not in head:
            lines = head.splitlines()
            foreign.append(f"{stage}: {lines[1] if len(lines) > 1 else head!r}")

    assert not foreign, "installed git hooks are not owned by beads:\n" + "\n".join(
        foreign,
    )


def test_generated_hook_entries_are_executable_argv(tmp_path: Path) -> None:
    """Installed git hooks must delegate to the single canonical runner.

    Why (gastown): beads owns the installed git hook shims, and they are the
    actual commit gate: each must delegate to `bd hooks run <stage>`.

    flext_infra codegen conform regenerates .pre-commit-config.yaml
    unconditionally (SSOT: flext_infra _constants/check.py
    PRE_COMMIT_CONFIG) and declares it a TRACKED managed artifact (the
    generated `.gitignore` deliberately un-ignores it). Upstream is law:
    the projection is version-controlled, while beads remains the only owner
    of the installed hooks — this file stays unused by the lifecycle.
    """
    for stage, shim in _hook_shims(tmp_path).items():
        assert shim.is_file(), f"{stage} shim missing; run `bd hooks install`"
        assert os.access(shim, os.X_OK), f"{stage} shim is not executable: {shim}"
        head = shim.read_text(errors="replace")[:400]
        # A generic `bd hooks run` accepts a shim wired to the wrong stage;
        # require the stage this file is installed as.
        assert f"bd hooks run {stage}" in head, (
            f"{stage} shim does not delegate to `bd hooks run {stage}`:\n{head}"
        )

    tracked = subprocess.run(
        ["git", "ls-files", "--error-unmatch", ".pre-commit-config.yaml"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert tracked.returncode == 0, (
        ".pre-commit-config.yaml is a generated, tracked managed artifact of "
        "flext_infra codegen conform; it must stay version-controlled so the "
        "rendered hook catalog travels with the repository, while beads keeps "
        "owning the installed hooks"
    )


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
