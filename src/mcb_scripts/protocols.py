"""Protocol definitions for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
src/mcb_scripts/protocols
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import Protocol, runtime_checkable

from flext_cli import FlextCliProtocols



class McbScriptsProtocols(FlextCliProtocols):
    """Protocol definitions for mcbscripts."""

    @runtime_checkable
    class McbScripts(Protocol):
        """MCB protocol namespace consumed by scaffolded test facades."""


p = McbScriptsProtocols

__all__: list[str] = ["McbScriptsProtocols", "p"]
