# AUTO-GENERATED FILE — Regenerate with: make gen
"""Scripts.docs.py package."""

from __future__ import annotations

from typing import TYPE_CHECKING

from types import MappingProxyType

from flext_core.lazy import build_lazy_import_map, install_lazy_exports

if TYPE_CHECKING:
    from .check_links import CheckLinksSettings, main, run
    from .check_outdated import CheckOutdatedSettings, OUTDATED_PATTERNS, SUPPRESS_RE
    from .check_source_refs import CheckSourceRefsSettings
__all__: tuple[str, ...] = (
    "OUTDATED_PATTERNS",
    "SUPPRESS_RE",
    "CheckLinksSettings",
    "CheckOutdatedSettings",
    "CheckSourceRefsSettings",
    "main",
    "run",
)

_LAZY_IMPORTS = MappingProxyType(
    build_lazy_import_map(
        MappingProxyType({
            ".check_links": ("CheckLinksSettings", "main", "run"),
            ".check_outdated": (
                "CheckOutdatedSettings",
                "OUTDATED_PATTERNS",
                "SUPPRESS_RE",
            ),
            ".check_source_refs": ("CheckSourceRefsSettings",),
        }),
        alias_groups=MappingProxyType({}),
        sort_keys=False,
    )
)

install_lazy_exports(__name__, globals(), _LAZY_IMPORTS, public_exports=__all__)
