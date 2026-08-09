"""Thin entrypoint for the workspace command framework.

Copied into <workspace>/scripts/dispatch.py by ~/.ai-hub distribution.
The imported module lives at <workspace>/scripts/lib/workspace_command.py.
"""

from __future__ import annotations

from lib.workspace_command import main

if __name__ == "__main__":
    raise SystemExit(main())
