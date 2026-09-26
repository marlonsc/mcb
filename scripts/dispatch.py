#!/usr/bin/env python3
"""Script-verb dispatcher for the generated Make surface.

The generator's `_dispatch` routes `<script-verb> WHAT=<what>` here with the
verb name as argv[1] and the selector in $WHAT. This dispatcher resolves the
handler as `<root>/<verb>/<what>.py|<what>.sh` across the declared
SCRIPT_ROOTS (`<root>/<verb>/all.sh` answers the bare and `all` selectors),
executes it, and prints the verb's handler menu when invoked without a
resolvable selector.

Dispatch invariants: verb and selector are allowlisted to `[a-z0-9_-]+`, the
resolved handler must live inside the declared script roots, and the child
process runs argv-listed without a shell.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

_NAME: re.Pattern[str] = re.compile(r"^[a-z0-9_-]+$")


def _repo_root() -> Path:
    # The dispatcher lives at <root>/scripts/dispatch.py: its own directory IS
    # the script root, so the roots declared for the generated Makefile and
    # this file cannot drift apart.
    return Path(__file__).resolve().parent


def _contained(handler: Path) -> bool:
    return handler.resolve().is_relative_to(_repo_root())


def _handlers_for(verb: str) -> list[Path]:
    verb_dir = _repo_root() / verb
    if verb_dir.is_dir():
        return sorted(verb_dir.glob("*"))
    return []


def _resolve(verb: str, what: str) -> Path | None:
    normalized = what.replace("-", "_")
    # `all` answers all.sh naturally through its stem; an unknown selector
    # must never fall back to the whole-surface handler.
    for handler in _handlers_for(verb):
        if handler.stem == normalized:
            return handler
    return None


def _print_help(verb: str) -> int:
    handlers = [p for p in _handlers_for(verb) if p.suffix in {".py", ".sh"}]
    print(f"{verb} WHAT=<selector>")
    for handler in handlers:
        print(f"  {handler.stem}")
    return 0


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: dispatch.py <verb>", file=sys.stderr)
        return 2
    verb = argv[1]
    if not _NAME.fullmatch(verb):
        print(f"ERROR: invalid verb {verb!r}", file=sys.stderr)
        return 2
    what = os.environ.get("WHAT", "").strip()
    if not what:
        return _print_help(verb)
    if not _NAME.fullmatch(what):
        print(f"ERROR: invalid WHAT {what!r}", file=sys.stderr)
        return 2
    handler = _resolve(verb, what)
    if handler is None or not _contained(handler):
        _print_help(verb)
        print(f"ERROR: unsupported {verb} WHAT={what}", file=sys.stderr)
        return 2
    command = (
        [sys.executable, str(handler)]
        if handler.suffix == ".py"
        else ["bash", str(handler)]
    )
    return subprocess.run(command, check=False, shell=False).returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
