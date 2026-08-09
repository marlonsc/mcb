"""Default workspace SSOT for the ~/.ai-hub command framework.

Workspaces should override this file at <workspace>/scripts/lib/workspace.py.
The imported names are the contract required by scripts/lib/workspace_command.py.
"""

from __future__ import annotations

import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
AI_HUB = Path(os.environ.get("AI_HUB", str(Path.home() / ".ai-hub"))).expanduser()
LOCAL_VENV_BIN = ROOT / ".venv" / "bin"
LOCAL_PYTHON = ROOT / ".venv" / "bin" / "python"
AIHUB_VENV_BIN = AI_HUB / ".venv" / "bin"
USER_LOCAL_BIN = Path.home() / ".local" / "bin"
MISE_SHIMS = Path(
    os.environ.get(
        "WORKSPACE_MISE_SHIMS",
        os.environ.get("MISE_SHIMS", str(Path.home() / ".local/share/mise/shims")),
    )
).expanduser()
SUBMODULE_SCRIPT_ROOTS: tuple[Path, ...] = ()


def local_python_cmd() -> Path:
    """Return the required workspace-local .venv Python."""
    return LOCAL_PYTHON


def runtime_path(inherited: str | None = None) -> str:
    """Return canonical PATH: workspace venv, mise, ai-hub venv, user-local, inherited path."""
    entries = (LOCAL_VENV_BIN, MISE_SHIMS, AIHUB_VENV_BIN, USER_LOCAL_BIN)
    prefix = os.pathsep.join(str(path) for path in entries)
    if inherited:
        return f"{prefix}{os.pathsep}{inherited}"
    return prefix


def runtime_env(base: dict[str, str] | None = None) -> dict[str, str]:
    """Return the canonical environment for workspace-dispatched commands."""
    env = dict(os.environ if base is None else base)
    env["AI_HUB"] = str(AI_HUB)
    env["WORKSPACE_ROOT"] = str(ROOT)
    env["WORKSPACE_MISE_SHIMS"] = str(MISE_SHIMS)
    env["MISE_SHIMS"] = str(MISE_SHIMS)
    env["UV_PROJECT_ENVIRONMENT"] = str(ROOT / ".venv")
    env["VIRTUAL_ENV"] = str(ROOT / ".venv")
    env["PATH"] = runtime_path(env.get("PATH"))
    return env
