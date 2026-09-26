# AUTO-GENERATED FILE — Regenerate with: make gen
"""Mcb Scripts package."""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core.lazy import build_lazy_import_map, install_lazy_exports

if TYPE_CHECKING:
    from . import docs, qlty
    from .constants import McbConstants
    from .core import (
        BaseCommandSettings,
        BaseMcbSettings,
        McbService,
        c,
        configure_logging,
        get_logger,
        r,
    )
    from .result import McbResult
    from .service import McbScriptsService, McbScriptsService as s
    from .settings import McbSettings


__all__: tuple[str, ...] = (
    "BaseCommandSettings",
    "BaseMcbSettings",
    "McbConstants",
    "McbResult",
    "McbScriptsService",
    "McbService",
    "McbSettings",
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
            ".constants": ("McbConstants",),
            ".core": (
                "BaseCommandSettings",
                "BaseMcbSettings",
                "McbService",
                "c",
                "configure_logging",
                "get_logger",
                "r",
            ),
            ".docs": ("docs",),
            ".qlty": ("qlty",),
            ".result": ("McbResult",),
            ".service": ("McbScriptsService", "s"),
            ".settings": ("McbSettings",),
        }),
        alias_groups=MappingProxyType({}),
        sort_keys=False,
    )
)

install_lazy_exports(__name__, globals(), _LAZY_IMPORTS, public_exports=__all__)
