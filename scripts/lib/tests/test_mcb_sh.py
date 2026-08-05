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

from ._utilities.matchers import tm

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

    tm.that(result.returncode == 0, result.stderr)
    assert result.stdout.strip() == str(target_bin)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
