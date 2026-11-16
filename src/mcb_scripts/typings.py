"""Type aliases for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
src/mcb_scripts/typings
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_cli import FlextCliTypes



class McbScriptsTypes(FlextCliTypes):
    """Type aliases for mcbscripts."""

    class McbScripts:
        """MCB typing namespace consumed by scaffolded test facades."""


t = McbScriptsTypes

__all__: list[str] = ["McbScriptsTypes", "t"]
