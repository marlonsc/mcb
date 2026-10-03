"""Service base for scripts_lib tests.

Copyright (c) 2026 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import TYPE_CHECKING, override

from flext_tests import FlextTestsServiceBase as _FlextTestsServiceBase

if TYPE_CHECKING:
    from flext_core import p


class McbScriptsTestsServiceBase[TDomainResult: p.Base = p.Base](
    _FlextTestsServiceBase[TDomainResult],
):
    """Project-local test service base with flext-core result typing."""

    @override
    def execute(self) -> p.Result[TDomainResult]:
        """Execute domain service logic - must be implemented by subclasses."""
        msg = "subclasses must implement execute"
        raise NotImplementedError(msg)


s = McbScriptsTestsServiceBase

__all__: list[str] = ["McbScriptsTestsServiceBase", "s"]
