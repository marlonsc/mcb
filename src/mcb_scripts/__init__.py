# AUTO-GENERATED FILE — Regenerate with: make gen
"""Mcb Scripts package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core import build_lazy_import_map, install_lazy_exports

if TYPE_CHECKING:
    from mcb_scripts import docs, qlty
    from mcb_scripts.constants import (
        McbScriptsConstants,
        McbScriptsQltyCategory,
        McbScriptsSeverity,
        c,
    )
    from mcb_scripts.core import (
        McbScriptsBaseCommandSettings,
        McbScriptsBaseSettings,
        McbService,
        configure_logging,
        get_logger,
        r,
    )
    from mcb_scripts.result import McbResult
    from mcb_scripts.service import McbScriptsService, s
    from mcb_scripts.settings import McbScriptsSettings


__all__: tuple[str, ...] = (
    "McbResult",
    "McbScriptsBaseCommandSettings",
    "McbScriptsBaseSettings",
    "McbScriptsConstants",
    "McbScriptsQltyCategory",
    "McbScriptsService",
    "McbScriptsSettings",
    "McbScriptsSeverity",
    "McbService",
    "c",
    "configure_logging",
    "docs",
    "get_logger",
    "qlty",
    "r",
    "s",
)

_LAZY_IMPORTS = MappingProxyType(
    build_lazy_import_map(
        MappingProxyType({
            ".constants": (
                "McbScriptsConstants",
                "McbScriptsQltyCategory",
                "McbScriptsSeverity",
                "c",
            ),
            ".core": (
                "McbScriptsBaseCommandSettings",
                "McbScriptsBaseSettings",
                "McbService",
                "configure_logging",
                "get_logger",
                "r",
            ),
            ".docs": ("docs",),
            ".qlty": ("qlty",),
            ".result": ("McbResult",),
            ".service": ("McbScriptsService", "s"),
            ".settings": ("McbScriptsSettings",),
        }),
        alias_groups=MappingProxyType({}),
        sort_keys=False,
    ),
)

install_lazy_exports(__name__, globals(), _LAZY_IMPORTS, public_exports=__all__)
