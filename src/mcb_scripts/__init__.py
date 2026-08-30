# AUTO-GENERATED FILE — Regenerate with: make gen
"""Mcb Scripts package."""

from __future__ import annotations

from typing import TYPE_CHECKING

from types import MappingProxyType

from flext_core.lazy import build_lazy_import_map, install_lazy_exports

if TYPE_CHECKING:
    from . import docs as docs
    from . import qlty as qlty
    from typing import Final

    from .constants import McbConstants, McbConstants as c
    from .core import BaseCommandSettings, McbService
    from .logger import configure_logging, get_logger
    from .result import McbResult, r
    from .service import McbScriptsService, s
    from .settings import BaseMcbSettings, McbSettings
__all__: tuple[str, ...] = (
    "BaseCommandSettings",
    "BaseMcbSettings",
    "Final",
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
            ".constants": ("McbConstants", "c"),
            ".core": ("BaseCommandSettings", "McbService"),
            ".docs": ("docs",),
            ".logger": ("configure_logging", "get_logger"),
            ".qlty": ("qlty",),
            ".result": ("McbResult", "r"),
            ".service": ("McbScriptsService", "s"),
            ".settings": ("BaseMcbSettings", "McbSettings"),
            "typing": ("Final",),
        }),
        alias_groups=MappingProxyType({}),
        sort_keys=False,
    )
)

install_lazy_exports(__name__, globals(), _LAZY_IMPORTS, public_exports=__all__)
