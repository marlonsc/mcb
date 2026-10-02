# AUTO-GENERATED FILE — Regenerate with: make gen
"""Tests.python.scripts Lib. Fixtures package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType

from flext_core.lazy import build_lazy_import_map, install_lazy_exports

__all__: tuple[str, ...] = ()

_LAZY_IMPORTS = MappingProxyType(
    build_lazy_import_map(
        MappingProxyType({}),
        alias_groups=MappingProxyType({}),
        sort_keys=False,
    ),
)

install_lazy_exports(__name__, globals(), _LAZY_IMPORTS, public_exports=__all__)
