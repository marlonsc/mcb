"""Validate optimizer process selection through its command surface."""

from __future__ import annotations

import os
import stat
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
OPTIMIZER = ROOT / "scripts" / "dev-env-optimize.sh"


def _write_command(bin_dir: Path, name: str, body: str) -> None:
    command = bin_dir / name
    command.write_text(f"#!/usr/bin/env bash\n{body}\n", encoding="utf-8")
    command.chmod(command.stat().st_mode | stat.S_IXUSR)


def test_optimizer_reports_and_selects_only_project_owned_processes(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    foreign_root = tmp_path / "foreign"
    proc_root = tmp_path / "proc"
    bin_dir = tmp_path / "bin"
    kill_log = tmp_path / "kills"
    command_log = tmp_path / "commands"
    project_root.mkdir()
    foreign_root.mkdir()
    proc_root.mkdir()
    bin_dir.mkdir()

    for pid, cwd in (
        (101, project_root),
        (102, project_root),
        (111, project_root),
        (112, project_root),
        (211, foreign_root),
        (201, foreign_root),
    ):
        pid_dir = proc_root / str(pid)
        pid_dir.mkdir()
        (pid_dir / "cwd").symlink_to(cwd, target_is_directory=True)

    _write_command(
        bin_dir,
        "pgrep",
        """printf 'pgrep %s\\n' "$*" >> "$COMMAND_LOG"
case "$*" in
  *rust.analyzer*) printf '%s\\n' '101 rust-analyzer' '102 rust-analyzer' '201 rust-analyzer' ;;
  *serena*) printf '%s\n' '111 serena start-mcp-server' '112 serena start-mcp-server' '211 serena start-mcp-server' ;;
  *) exit 1 ;;
esac""",
    )
    _write_command(
        bin_dir,
        "ps",
        """printf 'ps %s\\n' "$*" >> "$COMMAND_LOG"
if [ "$*" = "-eo pid,etime,cmd" ]; then
  printf '%s\\n' '  102       45:00 cargo check' '  201       45:00 cargo test'
else
  exit 0
fi""",
    )
    _write_command(bin_dir, "free", "exit 0")
    _write_command(bin_dir, "record-kill", 'printf "%s\\n" "$2" >> "$KILL_LOG"')

    env = os.environ.copy()
    env.update({
        "PATH": f"{bin_dir}:{env['PATH']}",
        "PROJECT_ROOT": str(project_root),
        "PROC_ROOT": str(proc_root),
        "KILL_LOG": str(kill_log),
        "COMMAND_LOG": str(command_log),
        "KILL_COMMAND": str(bin_dir / "record-kill"),
        "KEEP_RA": "2",
        "CARGO_ZOMBIE_MIN": "30",
    })

    result = subprocess.run(
        ["bash", str(OPTIMIZER), "--apply"],
        cwd=project_root,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    assert kill_log.read_text(encoding="utf-8").splitlines() == ["102"]
    assert "PID 201" not in result.stdout
    assert "PID 201" not in result.stderr
    assert "PID 211" not in result.stdout
    assert "PID 211" not in result.stderr
    assert "Found 2 Serena MCP server process(es)." in result.stdout
    assert "PID 102 (cargo) running for 45 min" in result.stderr
    assert "Top memory consumers" not in result.stdout
    assert "Process counts" not in result.stdout
    command_lines = command_log.read_text(encoding="utf-8").splitlines()
    assert all(not line.startswith("ps aux") for line in command_lines)
    assert all(not line.startswith("pgrep -c") for line in command_lines)


def test_optimizer_canonicalizes_configured_project_root_symlink(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_link = tmp_path / "project-link"
    proc_root = tmp_path / "proc"
    bin_dir = tmp_path / "bin"
    kill_log = tmp_path / "kills"
    project_root.mkdir()
    project_link.symlink_to(project_root, target_is_directory=True)
    proc_root.mkdir()
    bin_dir.mkdir()
    for pid in (101, 102):
        pid_dir = proc_root / str(pid)
        pid_dir.mkdir()
        (pid_dir / "cwd").symlink_to(project_root, target_is_directory=True)

    _write_command(
        bin_dir,
        "pgrep",
        """case "$*" in
  *rust.analyzer*) printf '%s\\n' '101 rust-analyzer' '102 rust-analyzer' ;;
  *) exit 1 ;;
esac""",
    )
    _write_command(bin_dir, "ps", "exit 0")
    _write_command(bin_dir, "free", "exit 0")
    _write_command(bin_dir, "record-kill", 'printf "%s\\n" "$2" >> "$KILL_LOG"')
    env = os.environ.copy()
    env.update({
        "PATH": f"{bin_dir}:{env['PATH']}",
        "PROJECT_ROOT": str(project_link),
        "PROC_ROOT": str(proc_root),
        "KILL_LOG": str(kill_log),
        "KILL_COMMAND": str(bin_dir / "record-kill"),
        "KEEP_RA": "1",
    })

    result = subprocess.run(
        ["bash", str(OPTIMIZER), "--apply"],
        cwd=project_root,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    assert kill_log.read_text(encoding="utf-8").splitlines() == ["101"]
