# AUTO-GENERATED FILE — Regenerate with: make gen
"""Tests.python.scripts Lib package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core import build_lazy_import_map, install_lazy_exports

if TYPE_CHECKING:
    from tests.python.scripts_lib import _fixtures, _utilities
    from tests.python.scripts_lib.base import McbScriptsTestsServiceBase, s
    from tests.python.scripts_lib.constants import McbScriptsConstants, c
    from tests.python.scripts_lib.models import m
    from tests.python.scripts_lib.protocols import p
    from tests.python.scripts_lib.results import e, r
    from tests.python.scripts_lib.typings import t
    from tests.python.scripts_lib.utilities import u


__all__: tuple[str, ...] = (
    "McbScriptsConstants",
    "McbScriptsTestsServiceBase",
    "_fixtures",
    "_utilities",
    "c",
    "e",
    "m",
    "p",
    "r",
    "s",
    "t",
    "u",
)

_LAZY_IMPORTS = MappingProxyType(
    build_lazy_import_map(
        MappingProxyType({
            "._fixtures": ("_fixtures",),
            "._utilities": ("_utilities",),
            ".base": ("McbScriptsTestsServiceBase", "s"),
            ".constants": ("McbScriptsConstants", "c"),
            ".models": ("m",),
            ".protocols": ("p",),
            ".results": ("e", "r"),
            ".typings": ("t",),
            ".utilities": ("u",),
        }),
        alias_groups=MappingProxyType({}),
        sort_keys=False,
    ),
)

install_lazy_exports(__name__, globals(), _LAZY_IMPORTS, public_exports=__all__)
