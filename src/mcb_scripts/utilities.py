"""Utility functions for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
src/mcb_scripts/utilities
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_cli import FlextCliUtilities



class McbScriptsUtilities(FlextCliUtilities):
    """Utility functions for mcbscripts."""

    class McbScripts:
        """MCB utility namespace consumed by scaffolded test facades."""


u = McbScriptsUtilities

__all__: list[str] = ["McbScriptsUtilities", "u"]
