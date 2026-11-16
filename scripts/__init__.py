# AUTO-GENERATED FILE — Regenerate with: make gen
"""Scripts package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core import install_lazy_exports

if TYPE_CHECKING:
    from flext_cli import s

    from mcb_scripts import c, d, e, h, m, p, r, t, u, x
    from scripts import docs
    from scripts.constants import ScriptsMcbScriptsConstants
    from scripts.models import ScriptsMcbScriptsModels
    from scripts.protocols import ScriptsMcbScriptsProtocols
    from scripts.typings import ScriptsMcbScriptsTypes
    from scripts.utilities import ScriptsMcbScriptsUtilities


__all__: tuple[str, ...] = (
    "ScriptsMcbScriptsConstants",
    "ScriptsMcbScriptsModels",
    "ScriptsMcbScriptsProtocols",
    "ScriptsMcbScriptsTypes",
    "ScriptsMcbScriptsUtilities",
    "c",
    "d",
    "docs",
    "e",
    "h",
    "m",
    "p",
    "r",
    "s",
    "t",
    "u",
    "x",
)

install_lazy_exports(
    __name__,
    globals(),
    MappingProxyType({
        "ScriptsMcbScriptsConstants": ".constants",
        "ScriptsMcbScriptsModels": ".models",
        "ScriptsMcbScriptsProtocols": ".protocols",
        "ScriptsMcbScriptsTypes": ".typings",
        "ScriptsMcbScriptsUtilities": ".utilities",
        "c": "mcb_scripts",
        "d": "mcb_scripts",
        "docs": ".docs",
        "e": "mcb_scripts",
        "h": "mcb_scripts",
        "m": "mcb_scripts",
        "p": "mcb_scripts",
        "r": "mcb_scripts",
        "s": "flext_cli",
        "t": "mcb_scripts",
        "u": "mcb_scripts",
        "x": "mcb_scripts",
    }),
    public_exports=__all__,
)
