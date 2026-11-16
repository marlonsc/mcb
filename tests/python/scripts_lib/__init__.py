# AUTO-GENERATED FILE — Regenerate with: make gen
"""Tests.python.scripts Lib package.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from types import MappingProxyType
from typing import TYPE_CHECKING

from flext_core import install_lazy_exports

if TYPE_CHECKING:
    from tests.python.scripts_lib import _fixtures, _utilities


__all__: tuple[str, ...] = ("_fixtures", "_utilities")

install_lazy_exports(
    __name__,
    globals(),
    MappingProxyType({"_fixtures": "._fixtures", "_utilities": "._utilities"}),
    public_exports=__all__,
)
