#!/usr/bin/env python3
"""Wrapper for qlty analysis package.

This wrapper keeps compatibility with older invocations that used
`--markdown <path>` to request a markdown report.
"""

import sys
from qlty.main import main


if __name__ == "__main__":
    if "--markdown" in sys.argv:
        idx = sys.argv.index("--markdown")
        sys.argv[idx] = "--report-file"
    sys.exit(main())
