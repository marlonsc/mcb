"""MCB Python tooling constants facade (re-exports the _constants SSOT).

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from mcb_scripts._constants import (
    McbScriptsConstants,
    McbScriptsQltyCategory,
    McbScriptsSeverity,
)

c = McbScriptsConstants()

__all__ = [
    "McbScriptsConstants",
    "McbScriptsQltyCategory",
    "McbScriptsSeverity",
    "c",
]
