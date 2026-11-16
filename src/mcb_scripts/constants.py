"""Constants for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
src/mcb_scripts/constants
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_cli import FlextCliConstants



class McbScriptsConstants(FlextCliConstants):
    """Constants for mcbscripts."""

    class McbScripts:
        """MCB constant namespace consumed by scaffolded test facades."""


c = McbScriptsConstants

__all__: list[str] = ["McbScriptsConstants", "c"]
