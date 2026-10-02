# Copyright 2026 FLEXT
"""Generated shim: the Mise lock transaction lives in the flext-infra library.

Owner: flext-infra/src/flext_infra/bootstrap.py (stdlib-only and executable by
file path, so it works even when the workspace environment is broken). This
projection only locates the library and delegates; keep it minimal (Makefile
simplicity law, operator 2026-10-02).
"""

from __future__ import annotations

import os
import runpy
import sys
from pathlib import Path

LIBRARY_RELATIVE = Path("flext-infra") / "src" / "flext_infra" / "bootstrap.py"


def _library() -> Path | None:
    override = os.environ.get("FLEXT_INFRA_BOOTSTRAP")
    if override:
        candidate = Path(override)
        if candidate.is_file():
            return candidate
    for ancestor in Path(__file__).resolve().parents:
        candidate = ancestor / LIBRARY_RELATIVE
        if candidate.is_file():
            return candidate
    return None


def main() -> int:
    library = _library()
    if library is None:
        try:
            from flext_infra import bootstrap as library_module
        except ImportError:
            sys.stderr.write(
                "ERROR: flext-infra bootstrap library not found; set"
                " FLEXT_INFRA_BOOTSTRAP or run from a workspace checkout\n"
            )
            return 2
        return library_module.FlextInfraBootstrap.main(sys.argv[1:])
    runpy.run_path(str(library), run_name="__main__")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
