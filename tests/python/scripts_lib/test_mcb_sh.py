"""Lib Tests Test Mcb Sh.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[3]
MCB_SH = ROOT / "scripts" / "lib" / "mcb.sh"


def test_mcb_bin_prefers_workspace_binary_over_path_binary(temp_dir: Path) -> None:
    workspace = temp_dir / "workspace"
    script_dir = workspace / "scripts" / "lib"
    target_bin = workspace / "target" / "debug" / "mcb"
    path_bin_dir = temp_dir / "bin"
    path_bin = path_bin_dir / "mcb"

    script_dir.mkdir(parents=True)
    target_bin.parent.mkdir(parents=True)
    path_bin_dir.mkdir()
    shutil.copy(MCB_SH, script_dir / "mcb.sh")

    for binary in (target_bin, path_bin):
        binary.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR)

    env = os.environ.copy()
    env["PATH"] = f"{path_bin_dir}:{env['PATH']}"
    command = "source scripts/lib/mcb.sh; mcb_bin"
    result = subprocess.run(
        ["bash", "-c", command],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == str(target_bin)


def test_guard_scans_mcb_validate_production_source(temp_dir: Path) -> None:
    workspace = temp_dir / "workspace"
    script_dir = workspace / "scripts" / "lib"
    validator_src = workspace / "crates" / "mcb-validate" / "src"
    script_dir.mkdir(parents=True)
    validator_src.mkdir(parents=True)
    shutil.copy(MCB_SH, script_dir / "mcb.sh")
    violation = validator_src / "seeded_violation.rs"
    violation.write_text("fn violation() { todo!() }\n", encoding="utf-8")

    result = subprocess.run(
        ["bash", "scripts/lib/mcb.sh", "guard"],
        cwd=workspace,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 3
    assert "todo!()" in result.stderr


def test_guard_fails_when_ast_grep_is_unavailable(temp_dir: Path) -> None:
    workspace = temp_dir / "workspace"
    script_dir = workspace / "scripts" / "lib"
    source_dir = workspace / "crates" / "sample" / "src"
    bin_dir = temp_dir / "bin"
    script_dir.mkdir(parents=True)
    source_dir.mkdir(parents=True)
    bin_dir.mkdir()
    (bin_dir / "bash").symlink_to("/bin/bash")
    shutil.copy(MCB_SH, script_dir / "mcb.sh")
    (source_dir / "lib.rs").write_text("pub fn clean() {}\n", encoding="utf-8")
    env = os.environ.copy()
    env["PATH"] = str(bin_dir)

    result = subprocess.run(
        ["bash", "scripts/lib/mcb.sh", "guard"],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    assert "ast-grep" in result.stderr


def test_guard_fails_when_ast_grep_invocation_breaks(temp_dir: Path) -> None:
    workspace = temp_dir / "workspace"
    script_dir = workspace / "scripts" / "lib"
    source_dir = workspace / "crates" / "sample" / "src"
    bin_dir = temp_dir / "bin"
    script_dir.mkdir(parents=True)
    source_dir.mkdir(parents=True)
    bin_dir.mkdir()
    shutil.copy(MCB_SH, script_dir / "mcb.sh")
    (source_dir / "lib.rs").write_text("pub fn clean() {}\n", encoding="utf-8")
    ast_grep = bin_dir / "ast-grep"
    ast_grep.write_text(
        '#!/bin/sh\n[ "$1" = "--version" ] && exit 0\nexit 1\n', encoding="utf-8"
    )
    ast_grep.chmod(ast_grep.stat().st_mode | stat.S_IXUSR)
    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}:/usr/bin:/bin"

    result = subprocess.run(
        ["bash", "scripts/lib/mcb.sh", "guard"],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode != 0
    assert "invocation failed" in result.stderr


def test_guard_accepts_successful_ast_grep_with_zero_matches(temp_dir: Path) -> None:
    workspace = temp_dir / "workspace"
    script_dir = workspace / "scripts" / "lib"
    source_dir = workspace / "crates" / "sample" / "src"
    bin_dir = temp_dir / "bin"
    script_dir.mkdir(parents=True)
    source_dir.mkdir(parents=True)
    bin_dir.mkdir()
    shutil.copy(MCB_SH, script_dir / "mcb.sh")
    (source_dir / "lib.rs").write_text("pub fn clean() {}\n", encoding="utf-8")
    ast_grep = bin_dir / "ast-grep"
    ast_grep.write_text(
        '#!/bin/sh\n[ "$1" = "--version" ] && exit 0\nprintf \'[]\\n\'\nexit 1\n',
        encoding="utf-8",
    )
    ast_grep.chmod(ast_grep.stat().st_mode | stat.S_IXUSR)
    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}:/usr/bin:/bin"

    result = subprocess.run(
        ["bash", "scripts/lib/mcb.sh", "guard"],
        cwd=workspace,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    assert "guard: clean" in result.stderr


def test_conflict_marker_guard_rejects_tracked_markers(temp_dir: Path) -> None:
    workspace = temp_dir / "workspace"
    script_dir = workspace / "scripts" / "lib"
    script_dir.mkdir(parents=True)
    shutil.copy(MCB_SH, script_dir / "mcb.sh")
    marker_file = workspace / "conflicted.txt"
    marker_file.write_text(
        ("<" * 7) + " HEAD\nleft\n" + ("=" * 7) + "\nright\n" + (">" * 7) + " branch\n",
        encoding="utf-8",
    )
    subprocess.run(["git", "init", "-q"], cwd=workspace, check=True)
    subprocess.run(["git", "add", "conflicted.txt"], cwd=workspace, check=True)

    result = subprocess.run(
        ["bash", "scripts/lib/mcb.sh", "guard"],
        cwd=workspace,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 3
    assert "conflicted.txt" in result.stderr


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
