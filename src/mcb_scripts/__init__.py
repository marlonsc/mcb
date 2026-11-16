# AUTO-GENERATED FILE — Regenerate with: make gen
"""Mcb Scripts package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core import install_lazy_exports

if TYPE_CHECKING:
    from flext_cli import d, e, h, r, s, x

    from mcb_scripts import docs
    from mcb_scripts.constants import McbScriptsConstants, c
    from mcb_scripts.models import McbScriptsModels, m
    from mcb_scripts.protocols import McbScriptsProtocols, p
    from mcb_scripts.typings import McbScriptsTypes, t
    from mcb_scripts.utilities import McbScriptsUtilities, u


__all__: tuple[str, ...] = (
    "McbScriptsConstants",
    "McbScriptsModels",
    "McbScriptsProtocols",
    "McbScriptsTypes",
    "McbScriptsUtilities",
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
        "McbScriptsConstants": ".constants",
        "McbScriptsModels": ".models",
        "McbScriptsProtocols": ".protocols",
        "McbScriptsTypes": ".typings",
        "McbScriptsUtilities": ".utilities",
        "c": ".constants",
        "d": "flext_cli",
        "docs": ".docs",
        "e": "flext_cli",
        "h": "flext_cli",
        "m": ".models",
        "p": ".protocols",
        "r": "flext_cli",
        "s": "flext_cli",
        "t": ".typings",
        "u": ".utilities",
        "x": "flext_cli",
    }),
    public_exports=__all__,
)
