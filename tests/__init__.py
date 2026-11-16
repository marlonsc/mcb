# AUTO-GENERATED FILE — Regenerate with: make gen
"""Tests package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core import install_lazy_exports

if TYPE_CHECKING:
    from flext_tests import api, d, e, h, r, s, td, tf, tk, tm, x

    from tests.constants import TestsMcbScriptsConstants, c
    from tests.models import TestsMcbScriptsModels, m
    from tests.protocols import TestsMcbScriptsProtocols, p
    from tests.typings import TestsMcbScriptsTypes, t
    from tests.utilities import TestsMcbScriptsUtilities, u


__all__: tuple[str, ...] = (
    "TestsMcbScriptsConstants",
    "TestsMcbScriptsModels",
    "TestsMcbScriptsProtocols",
    "TestsMcbScriptsTypes",
    "TestsMcbScriptsUtilities",
    "api",
    "c",
    "d",
    "e",
    "h",
    "m",
    "p",
    "r",
    "s",
    "t",
    "td",
    "tf",
    "tk",
    "tm",
    "u",
    "x",
)

install_lazy_exports(
    __name__,
    globals(),
    MappingProxyType({
        "TestsMcbScriptsConstants": ".constants",
        "TestsMcbScriptsModels": ".models",
        "TestsMcbScriptsProtocols": ".protocols",
        "TestsMcbScriptsTypes": ".typings",
        "TestsMcbScriptsUtilities": ".utilities",
        "api": "flext_tests",
        "c": ".constants",
        "d": "flext_tests",
        "e": "flext_tests",
        "h": "flext_tests",
        "m": ".models",
        "p": ".protocols",
        "r": "flext_tests",
        "s": "flext_tests",
        "t": ".typings",
        "td": "flext_tests",
        "tf": "flext_tests",
        "tk": "flext_tests",
        "tm": "flext_tests",
        "u": ".utilities",
        "x": "flext_tests",
    }),
    public_exports=__all__,
)
