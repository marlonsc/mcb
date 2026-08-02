#!/usr/bin/env python3
"""Analyze Qlty.

Thin entrypoint that delegates to scripts/qlty/main.py.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import sys
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

from qlty.main import main  # noqa: E402

if __name__ == "__main__":
    main()
